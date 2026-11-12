//! Native render records and ABI-stable shared type namespace.
// UniFFI includes module_path!() in type metadata. These responsibility files
// share this namespace to preserve existing Swift/Kotlin ABI checksums.
include!("types/measurements.rs");
include!("types/analytics.rs");
include!("types/settings.rs");
include!("types/options.rs");

use std::collections::HashMap;

use oriha_i18n::I18nLocale;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct RenderInput {
    pub markdown: String,
    pub locale: Option<String>,
    pub plan: Plan,
    pub rendering_options: RenderingOptions,
    pub print: Option<PrintOptions>,
    pub render_mode: RenderMode,
    pub watermark: Option<WatermarkOptions>,
    pub source_attributes: bool,
    pub table_of_contents: bool,
    pub viewport: Option<Viewport>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct RenderingOptions {
    pub color_scheme: ColorScheme,
    pub appearance_style: AppearanceStyle,
    pub syntax_theme: SyntaxTheme,
    pub custom_syntax_theme: Option<String>,
    pub heading_font: HeadingFont,
    pub custom_heading_font: Option<FontFamilySpec>,
    pub body_font: BodyFont,
    pub custom_body_font: Option<FontFamilySpec>,
    pub body_font_size_px: Option<f32>,
    pub code_font: CodeFont,
    pub custom_code_font: Option<FontFamilySpec>,
    pub table_border_style: TableBorderStyle,
    pub rounded_corners: bool,
    pub shadows: bool,
    pub full_table_data: bool,
    pub block_borders: bool,
}

impl Default for RenderInput {
    fn default() -> Self {
        Self {
            markdown: String::new(),
            locale: None,
            plan: Plan::Free,
            rendering_options: RenderingOptions::default(),
            print: None,
            render_mode: RenderMode::StyledHtml,
            watermark: None,
            source_attributes: false,
            table_of_contents: false,
            viewport: None,
        }
    }
}

impl Default for RenderingOptions {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Light,
            appearance_style: AppearanceStyle::Rich,
            syntax_theme: SyntaxTheme::GitHubLight,
            custom_syntax_theme: None,
            heading_font: HeadingFont::SystemDefault,
            custom_heading_font: None,
            body_font: BodyFont::SystemDefault,
            custom_body_font: None,
            body_font_size_px: None,
            code_font: CodeFont::SystemDefault,
            custom_code_font: None,
            table_border_style: TableBorderStyle::Inner,
            rounded_corners: true,
            shadows: true,
            full_table_data: false,
            block_borders: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct RenderOutput {
    pub html: String,
    pub css: Option<String>,
    pub stats: RenderStats,
    pub diagnostics: Vec<RenderDiagnostic>,
    pub assets: Vec<RenderAsset>,
    pub source_map: Vec<RenderedBlock>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct ExportHtmlOutput {
    pub html: String,
    pub stats: RenderStats,
    pub diagnostics: Vec<RenderDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct HtmlExportOptions {
    pub minify: bool,
    pub obfuscate: bool,
    pub preserve_heading_anchors: bool,
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

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct PreparedMarkdownLayout {
    pub version: u32,
    pub input: RenderInput,
    pub text_measurements: TextMeasurementBatch,
    pub policy_diagnostics: Vec<RenderDiagnostic>,
    #[serde(default)]
    pub policy_stats: RenderStats,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct PreparedMarkdownPatch {
    pub version: u32,
    pub previous_snapshot: Option<RenderSnapshot>,
    pub input: RenderInput,
    pub text_measurements: TextMeasurementBatch,
    pub policy_diagnostics: Vec<RenderDiagnostic>,
    #[serde(default)]
    pub policy_stats: RenderStats,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct PreparedHtmlExport {
    pub version: u32,
    pub input: RenderInput,
    pub options: HtmlExportOptions,
    pub text_measurements: TextMeasurementBatch,
    pub policy_diagnostics: Vec<RenderDiagnostic>,
    #[serde(default)]
    pub policy_stats: RenderStats,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct NativeBridgeVersionInfo {
    pub version: String,
    pub git_short_hash: String,
}

impl From<BridgeSettingsInput> for RenderInput {
    fn from(value: BridgeSettingsInput) -> Self {
        Self {
            markdown: String::new(),
            locale: value.locale,
            plan: value.plan,
            rendering_options: value.rendering_options,
            print: None,
            render_mode: value.mode,
            watermark: None,
            source_attributes: false,
            table_of_contents: false,
            viewport: value.viewport,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct WatermarkOptions {
    pub text: String,
    pub image_src: Option<String>,
    pub opacity: Option<f32>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct PrintOptions {
    pub page_size: Option<String>,
    pub margin: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct FontFamilySpec {
    pub family: String,
    #[serde(default)]
    pub fallbacks: Vec<String>,
}

impl FontFamilySpec {
    pub(crate) fn system() -> Self {
        Self {
            family: "-apple-system".to_string(),
            fallbacks: vec![
                "BlinkMacSystemFont".to_string(),
                "\"Segoe UI\"".to_string(),
                "Helvetica".to_string(),
                "Arial".to_string(),
                "sans-serif".to_string(),
            ],
        }
    }

    pub(crate) fn system_mono() -> Self {
        Self {
            family: "\"SFMono-Regular\"".to_string(),
            fallbacks: vec![
                "Consolas".to_string(),
                "\"Liberation Mono\"".to_string(),
                "Menlo".to_string(),
                "Monaco".to_string(),
                "\"Courier New\"".to_string(),
                "monospace".to_string(),
            ],
        }
    }
}

impl Default for FontFamilySpec {
    fn default() -> Self {
        Self::system()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct Viewport {
    pub width: u32,
    pub scale: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct RenderStats {
    #[serde(default = "render_stats_schema_version")]
    pub schema_version: u32,
    pub total_blocks: u32,
    pub blocks_by_kind: Vec<KindCount>,
    pub headings_by_level: Vec<KindCount>,
    pub code_blocks_by_language_kind: Vec<KindCount>,
    pub callouts_by_kind: Vec<KindCount>,
    pub locked_features_by_kind: Vec<KindCount>,
    pub unsupported_by_kind: Vec<KindCount>,
    pub issues_by_kind: Vec<KindCount>,
}

pub(crate) const fn render_stats_schema_version() -> u32 {
    1
}

impl Default for RenderStats {
    fn default() -> Self {
        Self {
            schema_version: 1,
            total_blocks: 0,
            blocks_by_kind: Vec::new(),
            headings_by_level: Vec::new(),
            code_blocks_by_language_kind: Vec::new(),
            callouts_by_kind: Vec::new(),
            locked_features_by_kind: Vec::new(),
            unsupported_by_kind: Vec::new(),
            issues_by_kind: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct KindCount {
    pub kind: String,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct RenderDiagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub source_range: Option<SourceRange>,
    pub block_id: Option<String>,
    pub engine: DiagnosticEngine,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct RenderAsset {
    pub id: String,
    pub kind: AssetKind,
    pub mime_type: String,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct RenderedBlock {
    pub id: String,
    pub kind: String,
    pub source_range: SourceRange,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct SourceRange {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct RenderSnapshot {
    pub schema_version: u32,
    pub settings_fingerprint: String,
    pub next_stable_index: u64,
    pub blocks: Vec<RenderSnapshotBlock>,
    pub tail_hash: String,
    pub tail_html_byte_length: u64,
    pub output_resource_fingerprint: String,
}

impl Default for RenderSnapshot {
    fn default() -> Self {
        Self {
            schema_version: oriha_core::render_snapshot_schema_version(),
            settings_fingerprint: String::new(),
            next_stable_index: 1,
            blocks: Vec::new(),
            tail_hash: String::new(),
            tail_html_byte_length: 0,
            output_resource_fingerprint: String::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct RenderSnapshotBlock {
    pub stable_id: String,
    pub kind: String,
    pub source_hash: String,
    pub html_hash: String,
    pub source_range: SourceRange,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
#[serde(rename_all = "camelCase")]
pub enum RenderPatchStrategy {
    Incremental,
    ReplaceAll,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
#[serde(rename_all = "camelCase")]
pub enum RenderPatchFallbackReason {
    MissingSnapshot,
    IncompatibleSnapshot,
    TailChanged,
    OutputTruncated,
    InvalidInput,
    EnginePanic,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
#[serde(rename_all = "camelCase")]
pub enum RenderPatchOperationKind {
    ReplaceAll,
    ReplaceBlocks,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct RenderPatchOperation {
    pub op: RenderPatchOperationKind,
    pub html: String,
    pub remove_stable_ids: Vec<String>,
    pub insert_after_stable_id: Option<String>,
    pub insert_before_stable_id: Option<String>,
}

impl Default for RenderPatchOperation {
    fn default() -> Self {
        Self {
            op: RenderPatchOperationKind::ReplaceAll,
            html: String::new(),
            remove_stable_ids: Vec::new(),
            insert_after_stable_id: None,
            insert_before_stable_id: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct RenderPatchOutput {
    pub output: RenderOutput,
    pub snapshot: RenderSnapshot,
    pub strategy: RenderPatchStrategy,
    pub fallback_reason: Option<RenderPatchFallbackReason>,
    pub patches: Vec<RenderPatchOperation>,
}
