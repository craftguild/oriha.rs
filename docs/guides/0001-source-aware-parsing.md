# Migrating Rust consumers to source-aware parsing

ADR-0001 changes the public Rust inline IR. JSON, prepared-layout, patch, HTML
export, UniFFI, WASM, and Expo contracts retain their existing formats. The parsed
IR and annotation APIs are available only to Rust consumers.

## Parse once and reuse the result

```rust
use oriha_core::{parse, render_parsed, RenderInput};

let parsed = parse(RenderInput {
    markdown: "Hello @ali**ce**".into(),
    ..Default::default()
});
assert_eq!(parsed.source(), "Hello @ali**ce**");
let first = render_parsed(&parsed);
let second = render_parsed(&parsed);
assert_eq!(first.html, second.html);
```

`OrihaEngine::parse` and `OrihaEngine::render_parsed` provide the same
operations with custom limits, hooks, and renderers. Existing `render(input)`
uses the same parsing and rendering stages. Rendering a parsed document uses its
stored effective settings and never parses Markdown again. It has no argument
for replacement source or a different parsing feature profile.

`before_document_parse` and `after_document_parse` execute during parsing only.
Their effective input, render context, diagnostics, assets, and render state are
retained. Block hooks and finalization hooks execute on each render, in their
existing order, using a working copy. Changes during one render do not accumulate
in the parsed document. Hooks themselves can retain external mutable state;
applications remain responsible for the behavior of that external state.

Parse-only calls enforce source limits and convert engine panics into diagnostics.
Inspect `ParsedDocument::diagnostics()` even when no HTML is requested. The
original caller input remains accessible through `source()` when a hook replaces
the effective Markdown. Replacement input without explicit correspondence has
`Generated` provenance.

## Match kinds through `InlineKind`

Previously, a consumer matched or constructed `InlineNode::Text(value)`. Match
`node.kind` now and retain the node's `source_mapping` when splitting text:

```rust
use oriha_core::ir::{InlineKind, InlineNode};

let generated = InlineNode::generated(InlineKind::Text("application text".into()));
if let InlineKind::Text(value) = &generated.kind {
    assert_eq!(value, "application text");
}
```

All former variants moved to `InlineKind`. `InlineKind::Annotation` is new.
Custom renderers should recurse through its children when computing plain text,
heading anchors, or visible-text statistics, and render its typed metadata when
decorating HTML. The built-in renderer already handles these cases.

`InlineNode::generated(kind)` and `InlineKind::into()` construct generated nodes.
Parser-created text nodes carry `TextMapping` segments. Do not copy a text node's
old mapping onto unrelated replacement text. After document hooks, changed Text,
Code, or Emoji values with an unchanged mapping, missing coverage, or invalid correspondence become
`Generated`. To supply explicit correspondence, provide complete valid segments
and describe a transformation with the appropriate provenance and `exact: false`.

Block normalization state is internal. Consumers constructing blocks or table
cells should use `BlockNode::new(id, kind, span)` or
`TableCellNode::new(source, children)`, then update their public fields.

## Read original-source correspondence

Every new mapping range is a zero-based, half-open UTF-8 byte interval. Text
mapping `display` ranges index the text node; region mappings index the region's
visible `text`. `RegionMapping::node` and `node_range` connect a region segment to
its text node. `source` ranges index the complete original caller input.

- `Literal` describes authored text.
- `Escape` retains the backslash and escaped character's source extent.
- `CharacterReference` retains the whole entity, including transformations into
  multiple Unicode code points.
- `Normalized` describes transformations such as expanded tabs, normalized line
  endings, and emoji shortcodes. Emoji nodes retain their shortcode extent.
- `Generated` has no original-source correspondence.

Only `exact` segments permit bytewise source subrange projection. For a segment
with `exact: false`, a visible subrange retains the complete source extent. Source
and display lengths can differ, and multiple segments can refer to the same source
extent. Never infer a position by searching the source for visible text.

Existing `BlockNode.span`, heading spans, and normalized block attributes keep
their previous semantics, including local container coordinates. Existing output
`source_map` values remain line-and-column `SourceRange` values. They are separate
from these new inline byte mappings.

## Match regions and apply annotations

`text_regions()` joins text across emphasis, strong emphasis, strikethrough, and
link labels. Destinations and titles are excluded. Blocks, individual table cells,
soft/hard breaks, code, math, HTML nodes, images, emoji nodes, and footnote references
end a region. Image alt text, generated callout labels, and metadata are excluded.
Only rendered footnote bodies are included, in main-flow reference order. Unused
definitions remain in IR but produce no visible region. Nested-only references
follow the default renderer's existing reference snapshot behavior. Text between
source HTML tags remains eligible in separate regions bounded by the HTML nodes.

A `RegionId` belongs to exactly one `ParsedDocument` instance. Cloning or applying
annotations creates a new instance with fresh IDs. Obtain regions again before
applying further annotations. These IDs are unrelated to incremental stable IDs
and do not survive source edits.

```rust
use oriha_core::{
    AnnotationAttributes, AnnotationMetadata, TextAnnotation,
};
let region = parsed.text_regions().next().unwrap();
let decorated = parsed.apply_annotations(&[TextAnnotation {
    region: region.id,
    range: 6..12,
    metadata: AnnotationMetadata {
        kind: "mention".into(),
        id: "occurrence-1".into(),
        attributes: AnnotationAttributes {
            classes: vec!["mention".into()],
            ..Default::default()
        },
    },
}]).expect("valid annotation");
```

The complete request is validated before any result is created. Empty, reversed,
out-of-bounds, non-UTF-8-boundary, overlapping, previously annotated, and foreign
ranges fail atomically. A range cannot extend outside its region. Adjacent
annotations are allowed. One annotation may span formatting and link boundaries
and any mixture of provenance. Its intersected text fragments receive the same
logical `id`, preserving valid HTML nesting. The original document is unchanged.

Kinds and identities must be nonempty. Classes are nonempty tokens without
whitespace or control characters. Data keys are suffixes beginning with an ASCII
lowercase letter and containing only lowercase letters, digits, and hyphens.
The reserved `oriha-annotation-id` and `oriha-annotation-kind` keys cannot be supplied.
For example, key `recipient` renders as `data-recipient`. Text and attribute values
are escaped by the engine. Arbitrary HTML strings are unnecessary.

Source-authored HTML remains `InlineKind::Html`; it never becomes annotation IR.
A source author can still write HTML attributes that resemble annotation output.
Retain existing HTML sanitization, and derive trusted delivery metadata from the
application's annotation request rather than trusting attributes in arbitrary HTML.

## Consumer example and measurements

Run the self-contained example:

```sh
cargo run -p oriha-core --example annotate_mentions --locked
```

[annotate_mentions.rs](../../crates/core/examples/annotate_mentions.rs) distinguishes
literal, escaped, even-backslash, and character-reference at-signs. It finds a
formatted mention without another Markdown parser. Handle syntax, email/URL
boundaries, case normalization, reserved words, and member lookup all live in the
example's consumer function. Delivery authorization and sanitization remain the
consumer's responsibility.

See [Verification methods](verification.md) for
parsing-time and retained-allocation methods, results, and tradeoffs.

The [Verification methods](verification.md)
records indexed mapping/hook processing, footnote visibility and mapped Code/Emoji
validation. These corrections do not change the public annotation request shape.

## Adoption gates for routing consumers

The APIs above provide the machinery to replace a second Markdown parser. They
do not establish equivalence with a consumer's existing candidate policy or prove
that a particular workload meets its latency budget. The
[Verification methods](verification.md)
records the evaluated source revision, observed differences, and verification scope.

### Decide candidate scope explicitly

`text_regions()` excludes image alt text, includes rendered footnote bodies,
and separates text on either side of an emoji node. For example, `word:rocket:@bob` exposes separate `word` and `@bob`
regions. A lexer that checks only the previous character within a region can now
accept that occurrence. A region boundary does not itself define the consumer's
word-boundary or delivery policy.

Define delivery eligibility and visible decoration separately, using the same
Markdown interpretation and common lexical/provenance rules. Delivery-eligible
source text can exceed the text available for visible annotation. The default
annotation scope does not include image alt text; consumers may read that field
from the parsed IR without adding it to `text_regions()`. oriha-core does not
resolve recipients.

A consumer may select these four cases as delivery candidates, subject to its
explicit handle, escape, membership and transactional recipient rules. This is
an example policy, separate from the producer's visible annotation contract:

| Source occurrence | Delivery candidate | Viewer annotation target |
| --- | --- | --- |
| `![description @bob](../migrations/image.png)` | `bob`, from authored image alt | None when alt is not displayed. |
| `![alt](../migrations/image.png)@bob` | `bob`, with an image boundary before it | The displayed `@bob`. |
| `@bob` in a referenced, rendered footnote body | `bob` | The displayed footnote occurrence. |
| `word:rocket:@bob` | `bob`, with an emoji boundary before it | The displayed `@bob`. |

Use `ParsedDocument::document()` and `InlineKind::Image { alt, .. }` for authored
alt text. Its inline children carry the same original-source `source_mapping`
and provenance metadata, including correspondence through normalized containers.
Traverse formatting according to the shared lexical policy and keep excluded
kinds/fields excluded. Restrict traversal to eligible content; blindly scanning
every retained footnote definition would include unused or nested-only bodies
that this decision does not approve. Image destinations and titles are separate
fields, not alt text.

Keep application delivery occurrences separate from `TextAnnotation` requests.
Create annotations only for eligible occurrences that have actual display
regions; an alt-only delivery candidate does not need a `RegionId` or an invented
visible span. Viewer annotation availability must not alter frozen recipients.
A viewer that does not display images can therefore deliver an alt-only mention
without showing a visible mention. Account for this consequence when selecting
application policy.

These four examples do not approve expanding matching to code, URL destinations
or titles, HTML attributes, generated content, unused definitions, or nested-only
footnote references. Retain established exclusions until a separate decision
changes them. Character-reference correctness and bounded resource behavior
remain producer requirements.

### Distinguish an empty result from incomplete parsing

`parse` returns a `ParsedDocument` even when parsing fails or stops at a limit.
`text_regions()` can therefore expose an empty or partial set. Use the Rust-only
status guard before treating that set as authoritative for routing:

```rust
use oriha_core::{ParseStatus, ParsedDocument};

fn require_complete(document: &ParsedDocument) -> Result<(), ParseStatus> {
    if document.parse_status() != ParseStatus::Complete {
        return Err(document.parse_status());
    }
    Ok(())
}
```

After this guard succeeds, an empty application candidate list is successful
extraction without candidates; `Err(Incomplete)` or `Err(Failed)` means extraction
is unavailable. The runnable example illustrates this guard and a visible-region
lexer. Broader delivery extraction, including image alt, is a separate consumer
policy. Inspect `diagnostics()` for details:

| Diagnostic | Current severity | Routing implication |
| --- | --- | --- |
| `engine.max_source_bytes_exceeded` | Error | Source was rejected. |
| `engine.panic` | Error | Parsing failed. |
| `engine.max_blocks_exceeded` | Warning | Remaining blocks were skipped. |
| `engine.max_nesting_depth_exceeded` | Warning | Nonempty nested content was skipped. |

Checking only `Severity::Error` misses block truncation. Return a distinct
unavailable/incomplete outcome for known parse failures and omissions; do not
convert it into a successful empty candidate set. Keep other diagnostics under
an explicit consumer policy.

The earlier evaluated revision `fb716e7` lacked this guard and could silently omit
quote/list text at depth zero. The remediated implementation reports Incomplete
and a warning. Status is captured before after-parse hooks and survives cloning,
annotations and repeated rendering, even if a hook clears diagnostics. It describes
engine parsing of effective input, not arbitrary hook-authored content or its
correspondence to the original caller source. Control the hook profile as part of
consumer integration. Ordinary recovery warnings do not imply Incomplete.

With a **64-level container limit**, a body containing a 66-level quote and
`@hidden` is a rejection case, not a successful empty candidate set. On depth
overflow, parsing reports `Incomplete`. An application should preserve the
author's input and report that quote/list nesting must be reduced before relying
on extracted recipients. Rendering fallback and persisted recipients remain
application responsibilities.

The existing `max_nesting_depth` implementation counts shared quote/list/callout
container crossings, so mixed-container boundaries also need consumer integration
coverage. This decision does not introduce a separate inline-formatting limit or
approve every other budget in the measured profile. The producer already checks
63/64/65 quote levels; consumer tests must also verify the no-commit/no-delivery
outcome rather than only the parser status.

Choose and test parsing limits for the accepted routing input domain. A Viewer
budget such as 4,096 blocks can truncate a valid 54 KB message. Presentation and
routing may use different resource budgets, while keeping the intended Markdown
interpretation consistent. Exercise limit boundaries and the unavailable outcome
in integration tests.

For example, the measured profile uses 65,536 source bytes, 4,096 blocks, depth
64, and 262,144 output bytes;
these profile values are evaluation settings rather than a blanket
approved application resource policy. Source limits apply to effective input.
Block limits retain their per-child-parser scope (direct blocks, not a global
descendant count). Container
depth now carries through child parsers: zero omits nonempty quote/list/callout
bodies, one permits one container crossing, and deeper containers require larger
budgets. Old child-depth resets could bypass positive limits; enforcing them may
omit content that previously rendered. Footnotes currently have inline bodies
and do not consume block-container depth; unused footnotes remain an ordinary
visibility rule. Inline formatting recursion has a separate implementation and
is outside this depth budget. Trailing blanks and collected definitions do not
consume blocks. Rendering output limits do not change parse status; check render
diagnostics separately when HTML availability matters. Choose limits from actual
workloads and bound document/annotation caches by measured memory costs.

### Measure the accepted workload and pin the source

The [Verification methods](verification.md)
records fresh paired multi-size timings, allocation profiles, complete pipeline
measurements, exact source identities, and compatibility gates. Extended-autolink
discovery now bounds lexical work and reuses failed-candidate validation. A later
whitespace-separated `@` or `:` no longer suppresses an earlier email/URL; URLs
with `//` keep at-signs in their path as URL text. These are intentional GFM
interpretation corrections with regressions. Use measured release workloads to
set an application budget before adoption. Reuse parsed results where applicable;
parsing, scanning, annotation, rendering, and retention perform different work.

Update the exact dependency revision and migrate public Rust IR usages together.
The evaluated source and the older `ae1846d` pin both identify the crate as 0.2.5;
that version number alone cannot identify this API. Validate the actual consumer
dependency graph, candidate/decoration behavior, sanitization and trusted metadata,
resource controls, and supported platforms before removing the second parser.
