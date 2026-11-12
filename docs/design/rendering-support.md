# Rendering support

Treat Markdown as untrusted input. Unsupported or failed input produces
`RenderOutput.diagnostics` and statistics instead of panicking.

## Markdown

Run all 652 CommonMark 0.31.2 official fixtures through the public renderer. The
CommonMark feature profile must match every expected HTML result after removing
renderer-specific wrappers/attributes. Product-profile extensions such as GFM,
callouts, code UI and raw-HTML safety have separate behavior and panic checks.

## Math and diagrams

LaTeX support is the subset fixed by the math parser/layout tests. Unknown
commands/environments and syntax failures produce `math.render_failed` and a
literal fallback or error figure. Mermaid support is fixed by its public API
tests and `crates/core/tests/mermaid.rs`; failures produce
`mermaid.render_failed`. PlantUML supports the tested sequence/class subsets;
unsupported kinds produce `plantuml.unsupported_diagram`, and parse/render
failures produce `plantuml.render_failed`.

## Boundaries

FFI/JSON/Wasm boundaries catch panics and return error values containing
`engine.panic` or `bridge.panic`. Internal `unwrap` / `expect` requires a logically
unreachable failure independent of untrusted input.
