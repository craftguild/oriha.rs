# ADR-0001: Source-Aware Parsing and Generic Text Annotations

- Status: **Accepted**
- Date: 2026-10-29
- Baseline examined: oriha-core 0.2.5 / `ae1846d`
- [Verification methods](../guides/verification.md)

## Context

A rendering consumer may use a second Markdown parser to extract mention
candidates and locate them in visible text, then use the public `BlockRenderer`
extension point to decorate matching inline IR.

This arrangement allows candidate extraction and rendering to interpret Markdown
differently. Visible text alone cannot determine how to handle these inputs:

| Source | Visible text | Example consumer policy |
| --- | --- | --- |
| `@alice` | `@alice` | Include as a candidate. |
| `\@alice` | `@alice` | Exclude because the at-sign was escaped. |
| `\\@alice` | `\@alice` | Include because the at-sign itself was not escaped. |
| `&#64;alice` | `@alice` | Include with knowledge that the at-sign came from a character reference. |
| `@ali**ce**` | `@alice` | Detect one occurrence and preserve the strong emphasis on `ce`. |

The current implementation has the following limitations:

- [lib.rs](../../crates/core/src/lib.rs) exposes the IR but keeps the parser module
  private. There is no public entry point for obtaining a parsed document without
  generating HTML.
- `InlineNode::Text(String)` in [ir.rs](../../crates/core/src/ir.rs) carries neither
  inline source ranges nor character provenance.
- [inline.rs](../../crates/core/src/parser/inline.rs) merges literal characters,
  escapes, and character references into text. Their provenance cannot reliably
  be recovered afterward.
- Block quotes, lists, and callouts recursively parse normalized substrings.
  Paragraph normalization and table-cell splitting also transform the input.
  Adding a parent's starting offset is therefore insufficient to recover original
  source positions.
- Public rendering paths in [render.rs](../../crates/core/src/render.rs) parse
  internally and cannot reuse a parsed document supplied by a caller.

Expose and preserve the information available to the existing parser rather
than adding another parser dependency.

## Agreed Premises

The following premises were confirmed with the user:

- Consider the public parsing API, original-source mapping, and generic text
  traversal and annotation API together.
- Breaking changes to the public Rust IR are acceptable.
- The original deliverable was this ADR. Implementation is now authorized by the
  implementation work; acceptance does not imply implementation completion.

## Decision

### 1. Separate Parsing Infrastructure from Mention Policy

oriha-core provides Markdown structure, visible text, original-source mapping,
annotation application, and rendering. The consumer defines handle syntax,
case normalization, reserved words, occurrence limits, and whether escaped or
character-reference text is eligible for matching.

Resolving recipients from membership, presence, or connection state remains a
consumer responsibility. Do not assign application-specific meanings to `@all`,
`@here`, or `@channel` inside the engine.

### 2. Expose Parsed Documents and Reuse Them for Rendering

Introduce `ParsedDocument`, which owns the original source, parsed IR, effective
input settings, diagnostics, and parse-stage state. Expose source and IR through
read-only accessors. Apply input limits and panic-to-diagnostic handling to the
parse-only boundary as well.

The required public API shape is shown below.

```rust
pub fn parse(input: RenderInput) -> ParsedDocument;
pub fn render_parsed(parsed: &ParsedDocument) -> RenderOutput;

impl OrihaEngine {
    pub fn parse(&self, input: RenderInput) -> ParsedDocument;
    pub fn render_parsed(&self, parsed: &ParsedDocument) -> RenderOutput;
}

impl ParsedDocument {
    pub fn source(&self) -> &str;
    pub fn document(&self) -> &Document;
    pub fn diagnostics(&self) -> &[RenderDiagnostic];
    pub fn text_regions(&self) -> impl Iterator<Item = TextRegion<'_>>;
    pub fn apply_annotations(
        &self,
        annotations: &[TextAnnotation],
    ) -> Result<ParsedDocument, AnnotationError>;
}
```

Top-level functions use the default engine. Existing `render` and the new entry
points share internal parsing and rendering stages rather than maintaining two
parsers. `render_parsed` uses the stored effective input settings without parsing
Markdown again. It does not accept replacement source or a different parsing
feature profile at render time.

Define hook lifecycle behavior as follows:

- Execute `before_document_parse` and `after_document_parse` exactly once during
  parsing. Retain their effective settings, IR changes, diagnostics, and other
  parse-stage state in `ParsedDocument`.
- Execute existing block-rendering and output-finalization hooks on every render.
- Render from a working copy of the retained state. Mutations by a hook or custom
  renderer during one render must not affect a later render of the same document.

Preserve existing hook ordering and output for `render(input)`. Document explicitly
that rendering a parsed document does not rerun its parse-stage hooks.

### 3. Preserve Character Provenance During Parsing

Change `InlineNode` into a structure containing a node kind and original-source
metadata, moving the existing enum variants into `InlineKind`. In addition to its
visible string, text carries mapping segments describing:

- A UTF-8 byte range within the visible text.
- One or more UTF-8 byte ranges in the original Markdown.
- Provenance: `Literal`, `Escape`, `CharacterReference`, `Normalized`, or
  `Generated`.

Ranges in the new source-mapping and annotation APIs are zero-based, half-open
UTF-8 byte intervals, `start..end`. Original-source ranges refer to the complete
input supplied by the caller, not a substring with quote or list prefixes removed.
Keep this information distinct from existing line-and-column `SourceRange` values.

Internal normalization produces a string together with a mapping to its parent
input. Compose these mappings through quotes, nested lists, callouts, paragraphs,
table cells, and footnotes to reach the original source. Handle CRLF, tab expansion,
removed prefixes and whitespace, and escaped table delimiters. Do not recover
positions by searching visible text in the original source.

For transformations that are not one-to-one, such as a character reference
expanding into multiple code points, preserve the source extent and transformation
kind. Do not assume that source and display lengths are equal. Emoji shortcode
expansion must also map back to its shortcode and remain distinguishable from
literal text.

Coalesce mappings into contiguous segments where possible instead of allocating
an independent object per byte. Text introduced or changed by a hook without
explicit source correspondence is `Generated`; do not invent its source range.

### 4. Expose Text Regions While Preserving Document Structure

`text_regions()` returns continuous text regions suitable for matching. Each
region provides an identifier scoped to its parsed document, visible text, and
mapping segments connecting it to inline nodes and original source.

Use these default traversal rules:

| Structure | Behavior |
| --- | --- |
| Paragraphs, headings, individual table cells, rendered footnote bodies, and callout bodies | Traverse each inline sequence separately. |
| Emphasis, strong emphasis, and strikethrough | Keep text inside and outside the formatting continuous. |
| Links | Traverse labels; exclude destinations and title attributes. |
| Code, math, HTML nodes, images, emoji nodes, and footnote references | Exclude and end the current region. |
| SoftBreak, HardBreak, block boundaries, and table-cell boundaries | End the current region; do not concatenate text across the boundary. |

Image alt text, code-language metadata, link definitions, other attributes, and
generated callout labels are outside the default traversal. Text that exists as
separate Text nodes between HTML tags remains eligible in separate regions bounded
by the HTML nodes.

For example, `@ali**ce**` yields `@alice` in one region. Text separated by code or
different table cells does not become one name. Excluded nodes remain in the
rendering IR.

Region identifiers belong to one parsed document and cannot be reused with a
different parsed document. They are neither edit-stable identifiers nor the
existing stable IDs used by incremental rendering.

### 5. Apply Annotations as Dedicated IR

The consumer supplies a region identifier, a range within its visible text,
annotation kind and identity, and class / data attributes. Validate these inputs
and return a new parsed document containing dedicated annotation IR. Do not mutate
the original `ParsedDocument`.

Apply the following rules:

- Reject empty ranges, out-of-bounds ranges, ranges that split a UTF-8 character,
  and region identifiers from a different parsed document.
- Reject annotations that cross region boundaries or overlap in visible text.
  Adjacent annotations are allowed.
- Validate the complete request before applying it. One invalid annotation must
  not produce a partially annotated result.
- Permit an annotation to cover characters with different provenance. The
  consumer uses mapping metadata to decide whether escapes and character
  references are eligible.
- Preserve existing emphasis and links. Attach the same annotation identity to
  intersected text fragments rather than generating illegally crossing HTML tags.

An annotation over `@ali**ce**` may render as separate fragments around `@ali` and
the strongly emphasized `ce`, but both fragments represent one logical annotation.
Repeated mentions remain distinct occurrences.

Default rendering uses `span` elements and typed class / data attributes. Escape
text and attribute values in the engine, and restrict attribute names to allowed
forms. Using annotations must not require injecting arbitrary HTML strings.

Source-authored HTML is never promoted to annotation IR. Consumers that treat HTML
attributes as trusted metadata must retain their existing sanitization and
distinction between source-authored and application-generated attributes. This
API does not replace a general HTML sanitizer.

## Compatibility and Public Surface

- Consumers matching or constructing `InlineNode` must migrate. Document the
  breaking change at release time and provide examples using `InlineKind`,
  provenance metadata, and the annotation API.
- Parsed documents and annotations are Rust contracts. This proposal does not add
  serialized IR or new public JSON, UniFFI, WASM, or Expo payloads.
- Preserve existing `RenderInput` / `RenderOutput` and prepared / patch / export
  JSON contracts. Do not change the coordinate systems of the existing block
  `source_map` or `BlockNode.span` as part of this work. Document their distinction
  from the new inline original-source mappings.
- Without annotations, preserve HTML / CSS, heading anchors, statistics,
  diagnostics, assets, and incremental-rendering behavior. Annotation wrappers do
  not count as new Markdown blocks or visible text.
- Custom renderers that arbitrarily replace rendered body content remain
  responsible for its consistency with parsed text.

## Alternatives Considered

### Expose the Existing Private parse_document Directly

This would provide IR without HTML generation, but provenance and original-source
positions would still be lost. Existing rendering would also parse again. It does
not meet the three requirements together.

### Recover Source Positions from Visible Text After Parsing

Repeated text, escapes, character references, formatting, and container
normalization make matching ambiguous. Preserve provenance when parsing instead.

### Preserve the Existing IR and Add Only an External Mapping Table

This reduces Rust type changes but requires extensions that replace or split nodes
to keep the table synchronized. Since breaking changes to the public IR are
acceptable, keep provenance on inline nodes. Internal indexes may still support
efficient region lookup.

### Adopt pulldown-cmark or Keep Two Parsers

Replacing the parser requires revalidating current CommonMark, GFM, callout, math,
and rendering contracts. The identified gap is preserving and exposing parsing
information, so extend the existing parser. Provide examples and verification
results that let consumers remove their additional parser incrementally.

### Add a Mention-Specific API to oriha-core

Handle syntax, reserved words, and delivery semantics differ between applications.
Expose generic text regions and annotations, leaving mention policy to consumers.

## Consequences and Tradeoffs

- Retaining parsed documents and provenance increases memory use and parser
  complexity. Nested normalization, tables, footnotes, and character references
  that expand into multiple characters require particular attention.
- The IR change requires migration work for consumers with custom renderers.
- Traversal, annotation, and rendering can share one interpretation of Markdown.
  Consumers can remove the workaround that decorates only when two parsers
  produce matching visible text.
- The first version rejects overlapping annotations. Overlapping annotation
  categories and tracking annotations across edits require separate contracts
  and are outside this decision.

## Validation Required for Implementation Completion

1. **Provenance and source positions:** Distinguish literal, escaped,
   even-backslash, and character-reference at-signs. Verify original UTF-8 ranges
   for Japanese text, emoji, CRLF, tabs, nested quotes and lists, callouts, escaped
   table pipes (`\|`), and footnotes.
2. **Traversal and annotations:** Cover `@ali**ce**`, link labels, and repeated
   occurrences. Demonstrate consumer matching rules for code, URLs, email
   addresses, and HTML attributes. Do not turn email-boundary rules into an
   engine-specific mention policy.
3. **Safe application:** Reject invalid ranges, overlaps, and foreign region
   identifiers without partial application. Quotes, `<`, and `&` in text or
   attribute values must not break HTML. Source-authored HTML must not create
   annotation IR.
4. **Reuse:** Prove that rendering parsed documents does not rerun the parser.
   Verify hook counts and ordering, propagation of parse-stage diagnostics, and
   independent repeated rendering.
5. **Regression:** Run the 652 CommonMark examples, GFM / callout tests, JSON
   contracts, incremental rendering, HTML export, math, and diagram tests. Check
   compatibility of existing output when no annotations are supplied.
6. **Consumer example:** Provide a Rust example implementing example mention
   candidate extraction and decoration without another Markdown parser. Preserve
   the consumer's delivery logic and existing sanitization.
7. **Cost:** Record parsing-time and retained-memory changes for unannotated
   input, short messages, large documents, deep nesting, and numerous escapes or
   character references.

## Review and Disposition

Accepted on 2026-10-29 after inspecting the existing block and
inline parsers, normalization helpers, render pipeline, and extension hooks.
The existing parser can retain correspondence at each transformation; replacing
it or recovering positions from visible strings is unnecessary. The render
pipeline already has a natural boundary after document hooks and before block
rendering. Retaining owned state at that boundary permits parse reuse while
preserving prepared and incremental rendering contracts.

Implementation details resolved during review:

- Preserve the original caller source separately from effective hook input.
  Arbitrary hook replacement without correspondence produces generated metadata.
- Retain owned parse context and state; borrow the rendering engine's hooks and
  renderers only during each render. Documents need not retain an engine lifetime.
- Use coalesced mapping segments through normalization. Keep existing block spans
  and normalized source attributes in their current coordinate system.
- Region identities are opaque document-instance identities. Annotation fragments
  carry typed metadata and preserve one supplied logical identity across wrappers.

The validation section remains a completion gate for parsing, mappings, regions,
annotations and consumer examples.

Authoring this ADR does not implement Rust changes or release a public API.

### Implementation details

The Rust API uses `TextMapping` (`display`, `source`, `provenance`, and `exact`)
and `TextProvenance`. Only exact mappings permit bytewise projection of source
subranges; transformed segments retain their entire source extent. Temporary
parent-coordinate normalization maps stay internal. New block and table-cell
constructors support consumers without exposing those internal coordinates.

`RegionId` is opaque. Cloning or annotating a parsed document creates fresh region
identities. Applying further annotations to a result requires obtaining its regions
again; ranges overlapping existing annotations are rejected. `AnnotationMetadata`
contains the kind, logical occurrence identity, and typed `AnnotationAttributes`.
Data keys are restricted suffixes, and the engine reserves its identity/kind keys.
These details resolve contracts left open in the API sketch without changing scope.

See the [Rust migration guide](../guides/0001-source-aware-parsing.md),
[consumer example](../../crates/core/examples/annotate_mentions.rs), and
[Verification methods](../guides/verification.md). Acceptance
remains separate from passing all implementation verification gates.

### Implementation verification

The required implementation gates passed on 2026-10-29. The focused core/bridge
command passed 200 tests; the workspace command passed 369 tests. Both preserve
one explicitly ignored, pre-existing opt-in full-GFM-conformance test; it is not
counted as passing. All 652 CommonMark examples pass. See the
[Verification methods](../guides/verification.md) for requirement-by-requirement
evidence, exact commands, consumer/example results, and startup-delay observations.

### Designer-review corrections

Follow-up review removes repeated full mapping/node scans and
revalidates Code/Emoji as well as Text values after document hooks. Only footnote
bodies selected by the existing default renderer's main-flow reference order
participate in visible regions; unused definitions stay in IR but introduce no
annotation candidates. Annotation application uses the same selection/order.
See [Verification methods](../guides/verification.md)
for regressions, timing scope and final verification, retaining the original
implementation report and review failures.

### Adoption addendum: observable parsing outcomes

The Rust-only `ParsedDocument::parse_status()` returns `ParseStatus::Complete`,
`Incomplete`, or `Failed`. Source rejection and a caught parsing panic are Failed;
skipped blocks or nonempty container bodies at the nesting limit are Incomplete.
This status is captured at the engine parsing boundary, before after-parse hooks
can edit diagnostics or IR. Cloning and immutable annotations preserve it;
rendering neither changes it nor accumulates parse diagnostics. Ordinary syntax
recovery warnings, excluded region kinds, and unreferenced footnotes do not make
parsing incomplete. Existing diagnostic severities stay unchanged, with a Warning
`engine.max_nesting_depth_exceeded` added for omitted container content.

Depth starts at zero and increases for each quote, list-item body, or callout-body
child block parse, including direct nested-list recursion. Child parsers inherit
that depth rather than resetting it. A depth of one permits one such crossing.
`max_blocks` retains its existing **per block-parser invocation** scope: it counts
that invocation's direct blocks, not all descendants or list-item wrappers.
Definitions and trailing blank lines do not consume a block budget. Source-byte
limits apply to effective document input; footnote bodies currently parse inline
content and do not create child block parsers or consume nesting depth. Inline
formatting recursion is outside this block-container depth budget. Shared omission
reporting carries child limit failures to the outer result without changing these
budget scopes. Output-byte limits affect rendering, not parse completeness.

Complete means the engine omitted no effective input because of these parsing
limits and did not fail. It does not certify application candidate policy or
correspondence of arbitrary hook-authored content to original caller input.
Before-parse hooks may replace effective source; after-parse hooks may deliberately
change visible content. Consumers must control their hook profile and provenance
policy. No status fields or parsed state are added to foreign or JSON interfaces.

Extended-autolink discovery now reuses bounded lexical windows and candidate
validation, including failed candidates and absent delimiters. Literal runs
materialize correspondence together; immutable source text and line indexes share
storage across context clones. Annotation application avoids an extra discarded
IR clone. These changes preserve region identity rules and immutable documents.
See the [Verification methods](../guides/verification.md)
for exact tested source, intentional link interpretation corrections, fresh cost
comparisons, dependency resolution and consumer acceptance boundaries.
