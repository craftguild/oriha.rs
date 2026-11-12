# ADR-0001 addendum: bounded transient memory

Date: 2026-10-29. Public ownership and APIs are unchanged.

The repeated consumer profile has three bodies below 65,536 source bytes:
5,040 plain lines with one candidate, 12,000 plain candidates, and 5,000
candidates split by strong formatting. An 8 MiB output cache does not bound
parser, region, annotation, or renderer working memory. Two active bodies can
overlap those allocations; queued bodies and allocator-retained pages add costs
outside the cache accounting.

The repository-owned `repeated_consumer` probe measures these separately without
hooks. In the initial serial run, formatted parsing retains 8,576,297 requested
bytes and peaks at 13,571,208 additional requested bytes during that stage.
Multiline parsing retains 4,987,313 and peaks at 8,884,929 bytes. Its region
projection allocates another 2,959,101 retained bytes; immutable annotation has
a 6,570,879-byte temporary peak and rendering a 4,522,497-byte temporary peak.
These are allocator requests, not RSS. Full traces and final repeated results
belong in the measurement report.

## Decision

Reduce two internal copies while retaining all source correspondence:

1. Consume inline tokens as they are emitted, building the same container tree.
   Keep at most one pending node so adjacent text merges and trailing-space
   trimming have exactly the previous token-boundary behavior. Keep delimiter
   pairing, mismatched-container handling, EOF recovery, and child ordering
   unchanged. The full token vector no longer overlaps the final inline tree.
2. Borrow block payloads in the default renderer rather than cloning every
   paragraph/heading/table payload. The renderer still receives its separate
   mutable document working copy. A callout may retain a small local payload
   clone where its helper requires a mutable whole block.

This does not change `ParsedDocument`, `InlineNode`, `TextRegion`, mappings,
provenance, region identities, annotation validation, completeness, output
fallback, hooks, or custom renderer ownership. There is no parsed-document cache,
application candidate limit, metadata deletion, or new Markdown parser. It does
not change strikethrough delimiter behavior or consumer region policy.

## Verification and remaining costs

Freeze allocation-instrumented and uninstrumented binaries before and after the
change. Repeat serial, two-active, and sixteen-queued schedules three times;
retain per-body stage traces, per-batch process requests, independently sampled
RSS, and cache-drop/worker-join quiescence. Measure routing projection plus scan
separately from rendering. Compare output/candidate/annotation invariants and run
the source-annotation, public CommonMark, extension, and robustness gates.

Eager owned region projections, immutable annotated-document copies, the render
working copy, retained normalized source, mapping vectors, cache capacity, and
system allocator retention remain measurable costs. Changing those public
ownership contracts would need another design decision. RSS staying elevated
after requested allocations are freed is not by itself evidence of a leak.
