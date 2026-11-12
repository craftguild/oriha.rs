//! Stable measurement identities, styles and approximate glyph metrics.
use super::*;

pub fn text_measurement_id(line: &str) -> String {
    format!("plantuml.inlineText.{:016x}", fnv1a_64(line.as_bytes()))
}

pub fn inline_text_measurement_request(
    line: &str,
    base_style: &InlineTextMeasurementStyle,
) -> InlineTextMeasurementRequest {
    let spans = parse_inline_text_spans(line);
    inline_text_measurement_request_from_spans(line, &spans, base_style)
}

pub(super) fn inline_text_measurement_request_from_spans(
    line: &str,
    spans: &[InlineTextSpan],
    base_style: &InlineTextMeasurementStyle,
) -> InlineTextMeasurementRequest {
    let text = spans
        .iter()
        .map(|span| span.text.as_str())
        .collect::<String>();
    let mut id_material = String::new();
    push_measurement_style_key(&mut id_material, base_style);
    id_material.push('\n');
    id_material.push_str(line);

    let run_specs = spans
        .iter()
        .map(|span| {
            let style = inline_measurement_style(base_style, &span.style);
            id_material.push('\n');
            id_material.push_str(&span.text);
            id_material.push('\t');
            push_measurement_style_key(&mut id_material, &style);
            (span.text.clone(), style)
        })
        .collect::<Vec<_>>();
    let id = format!(
        "plantuml.inlineText.{:016x}",
        fnv1a_64(id_material.as_bytes())
    );
    let runs = run_specs
        .into_iter()
        .enumerate()
        .map(|(index, (text, style))| InlineTextMeasurementRun {
            id: format!("{id}.run.{index}"),
            text,
            style,
        })
        .collect::<Vec<_>>();

    InlineTextMeasurementRequest {
        id,
        source: line.to_owned(),
        text,
        style: base_style.clone(),
        runs,
    }
}

pub(super) fn inline_measurement_style(
    base_style: &InlineTextMeasurementStyle,
    style: &InlineTextStyle,
) -> InlineTextMeasurementStyle {
    let mut measurement_style = base_style.clone();
    if style.monospace {
        measurement_style.font_family = Some("monospace".to_owned());
    }
    if let Some(font_size) = style.font_size {
        measurement_style.font_size = Some(f32::from(font_size));
    }
    if style.bold {
        measurement_style.font_weight = Some("bold".to_owned());
    }
    if style.italic {
        measurement_style.font_style = Some("italic".to_owned());
    }
    measurement_style
}

pub(super) fn push_measurement_style_key(output: &mut String, style: &InlineTextMeasurementStyle) {
    output.push_str(style.font_family.as_deref().unwrap_or(""));
    output.push('|');
    if let Some(font_size) = style.font_size {
        let _ = std::fmt::Write::write_fmt(output, format_args!("{font_size:.3}"));
    }
    output.push('|');
    output.push_str(style.font_weight.as_deref().unwrap_or(""));
    output.push('|');
    output.push_str(style.font_style.as_deref().unwrap_or(""));
}

pub fn approximate_inline_text_measurement(
    request: &InlineTextMeasurementRequest,
) -> InlineTextMeasurementResult {
    let mut x = 0.0;
    let runs = request
        .runs
        .iter()
        .enumerate()
        .map(|(index, run)| {
            let width = measure_inline_measurement_run(run);
            let measurement = InlineTextRunMeasurement {
                id: run.id.clone(),
                index,
                x,
                width,
            };
            x += width;
            measurement
        })
        .collect::<Vec<_>>();

    InlineTextMeasurementResult {
        id: request.id.clone(),
        width: x,
        runs,
    }
}

pub(super) fn measure_inline_measurement_run(run: &InlineTextMeasurementRun) -> f32 {
    let font_size_scale = run.style.font_size.unwrap_or(DEFAULT_FONT_SIZE) / DEFAULT_FONT_SIZE;
    let weight_scale = if run.style.font_weight.as_deref() == Some("bold") {
        1.05
    } else {
        1.0
    };
    let text_width = if run.style.font_family.as_deref() == Some("monospace") {
        run.text.chars().map(|_| 8.4).sum::<f32>()
    } else {
        run.text.chars().map(approx_inline_char_width).sum()
    };
    text_width * font_size_scale * weight_scale
}

pub(super) fn measure_inline_span(span: &InlineTextSpan) -> f32 {
    let font_size_scale = inline_style_font_size(&span.style) / DEFAULT_FONT_SIZE;
    let weight_scale = if span.style.bold { 1.05 } else { 1.0 };
    let text_width = if span.style.monospace {
        span.text.chars().map(|_| 8.4).sum::<f32>()
    } else {
        span.text.chars().map(approx_inline_char_width).sum()
    };
    text_width * font_size_scale * weight_scale
}

pub(super) fn approx_inline_char_width(character: char) -> f32 {
    match character {
        '\u{0300}'..='\u{036f}' | '\u{1ab0}'..='\u{1aff}' | '\u{1dc0}'..='\u{1dff}' => 0.0,
        ' ' => 4.0,
        '\t' => 16.0,
        'i' | 'l' | 'I' | '!' | '|' | '.' | ',' | '\'' | ':' | ';' => 3.5,
        't' | 'f' | 'r' | 'j' => 5.0,
        'm' | 'w' => 10.5,
        'M' | 'W' => 12.0,
        'A'..='Z' => 8.8,
        '0'..='9' => 7.2,
        _ if is_wide_character(character) => 14.0,
        _ => 7.2,
    }
}

pub(super) fn is_wide_character(character: char) -> bool {
    matches!(
        character,
        '\u{1100}'..='\u{11ff}'
            | '\u{2e80}'..='\u{a4cf}'
            | '\u{ac00}'..='\u{d7af}'
            | '\u{f900}'..='\u{faff}'
            | '\u{fe10}'..='\u{fe1f}'
            | '\u{fe30}'..='\u{fe6f}'
            | '\u{ff00}'..='\u{ffef}'
            | '\u{1f300}'..='\u{1faff}'
    )
}

pub(super) fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
