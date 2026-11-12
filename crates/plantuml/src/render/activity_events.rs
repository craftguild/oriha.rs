//! Timeline dispatch, activations and return events.
use super::*;

pub(super) fn render_timeline_event(
    svg: &mut String,
    layout: &SequenceLayout,
    event: &SequenceTimelineEventLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) {
    match event {
        SequenceTimelineEventLayout::Message(message) => {
            render_message(svg, message, theme, style, measurer);
        }
        SequenceTimelineEventLayout::Title(title) => {
            render_title(svg, layout, title, theme, style);
        }
        SequenceTimelineEventLayout::Note(note) => {
            render_note(svg, layout, note, theme, style, measurer);
        }
        SequenceTimelineEventLayout::GroupStart(group) => {
            render_group_start(svg, layout, group, theme, style);
        }
        SequenceTimelineEventLayout::GroupElse(group_else) => {
            render_group_else(svg, layout, group_else, theme, measurer);
        }
        SequenceTimelineEventLayout::GroupEnd(group_end) => {
            render_group_end(svg, layout, group_end, theme);
        }
        SequenceTimelineEventLayout::Divider(divider) => {
            render_divider(svg, layout, divider, theme, measurer);
        }
        SequenceTimelineEventLayout::Delay(delay) => {
            render_delay(svg, layout, delay, theme, style, measurer);
        }
        SequenceTimelineEventLayout::Activation(activation) => {
            render_activation(svg, layout, activation, theme, style, measurer);
        }
        SequenceTimelineEventLayout::Return(return_event) => {
            render_return(svg, layout, return_event, theme, style, measurer);
        }
        SequenceTimelineEventLayout::BoxStart(box_start) => {
            render_box_start(svg, layout, box_start, theme, style, measurer);
        }
        SequenceTimelineEventLayout::BoxEnd(box_end) => {
            render_box_end(svg, layout, box_end, theme);
        }
        SequenceTimelineEventLayout::Semantic(semantic_event) => {
            render_semantic_event(svg, layout, semantic_event, theme, style, measurer);
        }
    }
}

pub(super) fn render_activation(
    svg: &mut String,
    layout: &SequenceLayout,
    activation: &ActivationEventLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) {
    let x = participant_x(layout, &activation.participant).unwrap_or(layout.width / 2.0);
    let kind = activation_kind_name(activation.kind);
    let fill = activation
        .color
        .as_deref()
        .map(svg_color_value)
        .unwrap_or_else(|| Cow::Borrowed(theme.participant_fill.as_str()));
    let corner_radius = style
        .corner_radius
        .svg_px()
        .map(|px| format!(r#" rx="{px}" ry="{px}""#))
        .unwrap_or_default();

    write!(
        svg,
        r#"<g class="sequence-event activation activation-{kind} {kind}" data-event="activation" data-activation-kind="{kind}" data-participant="{}" data-line="{}">"#,
        escape_attr(&activation.participant),
        activation.line,
    )
    .expect("writing to String cannot fail");

    match activation.kind {
        ActivationKind::Activate | ActivationKind::Create => {
            write!(
                svg,
                r#"<rect x="{}" y="{}" width="12" height="26" fill="{}" stroke="{}"{}/>"#,
                format_number(x - 6.0),
                format_number(activation.y - 13.0),
                escape_attr(fill.as_ref()),
                escape_attr(&theme.participant_stroke),
                corner_radius,
            )
            .expect("writing to String cannot fail");
        }
        ActivationKind::Deactivate | ActivationKind::Destroy => {
            write!(
                svg,
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="2"/>"#,
                format_number(x - 8.0),
                format_number(activation.y),
                format_number(x + 8.0),
                format_number(activation.y),
                escape_attr(&theme.participant_stroke),
            )
            .expect("writing to String cannot fail");
        }
    }

    let label = format!("{kind} {}", activation.participant);
    render_text_element(
        svg,
        x + 12.0,
        activation.y + 4.0,
        "start",
        &theme.text_color,
        &label,
        measurer,
    );
    svg.push_str("</g>");
}

pub(super) fn render_return(
    svg: &mut String,
    layout: &SequenceLayout,
    return_event: &ReturnEventLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) {
    let (x, width) = participant_span_x_width(layout);
    write!(
        svg,
        r#"<g class="sequence-event return" data-event="return" data-line="{}"><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-dasharray="{}"/>"#,
        return_event.line,
        format_number(x),
        format_number(return_event.y),
        format_number(x + width),
        format_number(return_event.y),
        escape_attr(&theme.message_color),
        style.stroke_width.dasharray(),
    )
    .expect("writing to String cannot fail");
    if let Some(label) = &return_event.label {
        render_text_element(
            svg,
            x + width / 2.0,
            return_event.y - 8.0 - TEXT_LINE_HEIGHT * (text_line_count(label) - 1) as f32,
            "middle",
            &theme.text_color,
            label,
            measurer,
        );
    }
    svg.push_str("</g>");
}
