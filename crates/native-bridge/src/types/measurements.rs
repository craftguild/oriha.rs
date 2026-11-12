// Typed native text-measurement requests, runs and results.

#[derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct TextMeasurementBatch {
    pub requests: Vec<TextMeasurementRequest>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct TextMeasurementRequest {
    pub id: String,
    pub engine: String,
    pub kind: String,
    pub text: String,
    pub style: Option<TextMeasurementStyle>,
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<String>,
    pub font_style: Option<String>,
    pub runs: Option<Vec<TextMeasurementRun>>,
    pub width: Option<f32>,
    pub max_width: Option<f32>,
    pub wrap_mode: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct TextMeasurementStyle {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<String>,
    pub font_style: Option<String>,
    pub line_height: Option<f32>,
    pub letter_spacing: Option<f32>,
    pub text_transform: Option<String>,
    pub text_decoration_line: Option<String>,
    pub color: Option<String>,
    pub width: Option<f32>,
    pub max_width: Option<f32>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct TextMeasurementRun {
    pub id: Option<String>,
    pub text: String,
    pub style: Option<TextMeasurementStyle>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, uniffi::Record)]
#[serde(default, rename_all = "camelCase")]
pub struct TextMeasurementResults {
    pub results: Vec<TextMeasurementResult>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct TextMeasurementResult {
    pub id: String,
    pub width: f32,
    pub height: Option<f32>,
    pub line_count: Option<u32>,
    pub left: Option<f32>,
    pub right: Option<f32>,
    pub raw_width: Option<f32>,
    pub runs: Option<Vec<TextMeasurementRunResult>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct TextMeasurementRunResult {
    pub id: Option<String>,
    pub index: u32,
    pub width: f32,
    pub height: f32,
    pub rects: Vec<TextMeasurementRect>,
}

#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Record)]
#[serde(rename_all = "camelCase")]
pub struct TextMeasurementRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
