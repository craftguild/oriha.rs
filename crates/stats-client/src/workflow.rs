//! Outbox session transitions and generation-checked command orchestration.
mod delivery;
#[allow(unused_imports)]
use delivery::*;

use crate::envelope::{
    decode_stored_envelope, encode_envelope, envelope_for_payload, is_expired, retry_delay_ms,
    UsageAnalyticsOutboxEnvelope,
};
use crate::payload::encode_render_payload;
use crate::types::{
    HttpHeader, SendHttpContext, UsageAnalyticsOutboxCommand, UsageAnalyticsOutboxCommandKind,
    UsageAnalyticsOutboxCommandResult, UsageAnalyticsOutboxConfig,
    UsageAnalyticsOutboxDeliveryStateInput, UsageAnalyticsOutboxEnqueueStartInput,
    UsageAnalyticsOutboxResumeInput, UsageAnalyticsOutboxStartInput, UsageAnalyticsOutboxStep,
    UsageAnalyticsStoredEnvelope, CONTENT_TYPE_HEADER, STATS_API_KEY_HEADER,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DrainPlan {
    storage_commands: Vec<UsageAnalyticsOutboxCommand>,
    http_commands: Vec<UsageAnalyticsOutboxCommand>,
    next_drain_delay_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", tag = "phase")]
enum UsageAnalyticsOutboxSession {
    EnqueueStorageApplied {
        id: String,
        generation: u64,
        config: UsageAnalyticsOutboxConfig,
    },
    DrainLoaded {
        id: String,
        generation: u64,
        config: UsageAnalyticsOutboxConfig,
    },
    PlanStorageApplied {
        id: String,
        generation: u64,
        http_commands: Vec<UsageAnalyticsOutboxCommand>,
        next_drain_delay_ms: Option<u64>,
    },
    HttpSent {
        id: String,
        generation: u64,
        http_commands: Vec<UsageAnalyticsOutboxCommand>,
    },
    ResultStorageApplied {
        id: String,
        generation: u64,
        next_drain_delay_ms: Option<u64>,
    },
    Purged {
        id: String,
        generation: u64,
    },
}

pub fn start_enqueue(
    input: UsageAnalyticsOutboxEnqueueStartInput,
    now_ms: u64,
) -> UsageAnalyticsOutboxStep {
    if !input.is_delivery_enabled {
        return complete_step(None);
    }

    let Some(payload_json) = encode_render_payload(input.render_payload) else {
        return complete_step(None);
    };
    let Some(envelope) = envelope_for_payload(payload_json, now_ms) else {
        return complete_step(None);
    };
    let envelope_id = envelope.id.clone();
    let Some(envelope_json) = encode_envelope(&envelope) else {
        return complete_step(None);
    };

    step_with_session(
        UsageAnalyticsOutboxSession::EnqueueStorageApplied {
            id: new_session_id(),
            generation: input.delivery_generation,
            config: input.config,
        },
        vec![write_envelope_command(envelope_id, envelope_json)],
        None,
        false,
    )
}

pub fn start_drain(
    input: UsageAnalyticsOutboxStartInput,
    _now_ms: u64,
) -> UsageAnalyticsOutboxStep {
    if !input.is_delivery_enabled {
        return complete_step(None);
    }

    load_envelopes_step(input.config, input.delivery_generation)
}

pub fn start_delivery_state_change(
    input: UsageAnalyticsOutboxDeliveryStateInput,
) -> UsageAnalyticsOutboxStep {
    if input.is_delivery_enabled {
        return complete_step(None);
    }

    step_with_session(
        UsageAnalyticsOutboxSession::Purged {
            id: new_session_id(),
            generation: input.delivery_generation,
        },
        vec![UsageAnalyticsOutboxCommand {
            id: "purge-envelopes".to_string(),
            kind: UsageAnalyticsOutboxCommandKind::PurgeEnvelopes,
            envelope_id: None,
            envelope_json: None,
            http_context: None,
        }],
        None,
        false,
    )
}

pub fn resume_step(
    input: UsageAnalyticsOutboxResumeInput,
    now_ms: u64,
) -> UsageAnalyticsOutboxStep {
    let Some(session) = decode_session(&input.step_token) else {
        return complete_step(None);
    };

    if !session_allows_delivery_state(&session, input.is_delivery_enabled)
        || session_generation(&session) != input.delivery_generation
    {
        return complete_step(None);
    }

    match session {
        UsageAnalyticsOutboxSession::EnqueueStorageApplied {
            generation, config, ..
        } => {
            if all_commands_succeeded(&input.results) {
                load_envelopes_step(config, generation)
            } else {
                complete_step(None)
            }
        }
        UsageAnalyticsOutboxSession::DrainLoaded {
            generation, config, ..
        } => {
            let Some(result) = input.results.first().filter(|result| result.succeeded) else {
                return complete_step(None);
            };
            let plan = plan_loaded_envelopes(config, result.stored_envelopes.clone(), now_ms);
            drain_plan_step(plan, generation)
        }
        UsageAnalyticsOutboxSession::PlanStorageApplied {
            generation,
            http_commands,
            next_drain_delay_ms,
            ..
        } => {
            if !all_commands_succeeded(&input.results) {
                return complete_step(None);
            }
            http_or_complete_step(http_commands, generation, next_drain_delay_ms)
        }
        UsageAnalyticsOutboxSession::HttpSent {
            generation,
            http_commands,
            ..
        } => {
            let plan = apply_http_results_to_commands(http_commands, input.results, now_ms);
            result_storage_or_complete_step(plan, generation)
        }
        UsageAnalyticsOutboxSession::ResultStorageApplied {
            next_drain_delay_ms,
            ..
        } => {
            if all_commands_succeeded(&input.results) {
                complete_step(next_drain_delay_ms)
            } else {
                complete_step(None)
            }
        }
        UsageAnalyticsOutboxSession::Purged { .. } => complete_step(None),
    }
}

fn load_envelopes_step(
    config: UsageAnalyticsOutboxConfig,
    generation: u64,
) -> UsageAnalyticsOutboxStep {
    step_with_session(
        UsageAnalyticsOutboxSession::DrainLoaded {
            id: new_session_id(),
            generation,
            config,
        },
        vec![UsageAnalyticsOutboxCommand {
            id: "load-envelopes".to_string(),
            kind: UsageAnalyticsOutboxCommandKind::LoadEnvelopes,
            envelope_id: None,
            envelope_json: None,
            http_context: None,
        }],
        None,
        false,
    )
}

fn drain_plan_step(plan: DrainPlan, generation: u64) -> UsageAnalyticsOutboxStep {
    if plan.storage_commands.is_empty() {
        return http_or_complete_step(plan.http_commands, generation, plan.next_drain_delay_ms);
    }

    step_with_session(
        UsageAnalyticsOutboxSession::PlanStorageApplied {
            id: new_session_id(),
            generation,
            http_commands: plan.http_commands,
            next_drain_delay_ms: plan.next_drain_delay_ms,
        },
        plan.storage_commands,
        None,
        false,
    )
}

fn http_or_complete_step(
    http_commands: Vec<UsageAnalyticsOutboxCommand>,
    generation: u64,
    next_drain_delay_ms: Option<u64>,
) -> UsageAnalyticsOutboxStep {
    if http_commands.is_empty() {
        return complete_step(next_drain_delay_ms);
    }

    step_with_session(
        UsageAnalyticsOutboxSession::HttpSent {
            id: new_session_id(),
            generation,
            http_commands: http_commands.clone(),
        },
        http_commands,
        None,
        false,
    )
}

fn result_storage_or_complete_step(plan: DrainPlan, generation: u64) -> UsageAnalyticsOutboxStep {
    if plan.storage_commands.is_empty() {
        return complete_step(plan.next_drain_delay_ms);
    }

    step_with_session(
        UsageAnalyticsOutboxSession::ResultStorageApplied {
            id: new_session_id(),
            generation,
            next_drain_delay_ms: plan.next_drain_delay_ms,
        },
        plan.storage_commands,
        None,
        false,
    )
}

fn step_with_session(
    session: UsageAnalyticsOutboxSession,
    commands: Vec<UsageAnalyticsOutboxCommand>,
    next_drain_delay_ms: Option<u64>,
    is_complete: bool,
) -> UsageAnalyticsOutboxStep {
    UsageAnalyticsOutboxStep {
        step_token: encode_session(&session),
        commands,
        next_drain_delay_ms,
        is_complete,
    }
}

fn complete_step(next_drain_delay_ms: Option<u64>) -> UsageAnalyticsOutboxStep {
    UsageAnalyticsOutboxStep {
        step_token: None,
        commands: Vec::new(),
        next_drain_delay_ms,
        is_complete: true,
    }
}

fn encode_session(session: &UsageAnalyticsOutboxSession) -> Option<String> {
    serde_json::to_string(session).ok()
}

fn decode_session(token: &str) -> Option<UsageAnalyticsOutboxSession> {
    serde_json::from_str(token).ok()
}

fn session_generation(session: &UsageAnalyticsOutboxSession) -> u64 {
    match session {
        UsageAnalyticsOutboxSession::EnqueueStorageApplied { generation, .. }
        | UsageAnalyticsOutboxSession::DrainLoaded { generation, .. }
        | UsageAnalyticsOutboxSession::PlanStorageApplied { generation, .. }
        | UsageAnalyticsOutboxSession::HttpSent { generation, .. }
        | UsageAnalyticsOutboxSession::ResultStorageApplied { generation, .. }
        | UsageAnalyticsOutboxSession::Purged { generation, .. } => *generation,
    }
}

fn session_allows_delivery_state(
    session: &UsageAnalyticsOutboxSession,
    is_delivery_enabled: bool,
) -> bool {
    matches!(session, UsageAnalyticsOutboxSession::Purged { .. }) || is_delivery_enabled
}

fn all_commands_succeeded(results: &[UsageAnalyticsOutboxCommandResult]) -> bool {
    !results.is_empty() && results.iter().all(|result| result.succeeded)
}

fn new_session_id() -> String {
    Uuid::new_v4().to_string()
}
