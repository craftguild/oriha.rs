//! Telemetry outbox inputs, commands and results.
use super::*;

pub(super) fn records() -> Vec<RecordSpec> {
    vec![
        record(
            "UsageAnalyticsOutboxConfig",
            &[],
            vec![
                field("ingestUrl", primitive("string"), false),
                field("apiKey", primitive("string"), false),
                field("requestTimeoutMs", primitive("number"), false),
                field("maxQueuedEvents", primitive("number"), false),
                field("retentionMs", primitive("number"), false),
            ],
        ),
        record(
            "UsageAnalyticsRenderPayloadInput",
            &[],
            vec![
                field("platform", primitive("string"), false),
                field("appVersion", primitive("string"), false),
                field("buildVersion", primitive("string"), false),
                field("locale", primitive("string"), false),
                field("subscriptionPlan", primitive("string"), false),
                field("stats", named("RenderStats"), false),
                field("eventId", primitive("string"), true),
            ],
        ),
        record(
            "UsageAnalyticsStoredEnvelope",
            &[],
            vec![
                field("id", primitive("string"), false),
                field("envelopeJson", primitive("string"), false),
            ],
        ),
        record(
            "SendHttpContext",
            &[],
            vec![
                field("url", primitive("string"), false),
                field("method", primitive("string"), false),
                field("timeoutMs", primitive("number"), false),
                field("headers", array(named("HttpHeader")), false),
                field("body", primitive("string"), false),
            ],
        ),
        record(
            "HttpHeader",
            &[],
            vec![
                field("name", primitive("string"), false),
                field("value", primitive("string"), false),
            ],
        ),
        record(
            "UsageAnalyticsOutboxStartInput",
            &[],
            vec![
                field("config", named("UsageAnalyticsOutboxConfig"), false),
                field("isDeliveryEnabled", primitive("boolean"), false),
                field("deliveryGeneration", primitive("number"), false),
            ],
        ),
        record(
            "UsageAnalyticsOutboxEnqueueStartInput",
            &[],
            vec![
                field("config", named("UsageAnalyticsOutboxConfig"), false),
                field(
                    "renderPayload",
                    named("UsageAnalyticsRenderPayloadInput"),
                    false,
                ),
                field("isDeliveryEnabled", primitive("boolean"), false),
                field("deliveryGeneration", primitive("number"), false),
            ],
        ),
        record(
            "UsageAnalyticsOutboxDeliveryStateInput",
            &[],
            vec![
                field("isDeliveryEnabled", primitive("boolean"), false),
                field("deliveryGeneration", primitive("number"), false),
            ],
        ),
        record(
            "UsageAnalyticsOutboxResumeInput",
            &[],
            vec![
                field("stepToken", primitive("string"), false),
                field(
                    "results",
                    array(named("UsageAnalyticsOutboxCommandResult")),
                    false,
                ),
                field("isDeliveryEnabled", primitive("boolean"), false),
                field("deliveryGeneration", primitive("number"), false),
            ],
        ),
        record(
            "UsageAnalyticsOutboxStep",
            &[],
            vec![
                field("stepToken", primitive("string"), true),
                field(
                    "commands",
                    array(named("UsageAnalyticsOutboxCommand")),
                    false,
                ),
                field("nextDrainDelayMs", primitive("number"), true),
                field("isComplete", primitive("boolean"), false),
            ],
        ),
        record(
            "UsageAnalyticsOutboxCommand",
            &[],
            vec![
                field("id", primitive("string"), false),
                field("kind", named("UsageAnalyticsOutboxCommandKind"), false),
                field("envelopeId", primitive("string"), true),
                field("envelopeJson", primitive("string"), true),
                field("httpContext", named("SendHttpContext"), true),
            ],
        ),
        record(
            "UsageAnalyticsOutboxCommandResult",
            &[],
            vec![
                field("commandId", primitive("string"), false),
                field("succeeded", primitive("boolean"), false),
                field(
                    "storedEnvelopes",
                    array(named("UsageAnalyticsStoredEnvelope")),
                    false,
                ),
                field("statusCode", primitive("number"), true),
            ],
        ),
    ]
}
