//! Page titles, headers, footers and page-break chrome.
use super::*;

pub(super) fn render_page_chrome(svg: &mut String, layout: &SequenceLayout, theme: &PlantUmlTheme) {
    if let Some(header) = &layout.header {
        render_header_footer(svg, layout, header, theme);
    }
    if let Some(title) = &layout.title {
        render_page_title(svg, layout, title, theme);
    }
    if let Some(footer) = &layout.footer {
        render_header_footer(svg, layout, footer, theme);
    }
}

pub(super) fn render_page_title(
    svg: &mut String,
    layout: &SequenceLayout,
    title: &TextEventLayout,
    theme: &PlantUmlTheme,
) {
    render_positioned_text(
        svg,
        PositionedText {
            class_name: "sequence-title",
            data_event: "title",
            data_line: title.line,
            x: layout.width / 2.0,
            y: title.y,
            anchor: "middle",
            fill: &theme.text_color,
            text: &title.text,
            attrs: r#" font-weight="700""#,
        },
    );
}

pub(super) fn render_header_footer(
    svg: &mut String,
    layout: &SequenceLayout,
    event: &HeaderFooterLayout,
    theme: &PlantUmlTheme,
) {
    let (class_name, data_event, x, anchor) = match event.kind {
        HeaderFooterKind::Header => ("sequence-header", "header", layout.width - 12.0, "end"),
        HeaderFooterKind::Footer => ("sequence-footer", "footer", layout.width / 2.0, "middle"),
    };
    let text = page_counter_text(&event.text);
    let font_size = sequence_small_font_size(theme);
    let extra_attrs = format!(
        r#" font-size="{font_size}" data-header-footer-kind="{}" data-multiline="{}""#,
        header_footer_kind_name(event.kind),
        event.multiline
    );
    render_positioned_text(
        svg,
        PositionedText {
            class_name,
            data_event,
            data_line: event.line,
            x,
            y: event.y,
            anchor,
            fill: &theme.participant_stroke,
            text: &text,
            attrs: &extra_attrs,
        },
    );
}

pub(super) fn render_page_break(
    svg: &mut String,
    layout: &SequenceLayout,
    event: &PageBreakLayout,
    theme: &PlantUmlTheme,
) {
    write!(
        svg,
        r#"<g class="sequence-event page-break" data-event="newpage" data-line="{}"><line x1="0" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="1" stroke-dasharray="2 4"/>"#,
        event.line,
        format_number(event.y),
        format_number(layout.width),
        format_number(event.y),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");

    if let Some(label) = event.label.as_deref() {
        render_positioned_text(
            svg,
            PositionedText {
                class_name: "page-break-label",
                data_event: "newpage-label",
                data_line: event.line,
                x: layout.width / 2.0,
                y: event.y + 18.0,
                anchor: "middle",
                fill: &theme.text_color,
                text: label,
                attrs: r#" font-weight="700""#,
            },
        );
    }

    svg.push_str("</g>");
}

pub(super) fn render_positioned_text(svg: &mut String, text: PositionedText<'_>) {
    write!(
        svg,
        r#"<text class="{}" data-event="{}" data-line="{}" x="{}" y="{}" text-anchor="{}" fill="{}"{}>"#,
        text.class_name,
        text.data_event,
        text.data_line,
        format_number(text.x),
        format_number(text.y),
        text.anchor,
        escape_attr(text.fill),
        text.attrs,
    )
    .expect("writing to String cannot fail");
    for (index, line) in text_lines(text.text).iter().enumerate() {
        render_text_line_tspans(svg, text.x, index, line);
    }
    svg.push_str("</text>");
}

pub(super) fn page_counter_text(text: &str) -> String {
    text.replace("%page%", "1").replace("%lastpage%", "1")
}
