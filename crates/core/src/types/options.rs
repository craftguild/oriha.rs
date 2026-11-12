use super::*;

pub const DEFAULT_WATERMARK_TEXT: &str = "Generated with markup";
pub const DEFAULT_WATERMARK_URL: &str = "https://markup.progressiveworks.co";
pub const WATERMARK_URL_ENV_VAR: &str = "MARKUP_WATERMARK_URL";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceOptions {
    #[serde(default)]
    pub color_scheme: ColorScheme,
    #[serde(default = "default_appearance_style")]
    pub style: String,
    #[serde(default = "default_syntax_theme")]
    pub syntax_theme: String,
    #[serde(default = "FontFamilySpec::system")]
    pub heading_font_family: FontFamilySpec,
    #[serde(default = "FontFamilySpec::system")]
    pub body_font_family: FontFamilySpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_font_size_px: Option<f32>,
    #[serde(default = "FontFamilySpec::system_mono")]
    pub code_font_family: FontFamilySpec,
    #[serde(default)]
    pub table_border_style: TableBorderStyle,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub print: Option<PrintOptions>,
    #[serde(default = "default_true")]
    pub rounded_corners: bool,
    #[serde(default = "default_true")]
    pub shadows: bool,
    #[serde(default)]
    pub full_table_data: bool,
    #[serde(default)]
    pub block_borders: bool,
}

impl Default for AppearanceOptions {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::default(),
            style: default_appearance_style(),
            syntax_theme: default_syntax_theme(),
            heading_font_family: FontFamilySpec::system(),
            body_font_family: FontFamilySpec::system(),
            body_font_size_px: None,
            code_font_family: FontFamilySpec::system_mono(),
            table_border_style: TableBorderStyle::default(),
            print: None,
            rounded_corners: true,
            shadows: true,
            full_table_data: false,
            block_borders: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ColorScheme {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TableBorderStyle {
    None,
    #[default]
    Inner,
    All,
    Vertical,
    Horizontal,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FontFamilySpec {
    pub family: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallbacks: Vec<String>,
}

impl FontFamilySpec {
    pub fn from_css_stack(stack: impl Into<String>) -> Self {
        Self {
            family: stack.into(),
            fallbacks: Vec::new(),
        }
    }

    pub fn system() -> Self {
        Self {
            family: "-apple-system".to_string(),
            fallbacks: vec![
                "BlinkMacSystemFont".to_string(),
                "\"Segoe UI\"".to_string(),
                "sans-serif".to_string(),
            ],
        }
    }

    pub fn system_mono() -> Self {
        Self {
            family: "\"SFMono-Regular\"".to_string(),
            fallbacks: vec![
                "Consolas".to_string(),
                "\"Liberation Mono\"".to_string(),
                "monospace".to_string(),
            ],
        }
    }

    pub fn css_stack(&self) -> String {
        std::iter::once(self.family.as_str())
            .chain(self.fallbacks.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Default for FontFamilySpec {
    fn default() -> Self {
        Self::system()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrintOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WatermarkOptions {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_src: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
}

pub fn default_watermark_options() -> WatermarkOptions {
    let url = std::env::var(WATERMARK_URL_ENV_VAR)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_WATERMARK_URL.to_string());
    WatermarkOptions {
        text: DEFAULT_WATERMARK_TEXT.to_string(),
        image_src: Some(qr_svg_data_url(&url)),
        opacity: None,
    }
}

fn qr_svg_data_url(value: &str) -> String {
    let code =
        qrcode::QrCode::new(value.as_bytes()).expect("default watermark QR payload should fit");
    let svg = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(96, 96)
        .dark_color(qrcode::render::svg::Color("#111827"))
        .light_color(qrcode::render::svg::Color("transparent"))
        .build();
    format!(
        "data:image/svg+xml,{}",
        percent_encode_data_url_component(&svg)
    )
}

fn percent_encode_data_url_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b' ' | b'\n' => {
                encoded.push(char::from(byte))
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                encoded.push('%');
                encoded.push(char::from(HEX[usize::from(byte >> 4)]));
                encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
            }
        }
    }
    encoded.replace(' ', "%20").replace('\n', "")
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderOptions {
    #[serde(default, skip_serializing_if = "CodeBlockOptions::is_default")]
    pub code_block: CodeBlockOptions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watermark: Option<WatermarkOptions>,
    #[serde(default)]
    pub source_attributes: bool,
    #[serde(default)]
    pub table_of_contents: bool,
}

/// Optional code-block UI. Omitted fields retain the existing enabled output.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct CodeBlockOptions {
    pub show_language: bool,
    pub show_copy_button: bool,
    pub show_line_numbers: bool,
}

impl Default for CodeBlockOptions {
    fn default() -> Self {
        Self {
            show_language: true,
            show_copy_button: true,
            show_line_numbers: true,
        }
    }
}

impl CodeBlockOptions {
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

fn default_appearance_style() -> String {
    "rich".to_string()
}

fn default_syntax_theme() -> String {
    "github-light".to_string()
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderMode {
    #[default]
    StyledHtml,
    SemanticHtml,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureFlags {
    #[serde(default = "default_true")]
    pub gfm: bool,
    #[serde(default = "default_true")]
    pub callouts: bool,
    #[serde(default = "default_true")]
    pub syntax_highlight: bool,
    #[serde(default)]
    pub raw_html: bool,
    #[serde(default)]
    pub math: bool,
    #[serde(default)]
    pub mermaid: bool,
    #[serde(default)]
    pub plantuml: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            gfm: true,
            callouts: true,
            syntax_highlight: true,
            raw_html: false,
            math: false,
            mermaid: false,
            plantuml: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Viewport {
    pub width: u32,
    pub scale: f32,
}

#[cfg(test)]
#[path = "watermark_tests.rs"]
mod watermark_tests;
