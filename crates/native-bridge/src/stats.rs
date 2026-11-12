use crate::types::{KindCount, RenderStats};

pub(crate) fn merge_render_stats(target: &mut RenderStats, source: RenderStats) {
    target.total_blocks += source.total_blocks;
    merge_kind_counts(&mut target.blocks_by_kind, source.blocks_by_kind);
    merge_kind_counts(&mut target.headings_by_level, source.headings_by_level);
    merge_kind_counts(
        &mut target.code_blocks_by_language_kind,
        source.code_blocks_by_language_kind,
    );
    merge_kind_counts(&mut target.callouts_by_kind, source.callouts_by_kind);
    merge_kind_counts(
        &mut target.locked_features_by_kind,
        source.locked_features_by_kind,
    );
    merge_kind_counts(&mut target.unsupported_by_kind, source.unsupported_by_kind);
    merge_kind_counts(&mut target.issues_by_kind, source.issues_by_kind);
}

fn merge_kind_counts(target: &mut Vec<KindCount>, source: Vec<KindCount>) {
    for entry in source {
        increment_kind_count(target, &entry.kind, entry.count);
    }
}

pub(crate) fn increment_kind_count(counts: &mut Vec<KindCount>, kind: &str, count: u32) {
    if count == 0 {
        return;
    }

    if let Some(entry) = counts.iter_mut().find(|entry| entry.kind == kind) {
        entry.count += count;
    } else {
        counts.push(KindCount {
            kind: kind.to_string(),
            count,
        });
    }
}
