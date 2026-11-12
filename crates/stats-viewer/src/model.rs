use chrono::{DateTime, Timelike, Utc};
use oriha_core::RenderStats;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GroupBy {
    All,
    Hour,
    Day,
    Platform,
    AppVersion,
    BuildVersion,
    Locale,
    SubscriptionPlan,
}

impl GroupBy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Platform => "platform",
            Self::AppVersion => "app-version",
            Self::BuildVersion => "build-version",
            Self::Locale => "locale",
            Self::SubscriptionPlan => "subscription-plan",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsReport {
    pub source_file_count: usize,
    pub source_files: Vec<String>,
    pub skipped_invalid_line_count: u64,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub group_by: GroupBy,
    pub summary: ReportRow,
    pub groups: Vec<ReportRow>,
    pub dimensions: DimensionTotals,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportRow {
    pub key: String,
    pub renders: u64,
    pub total_blocks: u64,
    pub average_blocks_per_render: f64,
    pub first_received_at: Option<DateTime<Utc>>,
    pub last_received_at: Option<DateTime<Utc>>,
    pub issue_count: u64,
    pub locked_feature_count: u64,
    pub unsupported_count: u64,
    pub buckets: BucketTotals,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketTotals {
    pub blocks_by_kind: Vec<CountEntry>,
    pub headings_by_level: Vec<CountEntry>,
    pub code_blocks_by_language_kind: Vec<CountEntry>,
    pub callouts_by_kind: Vec<CountEntry>,
    pub locked_features_by_kind: Vec<CountEntry>,
    pub unsupported_by_kind: Vec<CountEntry>,
    pub issues_by_kind: Vec<CountEntry>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DimensionTotals {
    pub platforms: Vec<CountEntry>,
    pub app_versions: Vec<CountEntry>,
    pub build_versions: Vec<CountEntry>,
    pub locales: Vec<CountEntry>,
    pub subscription_plans: Vec<CountEntry>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CountEntry {
    pub kind: String,
    pub count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StoredRecord {
    pub(crate) received_at: DateTime<Utc>,
    pub(crate) payload: StoredPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StoredPayload {
    pub(crate) platform: String,
    pub(crate) app_version: String,
    pub(crate) build_version: String,
    pub(crate) locale: String,
    pub(crate) subscription_plan: String,
    pub(crate) stats: RenderStats,
}

pub(crate) fn group_key(record: &StoredRecord, group_by: GroupBy) -> String {
    match group_by {
        GroupBy::All => "all".to_string(),
        GroupBy::Hour => format!(
            "{} {:02}:00Z",
            record.received_at.format("%Y-%m-%d"),
            record.received_at.hour()
        ),
        GroupBy::Day => record.received_at.format("%Y-%m-%d").to_string(),
        GroupBy::Platform => record.payload.platform.clone(),
        GroupBy::AppVersion => record.payload.app_version.clone(),
        GroupBy::BuildVersion => record.payload.build_version.clone(),
        GroupBy::Locale => record.payload.locale.clone(),
        GroupBy::SubscriptionPlan => record.payload.subscription_plan.clone(),
    }
}
