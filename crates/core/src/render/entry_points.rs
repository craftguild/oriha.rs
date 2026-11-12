use super::*;

pub fn render(input: RenderInput) -> RenderOutput {
    catch_engine_panic(
        std::panic::AssertUnwindSafe(move || OrihaEngine::default().render(input)),
        |message| invalid_render_output("engine.panic", message),
    )
}

pub fn export_html(input: ExportHtmlInput) -> ExportHtmlOutput {
    crate::export_html::export_html(input)
}

pub fn export_html_json(input_json: &str) -> String {
    crate::export_html::export_html_json(input_json)
}

pub fn prepare_html_export(input: ExportHtmlInput) -> PreparedHtmlExport {
    crate::export_html::prepare_html_export(input)
}

pub fn prepare_html_export_json(input_json: &str) -> String {
    crate::export_html::prepare_html_export_json(input_json)
}

pub fn render_prepared_html_export(
    prepared: PreparedHtmlExport,
    metrics: TextMeasurementResults,
) -> ExportHtmlOutput {
    crate::export_html::render_prepared_html_export(prepared, metrics)
}

pub fn render_prepared_html_export_json(prepared_json: &str, text_metrics_json: &str) -> String {
    crate::export_html::render_prepared_html_export_json(prepared_json, text_metrics_json)
}

pub fn render_markdown_patch(input: RenderPatchInput) -> RenderPatchOutput {
    catch_engine_panic(
        std::panic::AssertUnwindSafe(move || OrihaEngine::default().render_patch(input)),
        |message| {
            invalid_render_patch_output(
                "engine.panic",
                message,
                RenderPatchFallbackReason::EnginePanic,
            )
        },
    )
}

/// Runs an engine entry point and converts a panic into a caller-supplied
/// fallback value so FFI consumers never observe unwinding across a language
/// boundary.
pub fn catch_engine_panic<T>(
    op: impl FnOnce() -> T + std::panic::UnwindSafe,
    fallback: impl FnOnce(String) -> T,
) -> T {
    match std::panic::catch_unwind(op) {
        Ok(value) => value,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "engine panicked without a message".to_string());
            fallback(format!("Markdown engine panicked: {message}"))
        }
    }
}

pub fn render_json(input_json: &str) -> String {
    let output = match serde_json::from_str::<RenderInput>(input_json) {
        Ok(input) => render(input),
        Err(error) => invalid_render_output(
            "input.invalid_json",
            format!("RenderInput JSON could not be parsed: {error}"),
        ),
    };

    serde_json::to_string(&output).expect("RenderOutput serialization should not fail")
}

pub fn render_markdown_patch_json(input_json: &str) -> String {
    let output = match serde_json::from_str::<RenderPatchInput>(input_json) {
        Ok(input) => render_markdown_patch(input),
        Err(error) => invalid_render_patch_output(
            "input.invalid_json",
            format!("RenderPatchInput JSON could not be parsed: {error}"),
            RenderPatchFallbackReason::InvalidInput,
        ),
    };

    serde_json::to_string(&output).expect("RenderPatchOutput serialization should not fail")
}

pub fn prepare_markdown_patch_json(input_json: &str) -> String {
    let prepared = match serde_json::from_str::<RenderPatchInput>(input_json) {
        Ok(input) => prepare_markdown_patch(input),
        Err(error) => PreparedMarkdownPatch::with_error(
            RenderPatchInput::default(),
            "input.invalid_json",
            format!("RenderPatchInput JSON could not be parsed: {error}"),
        ),
    };

    serde_json::to_string(&prepared).expect("PreparedMarkdownPatch serialization should not fail")
}

pub fn render_prepared_markdown_patch_json(prepared_json: &str, text_metrics_json: &str) -> String {
    let prepared = match serde_json::from_str::<PreparedMarkdownPatch>(prepared_json) {
        Ok(prepared) => prepared,
        Err(error) => {
            return serde_json::to_string(&invalid_render_patch_output(
                "prepared.invalid_json",
                format!("PreparedMarkdownPatch JSON could not be parsed: {error}"),
                RenderPatchFallbackReason::InvalidInput,
            ))
            .expect("RenderPatchOutput serialization should not fail");
        }
    };

    let metrics = match parse_text_measurement_results(text_metrics_json) {
        Ok(metrics) => metrics,
        Err(error) => {
            return serde_json::to_string(&invalid_render_patch_output(
                "text_metrics.invalid_json",
                format!("TextMeasurementResult JSON could not be parsed: {error}"),
                RenderPatchFallbackReason::InvalidInput,
            ))
            .expect("RenderPatchOutput serialization should not fail");
        }
    };

    let output = render_prepared_markdown_patch(prepared, metrics);
    serde_json::to_string(&output).expect("RenderPatchOutput serialization should not fail")
}

pub fn prepare_markdown_layout_json(input_json: &str) -> String {
    let prepared = match serde_json::from_str::<RenderInput>(input_json) {
        Ok(input) => prepare_markdown_layout(input),
        Err(error) => PreparedMarkdownLayout::with_error(
            RenderInput::default(),
            "input.invalid_json",
            format!("RenderInput JSON could not be parsed: {error}"),
        ),
    };

    serde_json::to_string(&prepared).expect("PreparedMarkdownLayout serialization should not fail")
}

pub fn render_prepared_markdown_json(prepared_json: &str, text_metrics_json: &str) -> String {
    let prepared = match serde_json::from_str::<PreparedMarkdownLayout>(prepared_json) {
        Ok(prepared) => prepared,
        Err(error) => {
            return serde_json::to_string(&invalid_render_output(
                "prepared.invalid_json",
                format!("PreparedMarkdownLayout JSON could not be parsed: {error}"),
            ))
            .expect("RenderOutput serialization should not fail");
        }
    };

    let metrics = match parse_text_measurement_results(text_metrics_json) {
        Ok(metrics) => metrics,
        Err(error) => {
            return serde_json::to_string(&invalid_render_output(
                "text_metrics.invalid_json",
                format!("TextMeasurementResult JSON could not be parsed: {error}"),
            ))
            .expect("RenderOutput serialization should not fail");
        }
    };

    let output = render_prepared_markdown(prepared, metrics);
    serde_json::to_string(&output).expect("RenderOutput serialization should not fail")
}
