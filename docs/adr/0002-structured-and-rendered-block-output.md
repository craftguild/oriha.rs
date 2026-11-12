# ADR-0002: Structured and Rendered Block Output

- Status: **Accepted**
- Date: 2026-11-01
- Baseline examined: oriha-core 0.2.5 / `5307c0ca5d3852e2eb538b594abdfadbd06a1ef6`
- Detailed contract: [Block derivation design](../design/block-derivation.md)
- [Verification methods](../guides/verification.md)

## Context

Content applications need semantic HTML per block, with renderer-owned parsing
and conversion. Other consumers should be able to construct their own UI from
typed blocks instead of receiving HTML. A single text string cannot preserve
inline formatting, links, image URLs, table cells or code metadata.

[ADR-0001](0001-source-aware-parsing-and-annotations.md) already exposes an owned
parsed document and source-aware IR. The current document renderer returns one
HTML document, while its `RenderedBlock` values describe source mapping rather
than block HTML. Re-parsing Markdown or splitting final HTML in the CMS would
duplicate producer responsibilities and risk inconsistent references and nesting.

The user selected both output modes for the producer. The CMS will use rendered
output and choose whether to retain explicit Null values. This authoring task
delivers design documents only; no implementation or adoption is authorized by
the presence of this ADR.

## Decision

### Two Output Modes from One Parsed Document

Add an independent `BlockOutputMode` with `Structured` and `Rendered` variants.
Expose input-based and parsed-document-based derivation through the default
engine and `OrihaEngine`. Use distinct typed result envelopes with a version,
mode, completion status and diagnostics. Reuse the parser and retained state.

Structured output is an owned tree with ID, kind, nullable plain text, typed
properties, rich inline content and ordered children. Materialize source-backed
values and retain references and all footnote definitions. The block and inline
mappings follow oriha-core's IR enums and preserve their nested semantics.

Rendered output consists of root block ID, kind and nullable semantic HTML,
without children. Nested containers render as complete HTML fragments. Append
visible footnotes as a generated root fragment. Return assets and diagnostics,
and generate no theme CSS. Preserve engine semantic markup, including classes
and data attributes.

### Producer Decisions and Consumer Policy

Match every actual BlockKind and InlineKind exhaustively in both conversion
paths and the built-in HTML/emission path. New variants must receive a deliberate
conversion or explicit non-representation decision before compilation succeeds.
Do not use wildcard fallbacks or infer Null from empty HTML.

The producer preserves explicit non-rendering decisions, initially blank lines
and link reference definitions when such nodes are present in IR. Structured
non-text nodes remain meaningful even when their text is null. The consumer
decides what to filter; CMS delivery policy does not belong in oriha-core.

### Hook and Compatibility Boundaries

Keep existing document APIs and serialized responses unchanged, including the
current source-map `RenderedBlock` type. Add new types for the new contract.
Parsing hooks run once; structured derivation does not render. Rendered
derivation executes block rendering and block hooks on isolated working state.

Complete-document finalization hooks and document-only decorations do not run
in the new block API because an arbitrary document rewrite cannot be mapped
back to fragments reliably. Their existing document-API behavior is preserved.
Add a provided output-policy method to BlockRenderer so existing custom
renderers default to HTML and can explicitly opt into Null without changing the
existing render method. The detailed design fixes this extension's behavior.

Apply retained parse limits and an aggregate output budget. Report incomplete
or failed derivation explicitly, with diagnostics and empty result collections.
Do not expose truncated output as complete. IDs belong to a parsed revision;
they are not persistent editor identities.

### Scope and Tracking

The new contract is Rust plus serializable DTOs. Native/Expo/WASM bridge exposure,
edit operations, stable block identity across edits and the CMS itself remain
outside this decision. Producer qualification and application adoption are
separate responsibilities; this ADR does not certify application integration.

## Alternatives Considered

| Alternative | Disposition |
| --- | --- |
| Expose only ID, kind and plain text | Loses content and attributes required to reconstruct supported documents. |
| Expose the current Rust IR directly as JSON | Leaks source offsets and parser normalization state; some payloads are not self-contained. |
| Reuse RenderMode for structured versus HTML | Mixes block representation with the existing styled/semantic HTML distinction. |
| Return HTML and children together | Duplicates subtree content and makes the normal rendering contract ambiguous. |
| Split Markdown and render each part | Breaks document-wide reference, heading and footnote state. |
| Split document HTML in the CMS | Adds another structural parser and ties the consumer to producer markup details. |
| Implement rendered now and structured later | Defers a selected capability and does not establish both IR projection decisions together. |
| Filter Null inside the producer | Imposes the CMS's delivery policy on every consumer. |

## Consequences

Consumers can choose typed structure or ready HTML while using one parser. The
CMS becomes a thin consumer of the rendered contract. Explicit Null remains
distinct from failure, non-text structured data and empty HTML.

The engine acquires owned public DTOs, exhaustive projections, a versioned
serialization shape and bounded output handling. These increase maintenance
responsibility. Compiler exhaustiveness enforces a decision, not its correctness;
all-kind fixtures and reference/layout regressions remain necessary.

Custom document-finalization transformations continue to require a document
API. Structured consumers must implement presentation for the rich data they
choose to display. Native bridges do not gain the feature automatically.

## Review and Acceptance Conditions

Review the complete mapping tables, signatures, wire examples, hook lifecycle,
failure/Null distinction, limits and existing API compatibility in the linked
design. Resolve concrete design defects and record a disposition before updating
this ADR and its index.

Future implementation must prove variant-addition compile failures against the
actual IR, all-kind structured fidelity, parse reuse, repeated-render isolation,
semantic-body equivalence, nested HTML without duplicate output, valid assets,
and bounded failure behavior. Those are future acceptance criteria, not test
results from this document-authoring task.

The target is a complete production v1 of the scoped Rust API, with a real
compiled consumer, release-build verification, compatibility coverage, dependency
policy checks and usable API/migration documentation. This does not imply a
package-major-version change or new bridge/product scope.

Response-time, throughput, high-load/stress, comparative benchmarks and
process-memory/RSS measurements are explicitly excluded from completion.
Resource limits, bounded work, correctness, security and error handling remain
mandatory. Do not use missing performance measurements to delay an otherwise
complete v1, or their exclusion to excuse incomplete or unsafe code.

## Status and Evidence

Accepted on 2026-11-01 after reviewing the actual IR, retained
parse state, source normalization, renderer selection and consumer boundary.
Acceptance does not establish implementation completion or consumer adoption. A later invocation
of the linked goal prompt authorizes its scoped review and implementation;
merely storing the prompt does not start that work. Implementation receipts must
be recorded separately. V1 completion makes no measured performance claim.

### Review decisions

The typed paragraph payload also exposes `tight` to preserve tight-list
semantics. Ordered list `start` is a normalized decimal string (or null for
unordered lists), matching HTML without integer overflow on programmatic IR.
Empty and duplicate IDs fail derivation; rendered working identities are checked
again after hooks. The fixed, bounded failure envelope is the minimum response
when a configured budget cannot contain any diagnostic envelope. These clarify
the proposed contract without changing producer/consumer ownership.

The existing additive renderer method and the explicit
document-finalization exception preserve ordinary document API behavior.

### Implementation-driven decisions

Rust rendered entries use `RenderedBlockContent::{Html, Null}` and an `html()`
accessor, preventing contradictory HTML/Null states. Serialization still exposes
the specified nullable `html` and `nullReason` fields. This refines the internal
sum type proposed in the design without changing the wire examples.

A real-process adversarial test exposed stack exhaustion in the existing inline
image-label parser. Recursive label parsing/link validation now has a 64-call
safety bound; combined inline formatting and programmatic tree depth is bounded
at 256. Exceeding safety bounds fails explicitly, rather than returning a
truncated complete document or aborting. Ordinary supported markup and all
existing parse/source-map/JSON contracts remain regression gates. The inline
parser and renderer were split into cohesive modules to keep changed
implementation files within the code-size target.

See [API and migration guidance](../guides/0002-block-output.md) and the
[Verification methods](../guides/verification.md) for actual implementation
evidence and the unreleased source identity.

### Independent production-review corrections

Independent review identified an inherited inline HTML bypass of `raw_html=false`.
The producer now escapes authored inline HTML in context-aware and context-free
rendering, including footnotes and nested inline content. This intentional
security correction applies to ordinary document output as well as block output;
API/wire contracts remain unchanged. Explicit raw-HTML enablement and GFM tag
filtering retain their prior behavior, while structured data keeps authored HTML.

Independent review identified false output-budget failures from charging empty
resource-delta wrappers per root. Charge serialized resource/diagnostic items
instead; empty deltas consume no budget. Exact final aggregate counting and
bounded construction remain required. Review regressions cover exact and
one-byte-below limits from zero through 500 roots, with all combinations of
new assets and diagnostics, and generated math/footnote resources.
