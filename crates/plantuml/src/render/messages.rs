//! Sequence message lines, labels and self-message paths.
use super::*;

pub(super) fn message_stroke_color<'a>(
    message: &'a MessageLayout,
    theme: &'a PlantUmlTheme,
) -> Cow<'a, str> {
    message
        .arrow_features
        .color
        .as_deref()
        .map(svg_color_value)
        .unwrap_or_else(|| Cow::Borrowed(theme.message_color.as_str()))
}

pub(super) fn render_message(
    svg: &mut String,
    message: &MessageLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) {
    let y = message.y;
    let label_x = (message.start_x + message.end_x) / 2.0;
    let label_y = y - 8.0 - TEXT_LINE_HEIGHT * (text_line_count(&message.label) - 1) as f32;
    let class_attr = message_class_attr(message);
    let message_color = message_stroke_color(message, theme);
    let feature_attrs = message_feature_attrs(message);
    let line_dash = match message.line_style {
        LineStyle::Solid => String::new(),
        LineStyle::Dashed => format!(r#" stroke-dasharray="{}""#, style.stroke_width.dasharray()),
    };
    let stroke_width = style.stroke_width.svg_px();
    let start_side = message_start_arrow_side(message);
    let end_side = message_end_arrow_side(message);

    write!(
        svg,
        r#"<g class="{}" data-from="{}" data-to="{}" data-line-style="{}" data-arrow="{}" data-direction="{}" data-line="{}"{}>"#,
        class_attr,
        escape_attr(&message.from),
        escape_attr(&message.to),
        match message.line_style {
            LineStyle::Solid => "solid",
            LineStyle::Dashed => "dashed",
        },
        escape_attr(&message.arrow_raw),
        message_direction_name(message.direction),
        message.line,
        feature_attrs,
    )
    .expect("writing to String cannot fail");

    if matches!(message.direction, MessageDirection::SelfMessage) {
        render_text_element(
            svg,
            message.start_x + ARROW_DECORATION_GAP,
            label_y,
            "start",
            &theme.text_color,
            &message.label,
            measurer,
        );
        render_self_message_path(
            svg,
            message,
            message_color.as_ref(),
            &line_dash,
            stroke_width,
        );
        render_arrow_side(
            svg,
            end_side,
            message.start_x,
            message.y + SELF_MESSAGE_HEIGHT,
            -1.0,
            message_color.as_ref(),
            style,
        );
        render_arrow_side(
            svg,
            start_side,
            message.start_x,
            message.y,
            1.0,
            message_color.as_ref(),
            style,
        );
    } else {
        let line_direction = message_line_direction(message);
        let line_start_x =
            message.start_x + line_direction * arrow_side_line_inset(start_side, style);
        let line_end_x = message.end_x - line_direction * arrow_side_line_inset(end_side, style);
        let (line_start_x, line_end_x) =
            non_inverted_message_line(message.start_x, message.end_x, line_start_x, line_end_x);

        render_text_element(
            svg,
            label_x,
            label_y,
            "middle",
            &theme.text_color,
            &message.label,
            measurer,
        );
        write!(
            svg,
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{stroke_width}"{} />"#,
            format_number(line_start_x),
            format_number(y),
            format_number(line_end_x),
            format_number(y),
            escape_attr(message_color.as_ref()),
            line_dash,
        )
        .expect("writing to String cannot fail");
        render_message_arrow_sides(
            svg,
            message,
            start_side,
            end_side,
            message_color.as_ref(),
            style,
        );
    }

    render_message_feature_markers(svg, message, theme, style);
    svg.push_str("</g>");
}

pub(super) fn render_self_message_path(
    svg: &mut String,
    message: &MessageLayout,
    message_color: &str,
    line_dash: &str,
    stroke_width: u16,
) {
    let x = message.start_x;
    let y = message.y;
    write!(
        svg,
        r#"<path class="message-line self-message-line" d="M {} {} H {} V {} H {}" fill="none" stroke="{}" stroke-width="{stroke_width}"{} />"#,
        format_number(x),
        format_number(y),
        format_number(x + SELF_MESSAGE_WIDTH),
        format_number(y + SELF_MESSAGE_HEIGHT),
        format_number(x),
        escape_attr(message_color),
        line_dash,
    )
    .expect("writing to String cannot fail");
}

pub(super) fn non_inverted_message_line(
    original_start_x: f32,
    original_end_x: f32,
    line_start_x: f32,
    line_end_x: f32,
) -> (f32, f32) {
    let original_direction = if original_end_x >= original_start_x {
        1.0
    } else {
        -1.0
    };
    if (line_end_x - line_start_x) * original_direction > 4.0 {
        (line_start_x, line_end_x)
    } else {
        (original_start_x, original_end_x)
    }
}
