//! Flowchart backend, shared model and rendering style primitives.
mod parsing;
#[allow(unused_imports)]
use parsing::*;
mod layout;
#[allow(unused_imports)]
use layout::*;
mod measurements;
#[allow(unused_imports)]
use measurements::*;
mod render;
#[allow(unused_imports)]
use render::*;

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use oriha_diagram::{DiagramCornerRadius, DiagramStrokeWidth, DiagramStyle};

use crate::backend::DiagramBackend;
use crate::pipeline::{
    deterministic_mermaid_diagram_id, sanitize_mermaid_diagram_id, MermaidDiagramKind,
    MermaidError, MermaidOptions, MermaidSvg,
};
use crate::text_layout::{self, MeasuredText, TextResolver, NODE_MAX_WIDTH};
use crate::{
    MermaidMeasurementKind, MermaidMeasurementRequest, MermaidMeasurementResult, MermaidTheme,
};

const MARGIN: f32 = 34.0;
const LAYER_GAP: f32 = 72.0;
const NODE_GAP: f32 = 34.0;
const NODE_PAD_X: f32 = 30.0;
const NODE_PAD_Y: f32 = 22.0;

pub(crate) struct FlowchartBackend;

impl DiagramBackend for FlowchartBackend {
    fn kind(&self) -> MermaidDiagramKind {
        MermaidDiagramKind::Flowchart
    }

    fn render(
        &self,
        source: &str,
        diagram_id: &str,
        options: &MermaidOptions,
        measurements: &BTreeMap<String, MermaidMeasurementResult>,
    ) -> Result<MermaidSvg, MermaidError> {
        let diagram = parse_flowchart_diagram(source).map_err(parse_error_to_mermaid)?;
        let theme = options.theme();
        let resolver = TextResolver::new(&theme, measurements);
        let layout = layout_flowchart(&diagram, &resolver);
        let resolved_id = resolved_diagram_id(source, diagram_id);
        let svg = render_flowchart_svg(&resolved_id, &layout, &theme, &options.style);
        Ok(MermaidSvg {
            svg,
            width: layout_dimension(layout.width),
            height: layout_dimension(layout.height),
            view_box: format!(
                "0 0 {} {}",
                format_svg_number(layout.width),
                format_svg_number(layout.height)
            ),
            kind: MermaidDiagramKind::Flowchart,
        })
    }

    fn collect_text_measurement_requests(
        &self,
        source: &str,
        options: &MermaidOptions,
    ) -> Result<Vec<MermaidMeasurementRequest>, MermaidError> {
        let diagram = parse_flowchart_diagram(source).map_err(parse_error_to_mermaid)?;
        Ok(collect_flowchart_measurement_requests(
            &diagram,
            &options.theme(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct FlowchartDiagram {
    direction: FlowDirection,
    nodes: Vec<FlowNode>,
    edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowDirection {
    TopDown,
    LeftRight,
}

#[derive(Debug, Clone, PartialEq)]
struct FlowNode {
    id: String,
    label: String,
    shape: FlowNodeShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowNodeShape {
    Rectangle,
    Round,
    Stadium,
    Diamond,
}

#[derive(Debug, Clone, PartialEq)]
struct FlowEdge {
    from: String,
    to: String,
    label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FlowchartParseError {
    line: Option<usize>,
    message: String,
}

impl FlowchartParseError {
    fn empty() -> Self {
        Self {
            line: None,
            message: "Mermaid source did not contain a diagram".to_owned(),
        }
    }

    fn unsupported_diagram(diagram_type: &str, line: usize) -> Self {
        Self {
            line: Some(line),
            message: format!(
                "Unsupported Mermaid diagram `{diagram_type}`. Supported subset: flowchart TD/LR, sequenceDiagram, stateDiagram-v2"
            ),
        }
    }

    fn unsupported_syntax(line: usize, syntax: &str) -> Self {
        Self {
            line: Some(line),
            message: format!("Unsupported Mermaid syntax in subset renderer: `{syntax}`"),
        }
    }
}

impl fmt::Display for FlowchartParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line}: {}", self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for FlowchartParseError {}

fn parse_error_to_mermaid(error: FlowchartParseError) -> MermaidError {
    MermaidError::Parse {
        kind: MermaidDiagramKind::Flowchart,
        line: error.line,
        message: error.message,
    }
}

#[derive(Debug)]
struct ParsedFlowEdge {
    from: String,
    to: String,
    label: Option<String>,
}

#[derive(Debug)]
struct ParsedFlowNode {
    id: String,
    label: String,
    shape: FlowNodeShape,
    explicit: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct FlowchartLayout {
    width: f32,
    height: f32,
    direction: FlowDirection,
    nodes: Vec<FlowNodeLayout>,
    edges: Vec<FlowEdgeLayout>,
}

#[derive(Debug, Clone, PartialEq)]
struct FlowNodeLayout {
    id: String,
    shape: FlowNodeShape,
    rect: Rect,
    text: MeasuredText,
}

#[derive(Debug, Clone, PartialEq)]
struct FlowEdgeLayout {
    label: Option<MeasuredText>,
    points: Vec<Point>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Rect {
    fn center_x(self) -> f32 {
        self.x + self.width / 2.0
    }

    fn center_y(self) -> f32 {
        self.y + self.height / 2.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Debug, Default)]
struct MeasurementCollector {
    seen: BTreeSet<String>,
    requests: Vec<MermaidMeasurementRequest>,
}

fn color<'a>(value: Option<&'a str>, fallback: &'a str) -> &'a str {
    value
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
}

fn corner_radius(style: &DiagramStyle, default: f32) -> f32 {
    match style.corner_radius {
        None => default,
        Some(DiagramCornerRadius::None) => 0.0,
        Some(DiagramCornerRadius::Xs) => 2.0,
        Some(DiagramCornerRadius::Sm) => 4.0,
        Some(DiagramCornerRadius::Md) => 6.0,
        Some(DiagramCornerRadius::Lg) => 8.0,
        Some(DiagramCornerRadius::Xl) => 12.0,
    }
}

fn stroke_width<'a>(style: &DiagramStyle, default: &'a str) -> &'a str {
    match style.stroke_width {
        None => default,
        Some(DiagramStrokeWidth::Thin) => "1",
        Some(DiagramStrokeWidth::Regular) => "2",
        Some(DiagramStrokeWidth::Bold) => "3",
    }
}

fn resolved_diagram_id(source: &str, diagram_id: &str) -> String {
    if diagram_id.trim().is_empty() {
        deterministic_mermaid_diagram_id(source)
    } else {
        sanitize_mermaid_diagram_id(diagram_id)
    }
}

fn layout_dimension(value: f32) -> u32 {
    if value.is_finite() && value > 0.0 {
        value.ceil() as u32
    } else {
        0
    }
}

fn format_svg_number(value: f32) -> String {
    format_number(value)
}

fn format_number(value: f32) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded.fract() == 0.0 {
        format!("{}", rounded as i32)
    } else {
        format!("{rounded:.2}")
    }
}

fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(value: &str) -> String {
    escape_text(value).replace('"', "&quot;")
}
