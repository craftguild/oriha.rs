# ADR-0003: Typed Responsive Image Presentation

- Status: **Accepted**
- Date: 2026-11-01
- Extends: [ADR-0002](0002-structured-and-rendered-block-output.md)
- [Verification methods](../guides/verification.md)

## Context

Applications need responsive images while consuming semantic HTML blocks.
They own local file inspection, variant generation,
format negotiation and caching. The renderer owns HTML structure and escaping.

The existing parsed-image hook can replace a destination but cannot provide
typed srcset candidates, sizes, intrinsic dimensions and responsive layout.
Default standalone/inline images also use a 300-pixel width when unspecified.
Patching final HTML in the consumer would couple it to renderer implementation
details and repeat the escaping and image-detection rules.

The original ADR publication was documentation only. The separately authored
implementation scope is separate; publishing an ADR does not implement
the extension or assert verified consumer adoption.

## Decision

Accepted after source review on 2026-11-01. The existing
contextual inline path covers headings, links, containers, table cells and
footnotes; standalone figures can use the same image emitter. Retained
derivation already isolates IR and excludes document finalizers. The extension
can preserve those contracts without changing structured data or bridges.

The hook borrows immutable prepared presentation data, avoiding a per-render
copy before budget checks. `ImageRenderRequest` carries the parsed destination,
title, alt nodes and `Standalone`/`Inline` placement. Validated presentation
fields are private. Width candidates require nonblank explicit sizes. Layout
is an explicit `ImageLayout::Fixed` or `Responsive` choice. Responsive CSS fits
the parent, uses automatic height and intersects authored width/height URL values with supplied intrinsic
dimensions as maximum display bounds. Intrinsic attributes describe pixels
independently. The clarified layout contract requires a small original
to retain its display size, shrinking only for narrower parent/author bounds.
Checked attribute appends run under the retained output bound as well as the
existing rendered-block scope; final aggregate accounting remains unchanged.

These values are a transient prepared-data lookup, with no persistent domain
lifecycle or business line items. Transition-history and Header-Detail storage
baselines do not apply. Implementation acceptance is separate from verification;
the issues above track production-v1 implementation and delivery.

### An Opt-In Typed Render Hook

Add a provided image-presentation method to `RenderHook`, defaulting to no
override. Resolve it at semantic image emission, after parse-stage hooks, using
the parsed destination, title, alt content, image placement and render context.
The first hook returning a presentation in registration order supplies it; no
override uses the existing built-in behavior unchanged.

The returned presentation carries a source URL, ordered typed candidates with
positive pixel-width descriptors, an optional sizes value, intrinsic dimensions,
and an explicit responsive-layout choice. It is not raw HTML, an arbitrary
attribute map or a preassembled srcset string. Do not add CMS route names or
filesystem objects to these public types.

Candidate widths must be positive and unique. Intrinsic dimensions must be a
positive pair when supplied; responsive candidates require an explicit sizes
value. Reject invalid presentation construction with a typed error rather than
silently correcting descriptors or publishing contradictory attributes. The
consumer remains responsible for the real pixels matching those descriptors;
the renderer cannot infer image dimensions from network or filesystem content.

The renderer emits each candidate using its URL sanitizer and HTML attribute
escaping. Encode whitespace and data commas within candidate URLs rather than
escaping a complete user-supplied srcset string. Preserve syntax-bearing URL
delimiters: a `data:` URL's first comma separates its metadata from its payload
under [RFC 2397](https://www.rfc-editor.org/info/rfc2397/), and must remain literal.
The srcset URL token ends at ASCII whitespace; this internal comma does not
separate candidates. Encode later payload commas, including trailing ones.
Preserve percent escapes and valid IPv6 authority brackets, and retain
unsafe-scheme rejection.
Treat sizes
as an escaped attribute value, never markup. Follow the
[HTML srcset requirements](https://html.spec.whatwg.org/multipage/images.html#srcset-attributes).

### One Semantic Image Path

Use the extension for standalone figures and contextual inline images,
including linked images, nested containers, table cells and footnotes. Preserve
alt text, title/caption semantics, CSS classes, block IDs, source correspondence
and existing semantic wrappers.

An opted-in responsive image fits its parent, preserves aspect ratio and emits
the supplied intrinsic width/height. Its figure and image must not retain a
default 300-pixel width. Existing explicit display dimensions remain display
bounds rather than crop instructions. Supplied intrinsic dimensions cap both
display axes, while automatic width/height preserve aspect ratio when a maximum
height or narrower parent applies. The source is never enlarged by default.
This corrects the initial density-driven upscaling identified during browser
qualification and authorized as a producer repair. The caller supplies layout-specific sizes;
the renderer does not assume an article width or manufacture candidate URLs.

Apply identical presentation semantics in the document and rendered-block APIs,
including derivation from a retained parsed document. Render-stage presentation
must not re-run parse hooks, mutate retained IR or leak state between repeated
renders. Raw HTML policy and whole-document finalization-hook boundaries remain
as defined in ADR-0002.

### Compatibility and Responsibility Boundaries

Existing `RenderHook` implementations continue to compile through the default
method. With no override, existing HTML, image styles and serialized responses
remain unchanged. Do not change `InlineKind`, structured-output DTOs, schema
version, Null decisions or resource contracts to add presentation metadata.
Structured derivation does not invoke this render hook.

The engine performs no image reads, HTTP requests, decoding, transformation or
format negotiation. The application prepares file information, dimensions,
candidate selection and URLs before rendering. Its hook supplies prepared data
through a lookup, without file/network access or image generation during the
render call. Codecs, origins, quality settings, width presets, browser Accept
policy and cache persistence remain consumer concerns. The extension emits HTML
attributes; it does not turn the renderer into an image-file generator.

Account for the expanded HTML using the existing rendering/output budgets.
Do not exempt new attributes from bounded output or return truncated rendered
blocks as Complete. No bridge exposure, arbitrary HTML replacement hook,
frontend CSS system or image-processing dependency is added by this decision.

### Repository Ownership

This repository owns HTML generation and producer qualification. Applications
own image processing, delivery and integration verification. Producer evidence
does not certify those application responsibilities.

## Alternatives Considered

| Alternative | Disposition |
| --- | --- |
| Consumer rewrites final HTML | Repeats semantic structure, URL handling and escaping outside the producer. |
| Put responsive fields in Markdown IR/structured DTOs | Mixes deployment presentation with parsed content and expands unrelated contracts. |
| Raw img attributes or a srcset string | Makes descriptor validation and safe serialization harder to centralize. |
| Replace all block renderers in the CMS | Forces the consumer to maintain figure, inline, table and footnote behavior. |
| Embed CMS/image-processing policy in oriha-core | Couples a generic renderer to one application's storage and delivery rules. |
| Change default images globally | Alters existing consumers when only opted-in responsive presentation is required. |

## Consequences and Verification

Consumers can supply delivery-specific image presentation while the renderer
retains ownership of semantic HTML. The public hook and typed presentation add
a compatibility surface, and all semantic image paths must share its behavior.
The renderer can validate representation, but cannot certify that a consumer URL
serves the declared pixels.

The implementation must cover standalone/inline/linked/nested/table/
footnote images through both document and rendered-block APIs, direct input and
retained parse. Verify registration order, no-override compatibility, malformed
presentation rejection, correct srcset delimiter encoding, unsafe URLs, IPv6,
escaped alt/title/sizes, repeated-render isolation and output-limit behavior.
Existing structured output must remain equal and must not invoke presentation
resolution. Include a compiled real consumer using the typed extension.

Run the repository's required compatibility, dependency and release gates and
the code-size assessment for implementation changes. Consumer
pixel/HTTP/browser delivery checks belong to application integration. Latency,
throughput, high-load/stress tests and process-memory/RSS benchmarks are not
completion criteria. Acceptance alone does not establish implementation or
consumer qualification. See the [Rust API guide](../guides/0003-image-presentation.md)
and [Verification methods](../guides/verification.md) for actual producer
results, source/artifact identity and limits. Consumer pixel/HTTP/browser
integration is not claimed by this producer evidence.
