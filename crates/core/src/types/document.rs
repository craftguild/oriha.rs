use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderInput {
    pub markdown: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(default)]
    pub appearance: AppearanceOptions,
    #[serde(default)]
    pub render_options: RenderOptions,
    #[serde(default)]
    pub mode: RenderMode,
    #[serde(default)]
    pub features: FeatureFlags,
    #[serde(default)]
    pub viewport: Option<Viewport>,
}

impl Default for RenderInput {
    fn default() -> Self {
        Self {
            markdown: String::new(),
            locale: None,
            appearance: AppearanceOptions::default(),
            render_options: RenderOptions::default(),
            mode: RenderMode::StyledHtml,
            features: FeatureFlags::default(),
            viewport: None,
        }
    }
}

#[cfg(test)]
impl RenderInput {
    pub(crate) fn default_for_test() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderOutput {
    pub html: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
    pub stats: RenderStats,
    pub diagnostics: Vec<RenderDiagnostic>,
    pub assets: Vec<RenderAsset>,
    pub source_map: Vec<RenderedBlock>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportHtmlInput {
    pub input: RenderInput,
    #[serde(default, deserialize_with = "default_html_export_options_on_null")]
    pub options: HtmlExportOptions,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HtmlExportOptions {
    #[serde(default = "default_true")]
    pub minify: bool,
    #[serde(default = "default_true")]
    pub obfuscate: bool,
    #[serde(default = "default_true")]
    pub preserve_heading_anchors: bool,
    #[serde(default = "default_true")]
    pub strip_oriha_data_attributes: bool,
}

impl Default for HtmlExportOptions {
    fn default() -> Self {
        Self {
            minify: true,
            obfuscate: true,
            preserve_heading_anchors: true,
            strip_oriha_data_attributes: true,
        }
    }
}

fn default_html_export_options_on_null<'de, D>(
    deserializer: D,
) -> Result<HtmlExportOptions, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<HtmlExportOptions>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportHtmlOutput {
    pub html: String,
    pub stats: RenderStats,
    pub diagnostics: Vec<RenderDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderDiagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_range: Option<SourceRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
    pub engine: DiagnosticEngine,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticEngine {
    Core,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SourceRange {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAsset {
    pub id: String,
    pub kind: AssetKind,
    pub mime_type: String,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetKind {
    Svg,
    Css,
    Other,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderedBlock {
    pub id: String,
    pub kind: String,
    pub source_range: SourceRange,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineLimits {
    pub max_source_bytes: usize,
    pub max_blocks: usize,
    pub max_nesting_depth: usize,
    pub max_output_bytes: usize,
}

impl Default for EngineLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 1_000_000,
            max_blocks: 20_000,
            max_nesting_depth: 256,
            max_output_bytes: 2_000_000,
        }
    }
}
