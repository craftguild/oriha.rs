// Typed native telemetry outbox inputs, commands and results.

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeUsageAnalyticsOutboxConfig {
    pub ingest_url: String,
    pub api_key: String,
    pub request_timeout_ms: u64,
    pub max_queued_events: u32,
    pub retention_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeUsageAnalyticsRenderPayloadInput {
    pub platform: String,
    pub app_version: String,
    pub build_version: String,
    pub locale: String,
    pub subscription_plan: String,
    pub stats: RenderStats,
    pub event_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeUsageAnalyticsStoredEnvelope {
    pub id: String,
    pub envelope_json: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeSendHttpContext {
    pub url: String,
    pub method: String,
    pub timeout_ms: u64,
    pub headers: Vec<NativeHttpHeader>,
    pub body: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeHttpHeader {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeUsageAnalyticsOutboxStartInput {
    pub config: NativeUsageAnalyticsOutboxConfig,
    pub is_delivery_enabled: bool,
    pub delivery_generation: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeUsageAnalyticsOutboxEnqueueStartInput {
    pub config: NativeUsageAnalyticsOutboxConfig,
    pub render_payload: NativeUsageAnalyticsRenderPayloadInput,
    pub is_delivery_enabled: bool,
    pub delivery_generation: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeUsageAnalyticsOutboxDeliveryStateInput {
    pub is_delivery_enabled: bool,
    pub delivery_generation: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeUsageAnalyticsOutboxResumeInput {
    pub step_token: String,
    pub results: Vec<NativeUsageAnalyticsOutboxCommandResult>,
    pub is_delivery_enabled: bool,
    pub delivery_generation: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct NativeUsageAnalyticsOutboxStep {
    pub step_token: Option<String>,
    pub commands: Vec<NativeUsageAnalyticsOutboxCommand>,
    pub next_drain_delay_ms: Option<u64>,
    pub is_complete: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeUsageAnalyticsOutboxCommand {
    pub id: String,
    pub kind: NativeUsageAnalyticsOutboxCommandKind,
    pub envelope_id: Option<String>,
    pub envelope_json: Option<String>,
    pub http_context: Option<NativeSendHttpContext>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
#[serde(rename_all = "camelCase")]
pub enum NativeUsageAnalyticsOutboxCommandKind {
    LoadEnvelopes,
    WriteEnvelope,
    DeleteEnvelope,
    PurgeEnvelopes,
    SendHttp,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct NativeUsageAnalyticsOutboxCommandResult {
    pub command_id: String,
    pub succeeded: bool,
    pub stored_envelopes: Vec<NativeUsageAnalyticsStoredEnvelope>,
    pub status_code: Option<u16>,
}
