mod backend;
mod backends;
mod class_diagram;
mod layout;
mod parser;
mod pipeline;
mod render;
mod sequence;
mod text_layout;

pub use oriha_diagram::{
    DiagramArrowheadStyle, DiagramCornerRadius, DiagramStrokeWidth, DiagramStyle,
};
pub use pipeline::{PlantUml, PlantUmlDiagramKind, PlantUmlError, PlantUmlOptions, PlantUmlSvg};
pub use text_layout::{
    approximate_inline_text_measurement, inline_text_measurement_request, text_measurement_id,
    visible_inline_text, ApproxTextMeasurer, CollectingTextMeasurer, InlineTextMeasurementRequest,
    InlineTextMeasurementResult, InlineTextMeasurementRun, InlineTextMeasurementStyle,
    InlineTextRunMeasurement, TextMeasurer,
};

#[doc(hidden)]
pub mod __private {
    pub use crate::layout::{
        layout_sequence, layout_sequence_with_text_measurer, LayoutOptions, MessageLayout,
        ParticipantLayout, SequenceLayout,
    };
    pub use crate::parser::parse_sequence;
    pub use crate::pipeline::{
        collect_plantuml_text_measurement_requests,
        collect_plantuml_text_measurement_requests_with_options,
        collect_plantuml_text_measurements, collect_plantuml_text_measurements_with_options,
        collect_sequence_text_measurements, detect_plantuml_diagram, detect_plantuml_diagram_kind,
        render_plantuml_source_svg, render_plantuml_source_svg_with_style,
        render_plantuml_source_svg_with_style_and_text_measurer, render_sequence_source_svg,
        render_sequence_source_svg_with_style,
        render_sequence_source_svg_with_style_and_text_measurer, PlantUmlDiagramDetection,
        PlantUmlDiagramDetectionReason, PlantUmlRenderError,
    };
    pub use crate::render::{
        render_sequence_svg, render_sequence_svg_with_style,
        render_sequence_svg_with_style_and_text_measurer, ArrowheadStyle, CornerRadius,
        PlantUmlStyle, PlantUmlTheme, RenderedPlantUml, StrokeWidth,
    };
    pub use crate::sequence::{
        Activation, ActivationKind, Autoactivate, Autonumber, AutonumberCommand, BoxBlock, BoxEnd,
        Delay, Divider, Group, GroupElse, GroupEnd, GroupKind, HeaderFooter, HeaderFooterKind,
        HideFootbox, HideUnlinked, LineStyle, Mainframe, Message, MessageActivationShortcut,
        MessageActivationTarget, MessageArrow, MessageArrowFeatures, MessageArrowHalf,
        MessageArrowHead, MessageDirection, MessageEndpoint, MessageEndpointKind, Note,
        NotePlacement, NoteShape, PageBreak, Participant, ParticipantDeclaration,
        ParticipantDisplay, ParticipantKind, ParticipantSpot, PlantUmlBlockKind, PlantUmlError,
        PlantUmlError as SequenceParseError, Pragma, Reference, Return, SequenceDiagram,
        SequenceEvent, SequenceMessage, SequenceSemanticEvent, SkinParam, Space, StyleBlock, Title,
    };
}
