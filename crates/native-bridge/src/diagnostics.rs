use crate::stats::increment_kind_count;
use crate::types::*;
use crate::{
    PREPARED_HTML_EXPORT_VERSION, PREPARED_MARKDOWN_LAYOUT_VERSION, PREPARED_MARKDOWN_PATCH_VERSION,
};

pub(crate) fn invalid_prepared_markdown_layout(
    code: &str,
    message: String,
) -> PreparedMarkdownLayout {
    PreparedMarkdownLayout {
        version: PREPARED_MARKDOWN_LAYOUT_VERSION,
        input: RenderInput::default(),
        text_measurements: TextMeasurementBatch::default(),
        policy_diagnostics: vec![bridge_diagnostic(Severity::Error, code, message)],
        policy_stats: RenderStats::default(),
    }
}

pub(crate) fn invalid_typed_prepared_markdown_patch(
    previous_snapshot: Option<RenderSnapshot>,
    input: RenderInput,
    code: &str,
    message: String,
) -> PreparedMarkdownPatch {
    PreparedMarkdownPatch {
        version: PREPARED_MARKDOWN_PATCH_VERSION,
        previous_snapshot,
        input,
        text_measurements: TextMeasurementBatch::default(),
        policy_diagnostics: vec![bridge_diagnostic(Severity::Error, code, message)],
        policy_stats: RenderStats::default(),
    }
}

pub(crate) fn invalid_typed_prepared_html_export(
    input: RenderInput,
    options: HtmlExportOptions,
    code: &str,
    message: String,
) -> PreparedHtmlExport {
    PreparedHtmlExport {
        version: PREPARED_HTML_EXPORT_VERSION,
        input,
        options,
        text_measurements: TextMeasurementBatch::default(),
        policy_diagnostics: vec![bridge_diagnostic(Severity::Error, code, message)],
        policy_stats: RenderStats::default(),
    }
}

pub(crate) fn invalid_render_output(code: &str, message: String) -> RenderOutput {
    let mut stats = RenderStats::default();
    record_issue_code(&mut stats, code);
    RenderOutput {
        html: String::new(),
        css: None,
        stats,
        diagnostics: vec![bridge_diagnostic(Severity::Error, code, message)],
        assets: Vec::new(),
        source_map: Vec::new(),
    }
}

pub(crate) fn invalid_export_html_output(code: &str, message: String) -> ExportHtmlOutput {
    let mut stats = RenderStats::default();
    record_issue_code(&mut stats, code);
    ExportHtmlOutput {
        html: String::new(),
        stats,
        diagnostics: vec![bridge_diagnostic(Severity::Error, code, message)],
    }
}

pub(crate) fn invalid_render_patch_output(
    code: &str,
    message: String,
    reason: RenderPatchFallbackReason,
) -> RenderPatchOutput {
    let output = invalid_render_output(code, message);
    let html = output.html.clone();
    RenderPatchOutput {
        output,
        snapshot: RenderSnapshot::default(),
        strategy: RenderPatchStrategy::ReplaceAll,
        fallback_reason: Some(reason),
        patches: vec![RenderPatchOperation {
            op: RenderPatchOperationKind::ReplaceAll,
            html,
            remove_stable_ids: Vec::new(),
            insert_after_stable_id: None,
            insert_before_stable_id: None,
        }],
    }
}

fn record_issue_code(stats: &mut RenderStats, code: &str) {
    let kind = match code {
        "bridge.panic" => "bridgePanic",
        _ => "unknownError",
    };
    increment_kind_count(&mut stats.issues_by_kind, kind, 1);
}

pub(crate) fn bridge_diagnostic(
    severity: Severity,
    code: &str,
    message: String,
) -> RenderDiagnostic {
    RenderDiagnostic {
        severity,
        code: code.to_string(),
        message,
        source_range: None,
        block_id: None,
        engine: DiagnosticEngine::Bridge,
    }
}

pub(crate) fn add_bridge_diagnostics(
    mut output: RenderOutput,
    diagnostics: Vec<RenderDiagnostic>,
) -> RenderOutput {
    output.diagnostics = merged_diagnostics(diagnostics, output.diagnostics);
    output
}

pub(crate) fn merged_diagnostics(
    diagnostics: Vec<RenderDiagnostic>,
    output_diagnostics: Vec<RenderDiagnostic>,
) -> Vec<RenderDiagnostic> {
    if diagnostics.is_empty() {
        return output_diagnostics;
    }

    let mut merged = diagnostics;
    merged.extend(output_diagnostics);
    merged
}
