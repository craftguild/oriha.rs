//! Transient render-patch model and candidate collection.
mod stable_ids;
pub(crate) use stable_ids::assign_stable_ids;
#[allow(unused_imports)]
use stable_ids::*;
mod operations;
#[allow(unused_imports)]
use operations::*;
pub(crate) use operations::{build_patch_output, replace_all_output};
mod snapshots;
#[allow(unused_imports)]
use snapshots::*;
pub(crate) use snapshots::{
    build_snapshot, empty_snapshot, output_resource_fingerprint, render_settings_fingerprint,
    snapshot_blocks_from_rendered, snapshot_settings_are_compatible,
};

use std::collections::{BTreeMap, BTreeSet};

use crate::ir::{BlockKind, BlockNode};
use crate::source::SourceDocument;
use crate::types::{
    AssetKind, RenderInput, RenderOutput, RenderPatchFallbackReason, RenderPatchOperation,
    RenderPatchOutput, RenderPatchStrategy, RenderSnapshot, RenderSnapshotBlock, SourceRange,
};

#[derive(Clone, Debug)]
pub(crate) struct PatchCandidate {
    pub block_id: String,
    pub kind: String,
    pub source_hash: String,
    pub source_range: SourceRange,
}

#[derive(Clone, Debug)]
pub(crate) struct AssignedPatchBlock {
    pub stable_id: String,
    pub kind: String,
    pub source_hash: String,
    pub source_range: SourceRange,
}

#[derive(Clone, Debug)]
pub(crate) struct PatchAssignment {
    pub by_block_id: BTreeMap<String, AssignedPatchBlock>,
    pub next_stable_index: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct RenderedPatchBlock {
    pub stable_id: String,
    pub kind: String,
    pub source_hash: String,
    pub html_hash: String,
    pub source_range: SourceRange,
    pub html: String,
}

pub(crate) fn collect_patch_candidates(
    blocks: &[BlockNode],
    source: &SourceDocument,
) -> Vec<PatchCandidate> {
    blocks
        .iter()
        .filter(|block| is_patchable_block(block))
        .map(|block| PatchCandidate {
            block_id: block.id.clone(),
            kind: patch_block_kind(&block.kind).to_string(),
            source_hash: stable_hash_hex(source.source_span(block.span)),
            source_range: source.to_source_range(block.span),
        })
        .collect()
}

pub(crate) fn html_hash(value: &str) -> String {
    stable_hash_hex(value)
}
