use std::fmt;

use serde::{Deserialize, Serialize};

mod backend;
mod backends;
mod parser;
mod pipeline;
mod text_layout;

pub use oriha_diagram::{
    DiagramArrowheadStyle, DiagramCornerRadius, DiagramStrokeWidth, DiagramStyle,
};
pub use pipeline::{Mermaid, MermaidDiagramKind, MermaidError, MermaidOptions, MermaidSvg};

pub(crate) use pipeline::MermaidTheme;

#[doc(hidden)]
pub mod __private {
    pub use crate::pipeline::{
        detect_mermaid_diagram, detect_mermaid_diagram_kind, MermaidDiagramDetection,
        MermaidDiagramDetectionReason,
    };
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MermaidMeasurementRequest {
    pub id: String,
    pub kind: MermaidMeasurementKind,
    pub text: String,
    pub style: MermaidMeasurementStyle,
    pub width: Option<f32>,
    pub max_width: Option<f32>,
    pub wrap_mode: Option<MermaidWrapMode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MermaidMeasurementResult {
    pub id: String,
    pub width: f32,
    pub height: f32,
    pub line_count: usize,
    pub left: Option<f32>,
    pub right: Option<f32>,
    pub raw_width: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MermaidMeasurementStyle {
    pub font_family: Option<String>,
    pub font_size: f32,
    pub font_weight: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MermaidMeasurementKind {
    Text,
    Wrapped,
    WrappedRaw,
    WrappedWithRawWidth,
    SvgComputedLength,
    SvgBBoxX,
    SvgBBoxXAsciiOverhang,
    SvgTitleBBoxX,
    SvgSimpleBBoxWidth,
    SvgSimpleBBoxHeight,
}

impl MermaidMeasurementKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Wrapped => "wrapped",
            Self::WrappedRaw => "wrappedRaw",
            Self::WrappedWithRawWidth => "wrappedWithRawWidth",
            Self::SvgComputedLength => "svgComputedLength",
            Self::SvgBBoxX => "svgBBoxX",
            Self::SvgBBoxXAsciiOverhang => "svgBBoxXAsciiOverhang",
            Self::SvgTitleBBoxX => "svgTitleBBoxX",
            Self::SvgSimpleBBoxWidth => "svgSimpleBBoxWidth",
            Self::SvgSimpleBBoxHeight => "svgSimpleBBoxHeight",
        }
    }
}

impl fmt::Display for MermaidMeasurementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MermaidWrapMode {
    SvgLike,
    SvgLikeSingleRun,
    HtmlLike,
}

impl MermaidWrapMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::SvgLike => "svgLike",
            Self::SvgLikeSingleRun => "svgLikeSingleRun",
            Self::HtmlLike => "htmlLike",
        }
    }
}

impl fmt::Display for MermaidWrapMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub(crate) fn format_css_px(value: f32) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded.fract() == 0.0 {
        format!("{}px", rounded as i32)
    } else {
        format!("{rounded}px")
    }
}

pub(crate) fn finite_positive_f32(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

pub(crate) fn finite_non_negative_f32(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        fallback
    }
}

pub(crate) fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    const FLOWCHART: &str = "flowchart TD\nA[Start] --> B[Done]\n";
    const SEQUENCE: &str = "sequenceDiagram\nparticipant App as Expo App\nApp->>Core: render\n";
    const STATE: &str =
        "stateDiagram-v2\n[*] --> Editing\nEditing --> Rendered: done\nRendered --> [*]\n";

    #[test]
    fn public_api_renders_supported_mermaid_subsets() {
        let flowchart = Mermaid::render(FLOWCHART).expect("flowchart renders");
        assert_eq!(flowchart.kind, MermaidDiagramKind::Flowchart);
        assert!(flowchart.svg.contains("Start"));
        assert!(flowchart.svg.contains("Done"));

        let sequence = Mermaid::render(SEQUENCE).expect("sequence renders");
        assert_eq!(sequence.kind, MermaidDiagramKind::Sequence);
        assert!(sequence.svg.contains("Expo App"));
        assert!(sequence.svg.contains("render"));

        let state = Mermaid::render(STATE).expect("state renders");
        assert_eq!(state.kind, MermaidDiagramKind::State);
        assert!(state.svg.contains("Editing"));
        assert!(state.svg.contains("Rendered"));
    }

    #[test]
    fn renders_basic_flowchart_svg_with_sanitized_mermaid_id() {
        let options = MermaidOptions::default();
        let rendered = Mermaid::new()
            .render_with_measurements(FLOWCHART, "demo one", &options, &BTreeMap::new())
            .expect("flowchart should render");

        assert!(rendered.svg.starts_with("<svg "));
        assert!(rendered.svg.contains(r#"id="mermaid-demo-one""#));
        assert!(rendered.svg.contains("Start"));
        assert!(rendered.svg.contains("Done"));
        assert_eq!(
            rendered.view_box,
            format!("0 0 {} {}", rendered.width, rendered.height)
        );
    }

    #[test]
    fn renders_example_subset_constructs() {
        let renderer = Mermaid::new();
        let flow_options = MermaidOptions::default();
        let flowchart = renderer
            .render_with_measurements(
                "flowchart TD\n  Start([Open Markdown]) --> Parse[Parse blocks and inline spans]\n  Parse --> HasDiagram{Diagram fence?}\n  HasDiagram -- Mermaid --> RenderMermaid[Render Mermaid SVG]\n  HasDiagram -- No --> RenderHtml[Render semantic HTML]\n",
                "flow-example",
                &flow_options,
                &BTreeMap::new(),
            )
            .expect("sample flowchart subset should render");
        assert!(flowchart.svg.contains("Diagram fence?"));
        assert!(flowchart.svg.contains("Mermaid"));
        assert!(flowchart.svg.contains("No"));

        let sequence_options = MermaidOptions::default();
        let sequence = renderer
            .render_with_measurements(
                "sequenceDiagram\n  autonumber\n  participant User\n  participant App as Expo App\n  User->>App: Open document\n  App-->>User: final HTML\n",
                "sequence-example",
                &sequence_options,
                &BTreeMap::new(),
            )
            .expect("sample sequence subset should render");
        assert!(sequence.svg.contains("1. Open document"));
        assert!(sequence.svg.contains("2. final HTML"));

        let state_options = MermaidOptions::default();
        let state = renderer
            .render_with_measurements(
                "stateDiagram-v2\n  [*] --> Editing\n  Editing --> Measuring: contains measured diagram\n  Measuring --> Rendered: metrics returned\n  Rendered --> [*]\n",
                "state-example",
                &state_options,
                &BTreeMap::new(),
            )
            .expect("sample state subset should render");
        assert!(state.svg.contains("contains measured diagram"));
        assert!(state.svg.contains("metrics returned"));
    }

    #[test]
    fn collects_text_measurement_requests_with_stable_ids() {
        let requests = Mermaid::new()
            .collect_text_measurement_requests(FLOWCHART, &MermaidOptions::default())
            .expect("flowchart should collect requests");

        assert!(!requests.is_empty());
        assert!(requests
            .iter()
            .all(|request| request.id.starts_with("mermaid.measurement.")));
        assert!(requests
            .iter()
            .any(|request| request.kind == MermaidMeasurementKind::Wrapped));
    }

    #[test]
    fn replays_external_measurements_and_falls_back_when_missing() {
        let options = MermaidOptions {
            font_family: "Inter".to_owned(),
            body_font_size_px: Some(18.0),
            ..MermaidOptions::default()
        };
        let request = Mermaid::new()
            .collect_text_measurement_requests("flowchart TD\nReplay[Replay]\n", &options)
            .expect("measurement collection should succeed")
            .into_iter()
            .find(|request| request.text == "Replay")
            .expect("Replay text should be measured");
        let mut measurements = BTreeMap::new();
        measurements.insert(
            request.id.clone(),
            MermaidMeasurementResult {
                id: request.id,
                width: 123.0,
                height: 45.0,
                line_count: 2,
                left: None,
                right: None,
                raw_width: None,
            },
        );

        let rendered = Mermaid::new()
            .render_with_measurements(
                "flowchart TD\nReplay[Replay]\n",
                "",
                &options,
                &measurements,
            )
            .expect("render should accept external metrics");
        let fallback = Mermaid::new()
            .render_with_options("flowchart TD\nMissing[Missing]\n", &options)
            .expect("render should fall back without metrics");

        assert!(rendered.svg.contains("Replay"));
        assert!(fallback.svg.contains("Missing"));
    }

    #[test]
    fn options_build_mermaid_config_like_theme_variables() {
        let options = MermaidOptions {
            dark_mode: true,
            font_family: "Inter, sans-serif".to_owned(),
            body_font_size_px: Some(14.0),
            background: Some("#101820".to_owned()),
            primary_color: Some("#234567".to_owned()),
            primary_text_color: Some("#ffffff".to_owned()),
            primary_border_color: Some("#345678".to_owned()),
            line_color: Some("#456789".to_owned()),
            text_color: Some("#eeeeee".to_owned()),
            main_bkg: Some("#123456".to_owned()),
            node_bkg: Some("#223344".to_owned()),
            node_border: Some("#334455".to_owned()),
            cluster_bkg: Some("#445566".to_owned()),
            cluster_border: Some("#556677".to_owned()),
            edge_label_background: Some("#667788".to_owned()),
            style: DiagramStyle {
                corner_radius: Some(DiagramCornerRadius::Xl),
                arrowhead: Some(DiagramArrowheadStyle::Kite),
                stroke_width: Some(DiagramStrokeWidth::Bold),
            },
        };
        let config = options.to_mermaid_config_value();

        assert_eq!(config["theme"], "base");
        assert_eq!(config["darkMode"], true);
        assert_eq!(config["fontFamily"], "Inter, sans-serif");
        assert_eq!(config["fontSize"], 14.0);
        assert_eq!(config["themeVariables"]["fontFamily"], "Inter, sans-serif");
        assert_eq!(config["themeVariables"]["fontSize"], "14px");
        assert_eq!(config["themeVariables"]["background"], "#101820");
        assert_eq!(config["themeVariables"]["primaryColor"], "#234567");
        assert_eq!(config["themeVariables"]["primaryTextColor"], "#ffffff");
        assert_eq!(config["themeVariables"]["primaryBorderColor"], "#345678");
        assert_eq!(config["themeVariables"]["lineColor"], "#456789");
        assert_eq!(config["themeVariables"]["textColor"], "#eeeeee");
        assert_eq!(config["themeVariables"]["mainBkg"], "#123456");
        assert_eq!(config["themeVariables"]["nodeBkg"], "#223344");
        assert_eq!(config["themeVariables"]["nodeBorder"], "#334455");
        assert_eq!(config["themeVariables"]["clusterBkg"], "#445566");
        assert_eq!(config["themeVariables"]["clusterBorder"], "#556677");
        assert_eq!(config["themeVariables"]["edgeLabelBackground"], "#667788");
        assert!(config.get("style").is_none());
    }

    #[test]
    fn body_font_size_option_defaults_invalid_values_to_sixteen_pixels() {
        let default_config = MermaidOptions::default().to_mermaid_config_value();
        let invalid_config = MermaidOptions {
            body_font_size_px: Some(f32::NAN),
            ..MermaidOptions::default()
        }
        .to_mermaid_config_value();

        assert_eq!(default_config["fontSize"], 16.0);
        assert_eq!(default_config["themeVariables"]["fontSize"], "16px");
        assert_eq!(invalid_config["fontSize"], 16.0);
        assert_eq!(invalid_config["themeVariables"]["fontSize"], "16px");
    }

    #[test]
    fn default_diagram_style_keeps_existing_svg_strokes_and_radii() {
        let rendered = Mermaid::render("flowchart TD\nA[Start] -->|go| B[Done]\n")
            .expect("flowchart renders with default style");

        assert_eq!(MermaidOptions::default().style, DiagramStyle::default());
        assert!(rendered.svg.contains(r#"stroke-width="1.4""#));
        assert!(rendered.svg.contains(r#"stroke-width="1.6""#));
        assert!(rendered
            .svg
            .contains(r##"rx="8" fill="#ffffff" stroke="#aeb8c2" stroke-width="1.4""##));
        assert!(rendered
            .svg
            .contains(r##"rx="6" fill="#ffffff" opacity="0.94""##));
    }

    #[test]
    fn explicit_diagram_style_changes_supported_svg_strokes_and_radii() {
        let options = MermaidOptions {
            style: DiagramStyle {
                corner_radius: Some(DiagramCornerRadius::Xs),
                arrowhead: Some(DiagramArrowheadStyle::Chevron),
                stroke_width: Some(DiagramStrokeWidth::Bold),
            },
            ..MermaidOptions::default()
        };

        let flowchart = Mermaid::new()
            .render_with_options("flowchart TD\nA[Start] -->|go| B[Done]\n", &options)
            .expect("flowchart renders with explicit style");
        let sequence = Mermaid::new()
            .render_with_options(SEQUENCE, &options)
            .expect("sequence renders with explicit style");

        assert!(flowchart
            .svg
            .contains(r##"rx="2" fill="#ffffff" stroke="#aeb8c2" stroke-width="3""##));
        assert!(flowchart
            .svg
            .contains(r##"rx="2" fill="#ffffff" opacity="0.94""##));
        assert!(flowchart.svg.contains(r#"stroke-width="3""#));
        assert!(!flowchart.svg.contains(r#"stroke-width="1.4""#));
        assert!(!flowchart.svg.contains(r#"stroke-width="1.6""#));

        assert!(sequence.svg.contains(r#"class="oriha-mermaid-lifeline""#));
        assert!(sequence
            .svg
            .contains(r#"stroke-width="3" stroke-dasharray="5 5""#));
        assert!(sequence.svg.contains(r#"class="oriha-mermaid-message""#));
        assert!(!sequence.svg.contains(r#"stroke-width="1.2""#));
        assert!(!sequence.svg.contains(r#"stroke-width="1.6""#));
    }

    #[test]
    fn unsupported_diagram_reports_kind() {
        let error = Mermaid::render("erDiagram\nA ||--o{ B : owns\n")
            .expect_err("unsupported diagram should fail");
        assert_eq!(
            error,
            MermaidError::UnsupportedDiagram {
                kind: MermaidDiagramKind::EntityRelationship
            }
        );
    }

    #[test]
    fn detector_reports_mermaid_directive_evidence_with_line_number() {
        let detection =
            __private::detect_mermaid_diagram("%% ignored\n\nsequenceDiagram\nA->>B: hello\n");

        assert_eq!(detection.kind, MermaidDiagramKind::Sequence);
        assert_eq!(
            detection.reason,
            __private::MermaidDiagramDetectionReason::DiagramDirective
        );
        assert_eq!(detection.line, Some(3));
    }

    #[test]
    fn detector_distinguishes_unsupported_unknown_and_empty_mermaid_sources() {
        let unsupported = __private::detect_mermaid_diagram("erDiagram\nA ||--o{ B : owns\n");
        assert_eq!(unsupported.kind, MermaidDiagramKind::EntityRelationship);
        assert_eq!(
            unsupported.reason,
            __private::MermaidDiagramDetectionReason::UnsupportedDirective
        );
        assert_eq!(unsupported.line, Some(1));

        let unknown = __private::detect_mermaid_diagram("notMermaid\nA --> B\n");
        assert_eq!(unknown.kind, MermaidDiagramKind::Unknown);
        assert_eq!(
            unknown.reason,
            __private::MermaidDiagramDetectionReason::UnknownDirective
        );
        assert_eq!(unknown.line, Some(1));

        let empty = __private::detect_mermaid_diagram("%% only comments\n\n");
        assert_eq!(empty.kind, MermaidDiagramKind::Unknown);
        assert_eq!(
            empty.reason,
            __private::MermaidDiagramDetectionReason::NoEvidence
        );
        assert_eq!(empty.line, None);
    }

    #[test]
    fn empty_diagram_id_uses_deterministic_mermaid_prefixed_hash() {
        let first = Mermaid::render(FLOWCHART).expect("flowchart renders");
        let second = Mermaid::render(FLOWCHART).expect("flowchart renders again");

        assert_eq!(first.svg, second.svg);
        assert!(first.svg.contains(r#"id="mermaid-"#));

        let options = MermaidOptions::default();
        let rendered = Mermaid::new()
            .render_with_measurements(FLOWCHART, " 1 demo ", &options, &BTreeMap::new())
            .expect("flowchart renders with custom id");
        assert!(rendered.svg.contains(r#"id="mermaid-1-demo""#));
    }
}
