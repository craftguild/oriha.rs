//! Public native function manifest.
use super::*;

pub(super) fn functions() -> Vec<FunctionSpec> {
    vec![
        function(
            "nativeBridgeVersionInfo",
            vec![],
            named("NativeBridgeVersionInfo"),
            true,
        ),
        function(
            "prepareMarkdownLayout",
            vec![param("input", named("BridgeRenderInput"))],
            named("PreparedMarkdownLayout"),
            true,
        ),
        function(
            "prepareMarkdownPatch",
            vec![
                param("previousSnapshot", nullable(named("RenderSnapshot"))),
                param("input", named("BridgeRenderInput")),
            ],
            named("PreparedMarkdownPatch"),
            true,
        ),
        function(
            "prepareHtmlExport",
            vec![
                param("input", named("BridgeRenderInput")),
                param(
                    "options",
                    union(vec![named("HtmlExportOptions"), undefined()]),
                ),
            ],
            named("PreparedHtmlExport"),
            true,
        ),
        function(
            "resolveRenderSettings",
            vec![param("input", named("BridgeSettingsInput"))],
            named("BridgeResolvedSettings"),
            true,
        ),
        function(
            "renderPreparedMarkdown",
            vec![
                param("layout", named("PreparedMarkdownLayout")),
                param("measureResults", array(named("TextMeasurementResult"))),
            ],
            named("RenderOutput"),
            true,
        ),
        function(
            "renderPreparedMarkdownPatch",
            vec![
                param("preparedPatch", named("PreparedMarkdownPatch")),
                param("measureResults", array(named("TextMeasurementResult"))),
            ],
            named("RenderPatchOutput"),
            true,
        ),
        function(
            "renderPreparedHtmlExport",
            vec![
                param("preparedExport", named("PreparedHtmlExport")),
                param("measureResults", array(named("TextMeasurementResult"))),
            ],
            named("ExportHtmlOutput"),
            true,
        ),
        function(
            "usageAnalyticsOutboxStartEnqueue",
            vec![
                param("input", named("UsageAnalyticsOutboxEnqueueStartInput")),
                param("nowMs", primitive("number")),
            ],
            named("UsageAnalyticsOutboxStep"),
            true,
        ),
        function(
            "usageAnalyticsOutboxStartDrain",
            vec![
                param("input", named("UsageAnalyticsOutboxStartInput")),
                param("nowMs", primitive("number")),
            ],
            named("UsageAnalyticsOutboxStep"),
            true,
        ),
        function(
            "usageAnalyticsOutboxStartDeliveryStateChange",
            vec![param(
                "input",
                named("UsageAnalyticsOutboxDeliveryStateInput"),
            )],
            named("UsageAnalyticsOutboxStep"),
            true,
        ),
        function(
            "usageAnalyticsOutboxResume",
            vec![
                param("input", named("UsageAnalyticsOutboxResumeInput")),
                param("nowMs", primitive("number")),
            ],
            named("UsageAnalyticsOutboxStep"),
            true,
        ),
    ]
}
