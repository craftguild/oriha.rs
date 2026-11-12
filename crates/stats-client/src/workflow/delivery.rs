//! Outbox retention, HTTP delivery outcomes, retry plans and storage commands.
use super::*;

pub(super) fn plan_loaded_envelopes(
    config: UsageAnalyticsOutboxConfig,
    envelopes: Vec<UsageAnalyticsStoredEnvelope>,
    now_ms: u64,
) -> DrainPlan {
    let mut plan = DrainPlan::default();
    let mut valid_envelopes = Vec::new();
    let mut deleted_ids = HashSet::new();

    for stored in envelopes {
        match decode_stored_envelope(&stored) {
            Some(envelope) if is_expired(&envelope, now_ms, config.retention_ms) => {
                push_delete(&mut plan.storage_commands, &stored.id, &mut deleted_ids);
            }
            Some(envelope) => valid_envelopes.push(envelope),
            None => push_delete(&mut plan.storage_commands, &stored.id, &mut deleted_ids),
        }
    }

    valid_envelopes.sort_by_key(|envelope| envelope.enqueued_at_ms);
    let overflow = valid_envelopes
        .len()
        .saturating_sub(config.max_queued_events as usize);
    for envelope in valid_envelopes.iter().take(overflow) {
        push_delete(&mut plan.storage_commands, &envelope.id, &mut deleted_ids);
    }

    let remaining = valid_envelopes
        .into_iter()
        .skip(overflow)
        .filter(|envelope| !deleted_ids.contains(&envelope.id))
        .collect::<Vec<_>>();

    for envelope in &remaining {
        if envelope.next_attempt_at_ms <= now_ms {
            plan.http_commands
                .push(send_http_command(envelope, &config));
        }
    }

    if plan.http_commands.is_empty() {
        plan.next_drain_delay_ms = next_delay_ms(&remaining, now_ms);
    }

    plan
}

pub(super) fn apply_http_results_to_commands(
    commands: Vec<UsageAnalyticsOutboxCommand>,
    results: Vec<UsageAnalyticsOutboxCommandResult>,
    now_ms: u64,
) -> DrainPlan {
    let mut plan = DrainPlan::default();
    let result_by_command = results
        .into_iter()
        .map(|result| (result.command_id.clone(), result))
        .collect::<HashMap<_, _>>();

    for command in commands {
        let Some(envelope_id) = command.envelope_id.clone() else {
            continue;
        };
        let envelope_json = command.envelope_json.clone().unwrap_or_default();
        let Some(mut envelope) = decode_stored_envelope(&UsageAnalyticsStoredEnvelope {
            id: envelope_id.clone(),
            envelope_json,
        }) else {
            plan.storage_commands
                .push(delete_envelope_command(envelope_id));
            continue;
        };

        let outcome = result_by_command
            .get(&command.id)
            .and_then(|result| result.status_code)
            .map(DeliveryOutcome::from_status_code)
            .unwrap_or(DeliveryOutcome::Retry);

        match outcome {
            DeliveryOutcome::Success | DeliveryOutcome::PermanentFailure => {
                plan.storage_commands
                    .push(delete_envelope_command(envelope.id));
            }
            DeliveryOutcome::Retry => {
                envelope.attempt_count = envelope.attempt_count.saturating_add(1);
                envelope.next_attempt_at_ms =
                    now_ms.saturating_add(retry_delay_ms(envelope.attempt_count));
                if let Some(envelope_json) = encode_envelope(&envelope) {
                    plan.storage_commands
                        .push(write_envelope_command(envelope.id, envelope_json));
                    plan.next_drain_delay_ms = min_delay(
                        plan.next_drain_delay_ms,
                        retry_delay_ms(envelope.attempt_count),
                    );
                }
            }
        }
    }

    plan
}

pub(super) fn write_envelope_command(
    envelope_id: String,
    envelope_json: String,
) -> UsageAnalyticsOutboxCommand {
    UsageAnalyticsOutboxCommand {
        id: format!("write-envelope:{envelope_id}"),
        kind: UsageAnalyticsOutboxCommandKind::WriteEnvelope,
        envelope_id: Some(envelope_id),
        envelope_json: Some(envelope_json),
        http_context: None,
    }
}

pub(super) fn delete_envelope_command(envelope_id: String) -> UsageAnalyticsOutboxCommand {
    UsageAnalyticsOutboxCommand {
        id: format!("delete-envelope:{envelope_id}"),
        kind: UsageAnalyticsOutboxCommandKind::DeleteEnvelope,
        envelope_id: Some(envelope_id),
        envelope_json: None,
        http_context: None,
    }
}

pub(super) fn push_delete(
    commands: &mut Vec<UsageAnalyticsOutboxCommand>,
    envelope_id: &str,
    deleted_ids: &mut HashSet<String>,
) {
    if deleted_ids.insert(envelope_id.to_string()) {
        commands.push(delete_envelope_command(envelope_id.to_string()));
    }
}

pub(super) fn send_http_command(
    envelope: &UsageAnalyticsOutboxEnvelope,
    config: &UsageAnalyticsOutboxConfig,
) -> UsageAnalyticsOutboxCommand {
    UsageAnalyticsOutboxCommand {
        id: envelope.id.clone(),
        kind: UsageAnalyticsOutboxCommandKind::SendHttp,
        envelope_id: Some(envelope.id.clone()),
        envelope_json: encode_envelope(envelope),
        http_context: Some(SendHttpContext {
            url: config.ingest_url.clone(),
            method: "POST".to_string(),
            timeout_ms: config.request_timeout_ms,
            headers: vec![
                HttpHeader {
                    name: CONTENT_TYPE_HEADER.to_string(),
                    value: "application/json".to_string(),
                },
                HttpHeader {
                    name: STATS_API_KEY_HEADER.to_string(),
                    value: config.api_key.clone(),
                },
            ],
            body: envelope.payload_json.clone(),
        }),
    }
}

pub(super) fn next_delay_ms(
    envelopes: &[UsageAnalyticsOutboxEnvelope],
    now_ms: u64,
) -> Option<u64> {
    envelopes
        .iter()
        .map(|envelope| envelope.next_attempt_at_ms.saturating_sub(now_ms))
        .min()
        .map(|delay| delay.max(1_000))
}

pub(super) fn min_delay(current: Option<u64>, next: u64) -> Option<u64> {
    Some(current.map_or(next, |current| current.min(next)))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeliveryOutcome {
    Success,
    PermanentFailure,
    Retry,
}

impl DeliveryOutcome {
    pub(super) fn from_status_code(status_code: u16) -> Self {
        match status_code {
            200..=299 => Self::Success,
            429 | 500..=599 => Self::Retry,
            400..=499 => Self::PermanentFailure,
            _ => Self::Retry,
        }
    }
}
