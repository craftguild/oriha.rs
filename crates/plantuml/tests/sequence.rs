use oriha_plantuml::{
    __private::{
        collect_sequence_text_measurements, detect_plantuml_diagram, detect_plantuml_diagram_kind,
        layout_sequence, parse_sequence, render_plantuml_source_svg, render_sequence_source_svg,
        render_sequence_source_svg_with_style,
        render_sequence_source_svg_with_style_and_text_measurer, render_sequence_svg,
        render_sequence_svg_with_style, ArrowheadStyle, CornerRadius, GroupKind, LayoutOptions,
        LineStyle, MessageArrowHead, NoteShape, ParticipantKind, PlantUmlDiagramDetectionReason,
        PlantUmlError, PlantUmlRenderError, PlantUmlStyle, PlantUmlTheme, RenderedPlantUml,
        SequenceEvent, StrokeWidth,
    },
    ApproxTextMeasurer, InlineTextMeasurementRequest, InlineTextMeasurementResult,
    InlineTextRunMeasurement, PlantUmlDiagramKind, TextMeasurer,
};

const SEQUENCE_01: &str = include_str!("fixtures/sequence/001.puml");
const SEQUENCE_01_REFERENCE_SVG: &str = include_str!("fixtures/sequence/001-golden.svg");
const SEQUENCE_FIXTURE_001: &str = include_str!("fixtures/sequence/001.puml");
const SEQUENCE_FIXTURE_002: &str = include_str!("fixtures/sequence/002.puml");
const SEQUENCE_FIXTURE_003: &str = include_str!("fixtures/sequence/003.puml");
const SEQUENCE_FIXTURE_005: &str = include_str!("fixtures/sequence/005.puml");
const SEQUENCE_FIXTURE_014: &str = include_str!("fixtures/sequence/014.puml");
const SEQUENCE_FIXTURE_015: &str = include_str!("fixtures/sequence/015.puml");
const SEQUENCE_FIXTURE_016: &str = include_str!("fixtures/sequence/016.puml");
const SEQUENCE_FIXTURE_018: &str = include_str!("fixtures/sequence/018.puml");
const SEQUENCE_FIXTURE_019: &str = include_str!("fixtures/sequence/019.puml");
const SEQUENCE_FIXTURE_021: &str = include_str!("fixtures/sequence/021.puml");
const SEQUENCE_FIXTURE_022: &str = include_str!("fixtures/sequence/022.puml");
const SEQUENCE_FIXTURE_023: &str = include_str!("fixtures/sequence/023.puml");
const SEQUENCE_FIXTURE_024: &str = include_str!("fixtures/sequence/024.puml");
const SEQUENCE_FIXTURE_025: &str = include_str!("fixtures/sequence/025.puml");
const SEQUENCE_FIXTURE_026: &str = include_str!("fixtures/sequence/026.puml");
const SEQUENCE_FIXTURE_027: &str = include_str!("fixtures/sequence/027.puml");
const SEQUENCE_FIXTURE_028: &str = include_str!("fixtures/sequence/028.puml");
const SEQUENCE_FIXTURE_029: &str = include_str!("fixtures/sequence/029.puml");
const SEQUENCE_FIXTURE_030: &str = include_str!("fixtures/sequence/030.puml");
const SEQUENCE_FIXTURE_031: &str = include_str!("fixtures/sequence/031.puml");
const SEQUENCE_FIXTURE_032: &str = include_str!("fixtures/sequence/032.puml");
const SEQUENCE_FIXTURE_033: &str = include_str!("fixtures/sequence/033.puml");
const SEQUENCE_FIXTURE_034: &str = include_str!("fixtures/sequence/034.puml");

#[test]
fn dispatcher_reports_explicit_non_sequence_start_directive_as_unsupported() {
    let source = "@startjson\n{\"name\":\"Alice\"}\n@endjson\n";

    assert_eq!(
        detect_plantuml_diagram_kind(source),
        PlantUmlDiagramKind::Json
    );

    let error =
        render_plantuml_source_svg(source, &LayoutOptions::default(), &PlantUmlTheme::default())
            .expect_err("explicit json PlantUML should be reported as unsupported");
    assert_eq!(
        error,
        PlantUmlRenderError::UnsupportedDiagram {
            kind: PlantUmlDiagramKind::Json
        }
    );
}

#[test]
fn detector_reports_start_directive_evidence_with_line_number() {
    let source = "@startjson\n{\"name\":\"Alice\"}\n@endjson\n";

    let detection = detect_plantuml_diagram(source);

    assert_eq!(detection.kind, PlantUmlDiagramKind::Json);
    assert_eq!(
        detection.reason,
        PlantUmlDiagramDetectionReason::StartDirective
    );
    assert_eq!(detection.line, Some(1));
}

#[test]
fn detector_reports_body_statement_evidence_with_line_number() {
    let source = "' comment\n@startuml\nAlice -> Bob: Hello\n@enduml\n";

    let detection = detect_plantuml_diagram(source);

    assert_eq!(detection.kind, PlantUmlDiagramKind::Sequence);
    assert_eq!(
        detection.reason,
        PlantUmlDiagramDetectionReason::SequenceStatement
    );
    assert_eq!(detection.line, Some(3));
}

#[test]
fn dispatcher_reports_explicit_unknown_start_directive_as_unsupported() {
    let source = "@startditaa\n+---+\n| A |\n+---+\n@endditaa\n";

    assert_eq!(
        detect_plantuml_diagram_kind(source),
        PlantUmlDiagramKind::Unknown
    );

    let error =
        render_plantuml_source_svg(source, &LayoutOptions::default(), &PlantUmlTheme::default())
            .expect_err("explicit unknown PlantUML start directive should be unsupported");
    assert_eq!(
        error,
        PlantUmlRenderError::UnsupportedDiagram {
            kind: PlantUmlDiagramKind::Unknown
        }
    );
}

#[test]
fn dispatcher_preserves_startuml_parse_error_fallback() {
    let source = "@startuml\nnot a sequence diagram\n@enduml\n";

    assert_eq!(
        detect_plantuml_diagram_kind(source),
        PlantUmlDiagramKind::Sequence
    );

    let error =
        render_plantuml_source_svg(source, &LayoutOptions::default(), &PlantUmlTheme::default())
            .expect_err("@startuml should still dispatch to the sequence parser");
    assert!(
        matches!(error, PlantUmlRenderError::Parse { .. }),
        "{error:?}"
    );
}

#[test]
fn dispatcher_ignores_block_comments_when_inferring_diagram_kind() {
    let source = "@startuml\n/'\nclass User\n'/\nAlice -> Bob: Hello\n@enduml\n";

    assert_eq!(
        detect_plantuml_diagram_kind(source),
        PlantUmlDiagramKind::Sequence
    );
    render_plantuml_source_svg(source, &LayoutOptions::default(), &PlantUmlTheme::default())
        .expect("class keyword inside a PlantUML block comment should not force class dispatch");
}

#[test]
fn sequence_text_measurement_collection_includes_rendered_page_counter_text() {
    let source = "@startuml\nfooter Page %page% of %lastpage%\nAlice -> Bob: Hello\n@enduml\n";
    let measurements =
        collect_sequence_text_measurements(source, &LayoutOptions::default(), &ApproxTextMeasurer)
            .expect("sequence should collect measurements");

    assert!(
        measurements.iter().any(|text| text == "Page 1 of 1"),
        "{measurements:?}"
    );
}

#[test]
fn external_text_measurements_drive_note_width_and_inline_annotation_bounds() {
    struct FixedTextMeasurer;

    impl TextMeasurer for FixedTextMeasurer {
        fn measure_inline_text_request(
            &self,
            request: &InlineTextMeasurementRequest,
        ) -> InlineTextMeasurementResult {
            let mut x = 0.0;
            let runs = request
                .runs
                .iter()
                .enumerate()
                .map(|(index, run)| {
                    let width = match run.text.as_str() {
                        "prefix " => 100.0,
                        "displayed" => 220.0,
                        "This is " => 123.0,
                        "waved" => 47.0,
                        "A" => 8.0,
                        _ => 24.0,
                    };
                    let measurement = InlineTextRunMeasurement {
                        id: run.id.clone(),
                        index,
                        x,
                        width,
                    };
                    x += width;
                    measurement
                })
                .collect();

            InlineTextMeasurementResult {
                id: request.id.clone(),
                width: x,
                runs,
            }
        }
    }

    let source = concat!(
        "@startuml\n",
        "participant A\n",
        "note right of A\n",
        "prefix <back:yellow><size:18>displayed</size></back>\n",
        "This is <w:#FF33FF>waved</w>\n",
        "end note\n",
        "@enduml\n",
    );
    let rendered = render_sequence_source_svg_with_style_and_text_measurer(
        source,
        &LayoutOptions::default(),
        &PlantUmlTheme::default(),
        &PlantUmlStyle::default(),
        &FixedTextMeasurer,
    )
    .expect("sequence should render with external text metrics");
    let note_group =
        svg_group_containing(&rendered.svg, r#"data-line="3""#).expect("note should render");
    let (note_left, _, note_right, _) = svg_first_rect_bounds(note_group);
    let note_width = note_right - note_left;
    assert!(
        (343.0..345.0).contains(&note_width),
        "note width should use measured line width plus note padding"
    );

    let text = svg_first_element(note_group, "text").expect("note text should render");
    let text_x = attr_value(text, "x")
        .expect("note text should have x")
        .parse::<f32>()
        .expect("note text x should be numeric");
    let background = svg_element_with_classes(note_group, "rect", &["inline-text-background"])
        .expect("background annotation should render");
    let background_x = attr_value(background, "x")
        .expect("background should have x")
        .parse::<f32>()
        .expect("background x should be numeric");
    assert!(
        (98.5..99.5).contains(&(background_x - text_x)),
        "background x should use externally measured prefix width"
    );

    let wave = svg_element_with_classes(note_group, "path", &["inline-text-decoration-wavy"])
        .expect("wavy annotation should render");
    let wave_x = svg_path_moveto_x(wave);
    assert!(
        (122.5..123.5).contains(&(wave_x - text_x)),
        "wavy x should use externally measured prefix width"
    );
}

#[test]
fn parses_sequence_01_participants_and_messages() {
    let diagram = parse_sequence(SEQUENCE_01).expect("fixture should parse");

    assert_eq!(diagram.participants.len(), 2);
    assert_eq!(diagram.participants[0].id, "Alice");
    assert_eq!(diagram.participants[1].id, "Bob");
    assert_eq!(diagram.messages.len(), 4);

    assert_eq!(diagram.messages[0].from, "Alice");
    assert_eq!(diagram.messages[0].to, "Bob");
    assert_eq!(diagram.messages[0].line_style, LineStyle::Solid);

    assert_eq!(diagram.messages[1].from, "Bob");
    assert_eq!(diagram.messages[1].to, "Alice");
    assert_eq!(diagram.messages[1].line_style, LineStyle::Dashed);

    assert_eq!(diagram.messages[2].from, "Alice");
    assert_eq!(diagram.messages[2].to, "Bob");
    assert_eq!(diagram.messages[2].line_style, LineStyle::Solid);

    assert_eq!(diagram.messages[3].from, "Bob");
    assert_eq!(diagram.messages[3].to, "Alice");
    assert_eq!(diagram.messages[3].line_style, LineStyle::Dashed);
}

#[test]
fn layout_has_positive_positions_and_monotonic_events() {
    let diagram = parse_sequence(SEQUENCE_01).expect("fixture should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());

    assert!(layout.width > 0.0);
    assert!(layout.height > 0.0);
    assert!(layout.view_box.starts_with("0 0 "));
    assert!(layout
        .participants
        .iter()
        .all(|participant| participant.x > 0.0));
    assert!(layout.messages.iter().all(|message| message.y > 0.0));

    for pair in layout.messages.windows(2) {
        assert!(pair[0].y < pair[1].y);
    }
}

#[test]
fn parses_message_when_participant_name_matches_declaration_keyword() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_005).expect("fixture 005 should parse");

    let participant_ids = diagram
        .participants
        .iter()
        .map(|participant| participant.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(participant_ids, ["Participant", "Bob"]);
    assert!(!participant_ids.contains(&"->"));

    assert_eq!(diagram.messages.len(), 1);
    assert_eq!(diagram.messages[0].from, "Participant");
    assert_eq!(diagram.messages[0].to, "Bob");

    let display = diagram.participants[0]
        .declaration
        .as_ref()
        .and_then(|declaration| declaration.display.as_ref())
        .expect("multiline participant display should be retained");
    assert_eq!(
        display.lines,
        ["    =Title", "    ----", "    \"\"SubTitle\"\""]
    );
}

#[test]
fn widens_participant_spacing_to_fit_horizontal_message_labels() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_001).expect("fixture 001 should parse");
    let options = LayoutOptions::default();
    let layout = layout_sequence(&diagram, &options);

    let alice = layout
        .participants
        .iter()
        .find(|participant| participant.id == "Alice")
        .expect("Alice should be laid out");
    let bob = layout
        .participants
        .iter()
        .find(|participant| participant.id == "Bob")
        .expect("Bob should be laid out");

    assert!(
        bob.x - alice.x > options.participant_gap,
        "Alice/Bob spacing should expand beyond the fixed default gap for long message labels"
    );
}

#[test]
fn renders_declared_participant_kinds_with_kind_specific_shapes() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_002).expect("fixture 002 should parse");
    let kinds = diagram
        .participants
        .iter()
        .map(|participant| participant.kind)
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            ParticipantKind::Participant,
            ParticipantKind::Actor,
            ParticipantKind::Boundary,
            ParticipantKind::Control,
            ParticipantKind::Entity,
            ParticipantKind::Database,
            ParticipantKind::Collections,
            ParticipantKind::Queue,
        ]
    );

    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    for kind in [
        "participant",
        "actor",
        "boundary",
        "control",
        "entity",
        "database",
        "collections",
        "queue",
    ] {
        assert!(
            rendered
                .svg
                .contains(&format!(r#"data-participant-kind="{kind}""#)),
            "rendered SVG should expose participant kind {kind:?}"
        );
        assert!(
            rendered.svg.contains(&format!("participant-kind-{kind}")),
            "rendered SVG should use kind-specific participant shape class for {kind:?}"
        );
    }
}

#[test]
fn renders_participant_declaration_colors() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_003).expect("fixture 003 should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    assert!(
        rendered.svg.contains(r#"fill="red""#),
        "named declaration color #red should render as SVG color red"
    );
    assert!(
        rendered.svg.contains(r##"fill="#99FF99""##),
        "hex declaration color should render on participant head/tail"
    );
    assert!(
        !rendered.svg.contains(r##"fill="#red""##),
        "PlantUML named colors should not be emitted as invalid SVG hex colors"
    );
    assert!(rendered.svg.contains(">I have a really<"));
    assert!(rendered.svg.contains(">long name<"));
    assert!(
        !rendered.svg.contains(r"I have a really\nlong name"),
        "participant display names should render escaped \\n as separate SVG text lines"
    );
}

#[test]
fn renders_named_plantuml_colors_on_messages_and_blocks_as_svg_colors() {
    let source = r##"@startuml
box "Internal" #LightBlue
Bob -[#red]> Alice : hello
Alice -> Bob --++ #gold : ok
end box
@enduml
"##;
    let diagram = parse_sequence(source).expect("named PlantUML colors should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    assert!(rendered.svg.contains(r#"stroke="red""#));
    assert!(rendered.svg.contains(r#"fill="gold""#));
    assert!(rendered.svg.contains(r#"fill="LightBlue""#));
    assert!(!rendered.svg.contains(r##"stroke="#red""##));
    assert!(!rendered.svg.contains(r##"fill="#gold""##));
    assert!(!rendered.svg.contains(r##"fill="#LightBlue""##));
}

#[test]
fn parses_colored_dashed_message_operator() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_015).expect("fixture 015 should parse");

    assert_eq!(diagram.messages.len(), 2);
    assert_eq!(diagram.messages[0].label, "hello");
    assert_eq!(diagram.messages[0].line_style, LineStyle::Solid);
    assert_eq!(diagram.messages[1].label, "ok");
    assert_eq!(diagram.messages[1].line_style, LineStyle::Dashed);

    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());
    let ok_group = svg_group_containing(&rendered.svg, r#"data-line="3""#)
        .expect("fixture 015 ok message should render");
    assert!(
        message_line(ok_group).contains("stroke-dasharray"),
        "colored dashed arrow should render a dashed line"
    );
}

#[test]
fn renders_plantuml_sequence_arrowhead_variants() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_014).expect("fixture 014 should parse");
    let messages = diagram
        .events
        .iter()
        .filter_map(|event| match event {
            SequenceEvent::Message(message) => Some(message),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(messages.len(), 10);
    assert!(messages[0].arrow.features.end_cross);
    assert_eq!(
        messages[2].arrow.features.end_head,
        Some(MessageArrowHead::Open)
    );
    assert_eq!(
        messages[3].arrow.features.end_head,
        Some(MessageArrowHead::HalfBottom)
    );
    assert_eq!(
        messages[4].arrow.features.end_head,
        Some(MessageArrowHead::HalfBottom)
    );
    assert_eq!(messages[4].to.id, "Bob");
    assert_eq!(
        messages[5].arrow.features.end_head,
        Some(MessageArrowHead::HalfTop)
    );
    assert_eq!(messages[5].to.id, "Bob");
    assert!(messages[6].arrow.features.end_circle);
    assert!(messages[7].arrow.features.end_circle);
    assert_eq!(messages[7].to.id, "Bob");
    assert_eq!(
        messages[8].arrow.features.start_head,
        Some(MessageArrowHead::Filled)
    );
    assert_eq!(
        messages[8].arrow.features.end_head,
        Some(MessageArrowHead::Filled)
    );
    assert_eq!(
        messages[9].arrow.features.start_head,
        Some(MessageArrowHead::Filled)
    );
    assert!(messages[9].arrow.features.end_circle);

    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());
    let lost_arrow_group = svg_group_containing(&rendered.svg, r#"data-arrow="-&gt;x""#)
        .expect("fixture 014 lost arrow should render");
    assert!(
        message_line(lost_arrow_group).contains(r#"x2="244""#),
        "lost arrow line should connect to the cross center"
    );
    let open_arrow_group = svg_group_containing(&rendered.svg, r#"data-arrow="-&gt;&gt;""#)
        .expect("fixture 014 open arrow should render");
    assert!(
        message_line(open_arrow_group).contains(r#"x2="244""#),
        "open arrow line should connect to the arrowhead tip"
    );

    for marker in [
        "arrowhead--cross",
        "arrowhead--triangle",
        "arrowhead--open",
        "arrowhead--half-bottom",
        "arrowhead--half-top",
        "arrowhead--circle",
        "data-start-arrow-head=\"filled\"",
        "data-end-arrow-head=\"open\"",
        "data-end-circle=\"true\"",
    ] {
        assert!(
            rendered.svg.contains(marker),
            "fixture 014 rendered SVG should contain marker {marker:?}"
        );
    }
}

#[test]
fn renders_autonumber_as_message_label_prefixes() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_016).expect("fixture 016 should parse");
    assert_eq!(diagram.messages[0].label, "Authentication Request");

    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    assert_eq!(layout.messages[0].label, "<b>1</b> Authentication Request");
    assert_eq!(layout.messages[1].label, "<b>2</b> Authentication Response");

    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());
    assert!(!rendered.svg.contains("semantic-autonumber"));
    assert!(!rendered.svg.contains("autonumber start"));
    assert!(rendered.svg.contains(r#"font-weight="bold">1</tspan>"#));
    assert!(rendered.svg.contains(r#"font-weight="bold">2</tspan>"#));
    assert!(rendered.svg.contains("> Authentication Request<"));
    assert!(rendered.svg.contains("> Authentication Response<"));
}

#[test]
fn renders_autonumber_start_stop_resume_formats() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_019).expect("fixture 019 should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let labels = layout
        .messages
        .iter()
        .map(|message| message.label.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        [
            "<b>[010]</b> Authentication Request",
            "<b>[020]</b> Authentication Response",
            "dummy",
            "<font color=red><b>Message 30</b></font> Yet another authentication Request",
            "<font color=red><b>Message 40</b></font> Yet another authentication Response",
            "dummy",
            "<font color=blue><b>Message 50</b></font> Yet another authentication Request",
            "<font color=blue><b>Message 51</b></font> Yet another authentication Response",
        ]
    );
}

#[test]
fn renders_autonumber_inline_markup_on_prefix_only() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_018).expect("fixture 018 should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    for marker in [
        r#"font-weight="bold">[001]</tspan>"#,
        r#"text-decoration="underline">15</tspan>"#,
        r##"fill="#FF0000">Message 40</tspan>"##,
        r#"<tspan> Authentication Request</tspan>"#,
        r#"<tspan> Another authentication Request</tspan>"#,
        r#"<tspan> Yet another authentication Request</tspan>"#,
    ] {
        assert!(
            rendered.svg.contains(marker),
            "fixture 018 autonumber SVG should contain marker {marker:?}"
        );
    }
    assert!(!rendered.svg.contains("&lt;b&gt;"));
    assert!(!rendered.svg.contains("&lt;font"));
}

#[test]
fn renders_autonumber_variables_unicode_escapes_and_creole_markup() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_021).expect("fixture 021 should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());

    assert_eq!(layout.messages[0].label, "<b>10</b>");
    assert_eq!(
        layout.messages[1].label,
        "<b>11</b> //This is the response 11//"
    );

    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());
    for marker in [
        "the %autonumber% works everywhere.",
        r#"font-weight="bold">10"#,
        r#"font-style="italic">This is the response 11"#,
    ] {
        assert!(
            rendered.svg.contains(marker),
            "fixture 021 rendered SVG should contain marker {marker:?}"
        );
    }
    assert!(!rendered.svg.contains("&lt;U+0025&gt;"));
    assert!(!rendered.svg.contains("%autonumber%//"));
    assert!(!rendered.svg.contains(">  the %autonumber%"));

    let note_group = svg_group_containing(&rendered.svg, r#"data-event="note""#)
        .expect("note group should render");
    assert!(note_group.contains(r#"data-targets="Bob""#));
    assert!(note_group.contains(r#"text-anchor="start""#));
    let note_rect = svg_first_element(note_group, "rect").expect("note rect should render");
    let note_width = attr_value(note_rect, "width")
        .expect("note rect should have width")
        .parse::<f32>()
        .expect("note width should be numeric");
    let note_x = attr_value(note_rect, "x")
        .expect("note rect should have x")
        .parse::<f32>()
        .expect("note x should be numeric");
    let note_y = attr_value(note_rect, "y")
        .expect("note rect should have y")
        .parse::<f32>()
        .expect("note y should be numeric");
    let note_height = attr_value(note_rect, "height")
        .expect("note rect should have height")
        .parse::<f32>()
        .expect("note height should be numeric");
    let bob_x = layout
        .participants
        .iter()
        .find(|participant| participant.id == "Bob")
        .expect("Bob should be laid out")
        .x;
    assert!(note_width > 240.0);
    assert!(note_x > bob_x);
    assert!((note_y + note_height / 2.0 - layout.messages[0].y).abs() < f32::EPSILON);
}

#[test]
fn renders_page_header_footer_and_title_as_page_chrome() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_022).expect("fixture 022 should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    for marker in [
        r#"<title>Example Title</title>"#,
        r#"class="sequence-header" data-event="header""#,
        r#"class="sequence-title" data-event="title""#,
        r#"class="sequence-footer" data-event="footer""#,
        "Page Header",
        "Page 1 of 1",
        "Example Title",
    ] {
        assert!(
            rendered.svg.contains(marker),
            "fixture 022 SVG should contain marker {marker:?}"
        );
    }
    for marker in [
        "semantic-header",
        "semantic-footer",
        "semantic-title",
        "header: Page Header",
        "footer: Page %page% of %lastpage%",
    ] {
        assert!(
            !rendered.svg.contains(marker),
            "fixture 022 SVG should not contain marker {marker:?}"
        );
    }
    assert!(layout.messages[0].y < 160.0);
}

#[test]
fn renders_newpage_as_page_break_separator() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_023).expect("fixture 023 should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    assert_eq!(count_svg_class(&rendered.svg, "page-break"), 2);
    assert!(!rendered.svg.contains("semantic-newpage"));

    let first_break = svg_group_containing(&rendered.svg, r#"data-line="6""#)
        .expect("first newpage separator should render");
    assert!(first_break.contains(r#"class="sequence-event page-break""#));
    assert!(first_break.contains("<line "));
    assert!(first_break.contains(r#"stroke-dasharray="2 4""#));
    assert!(!first_break.contains("<rect "));
    assert!(!first_break.contains(">newpage<"));

    let titled_break = svg_group_containing(&rendered.svg, r#"data-line="11""#)
        .expect("titled newpage separator should render");
    assert!(titled_break.contains("A title for the"));
    assert!(titled_break.contains("last page"));
    assert!(!titled_break.contains("<rect "));
}

#[test]
fn renders_alt_groups_as_framed_blocks_with_header_boxes() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_024).expect("fixture 024 should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    assert_eq!(count_svg_class(&rendered.svg, "group-frame"), 3);
    for marker in [
        r#"class="sequence-event group-frame group-frame-alt""#,
        r#"class="sequence-event group-frame group-frame-group""#,
        r#"class="sequence-event group-frame group-frame-loop""#,
        r#"class="group-label-box""#,
        r#"class="group-label" data-event="group-label""#,
        r#"class="group-condition" data-event="group-condition""#,
        ">alt</tspan>",
        ">[successful case]</tspan>",
        ">[some kind of failure]</tspan>",
        ">[Another type of failure]</tspan>",
        ">My own label</tspan>",
        ">loop</tspan>",
        ">[1000 times]</tspan>",
    ] {
        assert!(
            rendered.svg.contains(marker),
            "fixture 024 SVG should contain marker {marker:?}"
        );
    }

    assert!(!rendered.svg.contains(">end</text>"));
    assert!(!rendered.svg.contains(">alt successful case</tspan>"));
    assert!(!rendered.svg.contains(">else some kind of failure</tspan>"));
}

#[test]
fn renders_group_secondary_label_outside_header_box() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_025).expect("fixture 025 should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    let group_frame = svg_group_containing(&rendered.svg, r#"data-line="4""#)
        .expect("fixture 025 group frame should render");

    assert!(group_frame.contains(r#"class="group-label-box""#));
    assert!(group_frame.contains(r#"class="group-label" data-event="group-label""#));
    assert!(group_frame.contains(r#"class="group-condition" data-event="group-condition""#));
    assert!(group_frame.contains(">My own label</tspan>"));
    assert!(group_frame.contains(">[My own label 2]</tspan>"));
    assert!(!group_frame.contains(">My own label [My own label 2]</tspan>"));
}

#[test]
fn renders_partition_notes_inside_group_frames() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_026).expect("fixture 026 should parse");
    let groups = diagram
        .events
        .iter()
        .filter_map(|event| match event {
            SequenceEvent::GroupStart(group) => Some(group),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].kind, GroupKind::Partition);
    assert_eq!(groups[1].kind, GroupKind::Partition);

    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    for marker in [
        r#"class="sequence-event group-frame group-frame-partition""#,
        ">partition</tspan>",
        ">[p1]</tspan>",
        ">[p2]</tspan>",
    ] {
        assert!(
            rendered.svg.contains(marker),
            "fixture 026 rendered SVG should contain marker {marker:?}"
        );
    }

    assert_rect_contains_rect(
        svg_group_containing(&rendered.svg, r#"data-line="4""#)
            .expect("partition p1 frame should render"),
        svg_group_containing(&rendered.svg, r#"data-line="7""#).expect("right note should render"),
    );
    assert_rect_contains_rect(
        svg_group_containing(&rendered.svg, r#"data-line="10""#)
            .expect("partition p2 frame should render"),
        svg_group_containing(&rendered.svg, r#"data-line="12""#).expect("left note should render"),
    );
    assert_rect_horizontal_padding_at_least(
        svg_group_containing(&rendered.svg, r#"data-line="10""#)
            .expect("partition p2 frame should render"),
        svg_group_containing(&rendered.svg, r#"data-line="12""#).expect("left note should render"),
        10.0,
    );
}

#[test]
fn renders_teoz_pragma_partition_with_puma_style_frames() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_027).expect("fixture 027 should parse");
    assert!(
        diagram.events.iter().any(|event| {
            matches!(
                event,
                SequenceEvent::Pragma(pragma)
                    if pragma.key == "teoz" && pragma.value.as_deref() == Some("true")
            )
        }),
        "fixture 027 should contain the teoz pragma"
    );

    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    for marker in [
        r#"class="sequence-event group-frame group-frame-partition""#,
        ">partition</tspan>",
        ">[p1]</tspan>",
        ">[p2]</tspan>",
    ] {
        assert!(
            rendered.svg.contains(marker),
            "fixture 027 rendered SVG should keep puma-style partition marker {marker:?}"
        );
    }

    assert_rect_horizontal_padding_at_least(
        svg_group_containing(&rendered.svg, r#"data-line="11""#)
            .expect("partition p2 frame should render"),
        svg_group_containing(&rendered.svg, r#"data-line="13""#).expect("left note should render"),
        10.0,
    );
    assert!(!rendered.svg.contains("!pragma teoz true"));
}

#[test]
fn renders_attached_left_notes_left_of_messages_with_left_aligned_multiline_text() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_028).expect("fixture 028 should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    let hello_group = svg_group_containing(&rendered.svg, r#"data-line="2""#)
        .expect("hello message should render");
    let hello_line = message_line(hello_group);
    let hello_start_x = attr_value(hello_line, "x1")
        .expect("hello line should have x1")
        .parse::<f32>()
        .expect("hello x1 should be numeric");

    let first_note_group =
        svg_group_containing(&rendered.svg, r#"data-line="3""#).expect("first note should render");
    let (_, _, first_note_right, _) = svg_first_rect_bounds(first_note_group);
    assert!(
        first_note_right <= hello_start_x,
        "first left note should stay left of the hello message, note_right={first_note_right}, hello_start_x={hello_start_x}"
    );

    let multiline_note_group = svg_group_containing(&rendered.svg, r#"data-line="9""#)
        .expect("multiline note should render");
    let text = svg_first_element(multiline_note_group, "text").expect("note text should render");
    let text_x = attr_value(text, "x").expect("note text should have x");
    assert_eq!(attr_value(text, "text-anchor"), Some("start"));

    let tspan_x_values = svg_elements(multiline_note_group, "tspan")
        .into_iter()
        .filter_map(|element| attr_value(element, "x"))
        .collect::<Vec<_>>();
    assert_eq!(tspan_x_values, [text_x, text_x, text_x]);
    assert!(multiline_note_group.contains(">a note</tspan>"));
    assert!(multiline_note_group.contains(">can also be defined</tspan>"));
    assert!(multiline_note_group.contains(">on several lines</tspan>"));

    let self_message = layout
        .messages
        .iter()
        .find(|message| message.line == 8)
        .expect("self message should be laid out");
    let self_message_group = svg_group_containing(&rendered.svg, r#"data-line="8""#)
        .expect("self message should render");
    let self_message_label =
        svg_first_element(self_message_group, "text").expect("self message label should render");
    let self_message_label_x = attr_value(self_message_label, "x")
        .expect("self message label should have x")
        .parse::<f32>()
        .expect("self message label x should be numeric");
    assert_eq!(attr_value(self_message_label, "text-anchor"), Some("start"));
    assert!((self_message_label_x - (self_message.start_x + 8.0)).abs() < 0.01);
}

#[test]
fn renders_note_background_colors() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_029).expect("fixture 029 should parse");
    let notes = diagram
        .events
        .iter()
        .filter_map(|event| match event {
            SequenceEvent::Note(note) => Some(note),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(notes.len(), 5);
    assert_eq!(notes[0].color.as_deref(), Some("#aqua"));
    assert_eq!(notes[3].color.as_deref(), Some("#FFAAAA"));

    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    let aqua_note =
        svg_group_containing(&rendered.svg, r#"data-line="4""#).expect("aqua note should render");
    let aqua_rect = svg_first_element(aqua_note, "rect").expect("aqua note rect should render");
    assert_eq!(attr_value(aqua_rect, "fill"), Some("aqua"));

    let hex_note = svg_group_containing(&rendered.svg, r#"data-line="13""#)
        .expect("hex color note should render");
    let hex_rect = svg_first_element(hex_note, "rect").expect("hex note rect should render");
    assert_eq!(attr_value(hex_rect, "fill"), Some("#FFAAAA"));

    assert!(!rendered.svg.contains(r##"fill="#aqua""##));
}

#[test]
fn renders_hnote_as_hexagon_and_rnote_as_rectangle() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_030).expect("fixture 030 should parse");
    let notes = diagram
        .events
        .iter()
        .filter_map(|event| match event {
            SequenceEvent::Note(note) => Some(note),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(notes.len(), 4);
    assert_eq!(notes[0].shape, NoteShape::Hexagon);
    assert_eq!(notes[1].shape, NoteShape::Rectangle);
    assert_eq!(notes[2].shape, NoteShape::Rectangle);
    assert_eq!(notes[3].shape, NoteShape::Hexagon);

    let rendered = render_sequence_source_svg(
        SEQUENCE_FIXTURE_030,
        &LayoutOptions::default(),
        &PlantUmlTheme::default(),
    )
    .expect("fixture 030 renders");

    let idle_note =
        svg_group_containing(&rendered.svg, r#"data-line="3""#).expect("idle hnote should render");
    assert!(idle_note.contains(r#"data-note-shape="hexagon""#));
    assert!(idle_note.contains("<polygon "));
    assert!(!idle_note.contains("<rect "));

    let multiline_hnote = svg_group_containing(&rendered.svg, r#"data-line="14""#)
        .expect("multiline hnote should render");
    assert!(multiline_hnote.contains(r#"data-note-shape="hexagon""#));
    assert!(multiline_hnote.contains("<polygon "));
    assert!(!multiline_hnote.contains("<rect "));

    for line in ["5", "9"] {
        let rnote = svg_group_containing(&rendered.svg, &format!(r#"data-line="{line}""#))
            .expect("rnote should render");
        assert!(rnote.contains(r#"data-note-shape="rectangle""#));
        assert!(rnote.contains("<rect "));
        assert!(!rnote.contains("<polygon "));
    }
}

#[test]
fn renders_creole_monospace_quotes_without_literal_quotes() {
    let rendered = render_sequence_source_svg(
        SEQUENCE_FIXTURE_031,
        &LayoutOptions::default(),
        &PlantUmlTheme::default(),
    )
    .expect("fixture 031 renders");

    let old_method_note = svg_group_containing(&rendered.svg, r#"data-line="4""#)
        .expect("old method note should render");
    assert!(old_method_note.contains("Old method for note over all part. with:"));
    assert!(old_method_note.contains(r#"font-family="monospace">note over "#));
    assert!(old_method_note
        .contains(r#"font-style="italic" font-family="monospace">FirstPart, LastPart"#));
    assert!(!old_method_note.contains(r#">""note over "#));
    assert!(!old_method_note.contains(r#"LastPart</tspan><tspan>"".</tspan>"#));

    let across_note =
        svg_group_containing(&rendered.svg, r#"data-line="5""#).expect("across note should render");
    assert!(across_note.contains(r#"font-family="monospace">note across</tspan>"#));
    assert!(!across_note.contains(r#"""note across"""#));
}

#[test]
fn note_targets_define_implicit_participant_order_before_messages() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_032).expect("fixture 032 should parse");
    let participant_ids = diagram
        .participants
        .iter()
        .map(|participant| participant.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(participant_ids, ["Alice", "Bob"]);

    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let hello = layout
        .messages
        .iter()
        .find(|message| message.line == 4)
        .expect("hello message should be laid out");
    assert_eq!(hello.from, "Bob");
    assert_eq!(hello.to, "Alice");
    assert!(
        hello.start_x > hello.end_x,
        "Bob -> Alice should render as a right-to-left arrow when notes introduced Alice before Bob"
    );

    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());
    for (line, participant_id) in [("2", "Alice"), ("3", "Bob")] {
        let participant_x = layout
            .participants
            .iter()
            .find(|participant| participant.id == participant_id)
            .expect("note target participant should be laid out")
            .x;
        let note_group = svg_group_containing(&rendered.svg, &format!(r#"data-line="{line}""#))
            .expect("standalone note should render");
        let (left, _, right, _) = svg_first_rect_bounds(note_group);
        let note_center = (left + right) / 2.0;
        assert!(
            (note_center - participant_x).abs() < 0.01,
            "standalone note on line {line} should be centered on {participant_id}'s lifeline"
        );
    }

    let hello_group =
        svg_group_containing(&rendered.svg, r#"data-line="4""#).expect("hello should render");
    assert!(hello_group.contains(r#"data-from="Bob""#));
    assert!(hello_group.contains(r#"data-to="Alice""#));
    let line = message_line(hello_group);
    let x1 = attr_value(line, "x1")
        .expect("hello line should have x1")
        .parse::<f32>()
        .expect("hello x1 should be numeric");
    let x2 = attr_value(line, "x2")
        .expect("hello line should have x2")
        .parse::<f32>()
        .expect("hello x2 should be numeric");
    assert!(x1 > x2, "hello line should point from right to left");
}

#[test]
fn renders_decorated_note_targets_before_messages_with_visual_arrow_direction() {
    let rendered = render_sequence_source_svg(
        SEQUENCE_FIXTURE_033,
        &LayoutOptions::default(),
        &PlantUmlTheme::default(),
    )
    .expect("fixture 033 renders");

    let hello_group =
        svg_group_containing(&rendered.svg, r#"data-line="4""#).expect("hello should render");
    assert!(hello_group.contains(r#"data-from="Bob""#));
    assert!(hello_group.contains(r#"data-to="Alice""#));
    assert!(hello_group.contains(r#"data-direction="right-to-left""#));

    let line = message_line(hello_group);
    let x1 = attr_value(line, "x1")
        .expect("hello line should have x1")
        .parse::<f32>()
        .expect("hello x1 should be numeric");
    let x2 = attr_value(line, "x2")
        .expect("hello line should have x2")
        .parse::<f32>()
        .expect("hello x2 should be numeric");
    assert!(x1 > x2, "hello line should point from right to left");
}

#[test]
fn renders_sequence_creole_and_html_text_decorations() {
    let rendered = render_sequence_source_svg(
        SEQUENCE_FIXTURE_034,
        &LayoutOptions::default(),
        &PlantUmlTheme::default(),
    )
    .expect("fixture 034 renders");

    assert!(rendered
        .svg
        .contains(r#"font-weight="bold">Famous</tspan>"#));
    assert!(!rendered.svg.contains("The **Famous** Bob"));

    let hello_group =
        svg_group_containing(&rendered.svg, r#"data-line="5""#).expect("hello should render");
    assert!(hello_group.contains(r#"text-decoration="line-through">there</tspan>"#));
    assert!(!hello_group.contains("--there--"));

    let delay_group =
        svg_group_containing(&rendered.svg, r#"data-line="6""#).expect("delay should render");
    assert!(delay_group.contains(r#"inline-text-decoration-wavy""#));
    assert!(delay_group.contains(">long delay</tspan>"));
    assert!(!delay_group.contains("~~long delay~~"));

    let formatting_note = svg_group_containing(&rendered.svg, r#"data-line="8""#)
        .expect("formatting note should render");
    for marker in [
        r#"font-weight="bold">bold</tspan>"#,
        r#"font-style="italic">italics</tspan>"#,
        r#"font-family="monospace">monospaced</tspan>"#,
        r#"text-decoration="line-through">stroked</tspan>"#,
        r#"text-decoration="underline">underlined</tspan>"#,
        r#">waved</tspan>"#,
    ] {
        assert!(
            formatting_note.contains(marker),
            "formatting note should contain marker {marker:?}"
        );
    }
    assert!(formatting_note.contains(r#"inline-text-decoration-wavy""#));
    let waved_decoration =
        svg_element_with_classes(formatting_note, "path", &["inline-text-decoration-wavy"])
            .expect("waved text should render a wavy path");
    let waved_x = svg_path_moveto_x(waved_decoration);
    assert!(
        (68.0..72.0).contains(&waved_x),
        "waved decoration should start after the measured `This is ` prefix"
    );
    for raw_marker in [
        "\"\"monospaced\"\"",
        "--stroked--",
        "__underlined__",
        "~~waved~~",
    ] {
        assert!(
            !formatting_note.contains(raw_marker),
            "formatting note should consume raw marker {raw_marker:?}"
        );
    }

    let right_note =
        svg_group_containing(&rendered.svg, r#"data-line="18""#).expect("right note should render");
    assert!(right_note.contains(r#"font-size="18""#));
    assert!(right_note.contains(r#"data-background-color="cadetblue""#));
    assert!(right_note.contains(r#"class="inline-text-background""#));
    assert!(right_note.contains(r#"fill="cadetblue""#));
    assert!(!right_note.contains("paint-order:stroke"));
    let displayed_background =
        svg_element_with_classes(right_note, "rect", &["inline-text-background"])
            .expect("displayed text should render a background rect");
    let displayed_background_x = attr_value(displayed_background, "x")
        .expect("displayed background should have x")
        .parse::<f32>()
        .expect("displayed background x should be numeric");
    let right_note_text =
        svg_first_element(right_note, "text").expect("right note should have text");
    let right_note_text_x = attr_value(right_note_text, "x")
        .expect("right note text should have x")
        .parse::<f32>()
        .expect("right note text x should be numeric");
    assert!(
        (43.0..46.0).contains(&(displayed_background_x - right_note_text_x)),
        "displayed background should align with the displayed text"
    );
    assert!(right_note.contains(r#"text-decoration="underline">left of</tspan>"#));
    assert!(!right_note.contains("&lt;back:cadetblue&gt;"));
    assert!(!right_note.contains("__left of__"));

    let left_note =
        svg_group_containing(&rendered.svg, r#"data-line="22""#).expect("left note should render");
    assert!(left_note.contains(r#"style="text-decoration-color: #FF0000">This</tspan>"#));
    assert!(!left_note.contains(
        r#"text-decoration="underline" style="text-decoration-color: #FF0000">This</tspan>"#
    ));
    assert!(left_note.contains(r#"inline-text-decoration-underline""#));
    assert!(left_note.contains(r##"fill="#118888">displayed</tspan>"##));
    assert!(left_note.contains(r##"font-weight="bold" fill="#800080">left of</tspan>"##));
    assert!(!left_note.contains(
        r#"text-decoration="line-through" style="text-decoration-color: #FF0000">Alice</tspan>"#
    ));
    assert!(left_note.contains(r#"inline-text-decoration-line-through""#));
    let alice_strike =
        svg_element_with_classes(left_note, "line", &["inline-text-decoration-line-through"])
            .expect("Alice should render a colored strike line");
    let alice_strike_x1 = attr_value(alice_strike, "x1")
        .expect("Alice strike should have x1")
        .parse::<f32>()
        .expect("Alice strike x1 should be numeric");
    let alice_strike_x2 = attr_value(alice_strike, "x2")
        .expect("Alice strike should have x2")
        .parse::<f32>()
        .expect("Alice strike x2 should be numeric");
    assert!(
        alice_strike_x2 - alice_strike_x1 < 34.0,
        "colored strike should cover Alice only, not Bob"
    );
    assert!(left_note.contains(">Alice</tspan>"));
    assert!(!left_note.contains("&lt;color"));
    assert!(!left_note.contains("&lt;s:red&gt;"));

    let message_group = svg_group_containing(&rendered.svg, r#"data-line="17""#)
        .expect("formatted message should render");
    let message_y = attr_value(message_line(message_group), "y1")
        .expect("formatted message line should have y1")
        .parse::<f32>()
        .expect("formatted message y1 should be numeric");
    let (_, right_note_top, _, right_note_bottom) = svg_first_rect_bounds(right_note);
    assert!(
        right_note_top > message_y,
        "line 18 note should start below the formatted message line"
    );
    let (_, left_note_top, _, left_note_bottom) = svg_first_rect_bounds(left_note);
    assert!(
        right_note_bottom <= left_note_top || left_note_bottom <= right_note_top,
        "line 18 and line 22 notes should not overlap vertically"
    );

    let hosted_note = svg_group_containing(&rendered.svg, r#"data-line="26""#)
        .expect("hosted note should render");
    assert!(hosted_note.contains(r#"inline-text-decoration-wavy""#));
    assert!(!hosted_note.contains(r#"text-decoration="wavy underline""#));
    assert!(hosted_note.contains(">This is hosted</tspan>"));
    assert!(hosted_note.contains(r#"font-family="monospace">(Cannot decode)</tspan>"#));
    assert!(!hosted_note.contains("&lt;w:#FF33FF&gt;"));
    assert!(!hosted_note.contains("&lt;img sourceforge.jpg&gt;"));
}

#[test]
fn renders_multiline_participant_display_blocks() {
    let diagram = parse_sequence(SEQUENCE_FIXTURE_005).expect("fixture 005 should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    assert!(count_svg_class(&rendered.svg, "message") > 0);
    assert!(!rendered.svg.contains(r#"data-participant="-&gt;""#));
    assert!(rendered
        .svg
        .contains(r#"class="participant-display-title""#));
    assert!(rendered.svg.contains(r#"font-weight="600">Title</text>"#));
    assert!(rendered
        .svg
        .contains(r#"class="participant-display-subtitle""#));
    assert!(rendered
        .svg
        .contains(r#"font-style="italic">SubTitle</text>"#));
    assert!(!rendered.svg.contains(">    =Title<"));
    assert!(!rendered.svg.contains("\"\"SubTitle\"\""));
    assert!(
        rendered.svg.contains("participant-display-separator"),
        "display block separator should be rendered structurally"
    );
}

#[test]
fn renders_sequence_svg_with_theme_and_groups() {
    let theme = PlantUmlTheme {
        font_family: "Fira Code".to_owned(),
        font_size: 13,
        background_color: "#fffdf7".to_owned(),
        participant_fill: "#eeeeff".to_owned(),
        participant_stroke: "#222244".to_owned(),
        lifeline_color: "#999999".to_owned(),
        message_color: "#884400".to_owned(),
        text_color: "#101010".to_owned(),
    };

    let rendered = render_sequence_source_svg(SEQUENCE_01, &LayoutOptions::default(), &theme)
        .expect("fixture should render");

    assert!(rendered.svg.contains("viewBox=\"0 0"));
    assert!(rendered
        .svg
        .contains(&format!("viewBox=\"{}\"", rendered.view_box)));
    assert!(rendered.svg.contains("width=\""));
    assert!(rendered.svg.contains("height=\""));
    assert!(rendered.svg.contains("data-diagram-type=\"SEQUENCE\""));
    assert!(rendered.svg.contains("class=\"participant\""));
    assert!(count_svg_class(&rendered.svg, "message") > 0);
    assert!(rendered.svg.contains("font-family=\"Fira Code\""));
    assert!(rendered.svg.contains("fill=\"#fffdf7\""));
    assert!(rendered.svg.contains("stroke=\"#884400\""));
}

#[test]
fn default_theme_uses_tailwind_slate_palette() {
    let theme = PlantUmlTheme::default();

    assert_eq!(
        theme.font_family,
        r#"-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif"#
    );
    assert_eq!(theme.font_size, 14);
    assert_eq!(theme.background_color, "#ffffff");
    assert_eq!(theme.participant_fill, "#f8fafc");
    assert_eq!(theme.participant_stroke, "#94a3b8");
    assert_eq!(theme.lifeline_color, "#94a3b8");
    assert_eq!(theme.message_color, "#94a3b8");
    assert_eq!(theme.text_color, "#1e293b");

    let rendered = render_sequence_source_svg(SEQUENCE_01, &LayoutOptions::default(), &theme)
        .expect("fixture should render");

    assert!(rendered.svg.contains(
        r#"font-family="-apple-system, BlinkMacSystemFont, &quot;Segoe UI&quot;, sans-serif""#
    ));
    assert!(rendered.svg.contains(r##"fill="#f8fafc""##));
    assert!(rendered.svg.contains(r##"stroke="#94a3b8""##));
    assert!(rendered.svg.contains(r##"fill="#1e293b""##));
}

#[test]
fn renders_default_participants_without_corner_radius() {
    let diagram = parse_sequence(SEQUENCE_01).expect("fixture should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    assert!(rendered.svg.contains("class=\"participant\""));
    assert!(!rendered.svg.contains(" rx=\""));
    assert!(!rendered.svg.contains(" ry=\""));
}

#[test]
fn renders_participants_with_medium_corner_radius() {
    let diagram = parse_sequence(SEQUENCE_01).expect("fixture should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let style = PlantUmlStyle {
        corner_radius: CornerRadius::Md,
        arrowhead: ArrowheadStyle::Triangle,
        stroke_width: StrokeWidth::Regular,
    };
    let rendered = render_sequence_svg_with_style(&layout, &PlantUmlTheme::default(), &style);

    assert!(rendered.svg.contains("class=\"participant\""));
    assert!(rendered.svg.contains("rx=\"6\""));
    assert!(rendered.svg.contains("ry=\"6\""));
}

#[test]
fn renders_triangle_arrowheads_as_filled_polygons() {
    let diagram = parse_sequence(SEQUENCE_01).expect("fixture should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let style = PlantUmlStyle {
        corner_radius: CornerRadius::None,
        arrowhead: ArrowheadStyle::Triangle,
        stroke_width: StrokeWidth::Regular,
    };
    let rendered = render_sequence_svg_with_style(&layout, &PlantUmlTheme::default(), &style);

    let arrowhead = svg_element_with_classes(
        &rendered.svg,
        "polygon",
        &["arrowhead", "arrowhead--triangle"],
    )
    .expect("triangle arrowhead polygon should render");
    assert!(arrowhead.contains("fill=\""));
}

#[test]
fn renders_chevron_arrowheads_as_stroked_open_paths() {
    let diagram = parse_sequence(SEQUENCE_01).expect("fixture should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let style = PlantUmlStyle {
        corner_radius: CornerRadius::None,
        arrowhead: ArrowheadStyle::Chevron,
        stroke_width: StrokeWidth::Regular,
    };
    let rendered = render_sequence_svg_with_style(&layout, &PlantUmlTheme::default(), &style);

    let arrowhead =
        svg_element_with_classes(&rendered.svg, "path", &["arrowhead", "arrowhead--chevron"])
            .expect("chevron arrowhead path should render");
    assert!(arrowhead.contains("stroke=\""));
    assert!(arrowhead.contains("fill=\"none\""));
    assert!(!has_svg_element_with_classes(
        &rendered.svg,
        "polygon",
        &["arrowhead"]
    ));
}

#[test]
fn renders_kite_arrowheads_as_filled_closed_paths() {
    let diagram = parse_sequence(SEQUENCE_01).expect("fixture should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let theme = PlantUmlTheme {
        font_family: "Fira Code".to_owned(),
        font_size: 13,
        background_color: "#fffdf7".to_owned(),
        participant_fill: "#eeeeff".to_owned(),
        participant_stroke: "#222244".to_owned(),
        lifeline_color: "#999999".to_owned(),
        message_color: "#884400".to_owned(),
        text_color: "#101010".to_owned(),
    };
    let style = PlantUmlStyle {
        corner_radius: CornerRadius::None,
        arrowhead: ArrowheadStyle::Kite,
        stroke_width: StrokeWidth::Regular,
    };
    let rendered = render_sequence_svg_with_style(&layout, &theme, &style);

    let arrowhead =
        svg_element_with_classes(&rendered.svg, "path", &["arrowhead", "arrowhead--kite"])
            .expect("kite arrowhead path should render");
    assert_eq!(attr_value(arrowhead, "fill"), Some("#884400"));
    assert_eq!(attr_value(arrowhead, "stroke"), Some("#884400"));
    assert!(arrowhead.contains(" Z\"") || arrowhead.contains(" z\""));
}

#[test]
fn renders_thin_stroke_width_on_message_lines_and_arrowheads() {
    let diagram = parse_sequence(SEQUENCE_01).expect("fixture should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let style = PlantUmlStyle {
        corner_radius: CornerRadius::None,
        arrowhead: ArrowheadStyle::Chevron,
        stroke_width: StrokeWidth::Thin,
    };
    let rendered = render_sequence_svg_with_style(&layout, &PlantUmlTheme::default(), &style);

    let message = first_message_group(&rendered.svg).expect("message group should render");
    assert!(message_line(message).contains("stroke-width=\"1\""));
    assert!(arrowhead_element(message).contains("stroke-width=\"1\""));
}

#[test]
fn renders_bold_stroke_width_on_message_lines_and_arrowheads() {
    let diagram = parse_sequence(SEQUENCE_01).expect("fixture should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let style = PlantUmlStyle {
        corner_radius: CornerRadius::None,
        arrowhead: ArrowheadStyle::Chevron,
        stroke_width: StrokeWidth::Bold,
    };
    let rendered = render_sequence_svg_with_style(&layout, &PlantUmlTheme::default(), &style);

    let message = first_message_group(&rendered.svg).expect("message group should render");
    assert!(message_line(message).contains("stroke-width=\"3\""));
    assert!(arrowhead_element(message).contains("stroke-width=\"3\""));
}

#[test]
fn renders_different_dashed_message_dasharrays_for_thin_and_bold() {
    let thin = render_with_style(PlantUmlStyle {
        corner_radius: CornerRadius::None,
        arrowhead: ArrowheadStyle::Chevron,
        stroke_width: StrokeWidth::Thin,
    });
    let bold = render_with_style(PlantUmlStyle {
        corner_radius: CornerRadius::None,
        arrowhead: ArrowheadStyle::Chevron,
        stroke_width: StrokeWidth::Bold,
    });

    let thin_dasharray = dashed_message_line_dasharray(&thin.svg);
    let bold_dasharray = dashed_message_line_dasharray(&bold.svg);

    assert_ne!(thin_dasharray, bold_dasharray);
}

#[test]
fn applies_corner_radius_kite_arrowheads_and_bold_strokes_together() {
    let rendered = render_with_style(PlantUmlStyle {
        corner_radius: CornerRadius::Md,
        arrowhead: ArrowheadStyle::Kite,
        stroke_width: StrokeWidth::Bold,
    });

    assert!(rendered.svg.contains("rx=\"6\""));
    assert!(rendered.svg.contains("ry=\"6\""));

    let message = first_message_group(&rendered.svg).expect("message group should render");
    assert!(message_line(message).contains("stroke-width=\"3\""));

    let arrowhead = svg_element_with_classes(message, "path", &["arrowhead", "arrowhead--kite"])
        .expect("kite arrowhead path should render");
    assert!(arrowhead.contains("stroke-width=\"3\""));
    assert!(arrowhead.contains("fill=\""));
    assert!(arrowhead.contains("stroke=\""));
}

#[test]
fn source_renderer_applies_theme_and_style_together() {
    let theme = PlantUmlTheme {
        font_family: "Fira Code".to_owned(),
        font_size: 13,
        background_color: "#fffdf7".to_owned(),
        participant_fill: "#eeeeff".to_owned(),
        participant_stroke: "#222244".to_owned(),
        lifeline_color: "#999999".to_owned(),
        message_color: "#884400".to_owned(),
        text_color: "#101010".to_owned(),
    };
    let style = PlantUmlStyle {
        corner_radius: CornerRadius::Md,
        arrowhead: ArrowheadStyle::Chevron,
        stroke_width: StrokeWidth::Regular,
    };

    let rendered = render_sequence_source_svg_with_style(
        SEQUENCE_01,
        &LayoutOptions::default(),
        &theme,
        &style,
    )
    .expect("fixture should render");

    assert!(rendered.svg.contains("font-family=\"Fira Code\""));
    assert!(rendered.svg.contains("fill=\"#fffdf7\""));
    assert!(rendered.svg.contains("stroke=\"#884400\""));
    assert!(rendered.svg.contains("rx=\"6\""));
    assert!(rendered.svg.contains("ry=\"6\""));
    assert!(has_svg_element_with_classes(
        &rendered.svg,
        "path",
        &["arrowhead", "arrowhead--chevron"]
    ));
    assert!(!has_svg_element_with_classes(
        &rendered.svg,
        "polygon",
        &["arrowhead"]
    ));
}

#[test]
fn compares_key_surface_with_reference_svg() {
    assert!(SEQUENCE_01_REFERENCE_SVG.contains("data-diagram-type=\"SEQUENCE\""));
    assert_eq!(count_svg_class(SEQUENCE_01_REFERENCE_SVG, "participant"), 4);
    assert_eq!(count_svg_class(SEQUENCE_01_REFERENCE_SVG, "message"), 4);

    let rendered = render_sequence_source_svg(
        SEQUENCE_01,
        &LayoutOptions::default(),
        &PlantUmlTheme::default(),
    )
    .expect("fixture should render");

    for text in [
        "Alice",
        "Bob",
        "Authentication Request",
        "Authentication Response",
        "Another authentication Request",
        "Another authentication Response",
    ] {
        assert!(SEQUENCE_01_REFERENCE_SVG.contains(text));
        assert!(rendered.svg.contains(text));
    }

    assert_eq!(
        count_svg_class(&rendered.svg, "participant"),
        count_svg_class(SEQUENCE_01_REFERENCE_SVG, "participant")
    );
    assert_eq!(
        count_svg_class(&rendered.svg, "message"),
        count_svg_class(SEQUENCE_01_REFERENCE_SVG, "message")
    );
}

#[test]
fn renders_escaped_labels() {
    let source = "@startuml\nAlice -> Bob: <hello & \"world\">\n@enduml\n";
    let diagram = parse_sequence(source).expect("source should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    let rendered = render_sequence_svg(&layout, &PlantUmlTheme::default());

    assert!(rendered.svg.contains("&lt;hello &amp; \"world\"&gt;"));
    assert!(!rendered.svg.contains("<hello &"));
}

#[test]
fn parses_named_startuml_directive() {
    let source = "@startuml named-diagram\nAlice -> Bob: hi\n@enduml\n";

    let diagram = parse_sequence(source).expect("named startuml should parse");
    assert_eq!(diagram.messages.len(), 1);

    let rendered =
        render_plantuml_source_svg(source, &LayoutOptions::default(), &PlantUmlTheme::default())
            .expect("named startuml should dispatch as sequence");
    assert!(rendered.svg.contains("hi"));
}

#[test]
fn parses_message_colon_inside_quoted_endpoints() {
    let source = "@startuml\n\"A:B\" -> \"B:C\": payload: ok\n@enduml\n";

    let diagram = parse_sequence(source).expect("quoted endpoint colons should parse");

    assert_eq!(diagram.participants.len(), 2);
    assert_eq!(diagram.participants[0].id, "A:B");
    assert_eq!(diagram.participants[1].id, "B:C");
    assert_eq!(diagram.messages[0].from, "A:B");
    assert_eq!(diagram.messages[0].to, "B:C");
    assert_eq!(diagram.messages[0].label, "payload: ok");
}

#[test]
fn missing_boundaries_are_errors() {
    assert_eq!(
        parse_sequence("Alice -> Bob: hi\n").unwrap_err(),
        PlantUmlError::MissingStart
    );
    assert_eq!(
        parse_sequence("@startuml\nAlice -> Bob: hi\n").unwrap_err(),
        PlantUmlError::MissingEnd
    );
}

fn count_svg_class(svg: &str, class_name: &str) -> usize {
    svg.match_indices("class=\"")
        .filter(|(start, _)| {
            let value_start = start + "class=\"".len();
            let Some(value_end) = svg[value_start..].find('"') else {
                return false;
            };
            svg[value_start..value_start + value_end]
                .split_ascii_whitespace()
                .any(|name| name == class_name)
        })
        .count()
}

fn render_with_style(style: PlantUmlStyle) -> RenderedPlantUml {
    let diagram = parse_sequence(SEQUENCE_01).expect("fixture should parse");
    let layout = layout_sequence(&diagram, &LayoutOptions::default());
    render_sequence_svg_with_style(&layout, &PlantUmlTheme::default(), &style)
}

fn has_svg_element_with_classes(svg: &str, tag: &str, class_names: &[&str]) -> bool {
    svg_element_with_classes(svg, tag, class_names).is_some()
}

fn svg_element_with_classes<'a>(svg: &'a str, tag: &str, class_names: &[&str]) -> Option<&'a str> {
    let needle = format!("<{tag} ");
    svg.match_indices(&needle).find_map(|(start, _)| {
        let end = svg[start..].find('>')?;
        let element = &svg[start..start + end];
        class_names
            .iter()
            .all(|class_name| svg_element_has_class(element, class_name))
            .then_some(element)
    })
}

fn svg_first_element<'a>(svg: &'a str, tag: &str) -> Option<&'a str> {
    let needle = format!("<{tag} ");
    let start = svg.find(&needle)?;
    let end = svg[start..].find('>')?;
    Some(&svg[start..start + end])
}

fn svg_elements<'a>(svg: &'a str, tag: &str) -> Vec<&'a str> {
    let needle = format!("<{tag} ");
    svg.match_indices(&needle)
        .filter_map(|(start, _)| {
            let end = svg[start..].find('>')?;
            Some(&svg[start..start + end])
        })
        .collect()
}

fn svg_element_has_class(element: &str, class_name: &str) -> bool {
    let Some(class_start) = element.find("class=\"") else {
        return false;
    };
    let value_start = class_start + "class=\"".len();
    let Some(value_end) = element[value_start..].find('"') else {
        return false;
    };
    element[value_start..value_start + value_end]
        .split_ascii_whitespace()
        .any(|name| name == class_name)
}

fn first_message_group(svg: &str) -> Option<&str> {
    let start = svg.match_indices("<g ").find_map(|(start, _)| {
        let end = svg[start..].find('>')?;
        let element = &svg[start..start + end];
        svg_element_has_class(element, "message").then_some(start)
    })?;
    let end = svg[start..].find("</g>")?;
    Some(&svg[start..start + end])
}

fn svg_group_containing<'a>(svg: &'a str, marker: &str) -> Option<&'a str> {
    let marker_start = svg.find(marker)?;
    let start = svg[..marker_start].rfind("<g ")?;
    let end = svg[marker_start..].find("</g>")?;
    Some(&svg[start..marker_start + end])
}

fn assert_rect_contains_rect(container_group: &str, inner_group: &str) {
    let (container_left, container_top, container_right, container_bottom) =
        svg_first_rect_bounds(container_group);
    let (inner_left, inner_top, inner_right, inner_bottom) = svg_first_rect_bounds(inner_group);

    assert!(
        inner_left >= container_left && inner_right <= container_right,
        "inner rect x span {inner_left}..{inner_right} should fit in container {container_left}..{container_right}"
    );
    assert!(
        inner_top >= container_top && inner_bottom <= container_bottom,
        "inner rect y span {inner_top}..{inner_bottom} should fit in container {container_top}..{container_bottom}"
    );
}

fn assert_rect_horizontal_padding_at_least(
    container_group: &str,
    inner_group: &str,
    expected_padding: f32,
) {
    let (container_left, _, container_right, _) = svg_first_rect_bounds(container_group);
    let (inner_left, _, inner_right, _) = svg_first_rect_bounds(inner_group);
    let left_padding = inner_left - container_left;
    let right_padding = container_right - inner_right;

    assert!(
        left_padding >= expected_padding && right_padding >= expected_padding,
        "inner rect should keep at least {expected_padding}px horizontal padding in container, got left={left_padding}, right={right_padding}"
    );
}

fn svg_first_rect_bounds(svg: &str) -> (f32, f32, f32, f32) {
    let rect = svg_first_element(svg, "rect").expect("rect should render");
    let x = attr_value(rect, "x")
        .expect("rect should have x")
        .parse::<f32>()
        .expect("rect x should be numeric");
    let y = attr_value(rect, "y")
        .expect("rect should have y")
        .parse::<f32>()
        .expect("rect y should be numeric");
    let width = attr_value(rect, "width")
        .expect("rect should have width")
        .parse::<f32>()
        .expect("rect width should be numeric");
    let height = attr_value(rect, "height")
        .expect("rect should have height")
        .parse::<f32>()
        .expect("rect height should be numeric");

    (x, y, x + width, y + height)
}

fn dashed_message_line_dasharray(svg: &str) -> &str {
    let start = svg
        .find("data-line-style=\"dashed\"")
        .expect("dashed message should render");
    let group_end = svg[start..]
        .find("</g>")
        .expect("dashed message group should close");
    let group = &svg[start..start + group_end];
    let line = message_line(group);
    attr_value(line, "stroke-dasharray").expect("dashed message line should have dasharray")
}

fn message_line(message_group: &str) -> &str {
    let start = message_group
        .find("<line ")
        .expect("message line should render");
    let end = message_group[start..]
        .find('>')
        .expect("message line should close");
    &message_group[start..start + end]
}

fn arrowhead_element(message_group: &str) -> &str {
    svg_element_with_classes(message_group, "path", &["arrowhead"])
        .or_else(|| svg_element_with_classes(message_group, "polygon", &["arrowhead"]))
        .expect("arrowhead should render")
}

fn svg_path_moveto_x(path: &str) -> f32 {
    let data = attr_value(path, "d").expect("path should have d");
    let moveto = data
        .strip_prefix("M ")
        .or_else(|| data.strip_prefix('M'))
        .expect("path should start with moveto");
    let x = moveto
        .split_once(',')
        .expect("moveto should contain x,y")
        .0
        .trim();
    x.parse::<f32>().expect("moveto x should be numeric")
}

fn attr_value<'a>(element: &'a str, attr_name: &str) -> Option<&'a str> {
    let needle = format!("{attr_name}=\"");
    let value_start = element.find(&needle)? + needle.len();
    let value_end = element[value_start..].find('"')?;
    Some(&element[value_start..value_start + value_end])
}
