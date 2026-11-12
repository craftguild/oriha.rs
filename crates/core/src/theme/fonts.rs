//! Font-stack validation and proportional font-size variables.
use super::*;

pub(super) fn sanitize_font_stack(value: &str, fallback: &str) -> String {
    sanitize_css_value(value, fallback)
}

pub(super) fn sanitize_css_value(value: &str, fallback: &str) -> String {
    if is_safe_css_value(value) {
        value.to_string()
    } else {
        fallback.to_string()
    }
}

pub(super) fn is_safe_css_value(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| !ch.is_control() && !matches!(ch, ';' | '{' | '}' | '<' | '>'))
}

pub(crate) fn resolve_body_font_size_px(value: Option<f32>) -> f32 {
    value
        .filter(|value| value.is_finite())
        .unwrap_or(DEFAULT_BODY_FONT_SIZE_PX)
        .clamp(MIN_BODY_FONT_SIZE_PX, MAX_BODY_FONT_SIZE_PX)
}

pub(super) fn insert_font_size_variables(
    vars: &mut BTreeMap<String, String>,
    options: &AppearanceOptions,
) {
    let body_font_size = resolve_body_font_size_px(options.body_font_size_px);
    vars.insert(
        "--oriha-font-size-body".to_string(),
        format_css_px(body_font_size),
    );
    vars.insert(
        "--oriha-font-size-code-label".to_string(),
        format!("calc(var(--oriha-font-size-body) * {CODE_LABEL_FONT_SIZE_RATIO})"),
    );
    vars.insert(
        "--oriha-font-size-code-block".to_string(),
        format!("calc(var(--oriha-font-size-body) * {CODE_BLOCK_FONT_SIZE_RATIO})"),
    );
    vars.insert(
        "--oriha-font-size-small".to_string(),
        format!("calc(var(--oriha-font-size-body) * {CODE_LABEL_FONT_SIZE_RATIO})"),
    );
    vars.insert(
        "--oriha-font-size-watermark".to_string(),
        format!("calc(var(--oriha-font-size-body) * {WATERMARK_FONT_SIZE_RATIO})"),
    );
}

pub(super) fn format_css_px(value: f32) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    if rounded.fract().abs() < f32::EPSILON {
        format!("{rounded:.0}px")
    } else {
        let trimmed = format!("{rounded:.3}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
        format!("{trimmed}px")
    }
}
