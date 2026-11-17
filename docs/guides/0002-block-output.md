# Rust structured documents and rendered blocks

`oriha-core` derives a flat typed document (structured schema version 2) or semantic HTML root fragments (rendered schema version 1) from one retained parse. The structured version-1 tree and its DTOs have been replaced without a compatibility path. Ordinary document HTML, source-aware parsing, source-map `RenderedBlock`, native/JSON, prepared rendering, patching and export contracts retain their behavior. These Rust APIs are not automatically exposed by native bridges.

## Use the API

```rust
use oriha_core::{
    OrihaEngine, RenderInput, BlockOutput, BlockOutputMode,
    BlockOutputStatus, DocumentNodeKind, StructuredOptions,
};

let engine = OrihaEngine::default();
let parsed = engine.parse(RenderInput {
    markdown: "# Hello **Rust**".into(),
    ..Default::default()
});
let BlockOutput::Structured(document) = engine.derive_parsed_blocks_with_options(
    &parsed, BlockOutputMode::Structured,
    StructuredOptions { syntax: true, ..Default::default() },
) else { unreachable!() };
assert_eq!(document.status, BlockOutputStatus::Complete);
let heading = document.nodes.iter()
    .find(|node| node.id == document.root_ids[0]).unwrap();
if let DocumentNodeKind::Heading { level, child_ids, anchor } = &heading.content {
    assert_eq!(*level, 1);
    assert_eq!(anchor, "hello-rust");
    assert_eq!(child_ids.len(), 2);
}
```

`derive_blocks(input, mode)` parses once. The top-level `derive_parsed_blocks` and matching `OrihaEngine` methods reuse the retained source, parsing features, appearance, limits and parse-hook state. The default structured options are all off. `OrihaEngine::derive_parsed_blocks_with_options` prepares selected artifacts from that same parse; its options never change parsing features or structural identities. A different deriving engine supplies renderers and render hooks only for rendered mode. Owned results outlive the source, parsed document and engine.

Run the compiled consumer example with `cargo run -p oriha-core --example derive_blocks --release --locked`. It exercises both representations, typed references/footnotes and serialization with actual engine output.

## Flat structured document

`StructuredBlockOutput` contains `schema_version: 2`, `mode: Structured`, `status`, `root_ids`, `nodes`, `reference_ids`, `footnote_ids`, `fonts` and typed `diagnostics`. JSON uses camelCase. Each entry and ID is non-null. Body roots are ordered; retained definitions are separate forest roots, not ordinary visible body blocks.

Each `DocumentNode` has an opaque document-local `id` and a flattened tagged `DocumentNodeKind`. Actual required payloads live on the corresponding enum variant, without a generic property bag, unrelated optional fields or another per-kind lookup. Existing block IDs are preserved and reserved before assigning collision-free inline, row, cell and definition IDs. IDs are stable across optional preparation within one revision; they are not persistent editor or DOM identities.

| Kind | Typed payload |
| --- | --- |
| `document`, `blockQuote` | Ordered `childIds` |
| `paragraph` | Inline `childIds`, `tight` |
| `heading` | Inline `childIds`, `level`, producer `anchor` |
| `fencedCode` | `source`, nullable decoded `language`, `info`, `closed`, `indent`, optional `highlighting` and `svg` |
| `indentedCode` | Same code metadata and optional `highlighting` |
| `mathBlock` | `source`, `display`, `closed`, optional `svg` |
| `thematicBreak`, `blankLine` | Explicit semantic variant |
| `htmlBlock`, `linkReferenceDefinition` | Retained `source` |
| `list` | Item `childIds`, `ordered`, `loose`, `marker`, normalized decimal-string `start` or null |
| `listItem` | Block `childIds`, `marker`, optional task `checked` |
| `gfmTable` | `alignments` and ordered `rowIds` |
| `tableRow` | `header` and ordered `cellIds` |
| `tableCell` | Inline `childIds` |
| `callout` | Block `childIds`, `calloutKind`, optional custom `title` |
| `foreignHtml` | `source`, `origin`, `trustLevel` |
| `text`, `emoji` | Visible `text`; emoji also has `shortcode` |
| `code`, `math` | Literal `source`; math has optional `svg` |
| `html` | Authored `source`, `tagfilter` |
| `softBreak`, `hardBreak` | Explicit line-break semantic variant |
| `annotation` | Inline `childIds`, `annotationKind`, `annotationId`, `classes`, scoped `data` |
| `emphasis`, `strong`, `strikethrough` | Inline `childIds` |
| `link`, `image` | `destination`, `normalizedDestination`, optional `title`, and label `childIds` or `altIds` |
| `footnoteReference` | `label`, optional `definitionId`, `ordinal`, `anchor`, `definitionAnchor` |
| `referenceDefinition` | Resolved `label`, `destination`, `normalizedDestination`, optional `title` |
| `footnoteDefinition` | `label`, inline `childIds`, `anchor`, `visible`, optional `ordinal`, `backlinkIds` |

The projection exhaustively matches the actual block and inline IR enums without wildcard fallback. Adding a syntax variant fails compilation until its structured projection is explicitly implemented. Behavioral tests cover semantic changes within an existing variant. No Markdown reparse or HTML extraction is involved.

Sources use the actual quote/list/callout normalization context. Inline text is already decoded; image alt content, annotations, foreign/authored HTML, unused definitions and duplicate footnotes remain present. Link-reference syntax is distinct from the retained resolved definition. Footnote resolution preserves last-definition precedence. `visible` preserves the renderer's main-flow definition snapshot; references in image alt or unused definitions do not become visible footnote references. Structural ownership is an ordered forest; semantic definition references do not own or recursively expand their targets.

`document.validate(max_nodes, max_depth, max_bytes)` rejects unsupported versions, incomplete status, duplicate/empty IDs, missing or illegal targets, repeated/cyclic ownership, unreachable nodes, invalid font dependencies and exhausted limits. Consumers validate before walking an untrusted document. Producer derivation validates the graph before returning success.

## Optional syntax and SVG preparation

`StructuredOptions` independently selects `syntax`, `math`, `mermaid` and `plantuml`, all off by default. These options control derivation, not parsing. Enable the normal math parsing feature when parsing math syntax. A Mermaid or PlantUML fence remains a fenced code node whether prepared or not.

Code highlighting reuses the HTML renderer's Rust tokenizer. `CodeHighlighting.lines` holds ordered `CodeLine.tokens`; each `SyntaxToken` has a finite `TokenKind` and literal `text`. Joining each row's token text and then joining rows with LF reconstructs the normalized code source exactly, including Unicode, whitespace, blank lines and a final empty line after a terminal LF. Unknown languages use plain tokens. Viewer colors, gutter policy, language labels and controls are outside this contract.

Math and supported fences carry `SvgContent` on their originating node alongside source and authoring metadata. It contains `markup`, optional intrinsic `width`, `height` and `baseline`, and required `fontIds`. The same engine rendering internals serve HTML and structured consumers. SVG has no HTML wrapper. Consumers retain explicit SVG validation and ID/reference scoping boundaries.

Shared `DocumentFont` entries contain `id`, `family`, truthful `mimeType`, typed `encoding` and `content`. The bundled math font uses `font/otf` and `base64`, and is returned once even for multiple equations. Preserve its [font license and notices](../../THIRD_PARTY_NOTICES.md). Diagrams use the configured body font stack. There is no general CSS/HTML resource escape hatch.

Failed or unsupported math/diagram rendering retains source, omits the unsuccessful SVG and adds a bounded warning with a stable code and owning node ID. No preparation means no preparation-failure warning. Global errors, panics or aggregate-budget exhaustion fail the whole derivation, rather than returning truncated success or disguising an abort as source fallback.

## Rendered HTML remains schema version 1

`RenderedBlockOutput` retains `schema_version: 1`, `mode`, `status`, `blocks`, `assets` and `diagnostics`. `RenderedHtmlBlock` has `id`, typed `kind` and `RenderedBlockContent::{Html, Null}`; `html()` exposes an optional string. JSON exposes `html` and `nullReason`. Empty HTML is a valid fragment; legal null reasons are `blankLine`, `referenceDefinition` and an explicit custom `unsupported` decision.

Rendered entries correspond to roots in IR order and contain each complete subtree. Shared rendering preserves heading de-duplication, links, references and footnote numbering. Visible footnotes append one collision-free generated `footnotes` entry. Joining non-null fragments preserves the existing semantic document body without document-only layout, theme CSS, watermark or article wrapper. Existing [code presentation options](code-block-options.md), responsive images, classes and `data-oriha-*` attributes remain in force.

`assets` retain `RenderAsset` identity, kind, MIME and content semantics, including the deduplicated math font. SVG/CSS assets are text; `Other` is not a universal text/base64 encoding. Ordinary HTML consumers retain their existing resource boundary.

## Hooks, limits and compatibility

Parse hooks run once. Structured derivation projects the retained post-parse IR without render hooks; optional preparation uses isolated engine state. Rendered derivation runs existing before/after block hooks on a working copy. Both modes exclude complete-document finalization hooks. `BlockRenderer::block_emission` defaults to HTML, preserving custom renderers that return an empty string. Ordinary document render APIs retain their finalization and emission behavior.

Check `status` before consuming the output. Incomplete parsing returns empty incomplete output; failed parsing, malformed IR, error diagnostics, global derivation failures or exhausted budgets return empty failed output. The retained source/block/container-depth limits remain enforced. Programmatic tree and inline depths have a 256-level safety bound, with a bounded aggregate graph walk. Recursive inline label parsing retains its separate 64-call bound.

`max_output_bytes` bounds the aggregate serialized UTF-8 document, including node payloads, ID lists, tokens, SVG, fonts, diagnostics and JSON escaping. Early accounting bounds owned construction and reserves font bytes before copying; a counting serializer checks the final aggregate without allocating another JSON document. Oversized diagnostics collapse to a bounded output-limit diagnostic. The minimal failure envelope may exceed an impossibly small configured limit. Custom hooks remain responsible for their internal allocations.

Authored/foreign HTML remains source data and does not authorize execution. HTML APIs continue to obey effective `raw_html` and tag-filter settings. The inherited inline raw-HTML security correction and exact rendered-resource accounting remain covered by regression tests.

Run `python3 scripts/check-block-exhaustiveness.py` to inject actual IR probe variants into a temporary source copy, verify `E0004` independently in every structured and HTML projection, and restore the copy. No probe enters the working tree. [Verification methods](verification.md) describe existing CommonMark, source-map, JSON/native, generation and product gates. [ADR 0005](../adr/0005-flat-structured-document-output.md) owns the flat contract; the earlier [design](../design/block-derivation.md) is retained as historical version-1 design evidence.

`render::builtin_html_anchors(&parsed)` and `render::normalize_link_destination(url)` remain available without HTML preparation. The structured contract carries heading and footnote anchors directly. Application URL policy, semantic markup, image inspection, provider embeds, galleries and search highlighting remain consumer responsibilities.
