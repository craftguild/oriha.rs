//! PlantUML render API, options and backend composition.
mod style;
#[allow(unused_imports)]
use style::*;
mod detection;
#[allow(unused_imports)]
use detection::*;
pub use detection::{detect_plantuml_diagram, detect_plantuml_diagram_kind};
mod measurements;
pub(crate) use measurements::collect_sequence_text_measurements_with_measurer;
#[allow(unused_imports)]
use measurements::*;
pub use measurements::{
    collect_plantuml_text_measurement_requests,
    collect_plantuml_text_measurement_requests_with_options, collect_plantuml_text_measurements,
    collect_plantuml_text_measurements_with_options, collect_sequence_text_measurements,
};

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;

use oriha_diagram::{DiagramArrowheadStyle, DiagramCornerRadius, DiagramStrokeWidth, DiagramStyle};

use crate::backend::DiagramBackend;
use crate::backends::{ClassBackend, SequenceBackend};
use crate::class_diagram::is_class_statement;
use crate::layout::{layout_sequence_with_text_measurer, LayoutOptions};
use crate::parser::parse_sequence;
use crate::render::{
    collect_sequence_render_text_measurements, render_sequence_svg_with_style_and_text_measurer,
    PlantUmlStyle, PlantUmlTheme, RenderedPlantUml,
};
use crate::sequence::PlantUmlError as SequenceParseError;
use crate::text_layout::{
    ApproxTextMeasurer, InlineTextMeasurementRequest, InlineTextMeasurementResult,
    InlineTextMeasurementStyle, TextMeasurer,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlantUmlDiagramKind {
    Sequence,
    Activity,
    Class,
    Component,
    Deployment,
    Dot,
    Gantt,
    Json,
    MindMap,
    Object,
    Salt,
    State,
    Timing,
    UseCase,
    Wbs,
    Yaml,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct PlantUmlDiagramDetection {
    pub kind: PlantUmlDiagramKind,
    pub reason: PlantUmlDiagramDetectionReason,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlantUmlDiagramDetectionReason {
    StartDirective,
    SequenceStatement,
    ClassStatement,
    UnsupportedStatement,
    DefaultToSequence,
    NoEvidence,
}

impl fmt::Display for PlantUmlDiagramKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Sequence => "sequence",
            Self::Activity => "activity",
            Self::Class => "class",
            Self::Component => "component",
            Self::Deployment => "deployment",
            Self::Dot => "dot",
            Self::Gantt => "gantt",
            Self::Json => "json",
            Self::MindMap => "mindmap",
            Self::Object => "object",
            Self::Salt => "salt",
            Self::State => "state",
            Self::Timing => "timing",
            Self::UseCase => "usecase",
            Self::Wbs => "wbs",
            Self::Yaml => "yaml",
            Self::Unknown => "unknown",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlantUmlError {
    Parse {
        kind: PlantUmlDiagramKind,
        line: Option<usize>,
        message: String,
    },
    Render {
        kind: PlantUmlDiagramKind,
        message: String,
    },
    UnsupportedDiagram {
        kind: PlantUmlDiagramKind,
    },
}

impl fmt::Display for PlantUmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse {
                kind,
                line,
                message,
            } => match line {
                Some(line) => write!(f, "{kind} PlantUML parse error at line {line}: {message}"),
                None => write!(f, "{kind} PlantUML parse error: {message}"),
            },
            Self::Render { kind, message } => {
                write!(f, "{kind} PlantUML render error: {message}")
            }
            Self::UnsupportedDiagram { kind } => {
                write!(f, "unsupported PlantUML diagram kind: {kind}")
            }
        }
    }
}

impl std::error::Error for PlantUmlError {}

pub type PlantUmlRenderError = PlantUmlError;

#[derive(Debug, Clone, PartialEq)]
pub struct PlantUmlOptions {
    pub font_family: String,
    pub body_font_size_px: Option<f32>,
    pub style: DiagramStyle,
}

impl Default for PlantUmlOptions {
    fn default() -> Self {
        let theme = PlantUmlTheme::default();
        Self {
            font_family: theme.font_family,
            body_font_size_px: None,
            style: DiagramStyle::default(),
        }
    }
}

impl PlantUmlOptions {
    pub fn base_text_style(&self) -> InlineTextMeasurementStyle {
        InlineTextMeasurementStyle {
            font_family: Some(self.font_family.clone()),
            font_size: Some(f32::from(self.resolved_font_size())),
            font_weight: None,
            font_style: None,
        }
    }

    pub(crate) fn layout_options(&self) -> LayoutOptions {
        LayoutOptions::default()
    }

    pub(crate) fn theme(&self) -> PlantUmlTheme {
        PlantUmlTheme {
            font_family: self.font_family.clone(),
            font_size: self.resolved_font_size(),
            ..PlantUmlTheme::default()
        }
    }

    pub(crate) fn resolved_font_size(&self) -> u16 {
        resolve_body_font_size_px(self.body_font_size_px)
            .round()
            .max(1.0) as u16
    }

    pub(crate) fn sequence_style(&self) -> PlantUmlStyle {
        sequence_style_from_diagram_style(self.style)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantUmlSvg {
    pub svg: String,
    pub width: u32,
    pub height: u32,
    pub view_box: String,
    pub kind: PlantUmlDiagramKind,
}

impl PlantUmlSvg {
    pub(crate) fn from_rendered(kind: PlantUmlDiagramKind, rendered: RenderedPlantUml) -> Self {
        Self {
            svg: rendered.svg,
            width: rendered.width,
            height: rendered.height,
            view_box: rendered.view_box,
            kind,
        }
    }
}

pub struct PlantUml {
    backends: &'static [&'static dyn DiagramBackend],
}

impl Default for PlantUml {
    fn default() -> Self {
        Self::new()
    }
}

impl PlantUml {
    pub fn new() -> Self {
        Self {
            backends: &[&SequenceBackend, &ClassBackend],
        }
    }

    pub fn render(input: &str) -> Result<PlantUmlSvg, PlantUmlError> {
        Self::new().render_with_options(input, &PlantUmlOptions::default())
    }

    pub fn render_with_options(
        &self,
        input: &str,
        options: &PlantUmlOptions,
    ) -> Result<PlantUmlSvg, PlantUmlError> {
        let measurer = RecordingApproxTextMeasurer::new(options.base_text_style());
        self.render_with_text_measurer(input, options, &measurer)
    }

    pub fn render_with_text_measurer(
        &self,
        input: &str,
        options: &PlantUmlOptions,
        measurer: &dyn TextMeasurer,
    ) -> Result<PlantUmlSvg, PlantUmlError> {
        let detection = detect_plantuml_diagram(input);
        let backend = self.backend(detection.kind.clone())?;
        backend.render(input, options, measurer)
    }

    pub fn collect_text_measurement_requests(
        &self,
        input: &str,
        options: &PlantUmlOptions,
    ) -> Result<Vec<InlineTextMeasurementRequest>, PlantUmlError> {
        let detection = detect_plantuml_diagram(input);
        let backend = self.backend(detection.kind.clone())?;
        backend.collect_text_measurement_requests(input, options, &options.base_text_style())
    }

    fn backend(
        &self,
        kind: PlantUmlDiagramKind,
    ) -> Result<&'static dyn DiagramBackend, PlantUmlError> {
        self.backends
            .iter()
            .copied()
            .find(|backend| backend.kind() == kind)
            .ok_or(PlantUmlError::UnsupportedDiagram { kind })
    }
}

pub fn render_sequence_source_svg(
    source: &str,
    options: &LayoutOptions,
    theme: &PlantUmlTheme,
) -> Result<RenderedPlantUml, SequenceParseError> {
    render_sequence_source_svg_with_style(source, options, theme, &PlantUmlStyle::default())
}

pub fn render_sequence_source_svg_with_style(
    source: &str,
    options: &LayoutOptions,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
) -> Result<RenderedPlantUml, SequenceParseError> {
    render_sequence_source_svg_with_style_and_text_measurer(
        source,
        options,
        theme,
        style,
        &ApproxTextMeasurer,
    )
}

pub fn render_sequence_source_svg_with_style_and_text_measurer(
    source: &str,
    options: &LayoutOptions,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) -> Result<RenderedPlantUml, SequenceParseError> {
    let diagram = parse_sequence(source)?;
    let layout = layout_sequence_with_text_measurer(&diagram, options, measurer);
    Ok(render_sequence_svg_with_style_and_text_measurer(
        &layout, theme, style, measurer,
    ))
}

pub fn render_plantuml_source_svg(
    source: &str,
    options: &LayoutOptions,
    theme: &PlantUmlTheme,
) -> Result<RenderedPlantUml, PlantUmlRenderError> {
    render_plantuml_source_svg_with_style(source, options, theme, &PlantUmlStyle::default())
}

pub fn render_plantuml_source_svg_with_style(
    source: &str,
    options: &LayoutOptions,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
) -> Result<RenderedPlantUml, PlantUmlRenderError> {
    render_plantuml_source_svg_with_style_and_text_measurer(
        source,
        options,
        theme,
        style,
        &ApproxTextMeasurer,
    )
}

pub fn render_plantuml_source_svg_with_style_and_text_measurer(
    source: &str,
    _options: &LayoutOptions,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) -> Result<RenderedPlantUml, PlantUmlRenderError> {
    let api_options = PlantUmlOptions {
        font_family: theme.font_family.clone(),
        body_font_size_px: Some(f32::from(theme.font_size)),
        style: diagram_style_from_sequence_style(style),
    };
    PlantUml::new()
        .render_with_text_measurer(source, &api_options, measurer)
        .map(|svg| RenderedPlantUml {
            svg: svg.svg,
            width: svg.width,
            height: svg.height,
            view_box: svg.view_box,
        })
}

enum StartDirectiveDiagramKind {
    GenericUml,
    Specific(PlantUmlDiagramKind),
}

struct DiagramKindCandidateLine {
    number: usize,
    text: String,
}

pub(crate) struct RecordingTextMeasurer<'a> {
    delegate: &'a dyn TextMeasurer,
    seen: RefCell<HashSet<String>>,
    lines: RefCell<Vec<String>>,
    requests: RefCell<Vec<InlineTextMeasurementRequest>>,
}

pub(crate) struct RecordingApproxTextMeasurer {
    base_style: InlineTextMeasurementStyle,
    seen: RefCell<HashSet<String>>,
    requests: RefCell<Vec<InlineTextMeasurementRequest>>,
}

pub(crate) fn sequence_error_to_plantuml(
    kind: PlantUmlDiagramKind,
    error: SequenceParseError,
) -> PlantUmlError {
    let line = match &error {
        SequenceParseError::MissingStart | SequenceParseError::MissingEnd => None,
        SequenceParseError::ContentAfterEnd { line }
        | SequenceParseError::InvalidMessage { line, .. }
        | SequenceParseError::InvalidStatement { line, .. }
        | SequenceParseError::UnsupportedStatement { line, .. }
        | SequenceParseError::UnexpectedBlockEnd { line, .. }
        | SequenceParseError::UnclosedBlock { line, .. } => Some(*line),
    };

    PlantUmlError::Parse {
        kind,
        line,
        message: error.to_string(),
    }
}
