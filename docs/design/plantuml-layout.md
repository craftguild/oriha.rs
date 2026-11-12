# PlantUML Layout Implementation Notes

Date: 2026-05-02
Status: implementation direction

## Context

The `oriha-plantuml` crate currently focuses on PlantUML sequence diagrams. The
parser already recognizes a useful subset of sequence syntax, and the fixture
corpus from the official PlantUML sequence documentation gives us a practical
compatibility baseline.

The current layout implementation is intentionally simple: participants are
placed in declaration order with a fixed horizontal gap, and timeline events are
placed with a fixed vertical gap. This is enough for early parser and renderer
work, but it will not scale to more realistic diagrams where labels, notes,
groups, boxes, activations, and long participant names need to negotiate space.

## Core Decision

Layout algorithms must be selected per diagram model.

PlantUML diagram families do not share one layout problem:

- Sequence diagrams use a time-ordered vertical axis. The source order is the
  semantic timeline, so the main layout problem is horizontal spacing,
  participant columns, message labels, notes, groups, boxes, and activation
  bars.
- Class, component, usecase, and deployment diagrams are node-link diagrams.
  These are closer to layered graph layout, orthogonal edge routing, or force
  layout problems.
- Activity and state diagrams are flow graphs, but their branch, merge, loop,
  and region semantics matter more than generic graph rendering.
- WBS and mindmap diagrams are tree layouts.
- Gantt and timing diagrams are time-scale layouts.
- JSON, YAML, packet, and table-like diagrams are tree, grid, or table layouts.

The shared layer should provide geometry primitives and rendering support. The
diagram-specific layer should own the layout planner.

```text
PlantUML source
  -> diagram-specific parser
  -> diagram-specific IR
  -> diagram-specific layout planner
  -> shared geometry primitives
  -> shared renderer / style backend
```

## What To Share

The reusable layout/rendering foundation should stay below diagram semantics:

- text measurement
- box, line, path, polygon, and text primitives
- spacing constraints
- collision checks
- coordinate normalization
- SVG emission helpers
- theme and style tokens
- visual summary and fixture comparison utilities

This keeps look and feel under our control while still allowing each diagram
type to use the right placement algorithm.

## Sequence Layout Direction

Sequence diagrams should not use a general Sugiyama layout as the main engine.
Sugiyama is useful for graph-like diagrams, but sequence diagrams have a fixed
semantic timeline. The better direction is a sequence-specific planner with a
constraint-based horizontal pass.

Recommended pipeline:

1. Measure

   Compute required sizes for participant heads, message labels, note text,
   group labels, dividers, references, and boxes.

2. Timeline layout

   Preserve source order. Assign each event a row and row height. Message rows,
   multiline notes, group headers, dividers, delays, and title/header/footer
   events can use different vertical spacing.

3. Horizontal constraints

   Treat participants as columns. Build constraints such as:

   - adjacent participant gap must fit participant head widths
   - message label between `A` and `B` must fit between their columns
   - self messages reserve space on one side of the participant
   - `note over A, B` must fit within or expand the `A..B` span
   - boxes must cover their participant range plus padding
   - group/ref/divider blocks may require full-width or participant-span width

4. Optional ordering

   Keep declaration order by default. If no explicit ordering is present, a
   later planner may use graph-style heuristics to reduce long message spans or
   crossings. This is the part where Sugiyama-style crossing minimization can be
   useful, but it should not override PlantUML sequence semantics.

5. Routing and rendering

   Once coordinates are known, route messages, activation bars, notes, groups,
   boxes, and participant heads/tails. Rendering remains style-driven and should
   not own placement decisions.

## Sugiyama Usage

Sugiyama is still useful, but not as a universal PlantUML layout engine.

Appropriate uses:

- class diagrams
- component diagrams
- deployment diagrams
- usecase diagrams
- parts of activity/state diagrams after semantic lowering
- optional participant ordering heuristics for sequence diagrams

Less appropriate uses:

- sequence timeline placement
- gantt or timing diagrams
- WBS/mindmap tree layout
- table/grid-like diagrams

For now, avoid adopting a Graphviz-oriented layout crate as the core dependency
for sequence rendering. Crates such as `layout-rs` provide DOT parsing and SVG
rendering machinery, but the project needs direct control over PlantUML
look-and-feel, SVG structure, and fixture summaries. A small internal
constraint planner is a better first step.

## Fixture Strategy

The current official sequence fixture corpus is useful for two distinct checks:

- parser and IR coverage
- visual/structural rendering checks

For rendering, exact SVG equality is not the primary goal. Prefer normalized
summaries and layout invariants:

- participant order is stable
- message Y positions are monotonic
- message endpoints preserve left/right relationship
- visible source text is retained
- long text is split into multiple text lines or spans
- notes and groups cover the intended participant span
- supported/unsupported fixture counts are explicit and tracked

PlantUML's own tests provide useful patterns:

- `FileFormat.DEBUG` emits drawing commands such as `RECTANGLE`, `LINE`,
  `TEXT`, `POLYGON`, and coordinates. This can become a stricter golden source
  than raw SVG for selected cases.
- SVG tests normalize XML and strip unstable attributes before comparison.

The practical path is to keep the official SVG pair view for human inspection,
and optionally add `*-golden.debug.txt` fixtures for selected hard layout cases.

## Compatibility Scope

Not every official sequence example should be treated as an implementation
priority.

Examples that are primarily style compatibility should not block parser and
layout progress. For example, official sequence fixture `065` is structurally
similar to `064`, but adds detailed `skinparam sequence { ... }` styling and
handwritten rendering. The unsupported behavior there is a style-block parsing
gap, not a sequence layout gap.

Near-term classification should distinguish:

- supported
- unsupported syntax
- parsed but style ignored
- rendered with known visual differences

This prevents style compatibility from being mistaken for core parser or layout
correctness.

## First Implementation Slice

The first layout improvement should stay narrow:

1. Add a text measurement abstraction with deterministic fallback metrics.
2. Derive participant head width from measured participant labels.
3. Derive message row height from message label line count.
4. Expand participant gaps based on message label width.
5. Expand note/group/box spans based on measured content.
6. Add invariant tests over supported official fixtures.
7. Keep the existing renderer API stable.

This slice improves real output without committing to a full graph layout engine.

## Non-Goals For The Next Slice

- Reimplement all PlantUML styling.
- Match PlantUML SVG output exactly.
- Adopt Graphviz as the layout backend.
- Use Sugiyama for sequence timeline placement.
- Support every official example before the parser and layout boundaries are
  stable.

## Suggested Module Shape

```text
crates/plantuml/src/
  layout.rs                         current public sequence layout surface
  layout/
    mod.rs
    measure.rs                      text and primitive measurement
    constraints.rs                  simple horizontal constraint solver
    sequence.rs                     sequence-specific planner
    graph.rs                        future graph-like planner entry point
```

The exact file split can wait until the first slice starts. The important
boundary is that measurement, constraints, and rendering stay separate.
