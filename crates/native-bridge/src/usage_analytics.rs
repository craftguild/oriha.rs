use oriha_stats_client as stats_client;

use crate::types::*;

#[uniffi::export]
pub fn usage_analytics_outbox_start_enqueue(
    input: NativeUsageAnalyticsOutboxEnqueueStartInput,
    now_ms: u64,
) -> NativeUsageAnalyticsOutboxStep {
    stats_client::start_enqueue(input.into(), now_ms).into()
}

#[uniffi::export]
pub fn usage_analytics_outbox_start_enqueue_json(input_json: String, now_ms: u64) -> String {
    let step =
        match serde_json::from_str::<NativeUsageAnalyticsOutboxEnqueueStartInput>(&input_json) {
            Ok(input) => usage_analytics_outbox_start_enqueue(input, now_ms),
            Err(_) => invalid_usage_analytics_step(),
        };

    serialize_step(&step)
}

#[uniffi::export]
pub fn usage_analytics_outbox_start_drain(
    input: NativeUsageAnalyticsOutboxStartInput,
    now_ms: u64,
) -> NativeUsageAnalyticsOutboxStep {
    stats_client::start_drain(input.into(), now_ms).into()
}

#[uniffi::export]
pub fn usage_analytics_outbox_start_drain_json(input_json: String, now_ms: u64) -> String {
    let step = match serde_json::from_str::<NativeUsageAnalyticsOutboxStartInput>(&input_json) {
        Ok(input) => usage_analytics_outbox_start_drain(input, now_ms),
        Err(_) => invalid_usage_analytics_step(),
    };

    serialize_step(&step)
}

#[uniffi::export]
pub fn usage_analytics_outbox_start_delivery_state_change(
    input: NativeUsageAnalyticsOutboxDeliveryStateInput,
) -> NativeUsageAnalyticsOutboxStep {
    stats_client::start_delivery_state_change(input.into()).into()
}

#[uniffi::export]
pub fn usage_analytics_outbox_start_delivery_state_change_json(input_json: String) -> String {
    let step =
        match serde_json::from_str::<NativeUsageAnalyticsOutboxDeliveryStateInput>(&input_json) {
            Ok(input) => usage_analytics_outbox_start_delivery_state_change(input),
            Err(_) => invalid_usage_analytics_step(),
        };

    serialize_step(&step)
}

#[uniffi::export]
pub fn usage_analytics_outbox_resume(
    input: NativeUsageAnalyticsOutboxResumeInput,
    now_ms: u64,
) -> NativeUsageAnalyticsOutboxStep {
    stats_client::resume_step(input.into(), now_ms).into()
}

#[uniffi::export]
pub fn usage_analytics_outbox_resume_json(input_json: String, now_ms: u64) -> String {
    let step = match serde_json::from_str::<NativeUsageAnalyticsOutboxResumeInput>(&input_json) {
        Ok(input) => usage_analytics_outbox_resume(input, now_ms),
        Err(_) => invalid_usage_analytics_step(),
    };

    serialize_step(&step)
}

fn invalid_usage_analytics_step() -> NativeUsageAnalyticsOutboxStep {
    NativeUsageAnalyticsOutboxStep {
        is_complete: true,
        ..NativeUsageAnalyticsOutboxStep::default()
    }
}

fn serialize_step(step: &NativeUsageAnalyticsOutboxStep) -> String {
    serde_json::to_string(step)
        .expect("NativeUsageAnalyticsOutboxStep serialization should not fail")
}
