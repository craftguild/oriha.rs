//! JSON boundary contract snapshots.
//!
//! These functions are consumed by wasm, native bridges, and external JSON
//! callers. The snapshots make unintentional response-shape changes fail CI.

use oriha_core::{
    export_html_json, prepare_html_export_json, prepare_markdown_patch_json, render_json,
    render_markdown_patch, render_markdown_patch_json, render_prepared_html_export_json,
    render_prepared_markdown_patch_json, EngineLimits, FeatureFlags, OrihaEngine, RenderInput,
    RenderMode, RenderPatchFallbackReason, RenderPatchInput, RenderPatchOperation,
    RenderPatchStrategy,
};

const CONTRACT_MARKDOWN: &str = r#"# Title

Paragraph with *emphasis*, **strong**, `code`, and a [link](https://example.com).

- item one
- [ ] task item

> :::note Heads up
> Callout body.

| a | b |
|---|---|
| 1 | 2 |

```rust
fn main() {}
```

Math: $x^2$

```mermaid
flowchart TD
A --> B
```
"#;

fn contract_input() -> serde_json::Value {
    serde_json::json!({
        "markdown": CONTRACT_MARKDOWN,
        "mode": "semanticHtml",
        "features": {
            "gfm": true,
            "callouts": true,
            "syntaxHighlight": true,
            "math": true,
            "mermaid": true,
        },
    })
}

#[test]
fn render_json_contract_snapshot() {
    let input = serde_json::to_string(&contract_input()).expect("input should serialize");
    let output: serde_json::Value =
        serde_json::from_str(&render_json(&input)).expect("output should be valid JSON");
    insta::assert_json_snapshot!("render_json_output", output, {
        ".assets[].content" => "[asset content]",
    });
}

#[test]
fn render_json_invalid_input_contract_snapshot() {
    let output: serde_json::Value =
        serde_json::from_str(&render_json("{not json")).expect("output should be valid JSON");
    insta::assert_json_snapshot!("render_json_invalid_input", output);
}

#[test]
fn export_html_json_valid_contract_shape() {
    let input = serde_json::json!({
        "input": {
            "markdown": "# Title\n\nParagraph.",
            "mode": "styledHtml"
        }
    });
    let output: serde_json::Value = serde_json::from_str(&export_html_json(&input.to_string()))
        .expect("output should be valid JSON");

    assert!(output
        .get("html")
        .and_then(serde_json::Value::as_str)
        .is_some());
    assert!(output.get("stats").is_some());
    assert!(output.get("diagnostics").is_some());
    assert!(output.get("sourceMap").is_none());
    assert!(output.get("snapshot").is_none());
    assert!(output.get("assets").is_none());
}

#[test]
fn export_html_json_invalid_input_contract_shape() {
    let output: serde_json::Value =
        serde_json::from_str(&export_html_json("{not json")).expect("output should be valid JSON");

    assert_eq!(output["html"], "");
    assert_eq!(output["diagnostics"][0]["code"], "input.invalid_json");
    assert!(output.get("sourceMap").is_none());
    assert!(output.get("snapshot").is_none());
    assert!(output.get("assets").is_none());
}

#[test]
fn prepare_and_render_prepared_html_export_contract_shape() {
    let input = serde_json::json!({
        "input": {
            "markdown": "# Prepared\n\n```mermaid\nflowchart TD\nA --> B\n```",
            "features": { "mermaid": true }
        }
    });
    let prepared_json = prepare_html_export_json(&input.to_string());
    let prepared: serde_json::Value =
        serde_json::from_str(&prepared_json).expect("prepared should be valid JSON");
    assert!(prepared.get("textMeasurements").is_some());
    assert!(prepared.get("sourceMap").is_none());
    assert!(prepared.get("snapshot").is_none());

    let output: serde_json::Value =
        serde_json::from_str(&render_prepared_html_export_json(&prepared_json, "[]"))
            .expect("output should be valid JSON");
    assert!(output
        .get("html")
        .and_then(serde_json::Value::as_str)
        .is_some());
    assert!(output.get("stats").is_some());
    assert!(output.get("diagnostics").is_some());
    assert!(output.get("sourceMap").is_none());
    assert!(output.get("snapshot").is_none());
    assert!(output.get("assets").is_none());
}

#[test]
fn default_export_minifies_obfuscates_and_strips_oriha_data_attrs() {
    let input = serde_json::json!({
        "input": {
            "markdown": "# Title\n\nParagraph.",
            "mode": "styledHtml"
        }
    });
    let output: serde_json::Value = serde_json::from_str(&export_html_json(&input.to_string()))
        .expect("output should be valid JSON");
    let html = output["html"].as_str().expect("html should be a string");

    assert!(html.starts_with("<!doctype html><html><head>"));
    assert!(!html.contains('\n'));
    assert!(!html.contains("data-oriha-"));
    assert!(!html.contains("class=\"oriha-"));
    assert!(html.contains("class=\"c"));
}

#[test]
fn export_html_accepts_null_options_as_defaults() {
    let input = serde_json::json!({
        "input": {
            "markdown": "# Title\n\nParagraph.",
            "mode": "styledHtml"
        },
        "options": null
    });
    let output: serde_json::Value = serde_json::from_str(&export_html_json(&input.to_string()))
        .expect("output should be valid JSON");

    assert!(output["html"]
        .as_str()
        .is_some_and(|html| html.contains("Title") && !html.contains("data-oriha-")));
    assert!(output["diagnostics"].as_array().is_some_and(Vec::is_empty));
}

#[test]
fn export_html_obfuscates_syntax_token_classes() {
    let input = serde_json::json!({
        "input": {
            "markdown": "```rust\nfn main() {}\n```",
            "mode": "styledHtml"
        }
    });
    let output: serde_json::Value = serde_json::from_str(&export_html_json(&input.to_string()))
        .expect("output should be valid JSON");
    let html = output["html"].as_str().expect("html should be a string");

    assert!(!html.contains(r#"class="token keyword""#));
    assert!(!html.contains(".token.keyword"));
    assert!(!html.contains(".keyword"));
    assert!(html.contains("fn"));
}

#[test]
fn export_html_preserves_pre_code_whitespace_when_minifying() {
    let input = serde_json::json!({
        "input": {
            "markdown": "```text\n  a\n    b\n```",
            "mode": "styledHtml"
        }
    });
    let output: serde_json::Value = serde_json::from_str(&export_html_json(&input.to_string()))
        .expect("output should be valid JSON");
    let html = output["html"].as_str().expect("html should be a string");

    assert!(html.contains(">  a</span>"));
    assert!(html.contains(">    b</span>"));
}

#[test]
fn export_html_preserves_inline_separator_whitespace_when_minifying() {
    let input = serde_json::json!({
        "input": {
            "markdown": "**Strong** *Emphasis*",
            "mode": "styledHtml"
        }
    });
    let output: serde_json::Value = serde_json::from_str(&export_html_json(&input.to_string()))
        .expect("output should be valid JSON");
    let html = output["html"].as_str().expect("html should be a string");

    assert!(html.contains("Strong</strong> <em>Emphasis"));
}

#[test]
fn export_html_preserves_heading_id_and_markdown_hash_link_by_default() {
    let input = serde_json::json!({
        "input": {
            "markdown": "# Heading\n\n[Jump](#heading)",
            "mode": "styledHtml"
        }
    });
    let output: serde_json::Value = serde_json::from_str(&export_html_json(&input.to_string()))
        .expect("output should be valid JSON");
    let html = output["html"].as_str().expect("html should be a string");

    assert!(html.contains(r#"id="heading""#));
    assert!(html.contains(r##"href="#heading""##));
}

#[test]
fn export_html_rewrites_footnote_refs_when_ids_are_mangled() {
    let input = serde_json::json!({
        "input": {
            "markdown": "Body with note.[^n]\n\n[^n]: Footnote body.",
            "mode": "styledHtml"
        }
    });
    let output: serde_json::Value = serde_json::from_str(&export_html_json(&input.to_string()))
        .expect("output should be valid JSON");
    let html = output["html"].as_str().expect("html should be a string");

    assert!(!html.contains("fn-n"));
    assert!(!html.contains("fnref-n"));
    assert!(html.contains(r##"href="#e"##));
    assert!(html.contains(r#"id="e"#));
    assert!(html.contains(r#"aria-labelledby="e"#));
}

#[test]
fn export_html_preserves_raw_html_user_id_and_class() {
    let input = serde_json::json!({
        "input": {
            "markdown": "<div id=\"user-id\" class=\"user-class oriha-user-class\">Raw</div>",
            "mode": "styledHtml",
            "features": { "rawHtml": true }
        }
    });
    let output: serde_json::Value = serde_json::from_str(&export_html_json(&input.to_string()))
        .expect("output should be valid JSON");
    let html = output["html"].as_str().expect("html should be a string");

    assert!(html.contains(r#"id="user-id""#));
    assert!(html.contains(r#"class="user-class oriha-user-class""#));
}

#[test]
fn render_markdown_patch_initial_contract_snapshot() {
    let input = serde_json::json!({
        "input": {
            "markdown": "# Title\n\nAlpha\n\nBeta\n",
            "mode": "semanticHtml"
        }
    });
    let output: serde_json::Value =
        serde_json::from_str(&render_markdown_patch_json(&input.to_string()))
            .expect("output should be valid JSON");
    insta::assert_json_snapshot!("render_markdown_patch_initial_output", output);
}

#[test]
fn render_markdown_patch_invalid_input_contract_snapshot() {
    let output: serde_json::Value = serde_json::from_str(&render_markdown_patch_json("{not json"))
        .expect("output should be valid JSON");
    insta::assert_json_snapshot!("render_markdown_patch_invalid_input", output);
}

#[test]
fn prepare_invalid_input_contract_snapshot() {
    let prepared: serde_json::Value =
        serde_json::from_str(&oriha_core::prepare_markdown_layout_json("{not json"))
            .expect("prepared should be valid JSON");
    insta::assert_json_snapshot!("prepare_markdown_layout_invalid_input", prepared);
}

#[test]
fn prepare_and_render_prepared_contract_snapshot() {
    let input = serde_json::to_string(&contract_input()).expect("input should serialize");
    let prepared_json = oriha_core::prepare_markdown_layout_json(&input);
    let prepared: serde_json::Value =
        serde_json::from_str(&prepared_json).expect("prepared should be valid JSON");
    insta::assert_json_snapshot!("prepare_markdown_layout_output", prepared);

    let output_json = oriha_core::render_prepared_markdown_json(&prepared_json, "[]");
    let output: serde_json::Value =
        serde_json::from_str(&output_json).expect("output should be valid JSON");
    insta::assert_json_snapshot!("render_prepared_markdown_output", output, {
        ".assets[].content" => "[asset content]",
    });
}

#[test]
fn prepare_and_render_prepared_patch_contract_snapshot() {
    let initial = render_markdown_patch(RenderPatchInput {
        previous_snapshot: None,
        input: mermaid_patch_render_input("Old\n\n```mermaid\nflowchart TD\nA --> B\n```\n"),
    });
    let patch_input = RenderPatchInput {
        previous_snapshot: Some(initial.snapshot),
        input: mermaid_patch_render_input("New\n\n```mermaid\nflowchart TD\nA --> B\n```\n"),
    };
    let input = serde_json::to_string(&patch_input).expect("input should serialize");

    let prepared_json = prepare_markdown_patch_json(&input);
    let prepared: serde_json::Value =
        serde_json::from_str(&prepared_json).expect("prepared should be valid JSON");
    insta::assert_json_snapshot!("prepare_markdown_patch_output", prepared);

    let output_json = render_prepared_markdown_patch_json(&prepared_json, "[]");
    let output: serde_json::Value =
        serde_json::from_str(&output_json).expect("output should be valid JSON");
    insta::assert_json_snapshot!("render_prepared_markdown_patch_output", output);
}

#[test]
fn prepare_markdown_patch_invalid_input_contract_snapshot() {
    let prepared: serde_json::Value =
        serde_json::from_str(&prepare_markdown_patch_json("{not json"))
            .expect("prepared should be valid JSON");
    insta::assert_json_snapshot!("prepare_markdown_patch_invalid_input", prepared);
}

#[test]
fn render_prepared_markdown_patch_invalid_prepared_contract_snapshot() {
    let output: serde_json::Value =
        serde_json::from_str(&render_prepared_markdown_patch_json("{not json", "[]"))
            .expect("output should be valid JSON");
    insta::assert_json_snapshot!("render_prepared_markdown_patch_invalid_prepared", output);
}

#[test]
fn render_prepared_markdown_patch_invalid_text_metrics_contract_snapshot() {
    let input = serde_json::to_string(&RenderPatchInput {
        previous_snapshot: None,
        input: mermaid_patch_render_input("```mermaid\nflowchart TD\nA --> B\n```\n"),
    })
    .expect("input should serialize");
    let prepared_json = prepare_markdown_patch_json(&input);

    let output: serde_json::Value = serde_json::from_str(&render_prepared_markdown_patch_json(
        &prepared_json,
        "{not json",
    ))
    .expect("output should be valid JSON");
    insta::assert_json_snapshot!(
        "render_prepared_markdown_patch_invalid_text_metrics",
        output
    );
}

#[test]
fn render_input_defaults_are_stable() {
    let default_input =
        serde_json::to_value(RenderInput::default()).expect("input should serialize");
    insta::assert_json_snapshot!("render_input_defaults", default_input);
}

#[test]
fn render_markdown_patch_reuses_stable_ids_for_edited_same_kind_gap() {
    let first = render_markdown_patch(RenderPatchInput {
        previous_snapshot: None,
        input: patch_render_input("Alpha\n\nBeta\n\nGamma\n"),
    });
    let beta_id = first.snapshot.blocks[1].stable_id.clone();

    let second = render_markdown_patch(RenderPatchInput {
        previous_snapshot: Some(first.snapshot),
        input: patch_render_input("Alpha\n\nBeta edited\n\nGamma\n"),
    });

    assert_eq!(second.strategy, RenderPatchStrategy::Incremental);
    assert_eq!(second.fallback_reason, None);
    assert_eq!(second.snapshot.blocks[1].stable_id, beta_id);
    assert_eq!(second.patches.len(), 1);

    let RenderPatchOperation::ReplaceBlocks {
        remove_stable_ids,
        html,
        ..
    } = &second.patches[0]
    else {
        panic!("expected replaceBlocks patch");
    };
    assert_eq!(remove_stable_ids, &vec![beta_id.clone()]);
    assert!(html.contains(&format!(r#"data-oriha-stable-id="{beta_id}""#)));
    assert!(html.contains("Beta edited"));
}

#[test]
fn render_markdown_patch_replaces_same_source_when_rendered_html_changes() {
    let first = render_markdown_patch(RenderPatchInput {
        previous_snapshot: None,
        input: patch_render_input("# Repeat\n\n# Repeat\n"),
    });
    let first_heading_id = first.snapshot.blocks[0].stable_id.clone();

    let second = render_markdown_patch(RenderPatchInput {
        previous_snapshot: Some(first.snapshot),
        input: patch_render_input("Intro\n\n# Repeat\n\n# Repeat\n"),
    });

    assert_eq!(second.strategy, RenderPatchStrategy::Incremental);
    assert!(second.patches.iter().any(|patch| match patch {
        RenderPatchOperation::ReplaceBlocks {
            remove_stable_ids, ..
        } => remove_stable_ids.contains(&first_heading_id),
        RenderPatchOperation::ReplaceAll { .. } => false,
    }));
}

#[test]
fn render_markdown_patch_falls_back_when_output_is_truncated() {
    let markdown =
        "A long paragraph that will be truncated by the test engine limit.\n\nSecond paragraph.\n";
    let first = render_markdown_patch(RenderPatchInput {
        previous_snapshot: None,
        input: patch_render_input(markdown),
    });

    let second = OrihaEngine::default()
        .with_limits(EngineLimits {
            max_output_bytes: 80,
            ..EngineLimits::default()
        })
        .render_patch(RenderPatchInput {
            previous_snapshot: Some(first.snapshot),
            input: patch_render_input(markdown),
        });

    assert_eq!(second.strategy, RenderPatchStrategy::ReplaceAll);
    assert_eq!(
        second.fallback_reason,
        Some(RenderPatchFallbackReason::OutputTruncated)
    );
    assert!(second.snapshot.settings_fingerprint.is_empty());
    assert!(second.snapshot.blocks.is_empty());
    assert!(matches!(
        second.patches.as_slice(),
        [RenderPatchOperation::ReplaceAll { .. }]
    ));

    let third = render_markdown_patch(RenderPatchInput {
        previous_snapshot: Some(second.snapshot),
        input: patch_render_input(markdown),
    });
    assert_eq!(third.strategy, RenderPatchStrategy::ReplaceAll);
    assert_eq!(
        third.fallback_reason,
        Some(RenderPatchFallbackReason::IncompatibleSnapshot)
    );
}

#[test]
fn render_markdown_patch_keeps_stable_attr_when_code_contains_attr_name() {
    let output = render_markdown_patch(RenderPatchInput {
        previous_snapshot: None,
        input: patch_render_input("```text\ndata-oriha-stable-id=\n```\n"),
    });

    assert!(output.html_contains_top_level_stable_code_block());
}

#[test]
fn render_markdown_patch_falls_back_when_tail_changes() {
    let first = render_markdown_patch(RenderPatchInput {
        previous_snapshot: None,
        input: patch_render_input("Body with note.[^n]\n\n[^n]: Footnote body.\n"),
    });

    let second = render_markdown_patch(RenderPatchInput {
        previous_snapshot: Some(first.snapshot),
        input: patch_render_input("Body without note.\n"),
    });

    assert_eq!(second.strategy, RenderPatchStrategy::ReplaceAll);
    assert_eq!(
        second.fallback_reason,
        Some(RenderPatchFallbackReason::TailChanged)
    );
    assert!(matches!(
        second.patches.as_slice(),
        [RenderPatchOperation::ReplaceAll { .. }]
    ));
}

#[test]
fn render_markdown_patch_falls_back_when_styled_output_resources_change() {
    let first = render_markdown_patch(RenderPatchInput {
        previous_snapshot: None,
        input: styled_math_patch_render_input("Alpha\n"),
    });
    let first_resource_fingerprint = first.snapshot.output_resource_fingerprint.clone();

    let second = render_markdown_patch(RenderPatchInput {
        previous_snapshot: Some(first.snapshot),
        input: styled_math_patch_render_input("Alpha\n\nMath: $x^2$\n"),
    });

    assert_ne!(
        second.snapshot.output_resource_fingerprint,
        first_resource_fingerprint
    );
    assert_eq!(second.strategy, RenderPatchStrategy::ReplaceAll);
    assert_eq!(
        second.fallback_reason,
        Some(RenderPatchFallbackReason::IncompatibleSnapshot)
    );
    assert!(!second.output.assets.is_empty());
    assert!(second
        .output
        .css
        .as_deref()
        .is_some_and(|css| css.contains("@font-face")));
    assert!(matches!(
        second.patches.as_slice(),
        [RenderPatchOperation::ReplaceAll { .. }]
    ));
}

#[test]
fn render_markdown_patch_falls_back_for_unanchored_insert_when_tail_exists() {
    let first = render_markdown_patch(RenderPatchInput {
        previous_snapshot: None,
        input: patch_render_input("Alpha with note.[^n]\n\n[^n]: Footnote body.\n"),
    });
    let first_tail_hash = first.snapshot.tail_hash.clone();

    assert!(first.snapshot.tail_html_byte_length > 0);

    let second = render_markdown_patch(RenderPatchInput {
        previous_snapshot: Some(first.snapshot),
        input: patch_render_input("Beta with note.[^n]\n\n[^n]: Footnote body.\n"),
    });

    assert_eq!(second.snapshot.tail_hash, first_tail_hash);
    assert!(second.snapshot.tail_html_byte_length > 0);
    assert_eq!(second.strategy, RenderPatchStrategy::ReplaceAll);
    assert_eq!(
        second.fallback_reason,
        Some(RenderPatchFallbackReason::IncompatibleSnapshot)
    );
    assert!(matches!(
        second.patches.as_slice(),
        [RenderPatchOperation::ReplaceAll { .. }]
    ));
}

trait PatchOutputAssertions {
    fn html_contains_top_level_stable_code_block(&self) -> bool;
}

impl PatchOutputAssertions for oriha_core::RenderPatchOutput {
    fn html_contains_top_level_stable_code_block(&self) -> bool {
        self.output.html.contains(
            "data-oriha-code-copy=\"data-oriha-stable-id=\n\" data-oriha-stable-id=\"oriha-b-1\"",
        )
    }
}

fn patch_render_input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default()
    }
}

fn mermaid_patch_render_input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.to_string(),
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            mermaid: true,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    }
}

fn styled_math_patch_render_input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.to_string(),
        mode: RenderMode::StyledHtml,
        features: FeatureFlags {
            math: true,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    }
}
