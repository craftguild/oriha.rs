//! Bridge identity and prepared rendering records.
use super::*;

pub(super) fn records() -> Vec<RecordSpec> {
    vec![
        record(
            "NativeBridgeVersionInfo",
            &[],
            vec![
                field("version", primitive("string"), false),
                field("gitShortHash", primitive("string"), false),
            ],
        ),
        record(
            "PreparedMarkdownLayout",
            &[],
            vec![
                field("version", primitive("number"), false),
                field("input", named("BridgeRenderInput"), false),
                field("textMeasurements", named("TextMeasurementBatch"), false),
                field("policyDiagnostics", array(named("RenderDiagnostic")), false),
                field("policyStats", named("RenderStats"), false),
            ],
        ),
        record(
            "PreparedMarkdownPatch",
            &[],
            vec![
                field("version", primitive("number"), false),
                field("previousSnapshot", named("RenderSnapshot"), true),
                field("input", named("BridgeRenderInput"), false),
                field("textMeasurements", named("TextMeasurementBatch"), false),
                field("policyDiagnostics", array(named("RenderDiagnostic")), false),
                field("policyStats", named("RenderStats"), false),
            ],
        ),
        record(
            "PreparedHtmlExport",
            &[],
            vec![
                field("version", primitive("number"), false),
                field("input", named("BridgeRenderInput"), false),
                field("options", named("HtmlExportOptions"), false),
                field("textMeasurements", named("TextMeasurementBatch"), false),
                field("policyDiagnostics", array(named("RenderDiagnostic")), false),
                field("policyStats", named("RenderStats"), false),
            ],
        ),
    ]
}
