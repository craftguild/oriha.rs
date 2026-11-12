# Rust block output, schema version 1

`oriha-core` provides owned structured trees and semantic HTML fragments from the
same parser. This Rust API is included in the initial 0.1.0 source release.
Existing document JSON,
source-map `RenderedBlock`, `RenderMode`, prepared rendering, patching and export
API and wire contracts are preserved. The inline HTML security correction below
intentionally changes unsafe output when raw HTML is disabled. Bridges do not
expose this API automatically.

## Use the API

```rust
use oriha_core::{
    parse, derive_parsed_blocks, BlockOutput, BlockOutputMode,
    BlockOutputStatus, RenderInput,
};

let parsed = parse(RenderInput {
    markdown: "# Hello **Rust**".into(),
    ..Default::default()
});
let BlockOutput::Rendered(output) =
    derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)
else { unreachable!() };
assert_eq!(output.status, BlockOutputStatus::Complete);
for block in &output.blocks {
    if let Some(html) = block.html() {
        // Supply HTML to your presentation layer, including empty strings.
        println!("{html}");
    }
}
```

`derive_blocks(input, mode)` parses once and derives an owned result. Both
functions are also methods on `OrihaEngine`. `derive_parsed_blocks` reuses
the retained source, effective features, appearance, limits and parse-hook
state. It makes no block-parser or inline-parser calls. A different deriving
engine supplies its renderers and render hooks, while the parsed settings win.
The result can outlive the source, parsed document and engine.

Rendered fragments also retain the [code-block presentation options](code-block-options.md).
Their defaults match complete styled and semantic HTML; structured code
properties are independent of these display switches.

Run the complete consumer example:

```sh
cargo build -p oriha-core --release --locked
cargo run -p oriha-core --example derive_blocks --release --locked
```

The example exercises typed structured data, reference/footnote retention,
rendered fragments, both entry points and JSON serialization using the release
library. It asserts the results instead of providing fixture responses.

## Typed structured data

`BlockOutput::Structured` contains `StructuredBlockOutput`: `schema_version`,
`mode`, `status`, `blocks`, `references`, `footnotes` and `diagnostics`.
Each `StructuredBlock` owns its `id`, nullable `text`, typed `content:
StructuredBlockProperties`, and ordered nullable `children`.

Serialization flattens `content` into `kind` and `properties`. Every property
variant is a distinct Rust payload. A heading cannot carry list properties.
No parser offsets, source mappings or normalization buffers enter this schema.

| Kind | Properties |
| --- | --- |
| `document`, `thematicBreak`, `htmlBlock`, `blockQuote`, `blankLine` | Empty object |
| `paragraph` | `richText`, `tight` (tight-list paragraph) |
| `heading` | `level`, `richText` |
| `fencedCode`, `indentedCode` | Nullable `language`, `info`, `closed`, `indent` |
| `mathBlock` | `display`, `closed` |
| `linkReferenceDefinition` | Materialized `source` |
| `list` | `ordered`, `loose`, `marker`, nullable `start` |
| `listItem` | `marker`, nullable task `checked` |
| `gfmTable` | `alignments`, `header`, `rows` |
| `callout` | `kind`, nullable `title` |
| `foreignHtml` | `origin`, `trustLevel` |

Ordered `start` is a normalized decimal string; unordered lists use null. This
preserves HTML start semantics even for programmatic markers beyond integer
range. Table alignments are `none`, `left`, `center` or `right`. Header cells and
row cells own `text` and `richText`. Trust levels are `untrusted`, `sanitized`
and `trusted`; retaining a trust label does not authorize HTML execution.

Own text excludes child blocks. Paragraphs/headings project their inline tree;
code/math/HTML carry their materialized values in `text`. Structural containers,
tables, reference definitions and thematic/blank nodes have null own text.
Empty text remains a string. Code uses the correct quote/list/callout source and
indentation normalization; consumers need no source buffer or Markdown parser.

`RichText` has nullable leaf `text`, typed `content: RichTextProperties` and
ordered nullable `children`. Its wire form also uses `kind` and `properties`:

| Kind | Text / properties / children |
| --- | --- |
| `text`, `code`, `math` | Leaf value, empty properties |
| `emoji` | Emoji leaf value, `shortcode` |
| `html` | Authored leaf value, `tagfilter` |
| `softBreak`, `hardBreak` | Newline leaf value, empty properties |
| `emphasis`, `strong`, `strikethrough` | Null leaf text, formatting children |
| `link`, `image` | Null leaf text, `url`, nullable `title`, label/alt children |
| `annotation` | Null leaf text, `kind`, `id`, `classes`, `data`, children |
| `footnoteReference` | Null leaf text, `label` |

Plain text recursively preserves formatting children and image alt text, keeps
code/math/HTML source, maps breaks to newline and footnote references to
`[^label]`. Text already decoded in the IR remains decoded. Annotation fragments
retain one supplied logical identity across formatting boundaries.

References map normalized labels to `destination` and nullable `title`.
Footnotes retain every definition in IR order, including unused and duplicate
definitions. Rich links already contain resolved destinations/titles. Rendering
uses the existing last-definition precedence for duplicate footnotes.

## Rendered HTML and explicit Null

`BlockOutput::Rendered` contains `RenderedBlockOutput`: `schema_version`,
`mode`, `status`, `blocks`, `assets` and `diagnostics`. A `RenderedHtmlBlock` owns
`id`, `kind` and `content: RenderedBlockContent`. Match `Html(String)` versus
`Null(BlockNullReason)` in Rust, or use `html() -> Option<&str>`.
This sum type prevents contradictory HTML/Null states. JSON exposes `html` and
`nullReason` directly:

```json
{"id":"block-1","kind":"blankLine","html":null,"nullReason":"blankLine"}
```

`Html("")` serializes as `"html":"","nullReason":null`. It remains a valid
fragment. Never infer Null from an empty string. Initial legal Null reasons are
`blankLine` and `referenceDefinition`. A custom renderer can explicitly select
`unsupported`, producing a diagnostic naming the kind and owning block.

Entries correspond to roots, in IR order. A list, quote, callout or Document
entry contains its complete subtree. Entries have no children; appending their
descendants would duplicate content. The engine renders each subtree once in a
shared context, preserving heading de-duplication, cross-root links, references
and footnote numbers. Visible footnotes append one generated `footnotes` entry.
Its ID is `oriha-generated-footnotes`, with the smallest available numeric suffix
when any IR node already owns that identity.

Classes and `data-oriha-*` attributes remain part of existing semantic markup.
There is no article wrapper, theme CSS, watermark or document layout decoration.
With the built-in renderer, joining non-null fragments in order equals the
semantic document body after excluding those document-only decorations.

Structured nullable array positions are reserved for deliberate future
non-representation decisions, with an unsupported-projection diagnostic. All
current IR variants are real structured nodes, including non-textual nodes.
Neither output mode applies the CMS's filtering policy. Parser-consumed blank
lines/reference definitions are not invented as IR nodes.

## Hooks and identities

Document parse hooks run once at parse time. Structured derivation projects the
retained post-parse IR and executes no render hooks or diagram layout. Rendered
derivation executes existing before/after block-render hooks on a working copy,
with one shared context and state. Repeated and concurrent derivations do not
mutate the parsed document. Returned IDs reflect deliberate render-hook changes;
otherwise both modes preserve the IR IDs.

The new block APIs execute neither `before_output_finalize` nor
`after_output_finalize`. Those complete-document hooks retain their ordinary
render/patch/export behavior. Arbitrary document rewrites cannot be attributed
to fragments. Consumers needing them must use the document APIs or an explicit
consumer block transformation.

`BlockRenderer::render` is unchanged. The additive provided
`block_emission(&self, block: &BlockNode) -> BlockEmission` defaults to `Html` for
existing custom renderers, including renderers producing empty HTML. Override it
to choose explicit Null. Ordinary document APIs ignore this emission method.

IDs must be nonempty and unique throughout the tree. Validation runs before
projection and again on the rendered working tree after hooks. Invalid IDs fail
the derivation; source identities are never silently invented. IDs belong to one
document revision and are unsuitable as edit-stable database keys.

## Status, limits and resources

Check `status` before consuming collections. `complete` means derivation
succeeded, including legal Null and supported warning fallbacks. An incomplete
parse returns `incomplete` with empty collections. Failed parsing, invalid IR,
error diagnostics, output-limit failure or a caught panic returns `failed` with
empty collections. Retain and inspect `diagnostics`; truncated output is never
reported complete. Failure collections include references/footnotes/assets too.

The retained input, per-parser-container block limit and container-depth limit
continue to apply. List-item wrappers are not an extra parser block budget.
Programmatic/hook-authored IR is validated against the corresponding bounds.
A safety ceiling of 256 tree levels protects programmatic wrappers and inline
formatting. Recursive inline label parsing and link validation have an additional
64-call safety bound; exceeding it fails parsing. These bounds prevent stack
exhaustion and do not produce successful partial documents. RAII guards restore
working limits and recursion state after failures.

`max_output_bytes` limits the aggregate UTF-8 JSON size: envelope, blocks,
properties, duplicated own text, references, footnotes, diagnostics and assets,
including JSON escaping. Early lower-bound accounting precedes structured
payload copies; built-in HTML buffers check remaining capacity on append;
resource registration checks the font before copying it. Incremental root and
resource accounting precedes further accumulation. A non-allocating counting
serializer verifies the exact final aggregate. Custom hooks/renderers remain
responsible for allocations they perform inside their existing String-based
extension methods; their returned output still receives aggregate validation.

Oversized diagnostic lists collapse to one bounded output-limit diagnostic.
The fixed failure envelope is the minimum possible response and can exceed a
configured limit too small to encode any error envelope.

Rendered `assets` preserve existing `RenderAsset` identity, kind, MIME and
content semantics, including math-font deduplication. The bundled math font is
`Other`, MIME `font/otf`, with base64 content. SVG/CSS asset content is text;
`Other` does not universally mean base64 or text. Font/CSS loading belongs to the
consumer. The block API generates no general theme stylesheet. Raw HTML remains
controlled by the retained feature setting; structured HTML is authored data,
not permission to execute it. Existing URL and annotation escaping remains in
force. This API does not replace a consumer's HTML sanitization policy.

## Compatibility and verification

JSON uses camelCase, `schemaVersion: 1` and a `structured` or `rendered` mode.
These DTOs are a public schema rather than a dump of internal IR. Incompatible
wire changes require a schema-version change; new kinds require an explicit
compatibility decision. Typed blocks and rich text preserve oriha-core's IR
semantics according to the [complete mappings](../design/block-derivation.md).

Both projections and built-in HTML/emission matches use the actual IR enums
exhaustively. Run `python3 scripts/check-block-exhaustiveness.py` to add real probe
variants in a temporary source copy and verify E0004 in each projection body.
No probe enters the working tree. See the [Verification methods](verification.md)
for source identity, all functional/dependency gates, limits and release evidence.

### Independent production-review corrections

Independent review found that the existing inline renderer ignored
`raw_html=false`. Authored inline HTML is now escaped in both inline rendering
paths, ordinary document output and rendered blocks, including nested formatting,
tables and footnotes. Effective retained settings govern parsed derivation.
`raw_html=true` retains enabled HTML behavior and the existing GFM tag filter.
That filter is not a sanitizer. Generated Markdown/annotation markup and
structured authored HTML data retain their existing representation. This is an
intentional security/contract correction to baseline HTML, without an API or
schema change.

Independent review found that temporary empty resource-array wrappers were
charged once per root. Incremental accounting now charges each newly registered
asset/diagnostic item, so an empty delta costs zero. Real envelope/array syntax
and commas remain covered by exact final counting serialization. A complete
result at its measured serialized-byte limit succeeds, including many roots;
one byte below that size fails with empty collections. Checked construction,
font pre-copy checks and aggregate resource limits remain enforced.

## Built-in HTML references without rendering

Rust consumers can call `oriha_core::render::builtin_html_anchors(&parsed)` for
built-in semantic HTML non-SVG DOM IDs in emission order, including canonical
heading and footnote IDs. `oriha_core::render::normalize_link_destination(url)`
uses the same destination normalization as the built-in HTML renderer. These
functions do not generate HTML, prepare images, render math/diagrams or measure
fonts. Custom renderers/hooks and IDs authored in raw HTML are outside the
anchor guarantee. Application URL policies and content selection remain with
the consumer. These APIs do not change Structured schema v1 or omit content.

Footnote IDs share a document-local namespace with built-in non-SVG block and
heading IDs. Existing IDs remain when available. Colliding definitions and
references use encoded-label fallbacks; the section title uses an available
alternative. Reference counters, definition links, first-reference backlinks
and the footnotes title use the same allocation as the anchor API. This fixes
distinct labels with equal slugs (for example `a b` and `a-b`) without changing
footnote definition precedence or numbering. Consumers obtain canonical IDs from
the API rather than reconstructing a slug or reference suffix.
