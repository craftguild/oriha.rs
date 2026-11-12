//! Incremental replacement operations and whole-output fallback.
use super::*;

pub(crate) fn build_patch_output(
    previous: Option<&RenderSnapshot>,
    output: RenderOutput,
    snapshot: RenderSnapshot,
    rendered_blocks: Vec<RenderedPatchBlock>,
) -> RenderPatchOutput {
    let Some(previous) = previous else {
        return replace_all_output(output, snapshot, RenderPatchFallbackReason::MissingSnapshot);
    };

    if !snapshot_is_patch_compatible(previous, &snapshot) {
        return replace_all_output(
            output,
            snapshot,
            RenderPatchFallbackReason::IncompatibleSnapshot,
        );
    }

    if previous.tail_hash != snapshot.tail_hash {
        return replace_all_output(output, snapshot, RenderPatchFallbackReason::TailChanged);
    }

    let patches = build_incremental_operations(&previous.blocks, &rendered_blocks);
    if has_unanchored_insert_with_tail(&snapshot, &patches) {
        return replace_all_output(
            output,
            snapshot,
            RenderPatchFallbackReason::IncompatibleSnapshot,
        );
    }

    RenderPatchOutput {
        output,
        snapshot,
        strategy: RenderPatchStrategy::Incremental,
        fallback_reason: None,
        patches,
    }
}

pub(crate) fn replace_all_output(
    output: RenderOutput,
    snapshot: RenderSnapshot,
    reason: RenderPatchFallbackReason,
) -> RenderPatchOutput {
    let html = output.html.clone();
    RenderPatchOutput {
        output,
        snapshot,
        strategy: RenderPatchStrategy::ReplaceAll,
        fallback_reason: Some(reason),
        patches: vec![RenderPatchOperation::ReplaceAll { html }],
    }
}

pub(super) fn has_unanchored_insert_with_tail(
    snapshot: &RenderSnapshot,
    patches: &[RenderPatchOperation],
) -> bool {
    snapshot.tail_html_byte_length > 0
        && patches.iter().any(|patch| match patch {
            RenderPatchOperation::ReplaceBlocks {
                insert_after_stable_id,
                insert_before_stable_id,
                html,
                ..
            } => {
                insert_after_stable_id.is_none()
                    && insert_before_stable_id.is_none()
                    && !html.is_empty()
            }
            RenderPatchOperation::ReplaceAll { .. } => false,
        })
}

pub(super) fn build_incremental_operations(
    previous: &[RenderSnapshotBlock],
    current: &[RenderedPatchBlock],
) -> Vec<RenderPatchOperation> {
    let anchors = lcs_by(
        previous,
        current,
        |block| {
            (
                block.stable_id.clone(),
                block.kind.clone(),
                block.source_hash.clone(),
                block.html_hash.clone(),
            )
        },
        |block| {
            (
                block.stable_id.clone(),
                block.kind.clone(),
                block.source_hash.clone(),
                block.html_hash.clone(),
            )
        },
    );

    let mut operations = Vec::new();
    let mut previous_start = 0usize;
    let mut current_start = 0usize;
    let mut previous_anchor_id = None::<String>;

    for (previous_end, current_end) in anchors
        .iter()
        .copied()
        .chain(std::iter::once((previous.len(), current.len())))
    {
        push_replace_blocks_operation(
            &mut operations,
            &previous[previous_start..previous_end],
            &current[current_start..current_end],
            previous_anchor_id.clone(),
            previous
                .get(previous_end)
                .map(|block| block.stable_id.clone()),
        );

        previous_anchor_id = previous
            .get(previous_end)
            .map(|block| block.stable_id.clone())
            .or(previous_anchor_id);
        previous_start = previous_end.saturating_add(1);
        current_start = current_end.saturating_add(1);
    }

    operations
}

pub(super) fn push_replace_blocks_operation(
    operations: &mut Vec<RenderPatchOperation>,
    previous: &[RenderSnapshotBlock],
    current: &[RenderedPatchBlock],
    insert_after_stable_id: Option<String>,
    insert_before_stable_id: Option<String>,
) {
    if previous.is_empty() && current.is_empty() {
        return;
    }

    operations.push(RenderPatchOperation::ReplaceBlocks {
        remove_stable_ids: previous
            .iter()
            .map(|block| block.stable_id.clone())
            .collect(),
        insert_after_stable_id,
        insert_before_stable_id,
        html: current.iter().map(|block| block.html.as_str()).collect(),
    });
}
