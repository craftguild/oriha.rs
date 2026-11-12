use crate::model::{
    group_key, BucketTotals, CountEntry, DimensionTotals, GroupBy, ReportRow, StoredRecord,
};
use oriha_core::{KindCount, RenderStats};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub(crate) struct ReportAccumulator {
    group_by: GroupBy,
    summary: Aggregate,
    groups: BTreeMap<String, Aggregate>,
    dimensions: DimensionMaps,
}

impl ReportAccumulator {
    pub(crate) fn new(group_by: GroupBy) -> Self {
        Self {
            group_by,
            summary: Aggregate::default(),
            groups: BTreeMap::new(),
            dimensions: DimensionMaps::default(),
        }
    }

    pub(crate) fn record(&mut self, record: &StoredRecord) {
        self.summary.record(record);
        self.dimensions.record(record);
        if self.group_by != GroupBy::All {
            let key = group_key(record, self.group_by);
            self.groups.entry(key).or_default().record(record);
        }
    }

    pub(crate) fn into_report_parts(self) -> (ReportRow, Vec<ReportRow>, DimensionTotals) {
        (
            self.summary.into_row("all".to_string()),
            self.groups
                .into_iter()
                .map(|(key, aggregate)| aggregate.into_row(key))
                .collect(),
            self.dimensions.into_totals(),
        )
    }
}

#[derive(Clone, Debug, Default)]
struct Aggregate {
    renders: u64,
    total_blocks: u64,
    first_received_at: Option<chrono::DateTime<chrono::Utc>>,
    last_received_at: Option<chrono::DateTime<chrono::Utc>>,
    buckets: BucketMaps,
}

impl Aggregate {
    fn record(&mut self, record: &StoredRecord) {
        self.renders += 1;
        self.total_blocks += u64::from(record.payload.stats.total_blocks);
        self.first_received_at = Some(match self.first_received_at {
            Some(current) => current.min(record.received_at),
            None => record.received_at,
        });
        self.last_received_at = Some(match self.last_received_at {
            Some(current) => current.max(record.received_at),
            None => record.received_at,
        });
        self.buckets.record(&record.payload.stats);
    }

    fn into_row(self, key: String) -> ReportRow {
        let issue_count = self.buckets.issues_by_kind.values().sum();
        let locked_feature_count = self.buckets.locked_features_by_kind.values().sum();
        let unsupported_count = self.buckets.unsupported_by_kind.values().sum();
        ReportRow {
            key,
            renders: self.renders,
            total_blocks: self.total_blocks,
            average_blocks_per_render: if self.renders == 0 {
                0.0
            } else {
                self.total_blocks as f64 / self.renders as f64
            },
            first_received_at: self.first_received_at,
            last_received_at: self.last_received_at,
            issue_count,
            locked_feature_count,
            unsupported_count,
            buckets: self.buckets.into_totals(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct BucketMaps {
    blocks_by_kind: BTreeMap<String, u64>,
    headings_by_level: BTreeMap<String, u64>,
    code_blocks_by_language_kind: BTreeMap<String, u64>,
    callouts_by_kind: BTreeMap<String, u64>,
    locked_features_by_kind: BTreeMap<String, u64>,
    unsupported_by_kind: BTreeMap<String, u64>,
    issues_by_kind: BTreeMap<String, u64>,
}

impl BucketMaps {
    fn record(&mut self, stats: &RenderStats) {
        merge_kind_counts(&mut self.blocks_by_kind, &stats.blocks_by_kind);
        merge_kind_counts(&mut self.headings_by_level, &stats.headings_by_level);
        merge_kind_counts(
            &mut self.code_blocks_by_language_kind,
            &stats.code_blocks_by_language_kind,
        );
        merge_kind_counts(&mut self.callouts_by_kind, &stats.callouts_by_kind);
        merge_kind_counts(
            &mut self.locked_features_by_kind,
            &stats.locked_features_by_kind,
        );
        merge_kind_counts(&mut self.unsupported_by_kind, &stats.unsupported_by_kind);
        merge_kind_counts(&mut self.issues_by_kind, &stats.issues_by_kind);
    }

    fn into_totals(self) -> BucketTotals {
        BucketTotals {
            blocks_by_kind: count_entries(self.blocks_by_kind),
            headings_by_level: count_entries(self.headings_by_level),
            code_blocks_by_language_kind: count_entries(self.code_blocks_by_language_kind),
            callouts_by_kind: count_entries(self.callouts_by_kind),
            locked_features_by_kind: count_entries(self.locked_features_by_kind),
            unsupported_by_kind: count_entries(self.unsupported_by_kind),
            issues_by_kind: count_entries(self.issues_by_kind),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct DimensionMaps {
    platforms: BTreeMap<String, u64>,
    app_versions: BTreeMap<String, u64>,
    build_versions: BTreeMap<String, u64>,
    locales: BTreeMap<String, u64>,
    subscription_plans: BTreeMap<String, u64>,
}

impl DimensionMaps {
    fn record(&mut self, record: &StoredRecord) {
        increment_count(&mut self.platforms, &record.payload.platform, 1);
        increment_count(&mut self.app_versions, &record.payload.app_version, 1);
        increment_count(&mut self.build_versions, &record.payload.build_version, 1);
        increment_count(&mut self.locales, &record.payload.locale, 1);
        increment_count(
            &mut self.subscription_plans,
            &record.payload.subscription_plan,
            1,
        );
    }

    fn into_totals(self) -> DimensionTotals {
        DimensionTotals {
            platforms: count_entries(self.platforms),
            app_versions: count_entries(self.app_versions),
            build_versions: count_entries(self.build_versions),
            locales: count_entries(self.locales),
            subscription_plans: count_entries(self.subscription_plans),
        }
    }
}

fn merge_kind_counts(target: &mut BTreeMap<String, u64>, source: &[KindCount]) {
    for entry in source {
        increment_count(target, &entry.kind, u64::from(entry.count));
    }
}

fn increment_count(target: &mut BTreeMap<String, u64>, key: &str, count: u64) {
    *target.entry(key.to_string()).or_default() += count;
}

fn count_entries(counts: BTreeMap<String, u64>) -> Vec<CountEntry> {
    let mut entries = counts
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(kind, count)| CountEntry { kind, count })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.kind.cmp(&right.kind))
    });
    entries
}
