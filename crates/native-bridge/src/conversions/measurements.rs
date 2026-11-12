//! Lossless bridge/core measurement conversions with finite-number normalization.
use super::*;

impl From<TextMeasurementBatch> for oriha_core::TextMeasurementBatch {
    fn from(value: TextMeasurementBatch) -> Self {
        Self {
            requests: value.requests.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<oriha_core::TextMeasurementBatch> for TextMeasurementBatch {
    fn from(value: oriha_core::TextMeasurementBatch) -> Self {
        Self {
            requests: value.requests.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TextMeasurementRequest> for oriha_core::TextMeasurementRequest {
    fn from(value: TextMeasurementRequest) -> Self {
        Self {
            id: value.id,
            engine: value.engine,
            kind: value.kind,
            text: value.text,
            style: value.style.map(Into::into),
            font_family: value.font_family,
            font_size: value.font_size.and_then(non_negative_f32),
            font_weight: value.font_weight,
            font_style: value.font_style,
            runs: value
                .runs
                .map(|runs| runs.into_iter().map(Into::into).collect()),
            width: value.width.and_then(non_negative_f32),
            max_width: value.max_width.and_then(non_negative_f32),
            wrap_mode: value.wrap_mode,
        }
    }
}

impl From<oriha_core::TextMeasurementRequest> for TextMeasurementRequest {
    fn from(value: oriha_core::TextMeasurementRequest) -> Self {
        Self {
            id: value.id,
            engine: value.engine,
            kind: value.kind,
            text: value.text,
            style: value.style.map(Into::into),
            font_family: value.font_family,
            font_size: value.font_size.and_then(non_negative_f32),
            font_weight: value.font_weight,
            font_style: value.font_style,
            runs: value
                .runs
                .map(|runs| runs.into_iter().map(Into::into).collect()),
            width: value.width.and_then(non_negative_f32),
            max_width: value.max_width.and_then(non_negative_f32),
            wrap_mode: value.wrap_mode,
        }
    }
}

impl From<TextMeasurementStyle> for oriha_core::TextMeasurementStyle {
    fn from(value: TextMeasurementStyle) -> Self {
        Self {
            font_family: value.font_family,
            font_size: value.font_size.and_then(non_negative_f32),
            font_weight: value.font_weight,
            font_style: value.font_style,
            line_height: value.line_height.and_then(non_negative_f32),
            letter_spacing: value.letter_spacing.and_then(non_negative_f32),
            text_transform: value.text_transform,
            text_decoration_line: value.text_decoration_line,
            color: value.color,
            width: value.width.and_then(non_negative_f32),
            max_width: value.max_width.and_then(non_negative_f32),
        }
    }
}

impl From<oriha_core::TextMeasurementStyle> for TextMeasurementStyle {
    fn from(value: oriha_core::TextMeasurementStyle) -> Self {
        Self {
            font_family: value.font_family,
            font_size: value.font_size.and_then(non_negative_f32),
            font_weight: value.font_weight,
            font_style: value.font_style,
            line_height: value.line_height.and_then(non_negative_f32),
            letter_spacing: value.letter_spacing.and_then(non_negative_f32),
            text_transform: value.text_transform,
            text_decoration_line: value.text_decoration_line,
            color: value.color,
            width: value.width.and_then(non_negative_f32),
            max_width: value.max_width.and_then(non_negative_f32),
        }
    }
}

impl From<TextMeasurementRun> for oriha_core::TextMeasurementRun {
    fn from(value: TextMeasurementRun) -> Self {
        Self {
            id: value.id,
            text: value.text,
            style: value.style.map(Into::into),
        }
    }
}

impl From<oriha_core::TextMeasurementRun> for TextMeasurementRun {
    fn from(value: oriha_core::TextMeasurementRun) -> Self {
        Self {
            id: value.id,
            text: value.text,
            style: value.style.map(Into::into),
        }
    }
}

impl From<TextMeasurementResults> for oriha_core::TextMeasurementResults {
    fn from(value: TextMeasurementResults) -> Self {
        Self {
            results: value.results.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<oriha_core::TextMeasurementResults> for TextMeasurementResults {
    fn from(value: oriha_core::TextMeasurementResults) -> Self {
        Self {
            results: value.results.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TextMeasurementResult> for oriha_core::TextMeasurementResult {
    fn from(value: TextMeasurementResult) -> Self {
        Self {
            id: value.id,
            width: clamp_non_negative_f32(value.width),
            height: value.height.map(clamp_non_negative_f32),
            line_count: value.line_count.map(|count| count as usize),
            left: value.left.map(clamp_non_negative_f32),
            right: value.right.map(clamp_non_negative_f32),
            raw_width: value.raw_width.map(clamp_non_negative_f32),
            runs: value
                .runs
                .map(|runs| runs.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<oriha_core::TextMeasurementResult> for TextMeasurementResult {
    fn from(value: oriha_core::TextMeasurementResult) -> Self {
        Self {
            id: value.id,
            width: clamp_non_negative_f32(value.width),
            height: value.height.map(clamp_non_negative_f32),
            line_count: value.line_count.map(|count| count as u32),
            left: value.left.map(clamp_non_negative_f32),
            right: value.right.map(clamp_non_negative_f32),
            raw_width: value.raw_width.map(clamp_non_negative_f32),
            runs: value
                .runs
                .map(|runs| runs.into_iter().map(Into::into).collect()),
        }
    }
}

impl From<TextMeasurementRunResult> for oriha_core::TextMeasurementRunResult {
    fn from(value: TextMeasurementRunResult) -> Self {
        Self {
            id: value.id,
            index: value.index as usize,
            width: clamp_non_negative_f32(value.width),
            height: clamp_non_negative_f32(value.height),
            rects: value.rects.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<oriha_core::TextMeasurementRunResult> for TextMeasurementRunResult {
    fn from(value: oriha_core::TextMeasurementRunResult) -> Self {
        Self {
            id: value.id,
            index: value.index as u32,
            width: clamp_non_negative_f32(value.width),
            height: clamp_non_negative_f32(value.height),
            rects: value.rects.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TextMeasurementRect> for oriha_core::TextMeasurementRect {
    fn from(value: TextMeasurementRect) -> Self {
        Self {
            x: finite_f32_or_zero(value.x),
            y: finite_f32_or_zero(value.y),
            width: clamp_non_negative_f32(value.width),
            height: clamp_non_negative_f32(value.height),
        }
    }
}

impl From<oriha_core::TextMeasurementRect> for TextMeasurementRect {
    fn from(value: oriha_core::TextMeasurementRect) -> Self {
        Self {
            x: finite_f32_or_zero(value.x),
            y: finite_f32_or_zero(value.y),
            width: clamp_non_negative_f32(value.width),
            height: clamp_non_negative_f32(value.height),
        }
    }
}
