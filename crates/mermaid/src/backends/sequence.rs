//! Sequence backend, diagram model, layout and measurement orchestration.
mod parsing;
#[allow(unused_imports)]
use parsing::*;
mod render;
#[allow(unused_imports)]
use render::*;

use std::collections::{BTreeMap, BTreeSet};

use oriha_diagram::{DiagramCornerRadius, DiagramStrokeWidth, DiagramStyle};

use crate::backend::DiagramBackend;
use crate::parser::MermaidParseError;
use crate::pipeline::{
    deterministic_mermaid_diagram_id, parse_error_to_mermaid, sanitize_mermaid_diagram_id,
    MermaidDiagramKind, MermaidError, MermaidOptions, MermaidSvg,
};
use crate::text_layout::{self, MeasuredText, TextResolver};
use crate::{
    MermaidMeasurementKind, MermaidMeasurementRequest, MermaidMeasurementResult, MermaidTheme,
};

const MARGIN: f32 = 34.0;

pub(crate) struct SequenceBackend;

impl DiagramBackend for SequenceBackend {
    fn kind(&self) -> MermaidDiagramKind {
        MermaidDiagramKind::Sequence
    }

    fn render(
        &self,
        source: &str,
        diagram_id: &str,
        options: &MermaidOptions,
        measurements: &BTreeMap<String, MermaidMeasurementResult>,
    ) -> Result<MermaidSvg, MermaidError> {
        let diagram = parse_sequence_diagram(source)
            .map_err(|error| parse_error_to_mermaid(self.kind(), error))?;
        let theme = options.theme();
        let resolver = TextResolver::new(&theme, measurements);
        let layout = layout_sequence(&diagram, &resolver);
        Ok(render_sequence_svg(
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
        let diagram = parse_sequence_diagram(source)
            .map_err(|error| parse_error_to_mermaid(self.kind(), error))?;
        Ok(collect_sequence_measurement_requests(
            &diagram,
            &options.theme(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SequenceDiagram {
    autonumber: bool,
    participants: Vec<SequenceParticipant>,
    messages: Vec<SequenceMessage>,
}

#[derive(Debug, Clone, PartialEq)]
struct SequenceParticipant {
    id: String,
    label: String,
}

#[derive(Debug, Clone, PartialEq)]
struct SequenceMessage {
    from: String,
    to: String,
    label: String,
    dashed: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct SequenceLayout {
    width: f32,
    height: f32,
    participants: Vec<SequenceParticipantLayout>,
    messages: Vec<SequenceMessageLayout>,
    lifeline_top: f32,
    lifeline_bottom: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct SequenceParticipantLayout {
    id: String,
    rect: Rect,
    center_x: f32,
    text: MeasuredText,
}

#[derive(Debug, Clone, PartialEq)]
struct SequenceMessageLayout {
    from_x: f32,
    to_x: f32,
    y: f32,
    text: MeasuredText,
    number: Option<usize>,
    dashed: bool,
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

fn collect_sequence_measurement_requests(
    sequence: &SequenceDiagram,
    theme: &MermaidTheme,
) -> Vec<MermaidMeasurementRequest> {
    let mut collector = MeasurementCollector::default();
    for participant in &sequence.participants {
        collector.push(text_layout::measurement_request(
            theme,
            &participant.label,
            MermaidMeasurementKind::Text,
            None,
            Some("600"),
        ));
    }
    for message in &sequence.messages {
        collector.push(text_layout::measurement_request(
            theme,
            &message.label,
            MermaidMeasurementKind::Text,
            None,
            None,
        ));
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

fn layout_sequence(sequence: &SequenceDiagram, resolver: &TextResolver<'_>) -> SequenceLayout {
    let mut participants = Vec::new();
    let mut x = MARGIN;
    for participant in &sequence.participants {
        let text = resolver.measure_text(&participant.label, Some("600"));
        let width = (text.width + 34.0).max(120.0);
        let rect = Rect {
            x,
            y: MARGIN,
            width,
            height: 48.0,
        };
        participants.push(SequenceParticipantLayout {
            id: participant.id.clone(),
            center_x: rect.center_x(),
            rect,
            text,
        });
        x += width + 58.0;
    }
    let participant_by_id = participants
        .iter()
        .map(|participant| (participant.id.clone(), participant.center_x))
        .collect::<BTreeMap<_, _>>();
    let lifeline_top = MARGIN + 48.0;
    let mut messages = Vec::new();
    let mut y = lifeline_top + 48.0;
    for (index, message) in sequence.messages.iter().enumerate() {
        let from_x = participant_by_id[&message.from];
        let to_x = participant_by_id[&message.to];
        messages.push(SequenceMessageLayout {
            from_x,
            to_x,
            y,
            text: resolver.measure_text(&message.label, None),
            number: sequence.autonumber.then_some(index + 1),
            dashed: message.dashed,
        });
        y += 58.0;
    }

    let width = participants
        .last()
        .map(|participant| participant.rect.x + participant.rect.width + MARGIN)
        .unwrap_or(MARGIN * 2.0)
        .max(260.0);
    let height = (y + MARGIN).max(180.0);
    SequenceLayout {
        width,
        height,
        participants,
        messages,
        lifeline_top,
        lifeline_bottom: height - MARGIN,
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
    let rounded = (value * 100.0).round() / 100.0;
    if rounded.fract() == 0.0 {
        format!("{}", rounded as i32)
    } else {
        format!("{rounded:.2}")
    }
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
