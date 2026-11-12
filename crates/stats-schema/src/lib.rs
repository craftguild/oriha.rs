use oriha_core::{
    is_valid_render_stats_kind, render_stats_schema_version, sort_render_stats_kind_counts,
    KindCount, RenderStats, RenderStatsBucket,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use uuid::{Uuid, Variant, Version};

pub const USAGE_ANALYTICS_EVENT: &str = "markdown_render_complete";
pub const USAGE_ANALYTICS_SCHEMA_VERSION: u32 = 1;
pub const MAX_RENDER_STATS_COUNT: u32 = 1_000_000;

const MAX_LOCALE_LEN: usize = 32;
const MAX_APP_VERSION_LEN: usize = 64;
const MAX_BUILD_VERSION_LEN: usize = 64;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageAnalyticsPayload {
    pub event: String,
    pub schema_version: u32,
    pub platform: String,
    pub app_version: String,
    pub build_version: String,
    pub locale: String,
    pub subscription_plan: String,
    #[serde(default)]
    pub event_id: Option<String>,
    pub stats: UsageAnalyticsRenderStats,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageAnalyticsRenderStats {
    pub schema_version: u32,
    pub total_blocks: u32,
    pub blocks_by_kind: Vec<UsageAnalyticsKindCount>,
    pub headings_by_level: Vec<UsageAnalyticsKindCount>,
    pub code_blocks_by_language_kind: Vec<UsageAnalyticsKindCount>,
    pub callouts_by_kind: Vec<UsageAnalyticsKindCount>,
    pub locked_features_by_kind: Vec<UsageAnalyticsKindCount>,
    pub unsupported_by_kind: Vec<UsageAnalyticsKindCount>,
    pub issues_by_kind: Vec<UsageAnalyticsKindCount>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageAnalyticsKindCount {
    pub kind: String,
    pub count: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidatedUsageAnalyticsPayload {
    pub event: String,
    pub schema_version: u32,
    pub platform: String,
    pub app_version: String,
    pub build_version: String,
    pub locale: String,
    pub subscription_plan: String,
    #[serde(skip_serializing)]
    pub event_id: Option<Uuid>,
    pub stats: RenderStats,
}

impl ValidatedUsageAnalyticsPayload {
    pub fn try_from_payload(payload: UsageAnalyticsPayload) -> Result<Self, ValidationError> {
        validate_payload(payload)
    }

    pub fn canonical_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    pub fn hmac_digest_input(&self) -> Result<Vec<u8>, serde_json::Error> {
        self.canonical_json_bytes()
    }
}

pub fn validate_payload(
    payload: UsageAnalyticsPayload,
) -> Result<ValidatedUsageAnalyticsPayload, ValidationError> {
    validate_exact_str("event", &payload.event, USAGE_ANALYTICS_EVENT)?;
    validate_schema_version(
        "schemaVersion",
        payload.schema_version,
        USAGE_ANALYTICS_SCHEMA_VERSION,
    )?;
    validate_schema_version(
        "stats.schemaVersion",
        payload.stats.schema_version,
        render_stats_schema_version(),
    )?;
    validate_allowed(
        "platform",
        &payload.platform,
        &["macOS", "iOS", "Android", "Windows", "Web"],
    )?;
    validate_allowed(
        "subscriptionPlan",
        &payload.subscription_plan,
        &["free", "premium"],
    )?;
    validate_len("locale", &payload.locale, MAX_LOCALE_LEN)?;
    validate_len("appVersion", &payload.app_version, MAX_APP_VERSION_LEN)?;
    validate_len(
        "buildVersion",
        &payload.build_version,
        MAX_BUILD_VERSION_LEN,
    )?;
    let event_id = payload
        .event_id
        .as_deref()
        .map(validate_event_id)
        .transpose()?;

    if payload.stats.total_blocks > MAX_RENDER_STATS_COUNT {
        return Err(ValidationError::CountTooLarge {
            field: "stats.totalBlocks",
            max: MAX_RENDER_STATS_COUNT,
            actual: payload.stats.total_blocks,
        });
    }

    let blocks_by_kind = validate_kind_counts(
        RenderStatsBucket::BlocksByKind,
        "stats.blocksByKind",
        payload.stats.blocks_by_kind,
    )?;
    let blocks_by_kind_total = blocks_by_kind
        .iter()
        .map(|entry| u64::from(entry.count))
        .sum::<u64>();
    if blocks_by_kind_total != u64::from(payload.stats.total_blocks) {
        return Err(ValidationError::InvalidBlockTotal {
            total_blocks: payload.stats.total_blocks,
            blocks_by_kind_total,
        });
    }

    let stats = RenderStats {
        schema_version: render_stats_schema_version(),
        total_blocks: payload.stats.total_blocks,
        blocks_by_kind,
        headings_by_level: validate_kind_counts(
            RenderStatsBucket::HeadingsByLevel,
            "stats.headingsByLevel",
            payload.stats.headings_by_level,
        )?,
        code_blocks_by_language_kind: validate_kind_counts(
            RenderStatsBucket::CodeBlocksByLanguageKind,
            "stats.codeBlocksByLanguageKind",
            payload.stats.code_blocks_by_language_kind,
        )?,
        callouts_by_kind: validate_kind_counts(
            RenderStatsBucket::CalloutsByKind,
            "stats.calloutsByKind",
            payload.stats.callouts_by_kind,
        )?,
        locked_features_by_kind: validate_kind_counts(
            RenderStatsBucket::LockedFeaturesByKind,
            "stats.lockedFeaturesByKind",
            payload.stats.locked_features_by_kind,
        )?,
        unsupported_by_kind: validate_kind_counts(
            RenderStatsBucket::UnsupportedByKind,
            "stats.unsupportedByKind",
            payload.stats.unsupported_by_kind,
        )?,
        issues_by_kind: validate_kind_counts(
            RenderStatsBucket::IssuesByKind,
            "stats.issuesByKind",
            payload.stats.issues_by_kind,
        )?,
    };

    Ok(ValidatedUsageAnalyticsPayload {
        event: USAGE_ANALYTICS_EVENT.to_string(),
        schema_version: USAGE_ANALYTICS_SCHEMA_VERSION,
        platform: payload.platform,
        app_version: payload.app_version,
        build_version: payload.build_version,
        locale: payload.locale,
        subscription_plan: payload.subscription_plan,
        event_id,
        stats,
    })
}

pub fn validate_json_bytes(
    bytes: &[u8],
) -> Result<ValidatedUsageAnalyticsPayload, PayloadDecodeError> {
    let payload = serde_json::from_slice(bytes).map_err(PayloadDecodeError::Json)?;
    validate_payload(payload).map_err(PayloadDecodeError::Validation)
}

fn validate_kind_counts(
    bucket: RenderStatsBucket,
    field: &'static str,
    counts: Vec<UsageAnalyticsKindCount>,
) -> Result<Vec<KindCount>, ValidationError> {
    let mut merged = BTreeMap::<String, u32>::new();

    for entry in counts {
        if !is_valid_render_stats_kind(bucket, &entry.kind) {
            return Err(ValidationError::UnknownKind {
                field,
                kind: entry.kind,
            });
        }
        if entry.count == 0 {
            return Err(ValidationError::ZeroCount {
                field,
                kind: entry.kind,
            });
        }

        let next = merged
            .get(&entry.kind)
            .copied()
            .unwrap_or(0)
            .checked_add(entry.count)
            .ok_or(ValidationError::CountTooLarge {
                field,
                max: MAX_RENDER_STATS_COUNT,
                actual: u32::MAX,
            })?;

        if next > MAX_RENDER_STATS_COUNT {
            return Err(ValidationError::CountTooLarge {
                field,
                max: MAX_RENDER_STATS_COUNT,
                actual: next,
            });
        }

        merged.insert(entry.kind, next);
    }

    let mut canonical = merged
        .into_iter()
        .map(|(kind, count)| KindCount { kind, count })
        .collect::<Vec<_>>();
    sort_render_stats_kind_counts(bucket, &mut canonical);
    Ok(canonical)
}

fn validate_exact_str(
    field: &'static str,
    actual: &str,
    expected: &'static str,
) -> Result<(), ValidationError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ValidationError::InvalidValue {
            field,
            value: actual.to_string(),
        })
    }
}

fn validate_schema_version(
    field: &'static str,
    actual: u32,
    expected: u32,
) -> Result<(), ValidationError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ValidationError::InvalidSchemaVersion {
            field,
            expected,
            actual,
        })
    }
}

fn validate_allowed(
    field: &'static str,
    actual: &str,
    allowed: &'static [&'static str],
) -> Result<(), ValidationError> {
    if allowed.contains(&actual) {
        Ok(())
    } else {
        Err(ValidationError::InvalidValue {
            field,
            value: actual.to_string(),
        })
    }
}

fn validate_len(field: &'static str, actual: &str, max: usize) -> Result<(), ValidationError> {
    if actual.len() <= max {
        Ok(())
    } else {
        Err(ValidationError::StringTooLong {
            field,
            max,
            actual: actual.len(),
        })
    }
}

fn validate_event_id(actual: &str) -> Result<Uuid, ValidationError> {
    let uuid = Uuid::parse_str(actual).map_err(|_| ValidationError::InvalidEventId {
        value: actual.to_string(),
    })?;
    if uuid.get_version() == Some(Version::Random) && uuid.get_variant() == Variant::RFC4122 {
        Ok(uuid)
    } else {
        Err(ValidationError::InvalidEventId {
            value: actual.to_string(),
        })
    }
}

#[derive(Debug)]
pub enum PayloadDecodeError {
    Json(serde_json::Error),
    Validation(ValidationError),
}

impl fmt::Display for PayloadDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(err) => write!(f, "invalid JSON payload: {err}"),
            Self::Validation(err) => write!(f, "invalid usage analytics payload: {err}"),
        }
    }
}

impl Error for PayloadDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(err) => Some(err),
            Self::Validation(err) => Some(err),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationError {
    InvalidValue {
        field: &'static str,
        value: String,
    },
    InvalidSchemaVersion {
        field: &'static str,
        expected: u32,
        actual: u32,
    },
    StringTooLong {
        field: &'static str,
        max: usize,
        actual: usize,
    },
    UnknownKind {
        field: &'static str,
        kind: String,
    },
    ZeroCount {
        field: &'static str,
        kind: String,
    },
    CountTooLarge {
        field: &'static str,
        max: u32,
        actual: u32,
    },
    InvalidBlockTotal {
        total_blocks: u32,
        blocks_by_kind_total: u64,
    },
    InvalidEventId {
        value: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue { field, value } => {
                write!(f, "{field} has unsupported value {value:?}")
            }
            Self::InvalidSchemaVersion {
                field,
                expected,
                actual,
            } => write!(f, "{field} must be schema version {expected}, got {actual}"),
            Self::StringTooLong { field, max, actual } => {
                write!(f, "{field} length must be <= {max}, got {actual}")
            }
            Self::UnknownKind { field, kind } => {
                write!(f, "{field} contains unknown kind {kind:?}")
            }
            Self::ZeroCount { field, kind } => {
                write!(f, "{field} contains zero count for kind {kind:?}")
            }
            Self::CountTooLarge { field, max, actual } => {
                write!(f, "{field} count must be <= {max}, got {actual}")
            }
            Self::InvalidBlockTotal {
                total_blocks,
                blocks_by_kind_total,
            } => write!(
                f,
                "stats.totalBlocks must equal sum(stats.blocksByKind.count): {total_blocks} != {blocks_by_kind_total}"
            ),
            Self::InvalidEventId { value } => {
                write!(f, "eventId must be a UUID v4 value, got {value:?}")
            }
        }
    }
}

impl Error for ValidationError {}
