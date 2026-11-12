//! State-diagram backend, parsing model and measurement orchestration.
mod layout;
#[allow(unused_imports)]
use layout::*;
mod render;
#[allow(unused_imports)]
use render::*;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use oriha_diagram::{DiagramCornerRadius, DiagramStrokeWidth, DiagramStyle};

use crate::backend::DiagramBackend;
use crate::parser::{clean_identifier, non_empty, significant_lines, MermaidParseError};
use crate::pipeline::{
    deterministic_mermaid_diagram_id, parse_error_to_mermaid, sanitize_mermaid_diagram_id,
    MermaidDiagramKind, MermaidError, MermaidOptions, MermaidSvg,
};
use crate::text_layout::{self, MeasuredText, TextResolver, STATE_MAX_WIDTH};
use crate::{
    MermaidMeasurementKind, MermaidMeasurementRequest, MermaidMeasurementResult, MermaidTheme,
};

const MARGIN: f32 = 34.0;
const LAYER_GAP: f32 = 72.0;
const NODE_GAP: f32 = 34.0;
const NODE_PAD_X: f32 = 30.0;
const NODE_PAD_Y: f32 = 22.0;

pub(crate) struct StateBackend;

impl DiagramBackend for StateBackend {
    fn kind(&self) -> MermaidDiagramKind {
        MermaidDiagramKind::State
    }

    fn render(
        &self,
        source: &str,
        diagram_id: &str,
        options: &MermaidOptions,
        measurements: &BTreeMap<String, MermaidMeasurementResult>,
    ) -> Result<MermaidSvg, MermaidError> {
        let diagram = parse_state_diagram(source)
            .map_err(|error| parse_error_to_mermaid(self.kind(), error))?;
        let theme = options.theme();
        let resolver = TextResolver::new(&theme, measurements);
        let layout = layout_state(&diagram, &resolver);
        Ok(render_state_svg(
            source,
            diagram_id,
            &theme,
            &options.style,
            &layout,
        ))
    }

    fn collect_text_measurement_requests(
        &self,
        source: &str,
        options: &MermaidOptions,
    ) -> Result<Vec<MermaidMeasurementRequest>, MermaidError> {
        let diagram = parse_state_diagram(source)
            .map_err(|error| parse_error_to_mermaid(self.kind(), error))?;
        Ok(collect_state_measurement_requests(
            &diagram,
            &options.theme(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct StateDiagram {
    states: Vec<StateNode>,
    transitions: Vec<StateTransition>,
}

#[derive(Debug, Clone, PartialEq)]
struct StateNode {
    id: String,
    label: String,
    kind: StateNodeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateNodeKind {
    Initial,
    Final,
    State,
}

#[derive(Debug, Clone, PartialEq)]
struct StateTransition {
    from: String,
    to: String,
    label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct StateLayout {
    width: f32,
    height: f32,
    states: Vec<StateNodeLayout>,
    transitions: Vec<StateTransitionLayout>,
}

#[derive(Debug, Clone, PartialEq)]
struct StateNodeLayout {
    id: String,
    kind: StateNodeKind,
    rect: Rect,
    text: Option<MeasuredText>,
}

#[derive(Debug, Clone, PartialEq)]
struct StateTransitionLayout {
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

fn parse_state_diagram(source: &str) -> Result<StateDiagram, MermaidParseError> {
    let lines = significant_lines(source);
    if lines.is_empty() {
        return Err(MermaidParseError::empty());
    }
    parse_state(&lines)
}

fn parse_state(lines: &[(usize, String)]) -> Result<StateDiagram, MermaidParseError> {
    let mut states = Vec::new();
    let mut state_index = BTreeMap::new();
    let mut transitions = Vec::new();

    for (line_no, line) in lines.iter().skip(1) {
        if let Some(transition) = parse_state_transition(line, &mut states, &mut state_index) {
            transitions.push(transition);
            continue;
        }
        return Err(MermaidParseError::unsupported_syntax(*line_no, line));
    }

    if states.is_empty() {
        return Err(MermaidParseError::empty());
    }

    Ok(StateDiagram {
        states,
        transitions,
    })
}

fn parse_state_transition(
    line: &str,
    states: &mut Vec<StateNode>,
    index: &mut BTreeMap<String, usize>,
) -> Option<StateTransition> {
    let arrow_index = line.find("-->")?;
    let from_token = line[..arrow_index].trim();
    let rest = line[arrow_index + 3..].trim();
    let (to_token, label) = if let Some((to, label)) = rest.split_once(':') {
        (to.trim(), non_empty(label.trim()))
    } else {
        (rest, None)
    };
    let from = upsert_state_endpoint(states, index, from_token, true)?;
    let to = upsert_state_endpoint(states, index, to_token, false)?;
    Some(StateTransition { from, to, label })
}

fn upsert_state_endpoint(
    states: &mut Vec<StateNode>,
    index: &mut BTreeMap<String, usize>,
    token: &str,
    is_from: bool,
) -> Option<String> {
    let (id, label, kind) = if token == "[*]" {
        if is_from {
            (
                "__initial".to_owned(),
                String::new(),
                StateNodeKind::Initial,
            )
        } else {
            ("__final".to_owned(), String::new(), StateNodeKind::Final)
        }
    } else {
        let id = clean_identifier(token)?;
        (id.clone(), id, StateNodeKind::State)
    };

    if !index.contains_key(&id) {
        index.insert(id.clone(), states.len());
        states.push(StateNode {
            id: id.clone(),
            label,
            kind,
        });
    }
    Some(id)
}

fn collect_state_measurement_requests(
    state: &StateDiagram,
    theme: &MermaidTheme,
) -> Vec<MermaidMeasurementRequest> {
    let mut collector = MeasurementCollector::default();
    for node in &state.states {
        if node.kind == StateNodeKind::State {
            collector.push(text_layout::measurement_request(
                theme,
                &node.label,
                MermaidMeasurementKind::Wrapped,
                Some(STATE_MAX_WIDTH),
                Some("600"),
            ));
        }
    }
    for transition in &state.transitions {
        if let Some(label) = &transition.label {
            collector.push(text_layout::measurement_request(
                theme,
                label,
                MermaidMeasurementKind::Text,
                None,
                None,
            ));
        }
    }
    collector.into_requests()
}

#[derive(Debug, Default)]
struct MeasurementCollector {
    seen: BTreeSet<String>,
    requests: Vec<MermaidMeasurementRequest>,
}

impl MeasurementCollector {
    fn push(&mut self, request: MermaidMeasurementRequest) {
        if self.seen.insert(request.id.clone()) {
            self.requests.push(request);
        }
    }

    fn into_requests(self) -> Vec<MermaidMeasurementRequest> {
        self.requests
    }
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
