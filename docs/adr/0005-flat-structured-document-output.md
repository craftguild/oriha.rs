# ADR 0005: Flat structured documents with optional render preparation

Status: Accepted — Implemented; source consumer integration verified.

Date: 2026-11-17.

## Context

Oriha is a portable Markdown parser and renderer. Its existing HTML pipeline provides consistent engine-owned output for mobile, desktop and web consumers. That pipeline remains useful and must remain available. Other consumers need typed document data so they can construct their own presentation without parsing HTML or duplicating Markdown semantics.

[ADR 0002](0002-structured-and-rendered-block-output.md) introduced recursive structured blocks and rich inline trees alongside rendered HTML blocks. A recursive document is inconvenient for selection-based transports: a client must otherwise anticipate the Markdown nesting depth. The existing structured representation also leaves syntax highlighting and rendered math or diagrams available only through the HTML path. Consumers should not have to extract tokens or SVG from that HTML.

## Decision

### Replace the structured representation

Replace the existing structured schema with version 2. Do not retain the version-1 tree as a parallel structured representation or compatibility adapter. Preserve the existing document HTML APIs, rendered-block APIs, source-aware parse APIs, native/JSON contracts and their normal behavior. This decision changes the structured contract; it does not retire the HTML renderer or automatically expose a new native bridge.

The structured envelope contains `schemaVersion: 2`, the existing mode and completion status, `rootIds`, `nodes`, `referenceIds`, `footnoteIds`, `fonts` and typed `diagnostics`. All node collections and ID lists have non-null entries. `rootIds` orders the document body. `referenceIds` and `footnoteIds` identify retained definitions in the same node pool without making those definitions ordinary visible body roots.

`nodes` is one flat collection of a tagged Rust enum with typed variant payloads. Blocks, inline content, table rows, table cells and definitions all have document-local IDs. Nodes contain ordered ID references instead of nested nodes. Required text belongs to text-bearing variants; do not put a nullable `text` field on every node or add a generic key/value properties bag. Optional authoring values such as a link title or task state remain genuinely optional fields on the relevant variants.

IDs identify one parsed document revision, are opaque to consumers and are not persistent editor identities. Their assignment and structural ordering must not change when optional preparation is enabled or disabled. Preserve usable producer block identities and resolve collisions across block, inline, table and definition identities. Keep structural ownership distinct from semantic links: child ownership forms an ordered forest, while repeated footnote references can point to one retained definition without owning or recursively expanding it.

### Preserve Markdown semantics

Project the actual parser IR exhaustively. Resolve source-backed values while the parsed document is retained. Do not parse Markdown again, reconstruct semantics from HTML or invent source locations for generated nodes. The block and inline projection matches are exhaustive and contain no wildcard fallback: adding an IR variant fails compilation until its typed projection is implemented. Changes to semantics within an existing variant require behavior tests. Unsupported or malformed structure must not disappear silently.

| Source kind | Structured information to retain |
| --- | --- |
| Document, paragraph and heading | Ordered content IDs, paragraph tightness, heading level and producer anchor identity |
| Fenced and indented code | Normalized code source, language, info string, closure and indentation metadata |
| Block and inline math | Math source, display/inline semantics and closure where applicable |
| Quote, list and list item | Ordered ownership, marker, looseness, normalized decimal-string list start and optional task state |
| GFM table | Alignment plus ordered row IDs; header identity on rows; ordered cell IDs and inline content IDs |
| Callout | Kind, optional custom title and ordered body IDs |
| Text, emoji and inline code | Owned visible text or code source, and emoji shortcode where applicable |
| Emphasis, strong and strikethrough | Ordered inline child IDs |
| Link and image | Original and normalized destination, optional title, and ordered label or alt-content IDs |
| Soft and hard break, thematic break and blank line | Explicit semantic variants without invented text |
| Footnote reference and retained footnote definition | Labels, resolved references when available, ordered definition content and producer anchor/backlink information |
| Retained link references and link-reference syntax nodes | The distinction between resolved label/destination/title records and non-rendering source syntax, using typed variants |
| Annotation | Existing annotation kind, identity, classes, explicit annotation metadata and ordered child IDs |
| Authored HTML and foreign HTML | Source text and existing tag-filter, origin and trust metadata; execution permission remains separate |

Annotation extension metadata remains specific to the existing annotation contract. It is not an escape hatch for standard node attributes. Preserve the parser's current reference resolution and duplicate-definition behavior, including collision-free footnote anchors. Source HTML in a typed node is authored source, not permission to inject it into a viewer.

### Include optional preparation in the same output

The structured output itself carries syntax tokens and SVG on the nodes to which they belong. A consumer does not need a second result API followed by a join. Add explicit structured preparation options for syntax highlighting, math, Mermaid and PlantUML; all are off by default so a structure-only call remains possible. Options control derivation from an already parsed document and must not change parsing features or node IDs.

Code variants carry optional typed highlighting data composed of ordered lines and tokens. Each token has a finite token-kind enum and literal text, including plain tokens. Tokenization reuses the existing Rust highlighter; colors, CSS classes, line-number markup and copy controls are viewer decisions. Preserve empty lines, Unicode, whitespace and the distinction between code source and its terminal line ending. Unknown languages use plain tokens rather than failing the document.

Math variants and supported math/diagram fences carry optional typed SVG data directly on their originating node. Retain the Markdown source and fence metadata even when preparation succeeds; do not replace the semantic node with an untyped SVG-only node. SVG data contains markup, intrinsic sizing/baseline information needed for its display mode and IDs for required fonts. Its content is generated SVG, without an HTML figure, span or error wrapper.

Shared font dependencies are returned once in the document's typed `fonts` collection. Preserve the engine's MIME, encoding, family and licensing requirements; the current math renderer needs its bundled STIX font and must not be treated as a self-contained path-only SVG. Do not introduce a generic CSS/HTML resource escape hatch. Reuse the existing renderer internals for both HTML and structured preparation so their tokenization, math and diagram behavior do not drift.

YouTube, Spotify and Gist recognition, image galleries, application routes, filesystem image inspection, asset proxying and search-term highlighting are outside this contract. A viewer can derive those policies from typed links, images and text. The existing generic responsive-image HTML API remains valid for HTML consumers.

### Fail explicitly at the correct boundary

Keep the existing complete/incomplete/failed envelope and bounded diagnostics. Parse failure, invalid graph structure, duplicate IDs, missing owned nodes, unsupported projection or exhausted aggregate limits must not return a truncated document as complete. Apply node, depth, allocation and output budgets to source, tokens, SVG and fonts, not only to the base graph. Flat serialization does not remove the need to bound traversal depth.

A malformed or unsupported math/diagram rendering is a node-local preparation failure. Retain that node's source, omit its unsuccessful SVG and include a typed diagnostic with the owning node ID. Other nodes remain usable. Not-requested preparation is distinguishable from requested preparation failure through the explicit options and diagnostics. Cancellation or a derivation-wide safety failure still fails the derivation rather than being disguised as an ordinary source fallback.

Diagnostics contain severity, a stable code, a bounded message and node/source association when available. Consumers decide how much diagnostic detail to display. HTML rendering and structured preparation must use isolated working state and must not mutate the retained parse or each other's output.

## Alternatives considered

| Alternative | Reason not selected |
| --- | --- |
| Recursive structured nodes | Requires transport selection depth to follow arbitrary Markdown depth. |
| A generic node with many nullable attributes or string properties | Weakens the type contract and permits contradictory payloads. |
| Separate arrays for each node kind | Requires consumers to join identity records to kind-specific payloads. |
| Extract tokens and SVG from rendered HTML | Couples structured consumers to HTML markup and duplicates interpretation. |
| Separate preparation responses | Adds consumer-side assembly when the originating node can carry its result. |
| Always prepare every artifact | Performs unused work for text-only and structure-only consumers. |

## Consequences and compatibility

Structure-aware consumers gain one complete node pool whose transport selection depth is independent of Markdown nesting. They own their presentation, while HTML consumers retain engine-owned markup and cross-platform consistency. Typed preparation supplies the difficult rendering results without assigning application policy to Oriha.

This is a breaking change to the structured schema and its Rust DTOs. Record that change explicitly and migrate actual structured consumers, including text-feed projections. Do not apply the schema-version change to unrelated JSON, native or rendered-block envelopes. [ADR 0002](0002-structured-and-rendered-block-output.md) and its [detailed contract](../design/block-derivation.md) retain the historical structured design and current rendered-block provisions. The [Rust guide](../guides/0002-block-output.md) describes the version-2 implementation. Source integration with a generated GraphQL consumer has been verified; publication and complete consumer migration remain separate work.

## Acceptance and status

The implementation covers every current IR variant, deeply nested blocks and inlines, table/definition ownership, missing/duplicate/cyclic structural IDs, reference/footnote collisions, preparation-independent IDs and aggregate failure limits. Rust regressions verify literal code-line reconstruction, all supported SVG engines, shared fonts, inline sizing and node-local source fallback. Actual temporary enum additions produce compiler errors in the block and inline projections. Repeated derivation reuses parsing and remains isolated from ordinary HTML rendering.

The required repository check passes, including CommonMark, source mapping, JSON/native compatibility, HTML regressions and canonical generation. A separately built consumer executable serves the real generated union, optional tokens/SVG/fonts and typed images over HTTP. These results establish producer implementation and source integration; they do not establish publication, released dependency pins, full consumer adoption or unexecuted platform checks. No new throughput, benchmark or cross-platform application campaign is required by this ADR.
