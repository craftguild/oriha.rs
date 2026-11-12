use super::*;

pub(super) fn collect_plantuml_measurement_requests(
    blocks: &[BlockNode],
    source: &SourceDocument,
    options: &PlantUmlOptions,
    requests: &mut BTreeMap<String, TextMeasurementRequest>,
) {
    for block in blocks {
        if let BlockKind::FencedCode(node) = &block.kind {
            if code_block_language_is_plantuml(node) {
                let mut text =
                    normalize_fenced_code_text(source.source_span(node.body_span), node.indent);
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
                collect_plantuml_source_measurement_requests(&text, options, requests);
            }
        }
        if !block.children.is_empty() {
            collect_plantuml_measurement_requests(&block.children, source, options, requests);
        }
    }
}

pub(super) fn collect_plantuml_source_measurement_requests(
    source: &str,
    options: &PlantUmlOptions,
    requests: &mut BTreeMap<String, TextMeasurementRequest>,
) {
    let measurements = match PlantUml::new().collect_text_measurement_requests(source, options) {
        Ok(measurements) => measurements,
        Err(PlantUmlError::UnsupportedDiagram { .. }) => return,
        Err(_) => return,
    };
    for measurement in measurements {
        let id = measurement.id.clone();
        requests
            .entry(id.clone())
            .or_insert_with(|| plantuml_text_measurement_request(measurement));
    }
}

pub(super) fn collect_mermaid_measurement_requests(
    blocks: &[BlockNode],
    source: &SourceDocument,
    options: &MermaidOptions,
    requests: &mut BTreeMap<String, TextMeasurementRequest>,
) {
    for block in blocks {
        if let BlockKind::FencedCode(node) = &block.kind {
            if code_block_language_is_mermaid(node) {
                let mut text =
                    normalize_fenced_code_text(source.source_span(node.body_span), node.indent);
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
                collect_mermaid_source_measurement_requests(&text, options, requests);
            }
        }
        if !block.children.is_empty() {
            collect_mermaid_measurement_requests(&block.children, source, options, requests);
        }
    }
}

pub(super) fn collect_mermaid_source_measurement_requests(
    source: &str,
    options: &MermaidOptions,
    requests: &mut BTreeMap<String, TextMeasurementRequest>,
) {
    let measurements = match Mermaid::new().collect_text_measurement_requests(source, options) {
        Ok(measurements) => measurements,
        Err(_) => return,
    };
    for measurement in measurements {
        let id = measurement.id.clone();
        requests
            .entry(id)
            .or_insert_with(|| mermaid_text_measurement_request(measurement));
    }
}

pub(super) fn plantuml_text_measurement_request(
    measurement: InlineTextMeasurementRequest,
) -> TextMeasurementRequest {
    let style = text_measurement_style_from_plantuml(&measurement.style);
    TextMeasurementRequest {
        id: measurement.id,
        engine: "plantuml".to_string(),
        kind: "inlineText".to_string(),
        text: measurement.text,
        style: Some(style.clone()),
        font_family: style.font_family.clone(),
        font_size: style.font_size,
        font_weight: style.font_weight.clone(),
        font_style: style.font_style.clone(),
        runs: Some(
            measurement
                .runs
                .into_iter()
                .map(|run| TextMeasurementRun {
                    id: Some(run.id),
                    text: run.text,
                    style: Some(text_measurement_style_from_plantuml(&run.style)),
                })
                .collect(),
        ),
        width: None,
        max_width: None,
        wrap_mode: None,
    }
}

pub(super) fn mermaid_text_measurement_request(
    measurement: MermaidMeasurementRequest,
) -> TextMeasurementRequest {
    let style = text_measurement_style_from_mermaid(&measurement);
    TextMeasurementRequest {
        id: measurement.id,
        engine: "mermaid".to_string(),
        kind: measurement.kind.to_string(),
        text: measurement.text,
        style: Some(style.clone()),
        font_family: style.font_family.clone(),
        font_size: style.font_size,
        font_weight: style.font_weight.clone(),
        font_style: None,
        runs: None,
        width: measurement.width,
        max_width: measurement.max_width,
        wrap_mode: measurement.wrap_mode.map(|wrap_mode| wrap_mode.to_string()),
    }
}

pub(super) fn text_measurement_style_from_mermaid(
    measurement: &MermaidMeasurementRequest,
) -> TextMeasurementStyle {
    TextMeasurementStyle {
        font_family: measurement.style.font_family.clone(),
        font_size: Some(measurement.style.font_size),
        font_weight: measurement.style.font_weight.clone(),
        font_style: None,
        line_height: None,
        letter_spacing: None,
        text_transform: None,
        text_decoration_line: None,
        color: None,
        width: measurement.width,
        max_width: measurement.max_width,
    }
}

pub(super) fn text_measurement_style_from_plantuml(
    style: &InlineTextMeasurementStyle,
) -> TextMeasurementStyle {
    TextMeasurementStyle {
        font_family: style.font_family.clone(),
        font_size: style.font_size,
        font_weight: style.font_weight.clone(),
        font_style: style.font_style.clone(),
        line_height: None,
        letter_spacing: None,
        text_transform: None,
        text_decoration_line: None,
        color: None,
        width: None,
        max_width: None,
    }
}

pub(crate) fn parse_text_measurement_results(
    input: &str,
) -> Result<TextMeasurementResults, serde_json::Error> {
    if input.trim().is_empty() {
        return Ok(TextMeasurementResults::default());
    }

    let value = serde_json::from_str::<serde_json::Value>(input)?;
    let results = if value.is_array() {
        serde_json::from_value::<Vec<TextMeasurementResult>>(value)?
    } else {
        serde_json::from_value::<TextMeasurementResults>(value)?.results
    };

    Ok(TextMeasurementResults { results })
}
