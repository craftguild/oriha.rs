//! Snapshot construction, settings compatibility and resource fingerprints.
use super::*;

pub(crate) fn render_settings_fingerprint(input: &RenderInput) -> String {
    let mut settings = input.clone();
    settings.markdown.clear();
    let serialized =
        serde_json::to_string(&settings).expect("RenderInput settings should serialize");
    stable_hash_hex(&serialized)
}

pub(crate) fn snapshot_settings_are_compatible(
    snapshot: &RenderSnapshot,
    settings_fingerprint: &str,
) -> bool {
    snapshot.schema_version == crate::types::render_snapshot_schema_version()
        && snapshot.settings_fingerprint == settings_fingerprint
}

pub(super) fn snapshot_is_patch_compatible(
    previous: &RenderSnapshot,
    current: &RenderSnapshot,
) -> bool {
    snapshot_settings_are_compatible(previous, &current.settings_fingerprint)
        && previous.output_resource_fingerprint == current.output_resource_fingerprint
}

pub(crate) fn build_snapshot(
    settings_fingerprint: String,
    blocks: Vec<RenderSnapshotBlock>,
    next_stable_index: u64,
    tail_html: &str,
) -> RenderSnapshot {
    RenderSnapshot {
        schema_version: crate::types::render_snapshot_schema_version(),
        settings_fingerprint,
        next_stable_index,
        blocks,
        tail_hash: stable_hash_hex(tail_html),
        tail_html_byte_length: tail_html.len() as u64,
        output_resource_fingerprint: String::new(),
    }
}

pub(crate) fn output_resource_fingerprint(output: &RenderOutput) -> String {
    let mut encoded = String::new();
    match output.css.as_deref() {
        Some(css) => push_fingerprint_field(&mut encoded, "css", css),
        None => encoded.push_str("css:none;"),
    }
    encoded.push_str("assets:");
    encoded.push_str(&output.assets.len().to_string());
    encoded.push(';');
    for asset in &output.assets {
        push_fingerprint_field(&mut encoded, "asset.id", &asset.id);
        encoded.push_str("asset.kind:");
        encoded.push_str(asset_kind_fingerprint_value(&asset.kind));
        encoded.push(';');
        push_fingerprint_field(&mut encoded, "asset.mime", &asset.mime_type);
        push_fingerprint_field(&mut encoded, "asset.content", &asset.content);
    }
    stable_hash_hex(&encoded)
}

pub(crate) fn snapshot_blocks_from_rendered(
    rendered_blocks: &[RenderedPatchBlock],
) -> Vec<RenderSnapshotBlock> {
    rendered_blocks
        .iter()
        .map(|block| RenderSnapshotBlock {
            stable_id: block.stable_id.clone(),
            kind: block.kind.clone(),
            source_hash: block.source_hash.clone(),
            html_hash: block.html_hash.clone(),
            source_range: block.source_range,
        })
        .collect()
}

pub(crate) fn empty_snapshot(settings_fingerprint: String) -> RenderSnapshot {
    RenderSnapshot {
        settings_fingerprint,
        ..RenderSnapshot::default()
    }
}

pub(super) fn push_fingerprint_field(encoded: &mut String, label: &str, value: &str) {
    encoded.push_str(label);
    encoded.push(':');
    encoded.push_str(&value.len().to_string());
    encoded.push(':');
    encoded.push_str(&stable_hash_hex(value));
    encoded.push(';');
}

pub(super) fn asset_kind_fingerprint_value(kind: &AssetKind) -> &'static str {
    match kind {
        AssetKind::Svg => "svg",
        AssetKind::Css => "css",
        AssetKind::Other => "other",
    }
}

pub(super) fn is_patchable_block(block: &BlockNode) -> bool {
    !matches!(
        block.kind,
        BlockKind::BlankLine | BlockKind::LinkReferenceDefinition
    )
}

pub(super) fn patch_block_kind(kind: &BlockKind) -> &'static str {
    match kind {
        BlockKind::Document => "document",
        BlockKind::Paragraph(_) => "paragraph",
        BlockKind::Heading(_) => "heading",
        BlockKind::FencedCode(_) => "codeBlock",
        BlockKind::IndentedCode(_) => "indentedCode",
        BlockKind::MathBlock(_) => "mathBlock",
        BlockKind::ThematicBreak => "thematicBreak",
        BlockKind::HtmlBlock => "htmlBlock",
        BlockKind::LinkReferenceDefinition => "linkReferenceDefinition",
        BlockKind::BlockQuote => "blockQuote",
        BlockKind::List(_) => "list",
        BlockKind::ListItem(_) => "listItem",
        BlockKind::GfmTable(_) => "table",
        BlockKind::BlankLine => "blankLine",
        BlockKind::Callout(_) => "callout",
        BlockKind::ForeignHtml(_) => "foreignHtml",
    }
}
