use crate::types::UsageAnalyticsStoredEnvelope;
use oriha_stats_schema::validate_json_bytes;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageAnalyticsOutboxEnvelope {
    pub id: String,
    pub payload_json: String,
    pub enqueued_at_ms: u64,
    pub attempt_count: u32,
    pub next_attempt_at_ms: u64,
}

pub(crate) fn envelope_for_payload(
    payload_json: String,
    now_ms: u64,
) -> Option<UsageAnalyticsOutboxEnvelope> {
    let payload = validate_json_bytes(payload_json.as_bytes()).ok()?;
    let id = payload.event_id?.to_string();
    Some(UsageAnalyticsOutboxEnvelope {
        id,
        payload_json,
        enqueued_at_ms: now_ms,
        attempt_count: 0,
        next_attempt_at_ms: now_ms,
    })
}

pub(crate) fn decode_stored_envelope(
    stored: &UsageAnalyticsStoredEnvelope,
) -> Option<UsageAnalyticsOutboxEnvelope> {
    let envelope =
        serde_json::from_str::<UsageAnalyticsOutboxEnvelope>(&stored.envelope_json).ok()?;
    if envelope.id != stored.id {
        return None;
    }
    let payload = validate_json_bytes(envelope.payload_json.as_bytes()).ok()?;
    if payload.event_id?.to_string() != envelope.id {
        return None;
    }
    Some(envelope)
}

pub(crate) fn encode_envelope(envelope: &UsageAnalyticsOutboxEnvelope) -> Option<String> {
    serde_json::to_string(envelope).ok()
}

pub(crate) fn is_expired(
    envelope: &UsageAnalyticsOutboxEnvelope,
    now_ms: u64,
    retention_ms: u64,
) -> bool {
    now_ms.saturating_sub(envelope.enqueued_at_ms) > retention_ms
}

pub fn retry_delay_ms(attempt_count: u32) -> u64 {
    let exponent = attempt_count.saturating_sub(1).min(7);
    30_000_u64
        .saturating_mul(2_u64.pow(exponent))
        .min(3_600_000)
}
