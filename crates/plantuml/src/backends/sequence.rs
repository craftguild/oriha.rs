use crate::backend::DiagramBackend;
use crate::layout::layout_sequence_with_text_measurer;
use crate::parser::parse_sequence;
use crate::pipeline::{
    collect_sequence_text_measurements_with_measurer, sequence_error_to_plantuml,
    PlantUmlDiagramKind, PlantUmlError, PlantUmlOptions, PlantUmlSvg, RecordingApproxTextMeasurer,
};
use crate::render::render_sequence_svg_with_style_and_text_measurer;
use crate::text_layout::{InlineTextMeasurementRequest, InlineTextMeasurementStyle, TextMeasurer};

pub(crate) struct SequenceBackend;

impl DiagramBackend for SequenceBackend {
    fn kind(&self) -> PlantUmlDiagramKind {
        PlantUmlDiagramKind::Sequence
    }

    fn render(
        &self,
        source: &str,
        options: &PlantUmlOptions,
        measurer: &dyn TextMeasurer,
    ) -> Result<PlantUmlSvg, PlantUmlError> {
        let diagram = parse_sequence(source)
            .map_err(|error| sequence_error_to_plantuml(PlantUmlDiagramKind::Sequence, error))?;
        let layout =
            layout_sequence_with_text_measurer(&diagram, &options.layout_options(), measurer);
        Ok(PlantUmlSvg::from_rendered(
            PlantUmlDiagramKind::Sequence,
            render_sequence_svg_with_style_and_text_measurer(
                &layout,
                &options.theme(),
                &options.sequence_style(),
                measurer,
            ),
        ))
    }

    fn collect_text_measurement_requests(
        &self,
        source: &str,
        options: &PlantUmlOptions,
        base_style: &InlineTextMeasurementStyle,
    ) -> Result<Vec<InlineTextMeasurementRequest>, PlantUmlError> {
        let measurer = RecordingApproxTextMeasurer::new(base_style.clone());
        collect_sequence_text_measurements_with_measurer(
            source,
            &options.layout_options(),
            &measurer,
        )
        .map_err(|error| sequence_error_to_plantuml(PlantUmlDiagramKind::Sequence, error))?;
        Ok(measurer.into_requests())
    }
}
