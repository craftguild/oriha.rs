use super::*;

pub fn prepare_markdown_layout(input: RenderInput) -> PreparedMarkdownLayout {
    let fallback_input = input.clone();
    catch_engine_panic(
        std::panic::AssertUnwindSafe(move || prepare_markdown_layout_inner(input)),
        |message| PreparedMarkdownLayout::with_error(fallback_input, "engine.panic", message),
    )
}

pub(super) fn prepare_markdown_layout_inner(input: RenderInput) -> PreparedMarkdownLayout {
    let text_measurements = collect_text_measurement_batch(&input);
    PreparedMarkdownLayout::new(input, text_measurements)
}

pub fn prepare_markdown_patch(input: RenderPatchInput) -> PreparedMarkdownPatch {
    let fallback_input = input.clone();
    catch_engine_panic(
        std::panic::AssertUnwindSafe(move || prepare_markdown_patch_inner(input)),
        |message| PreparedMarkdownPatch::with_error(fallback_input, "engine.panic", message),
    )
}

pub(super) fn prepare_markdown_patch_inner(input: RenderPatchInput) -> PreparedMarkdownPatch {
    let text_measurements = collect_text_measurement_batch(&input.input);
    PreparedMarkdownPatch::new(input, text_measurements)
}

pub(super) fn collect_text_measurement_batch(input: &RenderInput) -> TextMeasurementBatch {
    let mut requests = BTreeMap::<String, TextMeasurementRequest>::new();
    if (input.features.plantuml || input.features.mermaid)
        && input.markdown.len() <= EngineLimits::default().max_source_bytes
    {
        let source = SourceDocument::new(input.markdown.clone());
        let document = parse_document(&input.markdown, &input.features, EngineLimits::default());
        if input.features.plantuml {
            let plantuml_options = plantuml_options_for_appearance(&input.appearance);
            collect_plantuml_measurement_requests(
                &document.blocks,
                &source,
                &plantuml_options,
                &mut requests,
            );
        }
        if input.features.mermaid {
            let mermaid_options = mermaid_options_for_appearance(&input.appearance);
            collect_mermaid_measurement_requests(
                &document.blocks,
                &source,
                &mermaid_options,
                &mut requests,
            );
        }
    }

    TextMeasurementBatch {
        requests: requests.into_values().collect(),
    }
}

pub fn render_prepared_markdown(
    prepared: PreparedMarkdownLayout,
    metrics: TextMeasurementResults,
) -> RenderOutput {
    if let Some(diagnostic) = prepared
        .diagnostics
        .iter()
        .find(|diagnostic| matches!(diagnostic.severity, Severity::Error))
    {
        return invalid_render_output(&diagnostic.code, diagnostic.message.clone());
    }

    catch_engine_panic(
        std::panic::AssertUnwindSafe(move || {
            let (plantuml_metrics, mermaid_metrics) = text_measurements_by_engine(metrics);
            OrihaEngine::default().render_with_text_metrics(
                prepared.input,
                plantuml_metrics,
                mermaid_metrics,
            )
        }),
        |message| invalid_render_output("engine.panic", message),
    )
}

pub fn render_prepared_markdown_patch(
    prepared: PreparedMarkdownPatch,
    metrics: TextMeasurementResults,
) -> RenderPatchOutput {
    if let Some(diagnostic) = prepared
        .diagnostics
        .iter()
        .find(|diagnostic| matches!(diagnostic.severity, Severity::Error))
    {
        return invalid_render_patch_output(
            &diagnostic.code,
            diagnostic.message.clone(),
            fallback_reason_for_error_code(&diagnostic.code),
        );
    }

    catch_engine_panic(
        std::panic::AssertUnwindSafe(move || {
            let (plantuml_metrics, mermaid_metrics) = text_measurements_by_engine(metrics);
            let previous_snapshot = prepared.previous_snapshot;
            let rendered = OrihaEngine::default().render_with_text_metrics_for_patch(
                prepared.input,
                plantuml_metrics,
                mermaid_metrics,
                previous_snapshot.as_ref(),
                true,
            );
            build_patch_output_from_pipeline(previous_snapshot.as_ref(), rendered)
        }),
        |message| {
            invalid_render_patch_output(
                "engine.panic",
                message,
                RenderPatchFallbackReason::EnginePanic,
            )
        },
    )
}

pub(super) fn fallback_reason_for_error_code(code: &str) -> RenderPatchFallbackReason {
    if code == "engine.panic" {
        RenderPatchFallbackReason::EnginePanic
    } else {
        RenderPatchFallbackReason::InvalidInput
    }
}

pub(super) fn text_measurements_by_engine(
    metrics: TextMeasurementResults,
) -> (
    BTreeMap<String, InlineTextMeasurementResult>,
    BTreeMap<String, MermaidMeasurementResult>,
) {
    let mut plantuml = BTreeMap::new();
    let mut mermaid = BTreeMap::new();
    for result in metrics.results {
        if !result.width.is_finite() || result.width < 0.0 {
            continue;
        }
        if result.id.starts_with("mermaid.") {
            let height = result.height.filter(|value| value.is_finite());
            let line_count = result.line_count;
            let left = result.left.filter(|value| value.is_finite());
            let right = result.right.filter(|value| value.is_finite());
            let raw_width = result.raw_width.filter(|value| value.is_finite());
            if result.width == 0.0
                && height.unwrap_or(0.0) == 0.0
                && line_count.unwrap_or(0) == 0
                && left.is_none()
                && right.is_none()
                && raw_width.is_none()
            {
                continue;
            }
            mermaid.insert(
                result.id.clone(),
                MermaidMeasurementResult {
                    id: result.id,
                    width: result.width,
                    height: height.unwrap_or(0.0),
                    line_count: line_count.unwrap_or(0),
                    left,
                    right,
                    raw_width,
                },
            );
        } else {
            let runs = result
                .runs
                .unwrap_or_default()
                .into_iter()
                .filter(|run| run.width.is_finite() && run.width >= 0.0)
                .map(|run| {
                    let x = run
                        .rects
                        .first()
                        .map(|rect| rect.x)
                        .filter(|x| x.is_finite())
                        .unwrap_or(0.0);
                    InlineTextRunMeasurement {
                        id: run.id.unwrap_or_default(),
                        index: run.index,
                        x,
                        width: run.width,
                    }
                })
                .collect();
            plantuml.insert(
                result.id.clone(),
                InlineTextMeasurementResult {
                    id: result.id,
                    width: result.width,
                    runs,
                },
            );
        }
    }
    (plantuml, mermaid)
}
