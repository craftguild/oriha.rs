use oriha_core::{
    prepare_markdown_layout_json, render, AppearanceOptions, FeatureFlags, FontFamilySpec,
    KindCount, RenderInput, RenderMode,
};

fn count_for(entries: &[KindCount], kind: &str) -> u32 {
    entries
        .iter()
        .find(|entry| entry.kind == kind)
        .map(|entry| entry.count)
        .unwrap_or(0)
}

fn mermaid_input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.to_owned(),
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            mermaid: true,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    }
}

fn mermaid_disabled_input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.to_owned(),
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            mermaid: false,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    }
}

fn mermaid_input_with_body_font(markdown: &str) -> RenderInput {
    RenderInput {
        appearance: AppearanceOptions {
            body_font_family: FontFamilySpec {
                family: "\"Mermaid Body\"".to_string(),
                fallbacks: vec!["sans-serif".to_string()],
            },
            body_font_size_px: Some(20.0),
            ..AppearanceOptions::default()
        },
        ..mermaid_input(markdown)
    }
}

#[test]
fn render_markdown_embeds_mermaid_svg_when_enabled() {
    let output = render(mermaid_input(
        "```mermaid\nflowchart TD\nA[Start] --> B[Done]\n```\n",
    ));

    assert!(
        output.html.contains(r#"class="oriha-mermaid""#),
        "{}",
        output.html
    );
    assert!(output.html.contains("<svg "), "{}", output.html);
    assert_eq!(
        count_for(&output.stats.code_blocks_by_language_kind, "mermaid"),
        1
    );
}

#[test]
fn render_markdown_uses_body_font_size_for_mermaid_svg() {
    let output = render(mermaid_input_with_body_font(
        "```mermaid\nflowchart TD\nA[Start] --> B[Done]\n```\n",
    ));

    assert!(output.html.contains(r#"font-size="20""#), "{}", output.html);
}

#[test]
fn render_markdown_keeps_mermaid_fence_as_code_block_when_disabled() {
    let output = render(mermaid_disabled_input(
        "```mermaid\nflowchart TD\nA[Start] --> B[Done]\n```\n",
    ));

    assert!(
        output.html.contains(r#"class="oriha-code-block""#),
        "{}",
        output.html
    );
    assert!(
        output.html.contains(r#"class="language-mermaid""#),
        "{}",
        output.html
    );
    assert!(!output.html.contains(r#"class="oriha-mermaid""#));
}

#[test]
fn mermaid_render_error_renders_error_figure_and_diagnostic() {
    let output = render(mermaid_input("```mermaid\nnot a mermaid diagram\n```\n"));

    assert!(
        output
            .html
            .contains(r#"class="oriha-mermaid oriha-mermaid--error""#),
        "{}",
        output.html
    );
    assert!(output
        .html
        .contains("Mermaid diagram could not be rendered"));
    assert!(output.html.contains("not a mermaid diagram"));
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "mermaid.render_failed"));
}

#[test]
fn prepared_markdown_collects_mermaid_measurements() {
    let input = serde_json::to_string(&mermaid_input_with_body_font(
        "```mermaid\nflowchart TD\nA[Start] --> B[Done]\n```\n",
    ))
    .expect("input serializes");
    let prepared_json = prepare_markdown_layout_json(&input);
    let prepared: serde_json::Value =
        serde_json::from_str(&prepared_json).expect("prepared layout is json");
    let requests = prepared["textMeasurements"]["requests"]
        .as_array()
        .expect("requests is an array");

    assert!(!requests.is_empty(), "{prepared_json}");
    assert!(
        requests
            .iter()
            .all(|request| request["engine"] == "mermaid"),
        "{prepared_json}"
    );
    assert!(
        requests
            .iter()
            .any(|request| request["kind"] == "wrapped" || request["kind"] == "text"),
        "{prepared_json}"
    );
    assert!(
        requests
            .iter()
            .any(|request| request["fontFamily"] == "\"Mermaid Body\",sans-serif"),
        "{prepared_json}"
    );
    assert!(
        requests
            .iter()
            .any(|request| request["fontSize"].as_f64() == Some(20.0)),
        "{prepared_json}"
    );
}
