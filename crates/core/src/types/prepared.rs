use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedMarkdownLayout {
    #[serde(default = "prepared_markdown_layout_version")]
    pub version: u32,
    pub input: RenderInput,
    #[serde(default)]
    pub text_measurements: TextMeasurementBatch,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<RenderDiagnostic>,
}

impl PreparedMarkdownLayout {
    pub fn new(input: RenderInput, text_measurements: TextMeasurementBatch) -> Self {
        Self {
            version: prepared_markdown_layout_version(),
            input,
            text_measurements,
            diagnostics: Vec::new(),
        }
    }

    pub fn with_error(input: RenderInput, code: &str, message: String) -> Self {
        Self {
            version: prepared_markdown_layout_version(),
            input,
            text_measurements: TextMeasurementBatch::default(),
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedMarkdownPatch {
    #[serde(default = "prepared_markdown_patch_version")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_snapshot: Option<RenderSnapshot>,
    pub input: RenderInput,
    #[serde(default)]
    pub text_measurements: TextMeasurementBatch,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<RenderDiagnostic>,
}

impl PreparedMarkdownPatch {
    pub fn new(input: RenderPatchInput, text_measurements: TextMeasurementBatch) -> Self {
        Self {
            version: prepared_markdown_patch_version(),
            previous_snapshot: input.previous_snapshot,
            input: input.input,
            text_measurements,
            diagnostics: Vec::new(),
        }
    }

    pub fn with_error(input: RenderPatchInput, code: &str, message: String) -> Self {
        Self {
            version: prepared_markdown_patch_version(),
            previous_snapshot: input.previous_snapshot,
            input: input.input,
            text_measurements: TextMeasurementBatch::default(),
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedHtmlExport {
    #[serde(default = "prepared_html_export_version")]
    pub version: u32,
    pub input: ExportHtmlInput,
    #[serde(default)]
    pub text_measurements: TextMeasurementBatch,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<RenderDiagnostic>,
}

impl PreparedHtmlExport {
    pub fn new(input: ExportHtmlInput, text_measurements: TextMeasurementBatch) -> Self {
        Self {
            version: prepared_html_export_version(),
            input,
            text_measurements,
            diagnostics: Vec::new(),
        }
    }

    pub fn with_error(input: ExportHtmlInput, code: &str, message: String) -> Self {
        Self {
            version: prepared_html_export_version(),
            input,
            text_measurements: TextMeasurementBatch::default(),
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
}

const fn prepared_markdown_layout_version() -> u32 {
    3
}

const fn prepared_markdown_patch_version() -> u32 {
    1
}

const fn prepared_html_export_version() -> u32 {
    2
}
