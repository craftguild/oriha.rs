use std::collections::BTreeMap;

use crate::pipeline::{MermaidDiagramKind, MermaidError, MermaidOptions, MermaidSvg};
use crate::{MermaidMeasurementRequest, MermaidMeasurementResult};

pub(crate) trait DiagramBackend: Sync {
    fn kind(&self) -> MermaidDiagramKind;

    fn render(
        &self,
        source: &str,
        diagram_id: &str,
        options: &MermaidOptions,
        measurements: &BTreeMap<String, MermaidMeasurementResult>,
    ) -> Result<MermaidSvg, MermaidError>;

    fn collect_text_measurement_requests(
        &self,
        source: &str,
        options: &MermaidOptions,
    ) -> Result<Vec<MermaidMeasurementRequest>, MermaidError>;
}
