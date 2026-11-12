//! Public text measurement collection and recording adapters.
use super::*;

pub fn collect_plantuml_text_measurements(
    source: &str,
    measurer: &dyn TextMeasurer,
) -> Result<Vec<String>, PlantUmlRenderError> {
    collect_plantuml_text_measurements_with_options(source, &LayoutOptions::default(), measurer)
}

pub fn collect_plantuml_text_measurements_with_options(
    source: &str,
    _options: &LayoutOptions,
    measurer: &dyn TextMeasurer,
) -> Result<Vec<String>, PlantUmlRenderError> {
    let recorder = RecordingTextMeasurer::new(measurer);
    let default_options = PlantUmlOptions::default();
    let api_options = PlantUmlOptions {
        font_family: recorder
            .base_inline_text_style()
            .font_family
            .unwrap_or_else(|| default_options.font_family.clone()),
        ..default_options
    };
    let detection = detect_plantuml_diagram(source);
    let backend = PlantUml::new().backend(detection.kind)?;
    backend.render(source, &api_options, &recorder)?;
    Ok(recorder.into_lines())
}

pub fn collect_plantuml_text_measurement_requests(
    source: &str,
    base_style: &InlineTextMeasurementStyle,
) -> Result<Vec<InlineTextMeasurementRequest>, PlantUmlRenderError> {
    collect_plantuml_text_measurement_requests_with_options(
        source,
        &LayoutOptions::default(),
        base_style,
    )
}

pub fn collect_plantuml_text_measurement_requests_with_options(
    source: &str,
    options: &LayoutOptions,
    base_style: &InlineTextMeasurementStyle,
) -> Result<Vec<InlineTextMeasurementRequest>, PlantUmlRenderError> {
    let default_options = PlantUmlOptions::default();
    let api_options = PlantUmlOptions {
        font_family: base_style
            .font_family
            .clone()
            .unwrap_or_else(|| default_options.font_family.clone()),
        body_font_size_px: base_style.font_size.or(default_options.body_font_size_px),
        ..default_options
    };
    let _ = options;
    let detection = detect_plantuml_diagram(source);
    let backend = PlantUml::new().backend(detection.kind)?;
    backend.collect_text_measurement_requests(source, &api_options, base_style)
}

pub fn collect_sequence_text_measurements(
    source: &str,
    options: &LayoutOptions,
    measurer: &dyn TextMeasurer,
) -> Result<Vec<String>, SequenceParseError> {
    let recorder = RecordingTextMeasurer::new(measurer);
    collect_sequence_text_measurements_with_measurer(source, options, &recorder)?;
    Ok(recorder.into_lines())
}

pub(crate) fn collect_sequence_text_measurements_with_measurer(
    source: &str,
    options: &LayoutOptions,
    measurer: &dyn TextMeasurer,
) -> Result<(), SequenceParseError> {
    let diagram = parse_sequence(source)?;
    let layout = layout_sequence_with_text_measurer(&diagram, options, measurer);
    collect_sequence_render_text_measurements(&layout, measurer);
    Ok(())
}

impl<'a> RecordingTextMeasurer<'a> {
    pub(crate) fn new(delegate: &'a dyn TextMeasurer) -> Self {
        Self {
            delegate,
            seen: RefCell::new(HashSet::new()),
            lines: RefCell::new(Vec::new()),
            requests: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn into_lines(self) -> Vec<String> {
        self.lines.into_inner()
    }
}

impl TextMeasurer for RecordingTextMeasurer<'_> {
    fn base_inline_text_style(&self) -> InlineTextMeasurementStyle {
        self.delegate.base_inline_text_style()
    }

    fn measure_inline_text_request(
        &self,
        request: &InlineTextMeasurementRequest,
    ) -> InlineTextMeasurementResult {
        let mut seen = self.seen.borrow_mut();
        if seen.insert(request.id.clone()) {
            self.lines.borrow_mut().push(request.source.clone());
            self.requests.borrow_mut().push(request.clone());
        }
        self.delegate.measure_inline_text_request(request)
    }
}

impl RecordingApproxTextMeasurer {
    pub(crate) fn new(base_style: InlineTextMeasurementStyle) -> Self {
        Self {
            base_style,
            seen: RefCell::new(HashSet::new()),
            requests: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn into_requests(self) -> Vec<InlineTextMeasurementRequest> {
        self.requests.into_inner()
    }
}

impl TextMeasurer for RecordingApproxTextMeasurer {
    fn base_inline_text_style(&self) -> InlineTextMeasurementStyle {
        self.base_style.clone()
    }

    fn measure_inline_text_request(
        &self,
        request: &InlineTextMeasurementRequest,
    ) -> InlineTextMeasurementResult {
        let mut seen = self.seen.borrow_mut();
        if seen.insert(request.id.clone()) {
            self.requests.borrow_mut().push(request.clone());
        }
        ApproxTextMeasurer.measure_inline_text_request(request)
    }
}
