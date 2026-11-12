//! Bridge/core snapshot and render-patch conversions.
use super::*;

impl From<RenderSnapshot> for oriha_core::RenderSnapshot {
    fn from(value: RenderSnapshot) -> Self {
        Self {
            schema_version: value.schema_version,
            settings_fingerprint: value.settings_fingerprint,
            next_stable_index: value.next_stable_index,
            blocks: value.blocks.into_iter().map(Into::into).collect(),
            tail_hash: value.tail_hash,
            tail_html_byte_length: value.tail_html_byte_length,
            output_resource_fingerprint: value.output_resource_fingerprint,
        }
    }
}

impl From<oriha_core::RenderSnapshot> for RenderSnapshot {
    fn from(value: oriha_core::RenderSnapshot) -> Self {
        Self {
            schema_version: value.schema_version,
            settings_fingerprint: value.settings_fingerprint,
            next_stable_index: value.next_stable_index,
            blocks: value.blocks.into_iter().map(Into::into).collect(),
            tail_hash: value.tail_hash,
            tail_html_byte_length: value.tail_html_byte_length,
            output_resource_fingerprint: value.output_resource_fingerprint,
        }
    }
}

impl From<RenderSnapshotBlock> for oriha_core::RenderSnapshotBlock {
    fn from(value: RenderSnapshotBlock) -> Self {
        Self {
            stable_id: value.stable_id,
            kind: value.kind,
            source_hash: value.source_hash,
            html_hash: value.html_hash,
            source_range: value.source_range.into(),
        }
    }
}

impl From<oriha_core::RenderSnapshotBlock> for RenderSnapshotBlock {
    fn from(value: oriha_core::RenderSnapshotBlock) -> Self {
        Self {
            stable_id: value.stable_id,
            kind: value.kind,
            source_hash: value.source_hash,
            html_hash: value.html_hash,
            source_range: value.source_range.into(),
        }
    }
}

impl From<oriha_core::RenderPatchStrategy> for RenderPatchStrategy {
    fn from(value: oriha_core::RenderPatchStrategy) -> Self {
        match value {
            oriha_core::RenderPatchStrategy::Incremental => Self::Incremental,
            oriha_core::RenderPatchStrategy::ReplaceAll => Self::ReplaceAll,
        }
    }
}

impl From<oriha_core::RenderPatchFallbackReason> for RenderPatchFallbackReason {
    fn from(value: oriha_core::RenderPatchFallbackReason) -> Self {
        match value {
            oriha_core::RenderPatchFallbackReason::MissingSnapshot => Self::MissingSnapshot,
            oriha_core::RenderPatchFallbackReason::IncompatibleSnapshot => {
                Self::IncompatibleSnapshot
            }
            oriha_core::RenderPatchFallbackReason::TailChanged => Self::TailChanged,
            oriha_core::RenderPatchFallbackReason::OutputTruncated => Self::OutputTruncated,
            oriha_core::RenderPatchFallbackReason::InvalidInput => Self::InvalidInput,
            oriha_core::RenderPatchFallbackReason::EnginePanic => Self::EnginePanic,
        }
    }
}

impl From<oriha_core::RenderPatchOperation> for RenderPatchOperation {
    fn from(value: oriha_core::RenderPatchOperation) -> Self {
        match value {
            oriha_core::RenderPatchOperation::ReplaceAll { html } => Self {
                op: RenderPatchOperationKind::ReplaceAll,
                html,
                remove_stable_ids: Vec::new(),
                insert_after_stable_id: None,
                insert_before_stable_id: None,
            },
            oriha_core::RenderPatchOperation::ReplaceBlocks {
                remove_stable_ids,
                insert_after_stable_id,
                insert_before_stable_id,
                html,
            } => Self {
                op: RenderPatchOperationKind::ReplaceBlocks,
                html,
                remove_stable_ids,
                insert_after_stable_id,
                insert_before_stable_id,
            },
        }
    }
}

impl From<oriha_core::RenderPatchOutput> for RenderPatchOutput {
    fn from(value: oriha_core::RenderPatchOutput) -> Self {
        Self {
            output: value.output.into(),
            snapshot: value.snapshot.into(),
            strategy: value.strategy.into(),
            fallback_reason: value.fallback_reason.map(Into::into),
            patches: value.patches.into_iter().map(Into::into).collect(),
        }
    }
}
