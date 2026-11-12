//! ABI-stable native rendering entry points, preparation and panic-safe JSON operations.
mod html_export;
#[allow(unused_imports)]
use html_export::*;

use crate::diagnostics::{
    add_bridge_diagnostics, invalid_export_html_output, invalid_prepared_markdown_layout,
    invalid_render_output, invalid_render_patch_output, invalid_typed_prepared_html_export,
    invalid_typed_prepared_markdown_patch, merged_diagnostics,
};
use crate::plan_policy::{apply_plan_policy, input_to_core, BridgeInput};
use crate::stats::merge_render_stats;
use crate::types::*;
use crate::{
    PREPARED_HTML_EXPORT_VERSION, PREPARED_MARKDOWN_LAYOUT_VERSION, PREPARED_MARKDOWN_PATCH_VERSION,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

#[uniffi::export]
pub fn prepare_markdown_patch(
    previous_snapshot: Option<RenderSnapshot>,
    input: RenderInput,
) -> PreparedMarkdownPatch {
    let fallback_snapshot = previous_snapshot.clone();
    let fallback_input = input.clone();
    oriha_core::catch_engine_panic(
        std::panic::AssertUnwindSafe(move || {
            prepare_markdown_patch_inner(previous_snapshot, input)
        }),
        |message| {
            invalid_typed_prepared_markdown_patch(
                fallback_snapshot,
                fallback_input,
                "bridge.panic",
                message,
            )
        },
    )
}

fn prepare_markdown_patch_inner(
    previous_snapshot: Option<RenderSnapshot>,
    input: RenderInput,
) -> PreparedMarkdownPatch {
    let BridgeInput {
        input,
        core_input,
        mut diagnostics,
        stats,
    } = apply_plan_policy(input);
    let prepared = oriha_core::prepare_markdown_patch(oriha_core::RenderPatchInput {
        previous_snapshot: previous_snapshot.clone().map(Into::into),
        input: core_input,
    });
    diagnostics.extend(prepared.diagnostics.into_iter().map(RenderDiagnostic::from));

    PreparedMarkdownPatch {
        version: PREPARED_MARKDOWN_PATCH_VERSION,
        previous_snapshot,
        input,
        text_measurements: prepared.text_measurements.into(),
        policy_diagnostics: diagnostics,
        policy_stats: stats,
    }
}

#[uniffi::export]
pub fn render_prepared_markdown_patch(
    prepared: PreparedMarkdownPatch,
    text_metrics: TextMeasurementResults,
) -> RenderPatchOutput {
    oriha_core::catch_engine_panic(
        std::panic::AssertUnwindSafe(move || {
            render_prepared_markdown_patch_inner(prepared, text_metrics)
        }),
        |message| {
            invalid_render_patch_output(
                "bridge.panic",
                message,
                RenderPatchFallbackReason::EnginePanic,
            )
        },
    )
}

fn render_prepared_markdown_patch_inner(
    prepared: PreparedMarkdownPatch,
    text_metrics: TextMeasurementResults,
) -> RenderPatchOutput {
    if let Some(diagnostic) = prepared
        .policy_diagnostics
        .iter()
        .find(|diagnostic| matches!(diagnostic.severity, Severity::Error))
    {
        return invalid_render_patch_output(
            &diagnostic.code,
            diagnostic.message.clone(),
            RenderPatchFallbackReason::InvalidInput,
        );
    }

    let policy_diagnostics = prepared.policy_diagnostics;
    let policy_stats = prepared.policy_stats;
    let core_prepared = oriha_core::PreparedMarkdownPatch {
        version: prepared.version,
        previous_snapshot: prepared.previous_snapshot.map(Into::into),
        input: input_to_core(prepared.input),
        text_measurements: prepared.text_measurements.into(),
        diagnostics: Vec::new(),
    };
    let mut output: RenderPatchOutput = oriha_core::render_prepared_markdown_patch(
        core_prepared,
        oriha_core::TextMeasurementResults::from(text_metrics),
    )
    .into();
    merge_render_stats(&mut output.output.stats, policy_stats);
    output.output = add_bridge_diagnostics(output.output, policy_diagnostics);
    output
}

#[uniffi::export]
pub fn prepare_html_export(
    input: RenderInput,
    options: Option<HtmlExportOptions>,
) -> PreparedHtmlExport {
    let fallback_input = input.clone();
    let fallback_options = options.clone().unwrap_or_default();
    oriha_core::catch_engine_panic(
        std::panic::AssertUnwindSafe(move || prepare_html_export_inner(input, options)),
        |message| {
            invalid_typed_prepared_html_export(
                fallback_input,
                fallback_options,
                "bridge.panic",
                message,
            )
        },
    )
}

#[uniffi::export]
pub fn prepare_html_export_json(input_json: String, options_json: Option<String>) -> String {
    let input = match parse_json::<RenderInput>(&input_json, "RenderInput") {
        Ok(input) => input,
        Err(message) => {
            return serialize_json(&invalid_typed_prepared_html_export(
                RenderInput::default(),
                HtmlExportOptions::default(),
                "input.invalid_json",
                message,
            ));
        }
    };
    let options = match parse_optional_json::<HtmlExportOptions>(
        options_json.as_deref(),
        "HtmlExportOptions",
    ) {
        Ok(options) => options,
        Err(message) => {
            return serialize_json(&invalid_typed_prepared_html_export(
                input,
                HtmlExportOptions::default(),
                "options.invalid_json",
                message,
            ));
        }
    };

    serialize_json(&prepare_html_export(input, options))
}

#[uniffi::export]
pub fn render_prepared_html_export(
    prepared: PreparedHtmlExport,
    text_metrics: TextMeasurementResults,
) -> ExportHtmlOutput {
    oriha_core::catch_engine_panic(
        std::panic::AssertUnwindSafe(move || {
            render_prepared_html_export_inner(prepared, text_metrics)
        }),
        |message| invalid_export_html_output("bridge.panic", message),
    )
}

#[uniffi::export]
pub fn render_prepared_html_export_json(
    prepared_json: String,
    text_metrics_json: String,
) -> String {
    let prepared = match parse_json::<PreparedHtmlExport>(&prepared_json, "PreparedHtmlExport") {
        Ok(prepared) => prepared,
        Err(message) => {
            return serialize_json(&invalid_export_html_output(
                "prepared.invalid_json",
                message,
            ));
        }
    };
    let text_metrics = match parse_text_measurement_results_json(&text_metrics_json) {
        Ok(text_metrics) => text_metrics,
        Err(message) => {
            return serialize_json(&invalid_export_html_output(
                "text_metrics.invalid_json",
                message,
            ));
        }
    };

    serialize_json(&render_prepared_html_export(prepared, text_metrics))
}

#[uniffi::export]
pub fn native_bridge_version_info() -> NativeBridgeVersionInfo {
    NativeBridgeVersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_short_hash: option_env!("ORIHA_NATIVE_BRIDGE_GIT_SHORT_HASH")
            .unwrap_or("unknown")
            .to_string(),
    }
}

#[uniffi::export]
pub fn prepare_markdown_layout(input: RenderInput) -> PreparedMarkdownLayout {
    oriha_core::catch_engine_panic(
        std::panic::AssertUnwindSafe(move || prepare_markdown_layout_inner(input)),
        |message| invalid_prepared_markdown_layout("bridge.panic", message),
    )
}

#[uniffi::export]
pub fn prepare_markdown_layout_json(input_json: String) -> String {
    let input = match parse_json::<RenderInput>(&input_json, "RenderInput") {
        Ok(input) => input,
        Err(message) => {
            return serialize_json(&invalid_prepared_markdown_layout(
                "input.invalid_json",
                message,
            ));
        }
    };

    serialize_json(&prepare_markdown_layout(input))
}

fn prepare_markdown_layout_inner(input: RenderInput) -> PreparedMarkdownLayout {
    let BridgeInput {
        input,
        core_input,
        mut diagnostics,
        stats,
    } = apply_plan_policy(input);
    let prepared = oriha_core::prepare_markdown_layout(core_input);
    diagnostics.extend(prepared.diagnostics.into_iter().map(RenderDiagnostic::from));

    PreparedMarkdownLayout {
        version: PREPARED_MARKDOWN_LAYOUT_VERSION,
        input,
        text_measurements: prepared.text_measurements.into(),
        policy_diagnostics: diagnostics,
        policy_stats: stats,
    }
}

#[uniffi::export]
pub fn render_prepared_markdown(
    prepared: PreparedMarkdownLayout,
    text_metrics: TextMeasurementResults,
) -> RenderOutput {
    oriha_core::catch_engine_panic(
        std::panic::AssertUnwindSafe(move || {
            render_prepared_markdown_inner(prepared, text_metrics)
        }),
        |message| invalid_render_output("bridge.panic", message),
    )
}

#[uniffi::export]
pub fn render_prepared_markdown_json(prepared_json: String, text_metrics_json: String) -> String {
    let prepared =
        match parse_json::<PreparedMarkdownLayout>(&prepared_json, "PreparedMarkdownLayout") {
            Ok(prepared) => prepared,
            Err(message) => {
                return serialize_json(&invalid_render_output("prepared.invalid_json", message));
            }
        };
    let text_metrics = match parse_text_measurement_results_json(&text_metrics_json) {
        Ok(text_metrics) => text_metrics,
        Err(message) => {
            return serialize_json(&invalid_render_output("text_metrics.invalid_json", message));
        }
    };

    serialize_json(&render_prepared_markdown(prepared, text_metrics))
}

fn render_prepared_markdown_inner(
    prepared: PreparedMarkdownLayout,
    text_metrics: TextMeasurementResults,
) -> RenderOutput {
    let core_prepared = oriha_core::PreparedMarkdownLayout {
        version: prepared.version,
        input: input_to_core(prepared.input),
        text_measurements: prepared.text_measurements.into(),
        diagnostics: Vec::new(),
    };
    let mut output: RenderOutput = oriha_core::render_prepared_markdown(
        core_prepared,
        oriha_core::TextMeasurementResults::from(text_metrics),
    )
    .into();
    merge_render_stats(&mut output.stats, prepared.policy_stats);
    add_bridge_diagnostics(output, prepared.policy_diagnostics)
}

#[uniffi::export]
pub fn prepare_markdown_patch_json(
    previous_snapshot_json: Option<String>,
    input_json: String,
) -> String {
    let previous_snapshot = match parse_optional_json::<RenderSnapshot>(
        previous_snapshot_json.as_deref(),
        "RenderSnapshot",
    ) {
        Ok(previous_snapshot) => previous_snapshot,
        Err(message) => {
            return serialize_json(&invalid_typed_prepared_markdown_patch(
                None,
                RenderInput::default(),
                "previous_snapshot.invalid_json",
                message,
            ));
        }
    };
    let input = match parse_json::<RenderInput>(&input_json, "RenderInput") {
        Ok(input) => input,
        Err(message) => {
            return serialize_json(&invalid_typed_prepared_markdown_patch(
                previous_snapshot,
                RenderInput::default(),
                "input.invalid_json",
                message,
            ));
        }
    };

    serialize_json(&prepare_markdown_patch(previous_snapshot, input))
}

#[uniffi::export]
pub fn render_prepared_markdown_patch_json(
    prepared_json: String,
    text_metrics_json: String,
) -> String {
    let prepared =
        match parse_json::<PreparedMarkdownPatch>(&prepared_json, "PreparedMarkdownPatch") {
            Ok(prepared) => prepared,
            Err(message) => {
                return serialize_json(&invalid_render_patch_output(
                    "prepared.invalid_json",
                    message,
                    RenderPatchFallbackReason::InvalidInput,
                ));
            }
        };
    let text_metrics = match parse_text_measurement_results_json(&text_metrics_json) {
        Ok(text_metrics) => text_metrics,
        Err(message) => {
            return serialize_json(&invalid_render_patch_output(
                "text_metrics.invalid_json",
                message,
                RenderPatchFallbackReason::InvalidInput,
            ));
        }
    };

    serialize_json(&render_prepared_markdown_patch(prepared, text_metrics))
}

fn parse_json<T: DeserializeOwned>(json: &str, type_name: &str) -> Result<T, String> {
    serde_json::from_str(json)
        .map_err(|error| format!("{type_name} JSON could not be parsed: {error}"))
}

fn parse_optional_json<T: DeserializeOwned>(
    json: Option<&str>,
    type_name: &str,
) -> Result<Option<T>, String> {
    match json {
        Some(json) => parse_json(json, type_name).map(Some),
        None => Ok(None),
    }
}

fn parse_text_measurement_results_json(json: &str) -> Result<TextMeasurementResults, String> {
    let value: serde_json::Value = serde_json::from_str(json)
        .map_err(|error| format!("TextMeasurementResult JSON could not be parsed: {error}"))?;
    let results = if value.is_array() {
        serde_json::from_value::<Vec<TextMeasurementResult>>(value)
    } else {
        serde_json::from_value::<TextMeasurementResults>(value).map(|value| value.results)
    }
    .map_err(|error| format!("TextMeasurementResult JSON could not be parsed: {error}"))?;

    Ok(TextMeasurementResults { results })
}

fn serialize_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("native bridge JSON serialization should not fail")
}
