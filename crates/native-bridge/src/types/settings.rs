// Typed settings controls, available options and resolved settings.

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct BridgeSettingsInput {
    pub locale: Option<String>,
    pub plan: Plan,
    pub rendering_options: RenderingOptions,
    pub mode: RenderMode,
    pub viewport: Option<Viewport>,
}

impl Default for BridgeSettingsInput {
    fn default() -> Self {
        let input = RenderInput::default();
        Self {
            locale: input.locale,
            plan: input.plan,
            rendering_options: input.rendering_options,
            mode: input.render_mode,
            viewport: input.viewport,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct BridgeResolvedSettings {
    pub resolved_locale: String,
    pub effective_input: BridgeEffectiveSettingsInput,
    pub controls: BridgeSettingsControls,
    pub available_options: BridgeAvailableOptions,
    pub features: Vec<BridgeFeature>,
    pub diagnostics: Vec<RenderDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct BridgeEffectiveSettingsInput {
    pub locale: String,
    pub plan: Plan,
    pub rendering_options: RenderingOptions,
    pub mode: RenderMode,
    pub viewport: Option<Viewport>,
}

impl BridgeEffectiveSettingsInput {
    pub(crate) fn from_input(input: RenderInput, locale: I18nLocale) -> Self {
        Self {
            locale: locale.code().to_string(),
            plan: input.plan,
            rendering_options: input.rendering_options,
            mode: input.render_mode,
            viewport: input.viewport,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct BridgeSettingsControls {
    pub plan: BridgeControlAvailability,
    pub color_scheme: BridgeControlAvailability,
    pub syntax_theme: BridgeControlAvailability,
    pub heading_font: BridgeControlAvailability,
    pub body_font: BridgeControlAvailability,
    pub code_font: BridgeControlAvailability,
    pub table_border_style: BridgeControlAvailability,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct BridgeControlAvailability {
    pub enabled: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct BridgeAvailableOptions {
    pub plan: Vec<BridgeOption>,
    pub color_scheme: Vec<BridgeOption>,
    pub syntax_theme: Vec<BridgeOption>,
    pub heading_font: Vec<BridgeFontOption>,
    pub body_font: Vec<BridgeFontOption>,
    pub code_font: Vec<BridgeFontOption>,
    pub table_border_style: Vec<BridgeOption>,
    pub locale: Vec<BridgeOption>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct BridgeOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub family: Option<String>,
    pub recommended: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct BridgeFontOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub family: Option<String>,
    pub recommended: bool,
    pub font_family: Option<FontFamilySpec>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
#[serde(rename_all = "camelCase")]
pub enum BridgeFeature {
    #[serde(rename = "gfm")]
    Gfm,
    #[serde(rename = "syntaxHighlight")]
    SyntaxHighlight,
    #[serde(rename = "darkMode")]
    DarkMode,
    #[serde(rename = "tables")]
    Tables,
    #[serde(rename = "taskLists")]
    TaskLists,
    #[serde(rename = "footnotes")]
    Footnotes,
    #[serde(rename = "rawHtml")]
    RawHtml,
    #[serde(rename = "pdfExport")]
    PdfExport,
    #[serde(rename = "callouts")]
    Callouts,
    #[serde(rename = "removePdfWatermark")]
    RemovePdfWatermark,
    #[serde(rename = "prioritySupport")]
    PrioritySupport,
    #[serde(rename = "plantuml")]
    Plantuml,
    #[serde(rename = "mermaid")]
    Mermaid,
    #[serde(rename = "math")]
    Math,
    #[serde(rename = "syntaxThemeCustomization")]
    SyntaxThemeCustomization,
    #[serde(rename = "fontCustomization")]
    FontCustomization,
}
