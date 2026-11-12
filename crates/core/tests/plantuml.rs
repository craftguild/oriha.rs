use oriha_core::{
    prepare_markdown_layout_json, render, render_prepared_markdown_json, AppearanceOptions,
    FeatureFlags, FontFamilySpec, KindCount, RenderInput, RenderMode, RenderOutput,
};
use serde_json::json;

fn count_for(entries: &[KindCount], kind: &str) -> u32 {
    entries
        .iter()
        .find(|entry| entry.kind == kind)
        .map(|entry| entry.count)
        .unwrap_or(0)
}

fn plantuml_input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.to_owned(),
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            plantuml: true,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    }
}

fn plantuml_disabled_input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.to_owned(),
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            plantuml: false,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    }
}

fn plantuml_input_with_body_font(markdown: &str) -> RenderInput {
    RenderInput {
        appearance: AppearanceOptions {
            body_font_family: FontFamilySpec {
                family: "\"PlantUML Body\"".to_string(),
                fallbacks: vec!["serif".to_string()],
            },
            body_font_size_px: Some(20.0),
            ..AppearanceOptions::default()
        },
        ..plantuml_input(markdown)
    }
}

#[test]
fn render_markdown_embeds_plantuml_svg_when_enabled() {
    let output = render(plantuml_input(
        "```plantuml\n@startuml\nAlice -> Bob: Hello\n@enduml\n```\n",
    ));

    assert!(
        output.html.contains(r#"class="oriha-plantuml""#),
        "{}",
        output.html
    );
    assert!(output.html.contains("<svg "), "{}", output.html);
    assert_eq!(
        count_for(&output.stats.code_blocks_by_language_kind, "plantuml"),
        1
    );
}

#[test]
fn render_markdown_uses_body_font_for_plantuml_svg() {
    let output = render(plantuml_input_with_body_font(
        "```plantuml\n@startuml\nAlice -> Bob: Hello\n@enduml\n```\n",
    ));

    assert!(
        output
            .html
            .contains(r#"font-family="&quot;PlantUML Body&quot;, serif""#),
        "{}",
        output.html,
    );
    assert!(output.html.contains(r#"font-size="20""#), "{}", output.html);
}

#[test]
fn render_markdown_keeps_plantuml_fence_as_code_block_when_disabled() {
    let output = render(plantuml_disabled_input(
        "```plantuml\n@startuml\nAlice -> Bob: Hello\n@enduml\n```\n",
    ));

    assert!(
        output.html.contains(r#"class="oriha-code-block""#),
        "{}",
        output.html
    );
    assert!(
        output.html.contains(r#"class="language-plantuml""#),
        "{}",
        output.html
    );
    assert!(!output.html.contains(r#"class="oriha-plantuml""#));
    assert!(output
        .html
        .contains(r#"class="oriha-code-copy-icon size-6""#));
}

#[test]
fn plantuml_parse_error_warns_and_falls_back_to_code_block() {
    let output = render(plantuml_input("```puml\nnot a sequence diagram\n```\n"));

    assert!(
        output.html.contains(r#"class="oriha-code-block""#),
        "{}",
        output.html
    );
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "plantuml.render_failed"));
}

#[test]
fn class_plantuml_diagram_renders_svg() {
    let output = render(plantuml_input(
        "```puml\n@startuml\nclass User\n@enduml\n```\n",
    ));

    assert!(
        output.html.contains(r#"class="oriha-plantuml""#),
        "{}",
        output.html
    );
    assert!(output.html.contains("<svg "), "{}", output.html);
    assert!(
        output.html.contains(r#"data-diagram-type="CLASS""#),
        "{}",
        output.html
    );
    assert!(output.html.contains("User"));
    assert!(
        !output.html.contains(r#"class="oriha-code-block-wrapper""#),
        "{}",
        output.html
    );
    assert_eq!(
        count_for(&output.stats.unsupported_by_kind, "plantumlDiagram"),
        0
    );
    assert_eq!(
        count_for(&output.stats.code_blocks_by_language_kind, "plantuml"),
        1
    );
    assert!(!output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "plantuml.unsupported_diagram"));
}

#[test]
fn explicit_unsupported_start_directive_renders_error_figure_and_diagnostic() {
    let output = render(plantuml_input(
        "```plantuml\n@startjson\n{\"name\":\"Alice\"}\n@endjson\n```\n",
    ));

    assert!(
        output
            .html
            .contains(r#"class="oriha-plantuml oriha-plantuml--error""#),
        "{}",
        output.html
    );
    assert!(
        output
            .html
            .contains("unsupported PlantUML diagram kind: json"),
        "{}",
        output.html
    );
    assert_eq!(
        count_for(&output.stats.unsupported_by_kind, "plantumlDiagram"),
        1
    );
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "plantuml.unsupported_diagram"));
}

#[test]
fn explicit_unknown_start_directive_renders_error_figure_and_diagnostic() {
    let output = render(plantuml_input(
        "```plantuml\n@startditaa\n+---+\n| A |\n+---+\n@endditaa\n```\n",
    ));

    assert!(
        output
            .html
            .contains(r#"class="oriha-plantuml oriha-plantuml--error""#),
        "{}",
        output.html
    );
    assert!(
        output
            .html
            .contains("unsupported PlantUML diagram kind: unknown"),
        "{}",
        output.html
    );
    assert!(
        !output.html.contains(r#"class="oriha-code-block-wrapper""#),
        "{}",
        output.html
    );
    assert_eq!(
        count_for(&output.stats.unsupported_by_kind, "plantumlDiagram"),
        1
    );
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "plantuml.unsupported_diagram"));
    assert!(!output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "plantuml.render_failed"));
}

#[test]
fn prepared_markdown_collects_text_measurements_and_finalizes_with_metrics() {
    let input = serde_json::to_string(&plantuml_input(
        "```plantuml\n@startuml\nparticipant Alice\nparticipant Bob\nAlice -> Bob: Hello\n@enduml\n```\n",
    ))
    .expect("input serializes");
    let prepared_json = prepare_markdown_layout_json(&input);
    let prepared: serde_json::Value =
        serde_json::from_str(&prepared_json).expect("prepared layout is json");
    let requests = prepared["textMeasurements"]["requests"]
        .as_array()
        .expect("requests is an array");

    assert!(
        requests.iter().any(|request| request["text"] == "Hello"),
        "{prepared_json}"
    );

    let results = requests
        .iter()
        .map(|request| {
            json!({
                "id": request["id"],
                "width": if request["text"] == "Hello" { 240.0 } else { 80.0 }
            })
        })
        .collect::<Vec<_>>();
    let output_json =
        render_prepared_markdown_json(&prepared_json, &json!({ "results": results }).to_string());
    let output: RenderOutput = serde_json::from_str(&output_json).expect("output is json");

    assert!(
        output.html.contains(r#"class="oriha-plantuml""#),
        "{}",
        output.html
    );
    assert!(output.html.contains("<svg "), "{}", output.html);
}

#[test]
fn prepared_markdown_uses_body_font_for_plantuml_measurements() {
    let input = serde_json::to_string(&plantuml_input_with_body_font(
        "```plantuml\n@startuml\nparticipant Alice\nparticipant Bob\nAlice -> Bob: Hello\n@enduml\n```\n",
    ))
    .expect("input serializes");
    let prepared_json = prepare_markdown_layout_json(&input);
    let prepared: serde_json::Value =
        serde_json::from_str(&prepared_json).expect("prepared layout is json");
    let requests = prepared["textMeasurements"]["requests"]
        .as_array()
        .expect("requests is an array");
    let request = requests
        .iter()
        .find(|request| request["text"] == "Hello")
        .expect("message label should be measured");

    assert_eq!(request["fontFamily"], "\"PlantUML Body\", serif");
    assert_eq!(request["fontSize"], 20.0);
    assert_eq!(request["style"]["fontFamily"], "\"PlantUML Body\", serif");
    assert!(request["runs"]
        .as_array()
        .is_some_and(|runs| !runs.is_empty()));
}

#[test]
fn prepared_markdown_skips_measurements_for_unsupported_plantuml_diagram() {
    let input = serde_json::to_string(&plantuml_input(
        "```plantuml\n@startjson\n{\"name\":\"Alice\"}\n@endjson\n```\n",
    ))
    .expect("input serializes");
    let prepared_json = prepare_markdown_layout_json(&input);
    let prepared: serde_json::Value =
        serde_json::from_str(&prepared_json).expect("prepared layout is json");
    let requests = prepared["textMeasurements"]["requests"]
        .as_array()
        .expect("requests is an array");

    assert!(requests.is_empty(), "{prepared_json}");
}

#[test]
fn prepared_markdown_measurement_requests_use_visible_plantuml_text() {
    let raw_note_line =
        "<back:lavender>背景色つき note text</back> と <w:#FF33FF>波線 mixed text</w>";
    let input = serde_json::to_string(&plantuml_input(&format!(
        "```plantuml\n@startuml\nparticipant A\nnote right of A\n  {raw_note_line}\nend note\n@enduml\n```\n",
    )))
    .expect("input serializes");
    let prepared_json = prepare_markdown_layout_json(&input);
    let prepared: serde_json::Value =
        serde_json::from_str(&prepared_json).expect("prepared layout is json");
    let requests = prepared["textMeasurements"]["requests"]
        .as_array()
        .expect("requests is an array");
    let request = requests
        .iter()
        .find(|request| request["text"] == "背景色つき note text と 波線 mixed text")
        .expect("visible PlantUML line should be measured");

    assert_eq!(
        request["text"], "背景色つき note text と 波線 mixed text",
        "{prepared_json}",
    );
    let runs = request["runs"]
        .as_array()
        .expect("PlantUML measurement request should include runs");
    assert_eq!(runs.len(), 3, "{prepared_json}");
    assert_eq!(runs[0]["text"], "背景色つき note text");
    assert_eq!(runs[1]["text"], " と ");
    assert_eq!(runs[2]["text"], "波線 mixed text");
    assert!(runs[2]["id"].as_str().is_some());
    assert!(
        requests.iter().all(|request| {
            request["text"]
                .as_str()
                .is_none_or(|text| !text.contains("<back:") && !text.contains("<w:"))
        }),
        "{prepared_json}",
    );
}
