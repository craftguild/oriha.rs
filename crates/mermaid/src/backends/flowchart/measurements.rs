//! Flowchart text-measurement requests and their collector.
use super::*;

pub(super) fn collect_flowchart_measurement_requests(
    diagram: &FlowchartDiagram,
    theme: &MermaidTheme,
) -> Vec<MermaidMeasurementRequest> {
    let mut collector = MeasurementCollector::default();
    for node in &diagram.nodes {
        collector.push(text_layout::measurement_request(
            theme,
            &node.label,
            MermaidMeasurementKind::Wrapped,
            Some(NODE_MAX_WIDTH),
            Some("600"),
        ));
    }
    for edge in &diagram.edges {
        if let Some(label) = &edge.label {
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

impl MeasurementCollector {
    pub(super) fn push(&mut self, request: MermaidMeasurementRequest) {
        if self.seen.insert(request.id.clone()) {
            self.requests.push(request);
        }
    }

    pub(super) fn into_requests(self) -> Vec<MermaidMeasurementRequest> {
        self.requests
    }
}
