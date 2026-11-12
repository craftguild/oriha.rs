mod document;
mod html_scan;
mod minify;
mod rewrite;

use crate::render::{prepare_markdown_layout, render, render_prepared_markdown};
use crate::types::{
    DiagnosticEngine, ExportHtmlInput, ExportHtmlOutput, HtmlExportOptions, PreparedHtmlExport,
    PreparedMarkdownLayout, RenderDiagnostic, RenderOutput, RenderStats, Severity,
    TextMeasurementResults,
};

pub fn export_html(input: ExportHtmlInput) -> ExportHtmlOutput {
    export_render_output(render(input.input), &input.options)
}

pub fn prepare_html_export(input: ExportHtmlInput) -> PreparedHtmlExport {
    let prepared = prepare_markdown_layout(input.input.clone());
    let mut export = PreparedHtmlExport::new(input, prepared.text_measurements);
    export.diagnostics = prepared.diagnostics;
    export
}

pub fn render_prepared_html_export(
    prepared: PreparedHtmlExport,
    metrics: TextMeasurementResults,
) -> ExportHtmlOutput {
    let options = prepared.input.options;
    let mut markdown_prepared =
        PreparedMarkdownLayout::new(prepared.input.input, prepared.text_measurements);
    markdown_prepared.diagnostics = prepared.diagnostics;
    export_render_output(
        render_prepared_markdown(markdown_prepared, metrics),
        &options,
    )
}

pub fn export_html_json(input_json: &str) -> String {
    let output = match serde_json::from_str::<ExportHtmlInput>(input_json) {
        Ok(input) => export_html(input),
        Err(error) => invalid_export_html_output(
            "input.invalid_json",
            format!("ExportHtmlInput JSON could not be parsed: {error}"),
        ),
    };

    serde_json::to_string(&output).expect("ExportHtmlOutput serialization should not fail")
}

pub fn prepare_html_export_json(input_json: &str) -> String {
    let prepared = match serde_json::from_str::<ExportHtmlInput>(input_json) {
        Ok(input) => prepare_html_export(input),
        Err(error) => PreparedHtmlExport::with_error(
            ExportHtmlInput::default(),
            "input.invalid_json",
            format!("ExportHtmlInput JSON could not be parsed: {error}"),
        ),
    };

    serde_json::to_string(&prepared).expect("PreparedHtmlExport serialization should not fail")
}

pub fn render_prepared_html_export_json(prepared_json: &str, text_metrics_json: &str) -> String {
    let prepared = match serde_json::from_str::<PreparedHtmlExport>(prepared_json) {
        Ok(prepared) => prepared,
        Err(error) => {
            return serde_json::to_string(&invalid_export_html_output(
                "prepared.invalid_json",
                format!("PreparedHtmlExport JSON could not be parsed: {error}"),
            ))
            .expect("ExportHtmlOutput serialization should not fail");
        }
    };

    let metrics = match crate::render::parse_text_measurement_results(text_metrics_json) {
        Ok(metrics) => metrics,
        Err(error) => {
            return serde_json::to_string(&invalid_export_html_output(
                "text_metrics.invalid_json",
                format!("TextMeasurementResult JSON could not be parsed: {error}"),
            ))
            .expect("ExportHtmlOutput serialization should not fail");
        }
    };

    let output = render_prepared_html_export(prepared, metrics);
    serde_json::to_string(&output).expect("ExportHtmlOutput serialization should not fail")
}

fn export_render_output(output: RenderOutput, options: &HtmlExportOptions) -> ExportHtmlOutput {
    let (html, mut css) =
        rewrite::rewrite_artifacts(output.html, output.css.unwrap_or_default(), options);

    if options.minify {
        css = minify::minify_css(&css);
    }

    let mut document = document::standalone_document(&html, &css);
    if options.minify {
        document = minify::minify_html(&document);
    }

    ExportHtmlOutput {
        html: document,
        stats: output.stats,
        diagnostics: output.diagnostics,
    }
}

fn invalid_export_html_output(code: &str, message: String) -> ExportHtmlOutput {
    let mut stats = RenderStats::default();
    stats.record_issue_code(code, Severity::Error);
    stats.finalize();
    ExportHtmlOutput {
        html: String::new(),
        stats,
        diagnostics: vec![RenderDiagnostic {
            severity: Severity::Error,
            code: code.to_string(),
            message,
            source_range: None,
            block_id: None,
            engine: DiagnosticEngine::Core,
        }],
    }
}
