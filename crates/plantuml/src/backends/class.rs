use crate::backend::DiagramBackend;
use crate::class_diagram::{
    collect_class_text_measurement_requests, render_class_source_svg_with_text_measurer,
};
use crate::pipeline::{PlantUmlDiagramKind, PlantUmlError, PlantUmlOptions, PlantUmlSvg};
use crate::text_layout::{InlineTextMeasurementRequest, InlineTextMeasurementStyle, TextMeasurer};

pub(crate) struct ClassBackend;

impl DiagramBackend for ClassBackend {
    fn kind(&self) -> PlantUmlDiagramKind {
        PlantUmlDiagramKind::Class
    }

    fn render(
        &self,
        source: &str,
        options: &PlantUmlOptions,
        measurer: &dyn TextMeasurer,
    ) -> Result<PlantUmlSvg, PlantUmlError> {
        render_class_source_svg_with_text_measurer(source, options, measurer)
    }

    fn collect_text_measurement_requests(
        &self,
        source: &str,
        options: &PlantUmlOptions,
        base_style: &InlineTextMeasurementStyle,
    ) -> Result<Vec<InlineTextMeasurementRequest>, PlantUmlError> {
        collect_class_text_measurement_requests(source, options, base_style)
    }
}
