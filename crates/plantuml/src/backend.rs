use crate::pipeline::{PlantUmlDiagramKind, PlantUmlError, PlantUmlOptions, PlantUmlSvg};
use crate::text_layout::{InlineTextMeasurementRequest, InlineTextMeasurementStyle, TextMeasurer};

pub(crate) trait DiagramBackend: Sync {
    fn kind(&self) -> PlantUmlDiagramKind;

    fn render(
        &self,
        source: &str,
        options: &PlantUmlOptions,
        measurer: &dyn TextMeasurer,
    ) -> Result<PlantUmlSvg, PlantUmlError>;

    fn collect_text_measurement_requests(
        &self,
        source: &str,
        options: &PlantUmlOptions,
        base_style: &InlineTextMeasurementStyle,
    ) -> Result<Vec<InlineTextMeasurementRequest>, PlantUmlError>;
}
