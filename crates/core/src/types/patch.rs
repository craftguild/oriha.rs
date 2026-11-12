use super::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPatchInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_snapshot: Option<RenderSnapshot>,
    pub input: RenderInput,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPatchOutput {
    pub output: RenderOutput,
    pub snapshot: RenderSnapshot,
    pub strategy: RenderPatchStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<RenderPatchFallbackReason>,
    pub patches: Vec<RenderPatchOperation>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RenderPatchStrategy {
    Incremental,
    ReplaceAll,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RenderPatchFallbackReason {
    MissingSnapshot,
    IncompatibleSnapshot,
    TailChanged,
    OutputTruncated,
    InvalidInput,
    EnginePanic,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "op", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum RenderPatchOperation {
    ReplaceAll {
        html: String,
    },
    ReplaceBlocks {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        remove_stable_ids: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        insert_after_stable_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        insert_before_stable_id: Option<String>,
        html: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenderSnapshot {
    #[serde(default = "render_snapshot_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub settings_fingerprint: String,
    #[serde(default = "default_next_stable_index")]
    pub next_stable_index: u64,
    #[serde(default)]
    pub blocks: Vec<RenderSnapshotBlock>,
    #[serde(default)]
    pub tail_hash: String,
    #[serde(default)]
    pub tail_html_byte_length: u64,
    #[serde(default)]
    pub output_resource_fingerprint: String,
}

impl Default for RenderSnapshot {
    fn default() -> Self {
        Self {
            schema_version: render_snapshot_schema_version(),
            settings_fingerprint: String::new(),
            next_stable_index: default_next_stable_index(),
            blocks: Vec::new(),
            tail_hash: String::new(),
            tail_html_byte_length: 0,
            output_resource_fingerprint: String::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenderSnapshotBlock {
    pub stable_id: String,
    pub kind: String,
    pub source_hash: String,
    #[serde(default)]
    pub html_hash: String,
    pub source_range: SourceRange,
}

pub const fn render_snapshot_schema_version() -> u32 {
    3
}

const fn default_next_stable_index() -> u64 {
    1
}
