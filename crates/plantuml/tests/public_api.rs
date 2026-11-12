use oriha_plantuml::{
    DiagramArrowheadStyle, DiagramCornerRadius, DiagramStrokeWidth, DiagramStyle, PlantUml,
    PlantUmlDiagramKind, PlantUmlError, PlantUmlOptions,
};

#[test]
fn public_api_renders_sequence_diagram() {
    let rendered = PlantUml::render("@startuml\nAlice -> Bob: Hello\n@enduml\n")
        .expect("sequence diagram should render");

    assert_eq!(rendered.kind, PlantUmlDiagramKind::Sequence);
    assert!(rendered.svg.contains(r#"data-diagram-type="SEQUENCE""#));
    assert!(rendered.svg.contains("Alice"));
    assert!(rendered.svg.contains("Bob"));
}

#[test]
fn public_api_applies_shared_diagram_style_to_sequence_diagram() {
    let options = PlantUmlOptions {
        style: DiagramStyle {
            corner_radius: Some(DiagramCornerRadius::Md),
            arrowhead: Some(DiagramArrowheadStyle::Kite),
            stroke_width: Some(DiagramStrokeWidth::Bold),
        },
        ..PlantUmlOptions::default()
    };

    let rendered = PlantUml::new()
        .render_with_options("@startuml\nAlice -> Bob: Hello\n@enduml\n", &options)
        .expect("sequence diagram should render with shared style");

    assert!(rendered.svg.contains(r#"rx="6""#));
    assert!(rendered
        .svg
        .contains(r#"class="arrowhead arrowhead--kite""#));
    assert!(rendered.svg.contains(r#"stroke-width="3""#));
}

#[test]
fn public_api_uses_body_font_size_option_for_svg_and_measurements() {
    let options = PlantUmlOptions {
        body_font_size_px: Some(20.0),
        ..PlantUmlOptions::default()
    };
    let rendered = PlantUml::new()
        .render_with_options("@startuml\nAlice -> Bob: Hello\n@enduml\n", &options)
        .expect("sequence diagram should render with body font size");
    let requests = PlantUml::new()
        .collect_text_measurement_requests("@startuml\nAlice -> Bob: Hello\n@enduml\n", &options)
        .expect("sequence diagram should collect measurements with body font size");

    assert!(rendered.svg.contains(r#"font-size="20""#));
    assert!(requests
        .iter()
        .any(|request| request.text == "Hello" && request.style.font_size == Some(20.0)));
}

#[test]
fn public_api_defaults_invalid_body_font_size_to_sixteen_pixels() {
    let invalid_options = PlantUmlOptions {
        body_font_size_px: Some(f32::NAN),
        ..PlantUmlOptions::default()
    };
    let default_rendered = PlantUml::new()
        .render_with_options(
            "@startuml\nAlice -> Bob: default size\n@enduml\n",
            &PlantUmlOptions::default(),
        )
        .expect("sequence diagram should render with default body font size");
    let invalid_rendered = PlantUml::new()
        .render_with_options(
            "@startuml\nAlice -> Bob: invalid size\n@enduml\n",
            &invalid_options,
        )
        .expect("sequence diagram should render with fallback body font size");

    assert!(default_rendered.svg.contains(r#"font-size="16""#));
    assert!(invalid_rendered.svg.contains(r#"font-size="16""#));
}

#[test]
fn public_api_uses_sequence_defaults_for_omitted_shared_style_fields() {
    let options = PlantUmlOptions {
        style: DiagramStyle::default(),
        ..PlantUmlOptions::default()
    };

    let rendered = PlantUml::new()
        .render_with_options("@startuml\nAlice -> Bob: Hello\n@enduml\n", &options)
        .expect("sequence diagram should render with default style");

    assert!(!rendered.svg.contains(r#" rx=""#));
    assert!(rendered
        .svg
        .contains(r#"class="arrowhead arrowhead--triangle""#));
    assert!(rendered.svg.contains(r#"stroke-width="1""#));
}

#[test]
fn public_api_renders_minimal_class_diagram() {
    let rendered = PlantUml::render(
        "@startuml\n\
         class User {\n\
         +id: int\n\
         +name(): String\n\
         }\n\
         interface Repository\n\
         User --> Repository : uses\n\
         @enduml\n",
    )
    .expect("class diagram should render");

    assert_eq!(rendered.kind, PlantUmlDiagramKind::Class);
    assert!(rendered.svg.contains(r#"data-diagram-type="CLASS""#));
    assert!(rendered.svg.contains("User"));
    assert!(rendered.svg.contains("&lt;&lt;interface&gt;&gt;"));
    assert!(rendered.svg.contains("+id: int"));
    assert!(rendered.svg.contains("+name(): String"));
    assert!(rendered.svg.contains("uses"));
}

#[test]
fn public_api_reports_explicit_unsupported_diagram_kind() {
    let error = PlantUml::render("@startjson\n{\"name\":\"Alice\"}\n@endjson\n")
        .expect_err("json diagrams are not supported yet");

    assert_eq!(
        error,
        PlantUmlError::UnsupportedDiagram {
            kind: PlantUmlDiagramKind::Json
        }
    );
}
