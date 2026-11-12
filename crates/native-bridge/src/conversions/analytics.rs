//! Native bridge and telemetry client command/result conversions.
use super::*;

impl From<NativeUsageAnalyticsOutboxConfig> for stats_client::UsageAnalyticsOutboxConfig {
    fn from(value: NativeUsageAnalyticsOutboxConfig) -> Self {
        Self {
            ingest_url: value.ingest_url,
            api_key: value.api_key,
            request_timeout_ms: value.request_timeout_ms,
            max_queued_events: value.max_queued_events,
            retention_ms: value.retention_ms,
        }
    }
}

impl From<NativeUsageAnalyticsRenderPayloadInput>
    for stats_client::UsageAnalyticsRenderPayloadInput
{
    fn from(value: NativeUsageAnalyticsRenderPayloadInput) -> Self {
        Self {
            platform: value.platform,
            app_version: value.app_version,
            build_version: value.build_version,
            locale: value.locale,
            subscription_plan: value.subscription_plan,
            stats: value.stats.into(),
            event_id: value.event_id,
        }
    }
}

impl From<NativeUsageAnalyticsStoredEnvelope> for stats_client::UsageAnalyticsStoredEnvelope {
    fn from(value: NativeUsageAnalyticsStoredEnvelope) -> Self {
        Self {
            id: value.id,
            envelope_json: value.envelope_json,
        }
    }
}

impl From<stats_client::SendHttpContext> for NativeSendHttpContext {
    fn from(value: stats_client::SendHttpContext) -> Self {
        Self {
            url: value.url,
            method: value.method,
            timeout_ms: value.timeout_ms,
            headers: value.headers.into_iter().map(Into::into).collect(),
            body: value.body,
        }
    }
}

impl From<NativeSendHttpContext> for stats_client::SendHttpContext {
    fn from(value: NativeSendHttpContext) -> Self {
        Self {
            url: value.url,
            method: value.method,
            timeout_ms: value.timeout_ms,
            headers: value.headers.into_iter().map(Into::into).collect(),
            body: value.body,
        }
    }
}

impl From<stats_client::HttpHeader> for NativeHttpHeader {
    fn from(value: stats_client::HttpHeader) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<NativeHttpHeader> for stats_client::HttpHeader {
    fn from(value: NativeHttpHeader) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

impl From<NativeUsageAnalyticsOutboxStartInput> for stats_client::UsageAnalyticsOutboxStartInput {
    fn from(value: NativeUsageAnalyticsOutboxStartInput) -> Self {
        Self {
            config: value.config.into(),
            is_delivery_enabled: value.is_delivery_enabled,
            delivery_generation: value.delivery_generation,
        }
    }
}

impl From<NativeUsageAnalyticsOutboxEnqueueStartInput>
    for stats_client::UsageAnalyticsOutboxEnqueueStartInput
{
    fn from(value: NativeUsageAnalyticsOutboxEnqueueStartInput) -> Self {
        Self {
            config: value.config.into(),
            render_payload: value.render_payload.into(),
            is_delivery_enabled: value.is_delivery_enabled,
            delivery_generation: value.delivery_generation,
        }
    }
}

impl From<NativeUsageAnalyticsOutboxDeliveryStateInput>
    for stats_client::UsageAnalyticsOutboxDeliveryStateInput
{
    fn from(value: NativeUsageAnalyticsOutboxDeliveryStateInput) -> Self {
        Self {
            is_delivery_enabled: value.is_delivery_enabled,
            delivery_generation: value.delivery_generation,
        }
    }
}

impl From<NativeUsageAnalyticsOutboxResumeInput> for stats_client::UsageAnalyticsOutboxResumeInput {
    fn from(value: NativeUsageAnalyticsOutboxResumeInput) -> Self {
        Self {
            step_token: value.step_token,
            results: value.results.into_iter().map(Into::into).collect(),
            is_delivery_enabled: value.is_delivery_enabled,
            delivery_generation: value.delivery_generation,
        }
    }
}

impl From<stats_client::UsageAnalyticsOutboxStep> for NativeUsageAnalyticsOutboxStep {
    fn from(value: stats_client::UsageAnalyticsOutboxStep) -> Self {
        Self {
            step_token: value.step_token,
            commands: value.commands.into_iter().map(Into::into).collect(),
            next_drain_delay_ms: value.next_drain_delay_ms,
            is_complete: value.is_complete,
        }
    }
}

impl From<stats_client::UsageAnalyticsOutboxCommand> for NativeUsageAnalyticsOutboxCommand {
    fn from(value: stats_client::UsageAnalyticsOutboxCommand) -> Self {
        Self {
            id: value.id,
            kind: value.kind.into(),
            envelope_id: value.envelope_id,
            envelope_json: value.envelope_json,
            http_context: value.http_context.map(Into::into),
        }
    }
}

impl From<stats_client::UsageAnalyticsOutboxCommandKind> for NativeUsageAnalyticsOutboxCommandKind {
    fn from(value: stats_client::UsageAnalyticsOutboxCommandKind) -> Self {
        match value {
            stats_client::UsageAnalyticsOutboxCommandKind::LoadEnvelopes => Self::LoadEnvelopes,
            stats_client::UsageAnalyticsOutboxCommandKind::WriteEnvelope => Self::WriteEnvelope,
            stats_client::UsageAnalyticsOutboxCommandKind::DeleteEnvelope => Self::DeleteEnvelope,
            stats_client::UsageAnalyticsOutboxCommandKind::PurgeEnvelopes => Self::PurgeEnvelopes,
            stats_client::UsageAnalyticsOutboxCommandKind::SendHttp => Self::SendHttp,
        }
    }
}

impl From<NativeUsageAnalyticsOutboxCommandResult>
    for stats_client::UsageAnalyticsOutboxCommandResult
{
    fn from(value: NativeUsageAnalyticsOutboxCommandResult) -> Self {
        Self {
            command_id: value.command_id,
            succeeded: value.succeeded,
            stored_envelopes: value.stored_envelopes.into_iter().map(Into::into).collect(),
            status_code: value.status_code,
        }
    }
}
