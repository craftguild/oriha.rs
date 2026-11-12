use std::collections::BTreeMap;

use crate::{
    finite_non_negative_f32, finite_positive_f32, fnv1a_64, MermaidMeasurementKind,
    MermaidMeasurementRequest, MermaidMeasurementResult, MermaidMeasurementStyle, MermaidTheme,
    MermaidWrapMode,
};

pub const NODE_MAX_WIDTH: f32 = 220.0;
pub const STATE_MAX_WIDTH: f32 = 190.0;

#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredText {
    pub text: String,
    pub lines: Vec<String>,
    pub width: f32,
    pub height: f32,
    pub line_count: usize,
    pub font_size: f32,
    pub font_weight: Option<String>,
}

pub struct TextResolver<'a> {
    theme: &'a MermaidTheme,
    measurements: &'a BTreeMap<String, MermaidMeasurementResult>,
}

impl<'a> TextResolver<'a> {
    pub fn new(
        theme: &'a MermaidTheme,
        measurements: &'a BTreeMap<String, MermaidMeasurementResult>,
    ) -> Self {
        Self {
            theme,
            measurements,
        }
    }

    pub fn measure_text(&self, text: &str, font_weight: Option<&str>) -> MeasuredText {
        self.measure(text, MermaidMeasurementKind::Text, None, font_weight)
    }

    pub fn measure_wrapped(
        &self,
        text: &str,
        max_width: f32,
        font_weight: Option<&str>,
    ) -> MeasuredText {
        self.measure(
            text,
            MermaidMeasurementKind::Wrapped,
            Some(max_width),
            font_weight,
        )
    }

    fn measure(
        &self,
        text: &str,
        kind: MermaidMeasurementKind,
        max_width: Option<f32>,
        font_weight: Option<&str>,
    ) -> MeasuredText {
        let request = measurement_request(self.theme, text, kind, max_width, font_weight);
        let font_size = request.style.font_size;
        let font_weight = request.style.font_weight.clone();
        let lines = match max_width {
            Some(max_width) => wrap_lines(text, max_width, font_size),
            None => split_lines(text),
        };
        let fallback = approximate_measurement(&lines, font_size);

        let (width, height, line_count) = if let Some(result) = self.measurements.get(&request.id) {
            (
                finite_non_negative_f32(result.width, fallback.0),
                finite_non_negative_f32(result.height, fallback.1),
                result.line_count.max(1),
            )
        } else {
            (fallback.0, fallback.1, lines.len().max(1))
        };

        MeasuredText {
            text: text.to_owned(),
            lines,
            width,
            height,
            line_count,
            font_size,
            font_weight,
        }
    }
}

pub fn measurement_request(
    theme: &MermaidTheme,
    text: &str,
    kind: MermaidMeasurementKind,
    max_width: Option<f32>,
    font_weight: Option<&str>,
) -> MermaidMeasurementRequest {
    let style = MermaidMeasurementStyle {
        font_family: Some(normalize_font_family_stack(&theme.font_family)),
        font_size: finite_positive_f32(theme.font_size, 16.0),
        font_weight: font_weight.map(str::to_owned),
    };
    let wrap_mode = match kind {
        MermaidMeasurementKind::Wrapped
        | MermaidMeasurementKind::WrappedRaw
        | MermaidMeasurementKind::WrappedWithRawWidth => Some(MermaidWrapMode::SvgLike),
        _ => None,
    };
    let id = mermaid_measurement_id(kind, text, &style, None, max_width, wrap_mode);
    MermaidMeasurementRequest {
        id,
        kind,
        text: text.to_owned(),
        style,
        width: None,
        max_width,
        wrap_mode,
    }
}

fn mermaid_measurement_id(
    kind: MermaidMeasurementKind,
    text: &str,
    style: &MermaidMeasurementStyle,
    width: Option<f32>,
    max_width: Option<f32>,
    wrap_mode: Option<MermaidWrapMode>,
) -> String {
    let mut bytes = Vec::new();
    push_hash_str(&mut bytes, kind.as_str());
    push_hash_str(&mut bytes, text);
    push_hash_optional_str(&mut bytes, style.font_family.as_deref());
    push_hash_f32(&mut bytes, style.font_size);
    push_hash_optional_str(&mut bytes, style.font_weight.as_deref());
    push_hash_optional_f32(&mut bytes, width);
    push_hash_optional_f32(&mut bytes, max_width);
    push_hash_optional_str(&mut bytes, wrap_mode.map(MermaidWrapMode::as_str));
    format!("mermaid.measurement.{}.{:016x}", kind, fnv1a_64(&bytes))
}

fn push_hash_str(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(value.as_bytes());
    bytes.push(0);
}

fn push_hash_optional_str(bytes: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(value) => {
            bytes.push(1);
            push_hash_str(bytes, value);
        }
        None => bytes.push(0),
    }
}

fn push_hash_f32(bytes: &mut Vec<u8>, value: f32) {
    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
}

fn push_hash_optional_f32(bytes: &mut Vec<u8>, value: Option<f32>) {
    match value {
        Some(value) => {
            bytes.push(1);
            push_hash_f32(bytes, value);
        }
        None => bytes.push(0),
    }
}

fn split_lines(text: &str) -> Vec<String> {
    let lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

fn wrap_lines(text: &str, max_width: f32, font_size: f32) -> Vec<String> {
    let max_units = (max_width / (font_size * 0.58)).max(8.0);
    let mut lines = Vec::new();
    for source_line in split_lines(text) {
        let mut current = String::new();
        let mut current_units = 0.0;
        for word in source_line.split_whitespace() {
            let word_units = approximate_text_units(word);
            let separator_units = if current.is_empty() { 0.0 } else { 1.0 };
            if !current.is_empty() && current_units + separator_units + word_units > max_units {
                lines.push(current);
                current = String::new();
                current_units = 0.0;
            }
            if !current.is_empty() {
                current.push(' ');
                current_units += separator_units;
            }
            current.push_str(word);
            current_units += word_units;
        }
        if current.is_empty() {
            lines.push(source_line);
        } else {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

fn approximate_measurement(lines: &[String], font_size: f32) -> (f32, f32) {
    let width = lines
        .iter()
        .map(|line| approximate_text_width(line, font_size))
        .fold(0.0, f32::max)
        .max(font_size * 2.0);
    let height = (lines.len().max(1) as f32) * font_size * 1.35;
    (width, height)
}

fn approximate_text_width(text: &str, font_size: f32) -> f32 {
    approximate_text_units(text) * font_size * 0.58
}

fn approximate_text_units(text: &str) -> f32 {
    text.chars()
        .map(|ch| {
            if ch.is_ascii_whitespace() {
                0.45
            } else if ch.is_ascii_punctuation() {
                0.55
            } else if ch.is_ascii() {
                1.0
            } else {
                1.75
            }
        })
        .sum()
}

fn normalize_font_family_stack(value: &str) -> String {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(",")
}
