//! Groups, dividers, titles, delays and participant boxes.
use super::*;

pub(super) fn render_group_start(
    _svg: &mut String,
    _layout: &SequenceLayout,
    _group: &GroupStartLayout,
    _theme: &PlantUmlTheme,
    _style: &PlantUmlStyle,
) {
}

pub(super) fn render_group_else(
    svg: &mut String,
    layout: &SequenceLayout,
    group_else: &GroupElseLayout,
    theme: &PlantUmlTheme,
    measurer: &dyn TextMeasurer,
) {
    let depth = group_depth_at_line(layout, group_else.line);
    let (x, width) = group_frame_x_width_containing_line(layout, group_else.line, measurer);
    let y = group_else.y;
    let label = group_else_label(group_else);

    write!(
        svg,
        r#"<g class="sequence-event group-else" data-event="group-else" data-line="{}" data-depth="{}"><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-dasharray="4 4"/>"#,
        group_else.line,
        depth,
        format_number(x),
        format_number(y),
        format_number(x + width),
        format_number(y),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
    render_text_element(
        svg,
        x + 10.0,
        y + 14.0,
        "start",
        &theme.text_color,
        &label,
        measurer,
    );
    svg.push_str("</g>");
}

pub(super) fn group_else_label(group_else: &GroupElseLayout) -> String {
    match &group_else.label {
        Some(label) => format!("[{label}]"),
        None => "else".to_owned(),
    }
}

pub(super) fn render_group_end(
    _svg: &mut String,
    _layout: &SequenceLayout,
    _group_end: &GroupEndLayout,
    _theme: &PlantUmlTheme,
) {
}

pub(super) fn render_divider(
    svg: &mut String,
    layout: &SequenceLayout,
    divider: &TextEventLayout,
    theme: &PlantUmlTheme,
    measurer: &dyn TextMeasurer,
) {
    let (x, width) = event_frame_x_width(layout);
    write!(
        svg,
        r#"<g class="sequence-event divider" data-event="divider" data-line="{}"><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/>"#,
        divider.line,
        format_number(x),
        format_number(divider.y),
        format_number(x + width),
        format_number(divider.y),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
    render_text_element(
        svg,
        x + width / 2.0,
        divider.y - 8.0 - TEXT_LINE_HEIGHT * (text_line_count(&divider.text) - 1) as f32,
        "middle",
        &theme.text_color,
        &divider.text,
        measurer,
    );
    svg.push_str("</g>");
}

pub(super) fn render_title(
    svg: &mut String,
    layout: &SequenceLayout,
    title: &TextEventLayout,
    theme: &PlantUmlTheme,
    _style: &PlantUmlStyle,
) {
    render_page_title(svg, layout, title, theme);
}

pub(super) fn render_delay(
    svg: &mut String,
    layout: &SequenceLayout,
    delay: &DelayLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) {
    let (x, width) = event_frame_x_width(layout);
    write!(
        svg,
        r#"<g class="sequence-event delay" data-event="delay" data-line="{}"><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-dasharray="{}"/>"#,
        delay.line,
        format_number(x),
        format_number(delay.y),
        format_number(x + width),
        format_number(delay.y),
        escape_attr(&theme.participant_stroke),
        style.stroke_width.dasharray(),
    )
    .expect("writing to String cannot fail");
    if let Some(text) = &delay.text {
        render_text_element(
            svg,
            x + width / 2.0,
            delay.y - 8.0 - TEXT_LINE_HEIGHT * (text_line_count(text) - 1) as f32,
            "middle",
            &theme.text_color,
            text,
            measurer,
        );
    }
    svg.push_str("</g>");
}

pub(super) fn render_box_start(
    svg: &mut String,
    layout: &SequenceLayout,
    box_start: &BoxStartLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) {
    let (x, width) = event_frame_x_width(layout);
    let y = box_start.y - 22.0;
    let title_line_count = box_start.title.as_deref().map(text_line_count).unwrap_or(1);
    let height = 34.0 + TEXT_LINE_HEIGHT * (title_line_count - 1) as f32;
    let fill = box_start
        .color
        .as_deref()
        .map(svg_color_value)
        .unwrap_or_else(|| Cow::Borrowed(theme.background_color.as_str()));
    let corner_radius = style
        .corner_radius
        .svg_px()
        .map(|px| format!(r#" rx="{px}" ry="{px}""#))
        .unwrap_or_default();

    write!(
        svg,
        r#"<g class="sequence-event box-start" data-event="box-start" data-line="{}"><rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}"{}/>"#,
        box_start.line,
        format_number(x),
        format_number(y),
        format_number(width),
        format_number(height),
        escape_attr(fill.as_ref()),
        escape_attr(&theme.participant_stroke),
        corner_radius,
    )
    .expect("writing to String cannot fail");
    if let Some(title) = &box_start.title {
        render_text_element(
            svg,
            x + 10.0,
            box_start.y - 4.0,
            "start",
            &theme.text_color,
            title,
            measurer,
        );
    }
    svg.push_str("</g>");
}

pub(super) fn render_box_end(
    svg: &mut String,
    layout: &SequenceLayout,
    box_end: &BoxEndLayout,
    theme: &PlantUmlTheme,
) {
    let (x, width) = event_frame_x_width(layout);
    write!(
        svg,
        r#"<g class="sequence-event box-end" data-event="box-end" data-line="{}"><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/></g>"#,
        box_end.line,
        format_number(x),
        format_number(box_end.y),
        format_number(x + width),
        format_number(box_end.y),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
}
