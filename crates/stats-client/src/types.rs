use oriha_core::RenderStats;
use serde::{Deserialize, Serialize};

pub const CONTENT_TYPE_HEADER: &str = "Content-Type";
pub const STATS_API_KEY_HEADER: &str = "X-ORIHA-Stats-Key";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct UsageAnalyticsOutboxConfig {
    pub ingest_url: String,
    pub api_key: String,
    pub request_timeout_ms: u64,
    pub max_queued_events: u32,
    pub retention_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageAnalyticsRenderPayloadInput {
    pub platform: String,
    pub app_version: String,
    pub build_version: String,
    pub locale: String,
    pub subscription_plan: String,
    pub stats: RenderStats,
    pub event_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct UsageAnalyticsStoredEnvelope {
    pub id: String,
    pub envelope_json: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SendHttpContext {
    pub url: String,
    pub method: String,
    pub timeout_ms: u64,
    pub headers: Vec<HttpHeader>,
    pub body: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageAnalyticsOutboxStartInput {
    pub config: UsageAnalyticsOutboxConfig,
    pub is_delivery_enabled: bool,
    pub delivery_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageAnalyticsOutboxEnqueueStartInput {
    pub config: UsageAnalyticsOutboxConfig,
    pub render_payload: UsageAnalyticsRenderPayloadInput,
    pub is_delivery_enabled: bool,
    pub delivery_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageAnalyticsOutboxDeliveryStateInput {
    pub is_delivery_enabled: bool,
    pub delivery_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageAnalyticsOutboxResumeInput {
    pub step_token: String,
    pub results: Vec<UsageAnalyticsOutboxCommandResult>,
    pub is_delivery_enabled: bool,
    pub delivery_generation: u64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct UsageAnalyticsOutboxStep {
    pub step_token: Option<String>,
    pub commands: Vec<UsageAnalyticsOutboxCommand>,
    pub next_drain_delay_ms: Option<u64>,
    pub is_complete: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct UsageAnalyticsOutboxCommand {
    pub id: String,
    pub kind: UsageAnalyticsOutboxCommandKind,
    pub envelope_id: Option<String>,
    pub envelope_json: Option<String>,
    pub http_context: Option<SendHttpContext>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum UsageAnalyticsOutboxCommandKind {
    LoadEnvelopes,
    WriteEnvelope,
    DeleteEnvelope,
    PurgeEnvelopes,
    SendHttp,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UsageAnalyticsOutboxCommandResult {
    pub command_id: String,
    pub succeeded: bool,
    pub stored_envelopes: Vec<UsageAnalyticsStoredEnvelope>,
    pub status_code: Option<u16>,
}
