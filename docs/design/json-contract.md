# JSON contract versioning

The `oriha-core` JSON entry points (`render_json`,
`prepare_markdown_layout_json`, `render_prepared_markdown_json`) are stable
contracts shared by Wasm and external JSON callers. Prefer the
[typed native contract](native-bridge-contract.md) at native FFI boundaries. Add a
JSON interface only when persistence, transport or an external protocol needs it.

`crates/core/tests/json_contract.rs` fixes response shapes with insta snapshots.
Review intentional differences as contract changes; repair accidental changes in
the implementation. Never silently update snapshots. `RenderInput::default()`
also defines omitted-field semantics, fixed by `render_input_defaults`.

Assume clients cannot all update together. Additive fields, enum values and
diagnostic codes are compatible under this contract. Removing/renaming fields,
changing types or changing existing meanings is breaking. Increment the affected
schema/version and use a `BREAKING CHANGE:` commit footer, including for
`PREPARED_MARKDOWN_LAYOUT_VERSION` and `render_stats_schema_version()`.

Snapshots redact `assets[].content` as `[asset content]`. They fix counts, kinds,
MIME types, IDs and other metadata, rather than font or SVG payload bytes.
