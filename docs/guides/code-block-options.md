# Code-block presentation options

The `oriha-core` Rust and JSON APIs accept `renderOptions.codeBlock` for both
`styledHtml` and `semanticHtml`. Every switch defaults to `true`, including
when the object or individual fields are omitted. Existing callers keep their
language labels, Copy buttons and line numbers.
Serialization omits an all-default `codeBlock` object to preserve existing
default input, prepared-output and patch contracts.

```json
{
  "markdown": "```rust\nlet x = 1;\n```",
  "mode": "semanticHtml",
  "renderOptions": {
    "codeBlock": {
      "showLanguage": true,
      "showCopyButton": false,
      "showLineNumbers": true
    }
  }
}
```

The same settings are available in Rust:

```rust
use oriha_core::{CodeBlockOptions, RenderInput, RenderOptions};

let input = RenderInput {
    markdown: "```rust\nlet x = 1;\n```".into(),
    render_options: RenderOptions {
        code_block: CodeBlockOptions {
            show_copy_button: false,
            ..Default::default()
        },
        ..Default::default()
    },
    ..Default::default()
};
```

| Option | Default | Effect when false |
| --- | --- | --- |
| `showLanguage` | `true` | Omit the visible language label. |
| `showCopyButton` | `true` | Omit the Copy button and duplicate `data-oriha-code-copy` payload. |
| `showLineNumbers` | `true` | Omit the line-number gutter. |

When both header controls are disabled, no empty header is emitted. These
options apply to fenced and indented code; successful math and diagram SVGs
keep their existing output. Code text, syntax tokens and language classes/data
remain available when a label is hidden.

`parse` retains the settings for `render_parsed` and
`derive_parsed_blocks(..., BlockOutputMode::Rendered)`. Direct rendered-block
derivation uses them too. `Structured` output keeps its factual language,
info and text properties regardless of presentation settings.

Semantic HTML still has no theme CSS or clipboard event handlers. Consumers
style the emitted elements and supply any Copy interaction they enable.
These are core rendering options; the separate native bridge input retains
its existing defaults and wire contract.
