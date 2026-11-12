//! Public Mermaid rendering pipeline, options and stable diagram identifiers.
mod theme;
#[allow(unused_imports)]
use theme::*;
mod detection;
#[allow(unused_imports)]
pub(crate) use detection::parse_error_to_mermaid;
#[allow(unused_imports)]
use detection::*;
pub use detection::{detect_mermaid_diagram, detect_mermaid_diagram_kind};

use std::collections::BTreeMap;
use std::fmt;

use oriha_diagram::DiagramStyle;
use serde_json::{Map, Number, Value};

use crate::backend::DiagramBackend;
use crate::backends::{FlowchartBackend, SequenceBackend, StateBackend};
use crate::parser::MermaidParseError;
use crate::{
    finite_positive_f32, fnv1a_64, format_css_px, MermaidMeasurementRequest,
    MermaidMeasurementResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MermaidDiagramKind {
    Flowchart,
    Sequence,
    State,
    Class,
    EntityRelationship,
    Gantt,
    GitGraph,
    UserJourney,
    MindMap,
    Pie,
    Quadrant,
    Requirement,
    Timeline,
    C4,
    XyChart,
    ZenUml,
    Block,
    Sankey,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MermaidDiagramDetection {
    pub kind: MermaidDiagramKind,
    pub reason: MermaidDiagramDetectionReason,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MermaidDiagramDetectionReason {
    DiagramDirective,
    UnsupportedDirective,
    UnknownDirective,
    NoEvidence,
}

impl fmt::Display for MermaidDiagramKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Flowchart => "flowchart",
            Self::Sequence => "sequence",
            Self::State => "state",
            Self::Class => "class",
            Self::EntityRelationship => "entity-relationship",
            Self::Gantt => "gantt",
            Self::GitGraph => "gitgraph",
            Self::UserJourney => "journey",
            Self::MindMap => "mindmap",
            Self::Pie => "pie",
            Self::Quadrant => "quadrant",
            Self::Requirement => "requirement",
            Self::Timeline => "timeline",
            Self::C4 => "c4",
            Self::XyChart => "xychart",
            Self::ZenUml => "zenuml",
            Self::Block => "block",
            Self::Sankey => "sankey",
            Self::Unknown => "unknown",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MermaidError {
    Parse {
        kind: MermaidDiagramKind,
        line: Option<usize>,
        message: String,
    },
    Render {
        kind: MermaidDiagramKind,
        message: String,
    },
    UnsupportedDiagram {
        kind: MermaidDiagramKind,
    },
}

impl fmt::Display for MermaidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse {
                kind,
                line,
                message,
            } => {
                if *kind == MermaidDiagramKind::Unknown
                    && line.is_none()
                    && message == EMPTY_DIAGRAM_MESSAGE
                {
                    return f.write_str(message);
                }
                match line {
                    Some(line) => write!(f, "{kind} Mermaid parse error at line {line}: {message}"),
                    None => write!(f, "{kind} Mermaid parse error: {message}"),
                }
            }
            Self::Render { kind, message } => write!(f, "{kind} Mermaid render error: {message}"),
            Self::UnsupportedDiagram { kind } => {
                write!(f, "unsupported Mermaid diagram kind: {kind}")
            }
        }
    }
}

impl std::error::Error for MermaidError {}

#[derive(Debug, Clone, PartialEq)]
pub struct MermaidOptions {
    pub dark_mode: bool,
    pub font_family: String,
    pub body_font_size_px: Option<f32>,
    pub background: Option<String>,
    pub primary_color: Option<String>,
    pub primary_text_color: Option<String>,
    pub primary_border_color: Option<String>,
    pub line_color: Option<String>,
    pub text_color: Option<String>,
    pub main_bkg: Option<String>,
    pub node_bkg: Option<String>,
    pub node_border: Option<String>,
    pub cluster_bkg: Option<String>,
    pub cluster_border: Option<String>,
    pub edge_label_background: Option<String>,
    pub style: DiagramStyle,
}

impl Default for MermaidOptions {
    fn default() -> Self {
        let theme = MermaidTheme::default();
        Self {
            dark_mode: theme.dark_mode,
            font_family: theme.font_family,
            body_font_size_px: None,
            background: theme.background,
            primary_color: theme.primary_color,
            primary_text_color: theme.primary_text_color,
            primary_border_color: theme.primary_border_color,
            line_color: theme.line_color,
            text_color: theme.text_color,
            main_bkg: theme.main_bkg,
            node_bkg: theme.node_bkg,
            node_border: theme.node_border,
            cluster_bkg: theme.cluster_bkg,
            cluster_border: theme.cluster_border,
            edge_label_background: theme.edge_label_background,
            style: DiagramStyle::default(),
        }
    }
}

impl MermaidOptions {
    pub fn to_mermaid_config_value(&self) -> Value {
        self.theme().to_mermaid_config_value()
    }

    pub(crate) fn theme(&self) -> MermaidTheme {
        MermaidTheme {
            dark_mode: self.dark_mode,
            font_family: self.font_family.clone(),
            font_size: resolve_body_font_size_px(self.body_font_size_px),
            background: self.background.clone(),
            primary_color: self.primary_color.clone(),
            primary_text_color: self.primary_text_color.clone(),
            primary_border_color: self.primary_border_color.clone(),
            line_color: self.line_color.clone(),
            text_color: self.text_color.clone(),
            main_bkg: self.main_bkg.clone(),
            node_bkg: self.node_bkg.clone(),
            node_border: self.node_border.clone(),
            cluster_bkg: self.cluster_bkg.clone(),
            cluster_border: self.cluster_border.clone(),
            edge_label_background: self.edge_label_background.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MermaidTheme {
    pub dark_mode: bool,
    pub font_family: String,
    pub font_size: f32,
    pub background: Option<String>,
    pub primary_color: Option<String>,
    pub primary_text_color: Option<String>,
    pub primary_border_color: Option<String>,
    pub line_color: Option<String>,
    pub text_color: Option<String>,
    pub main_bkg: Option<String>,
    pub node_bkg: Option<String>,
    pub node_border: Option<String>,
    pub cluster_bkg: Option<String>,
    pub cluster_border: Option<String>,
    pub edge_label_background: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MermaidSvg {
    pub svg: String,
    pub width: u32,
    pub height: u32,
    pub view_box: String,
    pub kind: MermaidDiagramKind,
}

pub struct Mermaid {
    backends: &'static [&'static dyn DiagramBackend],
}

impl Default for Mermaid {
    fn default() -> Self {
        Self::new()
    }
}

impl Mermaid {
    pub fn new() -> Self {
        Self {
            backends: &[&FlowchartBackend, &SequenceBackend, &StateBackend],
        }
    }

    pub fn render(input: &str) -> Result<MermaidSvg, MermaidError> {
        Self::new().render_with_options(input, &MermaidOptions::default())
    }

    pub fn render_with_options(
        &self,
        input: &str,
        options: &MermaidOptions,
    ) -> Result<MermaidSvg, MermaidError> {
        self.render_with_measurements_and_id(input, "", options, &BTreeMap::new())
    }

    pub fn render_with_measurements(
        &self,
        input: &str,
        diagram_id: &str,
        options: &MermaidOptions,
        measurements: &BTreeMap<String, MermaidMeasurementResult>,
    ) -> Result<MermaidSvg, MermaidError> {
        self.render_with_measurements_and_id(input, diagram_id, options, measurements)
    }

    fn render_with_measurements_and_id(
        &self,
        input: &str,
        diagram_id: &str,
        options: &MermaidOptions,
        measurements: &BTreeMap<String, MermaidMeasurementResult>,
    ) -> Result<MermaidSvg, MermaidError> {
        if first_significant_line(input).is_none() {
            return Err(empty_diagram_error());
        }

        let detection = detect_mermaid_diagram(input);
        let backend = self.backend(detection.kind)?;
        backend.render(input, diagram_id, options, measurements)
    }

    pub fn collect_text_measurement_requests(
        &self,
        input: &str,
        options: &MermaidOptions,
    ) -> Result<Vec<MermaidMeasurementRequest>, MermaidError> {
        if first_significant_line(input).is_none() {
            return Err(empty_diagram_error());
        }

        let detection = detect_mermaid_diagram(input);
        let backend = self.backend(detection.kind)?;
        backend.collect_text_measurement_requests(input, options)
    }

    fn backend(
        &self,
        kind: MermaidDiagramKind,
    ) -> Result<&'static dyn DiagramBackend, MermaidError> {
        self.backends
            .iter()
            .copied()
            .find(|backend| backend.kind() == kind)
            .ok_or(MermaidError::UnsupportedDiagram { kind })
    }
}

pub fn deterministic_mermaid_diagram_id(source: &str) -> String {
    sanitize_mermaid_diagram_id(&format!("mermaid-{:016x}", fnv1a_64(source.as_bytes())))
}

pub fn sanitize_mermaid_diagram_id(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return "mermaid-untitled".to_owned();
    }

    let mut out = String::with_capacity(raw.len() + "mermaid-".len());
    let mut prev_dash = false;
    for ch in raw.chars() {
        let sanitized = if ch.is_ascii_alphanumeric() || matches!(ch, '_' | ':' | '.') {
            ch
        } else {
            '-'
        };
        if sanitized == '-' {
            if prev_dash {
                continue;
            }
            prev_dash = true;
        } else {
            prev_dash = false;
        }
        out.push(sanitized);
    }

    while out.ends_with('-') {
        out.pop();
    }
    while out.starts_with('-') {
        out.remove(0);
    }

    if out.is_empty() {
        return "mermaid-untitled".to_owned();
    }
    if out == "mermaid" {
        return "mermaid-untitled".to_owned();
    }
    if out.starts_with("mermaid-") {
        out
    } else {
        format!("mermaid-{out}")
    }
}

fn insert_optional_string(map: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        map.insert(key.to_owned(), Value::String(value.to_owned()));
    }
}

fn json_number_f32(value: f32) -> Value {
    Number::from_f64(f64::from(value))
        .map(Value::Number)
        .unwrap_or_else(|| Value::Number(Number::from(16)))
}

const EMPTY_DIAGRAM_MESSAGE: &str = "Mermaid source did not contain a diagram";
