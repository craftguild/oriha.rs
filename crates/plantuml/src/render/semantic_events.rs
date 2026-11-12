//! Semantic sequence event markers and rendering.
use super::*;

pub(super) fn render_semantic_event(
    svg: &mut String,
    layout: &SequenceLayout,
    event: &SemanticEventLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) {
    match event {
        SemanticEventLayout::Autonumber(event) => {
            let mut label = format!("autonumber {}", autonumber_command_name(event.command));
            if !event.args.is_empty() {
                label.push(' ');
                label.push_str(&event.args.join(" "));
            }
            render_semantic_marker(
                svg,
                layout,
                SemanticMarker {
                    class_name: "autonumber",
                    data_event: "autonumber",
                    line: event.line,
                    y: event.y,
                    label: label.trim_end(),
                    extra_attrs: "",
                },
                theme,
                style,
                measurer,
            );
        }
        SemanticEventLayout::PageBreak(event) => {
            render_page_break(svg, layout, event, theme);
        }
        SemanticEventLayout::HeaderFooter(event) => {
            render_header_footer(svg, layout, event, theme);
        }
        SemanticEventLayout::Reference(event) => {
            let extra_attrs = format!(
                r#" data-targets="{}" data-multiline="{}""#,
                escape_attr(&event.targets.join(",")),
                event.multiline
            );
            render_semantic_marker(
                svg,
                layout,
                SemanticMarker {
                    class_name: "reference",
                    data_event: "reference",
                    line: event.line,
                    y: event.y,
                    label: event.text.as_deref().unwrap_or("ref"),
                    extra_attrs: &extra_attrs,
                },
                theme,
                style,
                measurer,
            );
        }
        SemanticEventLayout::Space(event) => {
            let label = event
                .size
                .as_deref()
                .map_or("space".to_owned(), |size| format!("space {size}"));
            render_semantic_marker(
                svg,
                layout,
                SemanticMarker {
                    class_name: "space",
                    data_event: "space",
                    line: event.line,
                    y: event.y,
                    label: &label,
                    extra_attrs: "",
                },
                theme,
                style,
                measurer,
            );
        }
        SemanticEventLayout::Mainframe(event) => {
            render_semantic_marker(
                svg,
                layout,
                SemanticMarker {
                    class_name: "mainframe",
                    data_event: "mainframe",
                    line: event.line,
                    y: event.y,
                    label: event.title.as_deref().unwrap_or("mainframe"),
                    extra_attrs: "",
                },
                theme,
                style,
                measurer,
            );
        }
        SemanticEventLayout::StyleBlock(event) => {
            let extra_attrs = format!(
                r#" data-content-lines="{}""#,
                text_line_count(&event.content)
            );
            render_semantic_marker(
                svg,
                layout,
                SemanticMarker {
                    class_name: "style-block",
                    data_event: "style-block",
                    line: event.line,
                    y: event.y,
                    label: "style block",
                    extra_attrs: &extra_attrs,
                },
                theme,
                style,
                measurer,
            );
        }
        SemanticEventLayout::HideUnlinked(event) => {
            render_semantic_marker(
                svg,
                layout,
                SemanticMarker {
                    class_name: "hide-unlinked",
                    data_event: "hide-unlinked",
                    line: event.line,
                    y: event.y,
                    label: "hide unlinked",
                    extra_attrs: "",
                },
                theme,
                style,
                measurer,
            );
        }
        SemanticEventLayout::Autoactivate(event) => {
            let label = if event.enabled {
                "autoactivate on"
            } else {
                "autoactivate off"
            };
            render_semantic_marker(
                svg,
                layout,
                SemanticMarker {
                    class_name: "autoactivate",
                    data_event: "autoactivate",
                    line: event.line,
                    y: event.y,
                    label,
                    extra_attrs: "",
                },
                theme,
                style,
                measurer,
            );
        }
    }
}

pub(super) fn render_semantic_marker(
    svg: &mut String,
    layout: &SequenceLayout,
    marker: SemanticMarker<'_>,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) {
    let (x, width) = event_frame_x_width(layout);
    let y = marker.y - 18.0;
    let height = 28.0 + TEXT_LINE_HEIGHT * (text_line_count(marker.label) - 1) as f32;
    let corner_radius = style
        .corner_radius
        .svg_px()
        .map(|px| format!(r#" rx="{px}" ry="{px}""#))
        .unwrap_or_default();

    write!(
        svg,
        r#"<g class="sequence-event semantic-event semantic-{}" data-event="{}" data-line="{}"{}><rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-dasharray="{}"{}/>"#,
        marker.class_name,
        marker.data_event,
        marker.line,
        marker.extra_attrs,
        format_number(x),
        format_number(y),
        format_number(width),
        format_number(height),
        escape_attr(&theme.participant_stroke),
        style.stroke_width.dasharray(),
        corner_radius,
    )
    .expect("writing to String cannot fail");
    render_text_element(
        svg,
        x + width / 2.0,
        marker.y,
        "middle",
        &theme.text_color,
        marker.label,
        measurer,
    );
    svg.push_str("</g>");
}
