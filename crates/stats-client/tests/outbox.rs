use oriha_core::{KindCount, RenderStats};
use oriha_stats_client::*;
use std::collections::HashSet;

const EVENT_ID: &str = "550e8400-e29b-41d4-a716-446655440000";
const EVENT_ID_OLD: &str = "550e8400-e29b-41d4-a716-446655440003";
const EVENT_ID_2: &str = "550e8400-e29b-41d4-a716-446655440001";
const EVENT_ID_3: &str = "550e8400-e29b-41d4-a716-446655440002";

fn config() -> UsageAnalyticsOutboxConfig {
    UsageAnalyticsOutboxConfig {
        ingest_url: "https://stats.example.test/api/ingest".to_string(),
        api_key: "test-key".to_string(),
        request_timeout_ms: 5_000,
        max_queued_events: 200,
        retention_ms: 7 * 24 * 60 * 60 * 1_000,
    }
}

fn stats() -> RenderStats {
    RenderStats {
        schema_version: 1,
        total_blocks: 1,
        blocks_by_kind: vec![KindCount {
            kind: "paragraph".to_string(),
            count: 1,
        }],
        headings_by_level: Vec::new(),
        code_blocks_by_language_kind: Vec::new(),
        callouts_by_kind: Vec::new(),
        locked_features_by_kind: Vec::new(),
        unsupported_by_kind: Vec::new(),
        issues_by_kind: Vec::new(),
    }
}

fn render_payload(event_id: Option<&str>) -> UsageAnalyticsRenderPayloadInput {
    UsageAnalyticsRenderPayloadInput {
        platform: "macOS".to_string(),
        app_version: "1.0".to_string(),
        build_version: "1".to_string(),
        locale: "ja".to_string(),
        subscription_plan: "free".to_string(),
        stats: stats(),
        event_id: event_id.map(str::to_string),
    }
}

fn payload_json(event_id: Option<&str>) -> String {
    let step = start_enqueue(
        UsageAnalyticsOutboxEnqueueStartInput {
            config: config(),
            render_payload: render_payload(event_id),
            is_delivery_enabled: true,
            delivery_generation: 1,
        },
        1_000,
    );
    let envelope_json = step.commands[0]
        .envelope_json
        .as_deref()
        .expect("envelope json");
    let envelope =
        serde_json::from_str::<UsageAnalyticsOutboxEnvelope>(envelope_json).expect("envelope");
    envelope.payload_json
}

fn envelope_json(
    id: &str,
    enqueued_at_ms: u64,
    attempt_count: u32,
    next_attempt_at_ms: u64,
) -> String {
    let envelope = UsageAnalyticsOutboxEnvelope {
        id: id.to_string(),
        payload_json: payload_json(Some(id)),
        enqueued_at_ms,
        attempt_count,
        next_attempt_at_ms,
    };
    serde_json::to_string(&envelope).expect("envelope json")
}

fn stored(id: &str, envelope_json: String) -> UsageAnalyticsStoredEnvelope {
    UsageAnalyticsStoredEnvelope {
        id: id.to_string(),
        envelope_json,
    }
}

fn command_ok(command: &UsageAnalyticsOutboxCommand) -> UsageAnalyticsOutboxCommandResult {
    UsageAnalyticsOutboxCommandResult {
        command_id: command.id.clone(),
        succeeded: true,
        stored_envelopes: Vec::new(),
        status_code: None,
    }
}

fn command_loaded(
    command: &UsageAnalyticsOutboxCommand,
    envelopes: Vec<UsageAnalyticsStoredEnvelope>,
) -> UsageAnalyticsOutboxCommandResult {
    UsageAnalyticsOutboxCommandResult {
        command_id: command.id.clone(),
        succeeded: true,
        stored_envelopes: envelopes,
        status_code: None,
    }
}

fn command_status(
    command: &UsageAnalyticsOutboxCommand,
    status_code: Option<u16>,
) -> UsageAnalyticsOutboxCommandResult {
    UsageAnalyticsOutboxCommandResult {
        command_id: command.id.clone(),
        succeeded: true,
        stored_envelopes: Vec::new(),
        status_code,
    }
}

fn start_load(generation: u64) -> UsageAnalyticsOutboxStep {
    start_drain(
        UsageAnalyticsOutboxStartInput {
            config: config(),
            is_delivery_enabled: true,
            delivery_generation: generation,
        },
        1_000,
    )
}

fn send_step(envelope_json: String, now_ms: u64) -> UsageAnalyticsOutboxStep {
    let step = start_load(4);
    resume_step(
        UsageAnalyticsOutboxResumeInput {
            step_token: step.step_token.expect("step token"),
            results: vec![command_loaded(
                &step.commands[0],
                vec![stored(EVENT_ID, envelope_json)],
            )],
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        now_ms,
    )
}

#[test]
fn start_enqueue_generates_uuid_v4_event_id_for_render_payload() {
    let json = payload_json(None);
    let value = serde_json::from_str::<serde_json::Value>(&json).expect("payload value");
    let event_id = value
        .get("eventId")
        .and_then(|value| value.as_str())
        .expect("event id");

    assert!(uuid::Uuid::parse_str(event_id)
        .expect("uuid")
        .get_version()
        .is_some_and(|version| version == uuid::Version::Random));
    assert_eq!(value["event"], "markdown_render_complete");
    assert_eq!(value["stats"]["blocksByKind"][0]["kind"], "paragraph");
}

#[test]
fn start_enqueue_returns_write_command_and_then_loads_envelopes() {
    let step = start_enqueue(
        UsageAnalyticsOutboxEnqueueStartInput {
            config: config(),
            render_payload: render_payload(Some(EVENT_ID)),
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        1_000,
    );

    assert_eq!(
        step.commands[0].kind,
        UsageAnalyticsOutboxCommandKind::WriteEnvelope
    );
    assert_eq!(step.commands[0].envelope_id.as_deref(), Some(EVENT_ID));
    assert!(step.commands[0].envelope_json.is_some());

    let step = resume_step(
        UsageAnalyticsOutboxResumeInput {
            step_token: step.step_token.expect("step token"),
            results: vec![command_ok(&step.commands[0])],
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        1_000,
    );
    assert_eq!(
        step.commands[0].kind,
        UsageAnalyticsOutboxCommandKind::LoadEnvelopes
    );
}

#[test]
fn outbox_workflow_deletes_invalid_expired_and_overflow_envelopes() {
    let mut config = config();
    config.max_queued_events = 1;
    config.retention_ms = 100;

    let step = start_drain(
        UsageAnalyticsOutboxStartInput {
            config,
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        2_000,
    );
    let step = resume_step(
        UsageAnalyticsOutboxResumeInput {
            step_token: step.step_token.expect("step token"),
            results: vec![command_loaded(
                &step.commands[0],
                vec![
                    stored(EVENT_ID_OLD, envelope_json(EVENT_ID_OLD, 0, 0, 0)),
                    stored("invalid", "{".to_string()),
                    stored(EVENT_ID_2, envelope_json(EVENT_ID_2, 1_000, 0, 1_000)),
                    stored(EVENT_ID_3, envelope_json(EVENT_ID_3, 2_000, 0, 2_000)),
                ],
            )],
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        2_000,
    );

    let deleted = step
        .commands
        .iter()
        .map(|command| command.envelope_id.as_deref().expect("envelope id"))
        .collect::<HashSet<_>>();
    assert!(deleted.contains(EVENT_ID_OLD));
    assert!(deleted.contains("invalid"));
    assert!(deleted.contains(EVENT_ID_2));
    assert!(step
        .commands
        .iter()
        .all(|command| command.kind == UsageAnalyticsOutboxCommandKind::DeleteEnvelope));

    let results = step.commands.iter().map(command_ok).collect();
    let step = resume_step(
        UsageAnalyticsOutboxResumeInput {
            step_token: step.step_token.expect("step token"),
            results,
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        2_000,
    );
    assert_eq!(
        step.commands[0].kind,
        UsageAnalyticsOutboxCommandKind::SendHttp
    );
    assert_eq!(step.commands[0].envelope_id.as_deref(), Some(EVENT_ID_3));
}

#[test]
fn outbox_workflow_returns_send_http_context_for_due_envelope() {
    let step = send_step(envelope_json(EVENT_ID, 1_000, 0, 1_000), 1_000);

    assert_eq!(
        step.commands[0].kind,
        UsageAnalyticsOutboxCommandKind::SendHttp
    );
    assert_eq!(step.commands[0].id, EVENT_ID);
    let context = step.commands[0]
        .http_context
        .as_ref()
        .expect("http context");
    assert_eq!(context.url, "https://stats.example.test/api/ingest");
    assert_eq!(context.method, "POST");
    assert_eq!(context.timeout_ms, 5_000);
    assert!(context
        .headers
        .iter()
        .any(|header| header.name == CONTENT_TYPE_HEADER && header.value == "application/json"));
    assert!(context
        .headers
        .iter()
        .any(|header| header.name == STATS_API_KEY_HEADER && header.value == "test-key"));
}

#[test]
fn outbox_workflow_schedules_future_envelope_without_sending() {
    let step = send_step(envelope_json(EVENT_ID, 1_000, 1, 31_000), 1_000);

    assert!(step.is_complete);
    assert!(step.commands.is_empty());
    assert_eq!(step.next_drain_delay_ms, Some(30_000));
}

#[test]
fn outbox_workflow_deletes_success_and_permanent_failure() {
    for status_code in [202, 400] {
        let step = send_step(envelope_json(EVENT_ID, 1_000, 0, 1_000), 1_000);
        let step = resume_step(
            UsageAnalyticsOutboxResumeInput {
                step_token: step.step_token.expect("step token"),
                results: vec![command_status(&step.commands[0], Some(status_code))],
                is_delivery_enabled: true,
                delivery_generation: 4,
            },
            2_000,
        );

        assert_eq!(
            step.commands[0].kind,
            UsageAnalyticsOutboxCommandKind::DeleteEnvelope
        );
    }
}

#[test]
fn outbox_workflow_retries_network_429_and_500() {
    for status_code in [None, Some(429), Some(500)] {
        let step = send_step(envelope_json(EVENT_ID, 1_000, 0, 1_000), 1_000);
        let step = resume_step(
            UsageAnalyticsOutboxResumeInput {
                step_token: step.step_token.expect("step token"),
                results: vec![command_status(&step.commands[0], status_code)],
                is_delivery_enabled: true,
                delivery_generation: 4,
            },
            2_000,
        );

        assert_eq!(
            step.commands[0].kind,
            UsageAnalyticsOutboxCommandKind::WriteEnvelope
        );
        let step = resume_step(
            UsageAnalyticsOutboxResumeInput {
                step_token: step.step_token.expect("step token"),
                results: vec![command_ok(&step.commands[0])],
                is_delivery_enabled: true,
                delivery_generation: 4,
            },
            2_000,
        );
        assert!(step.is_complete);
        assert_eq!(step.next_drain_delay_ms, Some(30_000));
    }
}

#[test]
fn retry_backoff_is_capped_at_one_hour() {
    assert_eq!(retry_delay_ms(1), 30_000);
    assert_eq!(retry_delay_ms(2), 60_000);
    assert_eq!(retry_delay_ms(8), 3_600_000);
    assert_eq!(retry_delay_ms(100), 3_600_000);
}

#[test]
fn outbox_workflow_enqueues_sends_and_deletes_accepted_payload() {
    let step = start_enqueue(
        UsageAnalyticsOutboxEnqueueStartInput {
            config: config(),
            render_payload: render_payload(Some(EVENT_ID)),
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        1_000,
    );
    let envelope = stored(
        EVENT_ID,
        step.commands[0].envelope_json.clone().expect("envelope"),
    );

    let step = resume_step(
        UsageAnalyticsOutboxResumeInput {
            step_token: step.step_token.expect("step token"),
            results: vec![command_ok(&step.commands[0])],
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        1_000,
    );
    let step = resume_step(
        UsageAnalyticsOutboxResumeInput {
            step_token: step.step_token.expect("step token"),
            results: vec![command_loaded(&step.commands[0], vec![envelope])],
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        1_000,
    );
    assert_eq!(
        step.commands[0].kind,
        UsageAnalyticsOutboxCommandKind::SendHttp
    );

    let step = resume_step(
        UsageAnalyticsOutboxResumeInput {
            step_token: step.step_token.expect("step token"),
            results: vec![command_status(&step.commands[0], Some(202))],
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        2_000,
    );
    assert_eq!(
        step.commands[0].kind,
        UsageAnalyticsOutboxCommandKind::DeleteEnvelope
    );

    let step = resume_step(
        UsageAnalyticsOutboxResumeInput {
            step_token: step.step_token.expect("step token"),
            results: vec![command_ok(&step.commands[0])],
            is_delivery_enabled: true,
            delivery_generation: 4,
        },
        2_000,
    );
    assert!(step.is_complete);
    assert!(step.commands.is_empty());
}

#[test]
fn outbox_workflow_ignores_stale_generation_and_purges_when_disabled() {
    let step = start_load(4);
    let stale = resume_step(
        UsageAnalyticsOutboxResumeInput {
            step_token: step.step_token.expect("step token"),
            results: vec![command_loaded(&step.commands[0], Vec::new())],
            is_delivery_enabled: true,
            delivery_generation: 5,
        },
        1_000,
    );
    assert!(stale.is_complete);
    assert!(stale.commands.is_empty());

    let purge = start_delivery_state_change(UsageAnalyticsOutboxDeliveryStateInput {
        is_delivery_enabled: false,
        delivery_generation: 6,
    });
    assert_eq!(
        purge.commands[0].kind,
        UsageAnalyticsOutboxCommandKind::PurgeEnvelopes
    );
}
