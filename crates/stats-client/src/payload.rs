use crate::types::UsageAnalyticsRenderPayloadInput;
use oriha_core::{KindCount, RenderStats};
use oriha_stats_schema::{
    validate_json_bytes, UsageAnalyticsKindCount, UsageAnalyticsPayload, UsageAnalyticsRenderStats,
    USAGE_ANALYTICS_EVENT, USAGE_ANALYTICS_SCHEMA_VERSION,
};
use uuid::Uuid;

pub(crate) fn encode_render_payload(input: UsageAnalyticsRenderPayloadInput) -> Option<String> {
    let event_id = input.event_id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let payload = UsageAnalyticsPayload {
        event: USAGE_ANALYTICS_EVENT.to_string(),
        schema_version: USAGE_ANALYTICS_SCHEMA_VERSION,
        platform: input.platform,
        app_version: input.app_version,
        build_version: input.build_version,
        locale: input.locale,
        subscription_plan: input.subscription_plan,
        event_id: Some(event_id.to_lowercase()),
        stats: render_stats_to_payload_stats(input.stats),
    };
    validate_and_encode_payload(payload)
}

fn validate_and_encode_payload(payload: UsageAnalyticsPayload) -> Option<String> {
    let bytes = serde_json::to_vec(&payload).ok()?;
    validate_json_bytes(&bytes).ok()?;
    String::from_utf8(bytes).ok()
}

fn render_stats_to_payload_stats(stats: RenderStats) -> UsageAnalyticsRenderStats {
    UsageAnalyticsRenderStats {
        schema_version: stats.schema_version,
        total_blocks: stats.total_blocks,
        blocks_by_kind: kind_counts(stats.blocks_by_kind),
        headings_by_level: kind_counts(stats.headings_by_level),
        code_blocks_by_language_kind: kind_counts(stats.code_blocks_by_language_kind),
        callouts_by_kind: kind_counts(stats.callouts_by_kind),
        locked_features_by_kind: kind_counts(stats.locked_features_by_kind),
        unsupported_by_kind: kind_counts(stats.unsupported_by_kind),
        issues_by_kind: kind_counts(stats.issues_by_kind),
    }
}

fn kind_counts(counts: Vec<KindCount>) -> Vec<UsageAnalyticsKindCount> {
    counts
        .into_iter()
        .map(|entry| UsageAnalyticsKindCount {
            kind: entry.kind,
            count: entry.count,
        })
        .collect()
}
