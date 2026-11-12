# Architecture Decision Records

ADRs record context, alternatives, decisions and consequences. `Proposed` means
under review, rather than accepted or implemented. Preserve numbers, dispositions
and supersession when moving documents. Track implementation work in Beads.

| ADR | Status | Decision |
| --- | --- | --- |
| [0001](0001-source-aware-parsing-and-annotations.md) | Accepted | Source-aware parsing and generic text annotations |
| [0002](0002-structured-and-rendered-block-output.md) | Accepted | Structured trees and semantic HTML blocks from one parsed document |
| [0003](0003-responsive-image-presentation.md) | Accepted | Opt-in typed responsive image presentation with renderer-owned HTML |
| [0004](0004-standards-v0.5-integration.md) | Accepted; worktree integration | Pinned standards v0.5.0 integration and local completion boundary |

The [ADR-0001 memory addendum](0001-memory-addendum.md) retains the associated
memory design. Public implementation contracts and examples are in the
[source-aware guide](../guides/0001-source-aware-parsing.md),
[block guide](../guides/0002-block-output.md),
[responsive image guide](../guides/0003-image-presentation.md) and
[block design](../design/block-derivation.md).

Original implementation prompts and experiment receipts are privately archived.
Their presence/removal does not authorize work or alter an ADR's disposition.
[Verification methods](../guides/verification.md) explain the reproducible public
checks. Correctness, resource safety and compatibility remain required;
performance campaigns are not added as release gates for ADR-0002/0003.
