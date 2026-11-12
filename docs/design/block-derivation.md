# Structured and Rendered Block Derivation

- Status: accepted contract; implementation tracked separately.
- Date: 2026-11-01.
- Baseline: oriha-core 0.2.5, repository revision `5307c0ca5d3852e2eb538b594abdfadbd06a1ef6`.
- Decision: [ADR-0002](../adr/0002-structured-and-rendered-block-output.md).
- Verification: [Verification methods](../guides/verification.md).

## Purpose and Ownership

Expose two block representations from the engine's existing parsed document.
`structured` lets a consumer construct its own presentation. `rendered` lets a
consumer display semantic HTML without interpreting Markdown or the IR.

The producer owns parsing, block boundaries, rich text, semantic conversion,
document references, diagnostics and explicit non-rendering decisions. Consumers
own delivery policy, filtering, CSS and application routes. The CMS will select
rendered output and remove entries whose HTML is null.

This document specifies a Rust API and a serializable data contract. It does not
add native, Expo, WASM or GraphQL bridges, an editor, persisted block identities,
or an application-specific endpoint. Implementation and adoption need separate
work. Typed blocks and rich text follow oriha-core's supported IR semantics
and the complete mappings below.

## Existing Boundary

[ADR-0001](../adr/0001-source-aware-parsing-and-annotations.md) already provides an
owned `ParsedDocument`, parse-only access, source-aware IR, annotations and
`render_parsed`. [The IR](../../crates/core/src/ir.rs) contains block and inline
enums, nested blocks, reference definitions and footnotes. Some values, such as
code bodies, still reference retained source or normalized container source.

The current `RenderOutput.html` is document HTML. Its existing `RenderedBlock`
type is source-map metadata, not a block containing HTML. Do not repurpose that
type or change its serialized contract.

The default parser currently consumes blank lines and link definitions without
necessarily emitting corresponding `BlockNode` values. The new API projects the
IR it receives; it must not fabricate missing nodes or add another parser to
recover them. The mapping below also covers programmatic and hook-authored IR.

## Public API and Envelopes

The public signatures are:

```rust
pub enum BlockOutputMode { Structured, Rendered }

pub enum BlockOutput {
    Structured(StructuredBlockOutput),
    Rendered(RenderedBlockOutput),
}

pub fn derive_blocks(input: RenderInput, mode: BlockOutputMode) -> BlockOutput;
pub fn derive_parsed_blocks(
    parsed: &ParsedDocument,
    mode: BlockOutputMode,
) -> BlockOutput;

impl OrihaEngine {
    pub fn derive_blocks(&self, input: RenderInput, mode: BlockOutputMode) -> BlockOutput;
    pub fn derive_parsed_blocks(
        &self,
        parsed: &ParsedDocument,
        mode: BlockOutputMode,
    ) -> BlockOutput;
}
```

Top-level functions use the default engine. `derive_blocks` parses once and
delegates to parsed derivation. `derive_parsed_blocks` never parses Markdown
again. It uses the retained effective features, source, limits and parse-stage
state. It neither replaces the source nor changes the parsing feature profile.

`BlockOutputMode` is independent of the existing `RenderMode`. Rendered block
derivation always uses semantic HTML and produces no theme CSS. Structured
derivation does not invoke HTML rendering or diagram layout. The retained
`RenderMode` and appearance settings remain intact for later ordinary renders;
rendered derivation changes only its own working render mode.

The serializable envelope has `schemaVersion: 1`, `mode`, `status`, `blocks` and
`diagnostics`. Status is `complete`, `incomplete` or `failed`. Structured output
also contains `references` and `footnotes`. Rendered output also contains
`assets`. Rust uses distinct output structs and enums instead of a record full
of optional fields for both modes. Use camelCase field names and the kind tags
defined below. A future incompatible wire change requires a schema-version
change; new kinds require a documented compatibility decision.

## Structured Representation

A structured block has `id`, `kind`, `text`, `properties` and `children`.
`children` is an ordered array of structured blocks. `text` is the block's own
plain-text projection; it excludes child blocks. It is null for non-textual
containers and structural nodes. Empty text is a valid string, not null.

`properties` is a discriminated Rust enum serialized together with `kind`.
Each kind has a typed payload; it is not an arbitrary JSON map or the internal
`attrs` map. This prevents a heading from carrying list properties. The
paragraph/heading and table-cell payloads contain ordered `richText` arrays.

Rich text is an owned inline tree with `kind`, nullable leaf `text`, typed
`properties`, and ordered inline `children`. It retains nested formatting,
links, images, equations and engine annotations. The inline tree preserves
the engine's nested IR without flattening annotation semantics. Inline nodes have no invented persistent IDs.

Structured `blocks`, block/inline `children` and `richText` arrays have nullable
entries in their Rust/serialization types. Every currently supported variant
has the concrete projection defined below. If a future variant deliberately
chooses non-representation, its explicit conversion arm emits a literal null in
place and an unsupported-projection diagnostic identifying the kind and owning
block. Do not silently drop that position. This differs from a valid node whose
own `text` is null; all current structural nodes remain concrete objects.

Plain-text projection decodes text as represented in the IR, keeps code and math
source, uses image alt text, and maps soft/hard breaks to newline characters.
It recursively reads inline containers, without their markup. A footnote
reference projects to `[^label]`. HTML nodes project to their authored source
text; permission to execute that HTML is a separate render feature.

For example, `## Hello **Rust**` has the following output shape. Example IDs are
illustrative; this is not a snapshot of implemented output.

```json
{
  "schemaVersion": 1,
  "mode": "structured",
  "status": "complete",
  "blocks": [
    {
      "id": "example-1",
      "kind": "heading",
      "text": "Hello Rust",
      "properties": {
        "level": 2,
        "richText": [
          { "kind": "text", "text": "Hello ", "properties": {}, "children": [] },
          {
            "kind": "strong",
            "text": null,
            "properties": {},
            "children": [
              { "kind": "text", "text": "Rust", "properties": {}, "children": [] }
            ]
          }
        ]
      },
      "children": []
    }
  ],
  "references": {},
  "footnotes": [],
  "diagnostics": []
}
```

### Complete Block Mapping

The `kind` tags mirror IR kinds, not output element names. An image-only
paragraph remains `paragraph`; a Mermaid fence remains `fencedCode`.
`richText` below means the typed inline array. Container children are projected
recursively, in IR order.

| IR variant | Kind | Structured payload and own text | Default rendered decision |
| --- | --- | --- | --- |
| `Document` | `document` | Empty properties; null text; child blocks | HTML of its children |
| `Paragraph` | `paragraph` | `richText`, `tight`; projected text | Paragraph or standalone-image HTML |
| `Heading` | `heading` | `level`, `richText`; projected text | Heading HTML |
| `FencedCode` | `fencedCode` | `language`, `info`, `closed`, `indent`; materialized code text | Code, math or diagram HTML |
| `IndentedCode` | `indentedCode` | `language`, `info`, `closed`, `indent`; normalized code text | Code HTML |
| `MathBlock` | `mathBlock` | `display`, `closed`; materialized math source text | Math HTML or existing feature fallback |
| `ThematicBreak` | `thematicBreak` | Empty properties; null text | Horizontal rule HTML |
| `HtmlBlock` | `htmlBlock` | Empty properties; authored source text | HTML wrapper with configured escaping |
| `LinkReferenceDefinition` | `linkReferenceDefinition` | `source` when present in IR; null text | Null: `referenceDefinition` |
| `BlockQuote` | `blockQuote` | Empty properties; null text; child blocks | Complete quote HTML |
| `List` | `list` | `ordered`, `loose`, `marker`, derived `start`; null text; child blocks | Complete list HTML |
| `ListItem` | `listItem` | `marker`, nullable `checked`; null text; child blocks | List-item HTML in its parent context |
| `GfmTable` | `gfmTable` | `alignments`, `header`, `rows`; null text | Complete table HTML |
| `BlankLine` | `blankLine` | Empty properties; null text | Null: `blankLine` |
| `Callout` | `callout` | `kind`, nullable `title`; null text; child blocks | Complete callout HTML |
| `ForeignHtml` | `foreignHtml` | `origin`, `trustLevel`; supplied HTML text | HTML wrapper with configured escaping |

Use null for the start of an unordered list; use a normalized decimal string
for an ordered start, avoiding integer overflow. Derive it
from the parser's marker using the same semantics as HTML rendering. Table
headers and rows contain ordered cells with `text` and `richText`. Resolve code,
math and HTML values against their actual source context, including normalized
list, quote and callout sources. Internal parsing offsets and normalization
strings are not public properties.

`references` maps the engine's normalized reference label to destination and
nullable title. `footnotes` is an ordered array of `{label, richText}` definitions,
including unreferenced definitions retained by the IR. Duplicate definitions
retain IR order; reference resolution uses the existing renderer's precedence.
Resolved inline links retain their URL and title, so consumers need no Markdown
reference parser. This is semantic IR projection, not byte-for-byte Markdown
round-trip preservation.

### Complete Inline Mapping

| IR variant | Kind | Leaf text / typed properties / inline children |
| --- | --- | --- |
| `Annotation` | `annotation` | Null text; annotation `kind`, `id`, classes and data attributes; children |
| `Text` | `text` | Decoded text; empty properties |
| `Emoji` | `emoji` | Emoji text; `shortcode` |
| `Code` | `code` | Code text; empty properties |
| `Math` | `math` | Math source text; empty properties |
| `Html` | `html` | Authored source text; `tagfilter` |
| `SoftBreak` | `softBreak` | Newline text; empty properties |
| `HardBreak` | `hardBreak` | Newline text; empty properties |
| `Emphasis` | `emphasis` | Null text; empty properties; children |
| `Strong` | `strong` | Null text; empty properties; children |
| `Strikethrough` | `strikethrough` | Null text; empty properties; children |
| `Link` | `link` | Null text; `url`, nullable `title`; children |
| `Image` | `image` | Null text; `url`, nullable `title`; children contain alt inlines |
| `FootnoteReference` | `footnoteReference` | Null text; `label` |

Do not discard an inline variant merely because it is not eligible for an
application's text search or mention policy. In particular, keep image alt text,
unreferenced footnote definitions, HTML and annotations in structured data.

## Rendered Representation

Use a new `RenderedHtmlBlock` type, keeping the existing source-map
`RenderedBlock` unchanged. Its wire fields are `id`, typed `kind`, `html: Option<String>` and
`nullReason`. Rust stores `RenderedBlockContent::{Html, Null}` and exposes an
`html()` accessor to prevent contradictory combinations. It has no `children`, structured properties or plain-text field.
An internal sum type distinguishes HTML from explicit Null; serialization emits
`nullReason: null` for HTML and a reason for Null. `Some("")` remains HTML.

Rendered `blocks` corresponds to the parsed document's root blocks, in order.
A list, quote, callout or programmatic Document root contains its complete
subtree HTML. Do not append its descendants as additional output entries, split
Markdown into pieces, or parse final HTML to discover boundaries. Preserve
existing semantic HTML markup, including engine classes and data attributes.
This mode supplies no theme CSS and does not promise class-free HTML.

For the same heading as above, the rendered shape is:

```json
{
  "schemaVersion": 1,
  "mode": "rendered",
  "status": "complete",
  "blocks": [
    {
      "id": "example-1",
      "kind": "heading",
      "html": "<h2 id=\"hello-rust\" class=\"oriha-heading oriha-heading--h2\" data-oriha-block-kind=\"heading\" data-oriha-block-id=\"example-1\" data-oriha-heading-level=\"2\">Hello <strong>Rust</strong></h2>",
      "nullReason": null
    }
  ],
  "assets": [],
  "diagnostics": []
}
```

Legal Null preserves the node's ID and kind. Initial Null reasons are
`blankLine` and `referenceDefinition`; a deliberately unsupported future kind
may choose `unsupported` explicitly in its conversion arm and report a
diagnostic. Null is never inferred from an empty string, an unknown kind tag,
or failure to find a renderer.

Render all roots through one shared document context. Collect HTML fragments
during the existing block traversal, so nested source normalization, reference
resolution, heading de-duplication, footnote numbering and render state remain
consistent. Append the existing footnote section as one synthetic `footnotes`
block only when that section is emitted. Use `oriha-generated-footnotes` as its
ID, adding the smallest available numeric suffix if a supplied node uses it.
Structured mode keeps footnote definitions in the envelope instead of appending
this rendered-only synthetic node.

Return existing `RenderAsset` values and diagnostics without discarding them.
Deduplicate assets using the engine's existing asset semantics. Math font
content is currently base64 with MIME `font/otf`; SVG/CSS asset content is text.
Do not claim every `Other` asset is text or base64. Consumers follow the
documented asset kind, identity and MIME contract. No general stylesheet is
generated in rendered mode.

## Hooks, Extensions and Identity

Parse-stage hooks run once during parse. Structured derivation projects the
retained post-parse IR and does not execute render hooks. Rendered derivation
runs existing block-render hooks on a working copy and shares their context
with ordinary semantic rendering. Neither mode mutates the retained document.

Document-output finalization hooks target complete document HTML. They are not
executed by the new block derivation API: arbitrary final document rewrites
cannot be attributed to individual fragments without a second HTML parser.
Keep their existing behavior unchanged on `render`, `render_parsed`, patch and
export APIs. A consumer requiring such a rewrite continues to use a document
API or performs its own explicitly scoped block transformation.

Retain the current `BlockRenderer::render` contract. Add a provided method
`block_emission(&self, block: &BlockNode) -> BlockEmission`, defaulting to
`BlockEmission::Html` for existing custom renderers. The built-in renderer
overrides it with the exhaustive HTML/Null policy above. A custom renderer can
explicitly opt into Null. Collect its HTML even if empty; do not guess its
policy from the fragment. Structured mode always represents the retained IR,
not a custom renderer's HTML. Document and test this additive extension.

Preserve each root's IR ID across both modes when render hooks do not transform
identity. Reject empty IDs and validate uniqueness across the structured tree and rendered
working tree, including after hooks;
report invalid hook-authored identities instead of silently inventing source
identities. IDs are document-local and revision-local, not editing-stable keys.
Reserved synthetic IDs are the only generated output identities. Rendering
options must not be stored by modifying the original `ParsedDocument`.

## Exhaustiveness, Failure and Resource Bounds

Structured block conversion, structured inline conversion, default HTML
conversion and default emission policy must match their actual IR enums
exhaustively, with no wildcard or catch-all conversion. Each new IR variant
must acquire its own typed representation or an explicit non-representation
decision, and its HTML/Null decision. Do not replace these decisions with string
registries or a generic Unknown branch. Compiler coverage forces a decision;
fixtures still establish whether the chosen behavior is correct.

Feature settings govern what the parser produces; output mode never broadens
them. A disabled diagram feature can produce an ordinary fenced-code block
according to existing behavior. Derivation supports every IR variant that is
present, independently of the application's chosen feature profile.

Carry parse completeness and diagnostics into the output. A retained incomplete
parse returns `status: incomplete` and empty result collections. A failed parse,
panic, invalid identity, error-severity derivation diagnostic or exceeded output
budget returns `status: failed` and empty result collections. Keep diagnostics
in either case; never return a successful-looking truncated HTML fragment.
Warnings accompanying a supported existing fallback may remain complete.
An explicit legal Null is not a failed derivation.

Reuse retained input, block-count and nesting limits. Apply `max_output_bytes`
to the aggregate UTF-8 serialization size of the selected output, including
rich text or HTML, properties, references, footnotes, assets and envelope fields.
If diagnostics themselves exceed the limit, return one bounded output-limit
diagnostic rather than the oversized list. A fixed bounded failure envelope is the minimum
response even when the configured limit cannot hold an envelope. Enforce the budget during output
construction, not only after allocating the full serialized result. These are
per-derivation limits, not a claim about the whole process's RSS.

The default built-in path must produce equivalent visible content when its
non-null root fragments and generated footnotes are placed in document order.
Compare it with the semantic document body's output, excluding the ordinary
outer article wrapper and explicitly document-only decorations. Block mode does
not emit document watermarks or document-level layout wrappers.

## Production v1 Delivery Scope

The target is a complete production v1 of this Rust block API within the scope
above. Both modes, typed serialization, all current IR kinds, errors, resource
limits and existing-client compatibility must work in real compiled consumers.
Do not substitute hard-coded examples, TODO paths, silent omission or an
unsupported Null for a currently required conversion. Supply a runnable consumer
example, usable API/migration documentation and reproducible correctness checks.
This milestone is not an instruction to change the package's major version.

Response-time targets, throughput targets, high-load/stress tests, benchmark
comparisons and process-memory/RSS measurements are not completion conditions.
They are optional diagnostics and must not become an implicit acceptance gate.
Existing unrelated performance issues or older ADR measurement work do not
become dependencies of this feature. No performance guarantee follows from v1
completion.

Resource bounds, early limit enforcement, safe failure, deterministic output,
parse reuse and maintainable algorithms remain required. Review for avoidable
repeated parsing, unchecked allocation and unbounded work; test structural limits
and retained-state ownership directly. Finite concurrency/isolation tests check
correctness, not load capacity. Do not weaken security, compatibility or ordinary
functional checks because performance qualification is excluded.

## Compatibility and Future Verification

Existing Rust document APIs, `RenderMode`, source-map metadata, JSON document
responses, hook lifecycle and existing clients retain their behavior. The new
output types are owned DTOs rather than serialization of the private parser
state. There is no automatic change to bridges or generated client types.

Implementation must include these checks; none has been executed by authoring
this document:

- Fixtures cover every BlockKind and InlineKind, including programmatic nodes
  that the default parser does not emit, attributes, normalized container source,
  tables, images, annotations, code, math and both diagram engines.
- Structured JSON retains formatting, URLs, optional values, reference and
  footnote data, Unicode and decoded text without a consumer-side Markdown parse.
- Reuse instrumentation proves parse-once behavior; hook counters prove the
  lifecycle above; repeated derivation leaves retained IR and later renders intact.
- Rendered output has no children or duplicated descendant entries. Null survives
  serialization, while empty HTML remains non-null. Footnote and heading links
  work across roots; body-equivalence checks cover existing semantic markup.
- A temporary copy of the actual source IR receives a probe variant in each
  enum. Compilation must fail in the relevant conversion bodies with E0004 until
  both mode decisions are supplied. A surrogate test enum is insufficient.
- Limits, incomplete parses, panics, unsupported decisions, custom renderers and
  asset-heavy documents cover failure status, bounded work and resource reporting.
- Run affected existing regression suites, formatting/lint checks and the
  implementation-only code-size assessment before any future implementation
  handoff. Store receipts separately from this proposed contract.
- Build the affected release artifact and execute the real Rust consumer example
  in both modes. Review dependency changes against the repository's advisory,
  license and source policies. Provide the supported API contract and concrete
  errors/limitations needed by a production caller. No benchmark report is needed.

Design acceptance and implementation status are tracked in Beads. Application
filtering and adoption require their own integration verification.

The linked goal prompt authorizes review and implementation only when a user
invokes it. Storing or editing the prompt does not execute it, change this
design's review status or make the feature ready for production.

Review accepted on 2026-11-01; see ADR review decisions.

Implementation decisions: recursive inline label parsing/link validation is
bounded at 64 calls to prevent a verified stack-overflow abort. Combined inline
formatting and programmatic tree depth has a 256-level safety ceiling, in
addition to retained container limits. Exceeding these bounds yields an explicit
failed result. See [migration guidance](../guides/0002-block-output.md) and the
[Verification methods](../guides/verification.md).
