//! Styled inline text, tspans and decoration geometry.
use super::*;

pub(super) fn render_text_element(
    svg: &mut String,
    x: f32,
    y: f32,
    anchor: &str,
    fill: &str,
    text: &str,
    measurer: &dyn TextMeasurer,
) {
    for (index, line) in text_lines(text).iter().enumerate() {
        render_inline_text_annotations(
            svg,
            InlineTextAnnotation {
                x,
                y,
                anchor,
                fill,
                line_index: index,
                line,
                measurer,
            },
        );
    }

    write!(
        svg,
        r#"<text x="{}" y="{}" text-anchor="{}" fill="{}">"#,
        format_number(x),
        format_number(y),
        anchor,
        escape_attr(fill),
    )
    .expect("writing to String cannot fail");

    for (index, line) in text_lines(text).iter().enumerate() {
        render_text_line_tspans(svg, x, index, line);
    }

    svg.push_str("</text>");
}

pub(super) fn render_inline_text_annotations(
    svg: &mut String,
    annotation: InlineTextAnnotation<'_>,
) {
    let line_layout = layout_inline_text_line_with_measurer(
        annotation.line,
        annotation.x,
        TextAnchor::from_svg_anchor(annotation.anchor),
        annotation.measurer,
    );
    if !line_layout.runs.iter().any(inline_run_needs_annotation) {
        return;
    }

    let baseline_y = annotation.y + TEXT_LINE_HEIGHT * annotation.line_index as f32;

    for run in &line_layout.runs {
        if run.width <= 0.0 {
            continue;
        }
        let font_size = inline_style_font_size(&run.style);
        if let Some(color) = &run.style.background_color {
            write!(
                svg,
                r#"<rect class="inline-text-background" x="{}" y="{}" width="{}" height="{}" fill="{}" opacity="0.65"/>"#,
                format_number(run.x - 1.0),
                format_number(baseline_y - font_size * 0.85),
                format_number(run.width + 2.0),
                format_number(font_size * 1.10),
                escape_attr(color),
            )
            .expect("writing to String cannot fail");
        }
        if let Some(color) = &run.style.decoration_color {
            if run.style.underline {
                render_inline_decoration_line(
                    svg,
                    "underline",
                    run.x,
                    run.x + run.width,
                    baseline_y + 2.0,
                    color,
                );
            }
            if run.style.strike {
                render_inline_decoration_line(
                    svg,
                    "line-through",
                    run.x,
                    run.x + run.width,
                    baseline_y - font_size * 0.25,
                    color,
                );
            }
            if run.style.wavy {
                render_inline_wavy_decoration(
                    svg,
                    run.x,
                    run.x + run.width,
                    baseline_y + 2.0,
                    color,
                );
            }
        } else if run.style.wavy {
            render_inline_wavy_decoration(
                svg,
                run.x,
                run.x + run.width,
                baseline_y + 2.0,
                annotation.fill,
            );
        }
    }
}

pub(super) fn inline_run_needs_annotation(run: &crate::text_layout::InlineTextRun) -> bool {
    run.style.background_color.is_some()
        || run.style.wavy
        || (run.style.decoration_color.is_some()
            && (run.style.underline || run.style.strike || run.style.wavy))
}

pub(super) fn render_inline_decoration_line(
    svg: &mut String,
    class_name: &str,
    x1: f32,
    x2: f32,
    y: f32,
    color: &str,
) {
    write!(
        svg,
        r#"<line class="inline-text-decoration inline-text-decoration-{class_name}" x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1"/>"#,
        format_number(x1),
        format_number(y),
        format_number(x2),
        format_number(y),
        escape_attr(color),
    )
    .expect("writing to String cannot fail");
}

pub(super) fn render_inline_wavy_decoration(
    svg: &mut String,
    x1: f32,
    x2: f32,
    y: f32,
    color: &str,
) {
    let width = x2 - x1;
    if width <= 0.0 {
        return;
    }
    let wave = 4.0;
    let mut data = format!("M {},{}", format_number(x1), format_number(y));
    let mut x = x1;
    let mut upward = true;
    while x < x2 {
        let next_x = (x + wave).min(x2);
        let control_x = (x + next_x) / 2.0;
        let control_y = if upward { y - 2.0 } else { y + 2.0 };
        write!(
            data,
            " Q {},{} {},{}",
            format_number(control_x),
            format_number(control_y),
            format_number(next_x),
            format_number(y),
        )
        .expect("writing to String cannot fail");
        x = next_x;
        upward = !upward;
    }

    write!(
        svg,
        r#"<path class="inline-text-decoration inline-text-decoration-wavy" d="{}" fill="none" stroke="{}" stroke-width="1"/>"#,
        escape_attr(&data),
        escape_attr(color),
    )
    .expect("writing to String cannot fail");
}

pub(super) fn render_text_line_tspans(svg: &mut String, x: f32, line_index: usize, line: &str) {
    let spans = parse_inline_text_spans(line);
    let dy_attr = if line_index == 0 { "" } else { r#" dy="16""# };

    if spans.len() == 1 && spans[0].style == InlineTextStyle::default() {
        write!(
            svg,
            r#"<tspan x="{}"{}>{}</tspan>"#,
            format_number(x),
            dy_attr,
            escape_text(&spans[0].text),
        )
        .expect("writing to String cannot fail");
        return;
    }

    for (index, span) in spans.iter().enumerate() {
        let position_attrs = if index == 0 {
            format!(r#" x="{}"{}"#, format_number(x), dy_attr)
        } else {
            String::new()
        };
        write!(
            svg,
            r#"<tspan{}{}>{}</tspan>"#,
            position_attrs,
            inline_text_style_attrs(&span.style),
            escape_text(&span.text),
        )
        .expect("writing to String cannot fail");
    }
}

pub(super) fn inline_text_style_attrs(style: &InlineTextStyle) -> String {
    let mut attrs = String::new();
    let mut style_declarations = Vec::new();
    let mut text_decorations = Vec::new();
    if style.bold {
        attrs.push_str(r#" font-weight="bold""#);
    }
    if style.italic {
        attrs.push_str(r#" font-style="italic""#);
    }
    if style.underline && style.decoration_color.is_none() {
        text_decorations.push("underline");
    }
    if style.strike && style.decoration_color.is_none() {
        text_decorations.push("line-through");
    }
    if !text_decorations.is_empty() {
        write!(
            attrs,
            r#" text-decoration="{}""#,
            text_decorations.join(" ")
        )
        .expect("writing to String cannot fail");
    }
    if style.monospace {
        attrs.push_str(r#" font-family="monospace""#);
    }
    if let Some(font_size) = style.font_size {
        write!(attrs, r#" font-size="{font_size}""#).expect("writing to String cannot fail");
    }
    if let Some(color) = &style.decoration_color {
        style_declarations.push(format!("text-decoration-color: {}", escape_attr(color)));
    }
    if let Some(color) = &style.background_color {
        write!(attrs, r#" data-background-color="{}""#, escape_attr(color))
            .expect("writing to String cannot fail");
    }
    if let Some(color) = &style.color {
        write!(attrs, r#" fill="{}""#, escape_attr(color)).expect("writing to String cannot fail");
    }
    if !style_declarations.is_empty() {
        write!(attrs, r#" style="{}""#, style_declarations.join(";"))
            .expect("writing to String cannot fail");
    }
    attrs
}

pub(super) fn text_line_count(text: &str) -> usize {
    text_lines(text).len().max(1)
}
