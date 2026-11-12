//! Prepared HTML export core operations; exported FFI entry points retain their stable parent namespace.
use super::*;

pub(super) fn prepare_html_export_inner(
    input: RenderInput,
    options: Option<HtmlExportOptions>,
) -> PreparedHtmlExport {
    let options = options.unwrap_or_default();
    let BridgeInput {
        input,
        core_input,
        mut diagnostics,
        stats,
    } = apply_plan_policy(input);
    let prepared = oriha_core::prepare_html_export(oriha_core::ExportHtmlInput {
        input: core_input,
        options: options.clone().into(),
    });
    diagnostics.extend(prepared.diagnostics.into_iter().map(RenderDiagnostic::from));

    PreparedHtmlExport {
        version: PREPARED_HTML_EXPORT_VERSION,
        input,
        options,
        text_measurements: prepared.text_measurements.into(),
        policy_diagnostics: diagnostics,
        policy_stats: stats,
    }
}

pub(super) fn render_prepared_html_export_inner(
    prepared: PreparedHtmlExport,
    text_metrics: TextMeasurementResults,
) -> ExportHtmlOutput {
    if let Some(diagnostic) = prepared
        .policy_diagnostics
        .iter()
        .find(|diagnostic| matches!(diagnostic.severity, Severity::Error))
    {
        return invalid_export_html_output(&diagnostic.code, diagnostic.message.clone());
    }

    let policy_diagnostics = prepared.policy_diagnostics;
    let policy_stats = prepared.policy_stats;
    let core_prepared = oriha_core::PreparedHtmlExport {
        version: prepared.version,
        input: oriha_core::ExportHtmlInput {
            input: input_to_core(prepared.input),
            options: prepared.options.into(),
        },
        text_measurements: prepared.text_measurements.into(),
        diagnostics: Vec::new(),
    };
    let mut output: ExportHtmlOutput = oriha_core::render_prepared_html_export(
        core_prepared,
        oriha_core::TextMeasurementResults::from(text_metrics),
    )
    .into();
    merge_render_stats(&mut output.stats, policy_stats);
    output.diagnostics = merged_diagnostics(policy_diagnostics, output.diagnostics);
    output
}
