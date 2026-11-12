//! Sequence SVG participants, lifelines, messages and text.
use super::*;

pub(super) fn render_sequence_svg(
    source: &str,
    diagram_id: &str,
    theme: &MermaidTheme,
    style: &DiagramStyle,
    layout: &SequenceLayout,
) -> MermaidSvg {
    let diagram_id = resolved_diagram_id(source, diagram_id);
    let svg = render_svg(&diagram_id, layout, theme, style);
    MermaidSvg {
        svg,
        width: layout_dimension(layout.width),
        height: layout_dimension(layout.height),
        view_box: format!(
            "0 0 {} {}",
            format_svg_number(layout.width),
            format_svg_number(layout.height)
        ),
        kind: MermaidDiagramKind::Sequence,
    }
}

pub(super) fn render_svg(
    diagram_id: &str,
    layout: &SequenceLayout,
    theme: &MermaidTheme,
    style: &DiagramStyle,
) -> String {
    let mut out = String::new();
    let width = format_number(layout.width);
    let height = format_number(layout.height);
    let arrow_id = format!("{diagram_id}-arrow");
    out.push_str(&format!(
        r#"<svg id="{}" xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img">"#,
        escape_attr(diagram_id)
    ));
    render_defs(&mut out, theme, &arrow_id);
    out.push_str(&format!(
        r#"<rect class="oriha-mermaid-svg-bg" x="0" y="0" width="{width}" height="{height}" rx="12" fill="{}"/>"#,
        escape_attr(color(theme.background.as_deref(), "transparent"))
    ));
    out.push_str(r#"<g class="oriha-mermaid-sequence">"#);
    for participant in &layout.participants {
        render_sequence_lifeline(
            &mut out,
            participant,
            layout.lifeline_top,
            layout.lifeline_bottom,
            theme,
            style,
        );
    }
    for message in &layout.messages {
        render_sequence_message(&mut out, message, theme, style, &arrow_id);
    }
    for participant in &layout.participants {
        render_sequence_participant(&mut out, participant, theme, style);
    }
    out.push_str("</g>");
    out.push_str("</svg>");
    out
}

pub(super) fn render_defs(out: &mut String, theme: &MermaidTheme, arrow_id: &str) {
    let line = escape_attr(color(theme.line_color.as_deref(), "#5f6b7a"));
    out.push_str(&format!(
        r#"<defs><marker id="{}" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="{line}"/></marker></defs>"#,
        escape_attr(arrow_id)
    ));
}

pub(super) fn render_sequence_participant(
    out: &mut String,
    participant: &SequenceParticipantLayout,
    theme: &MermaidTheme,
    style: &DiagramStyle,
) {
    let fill = escape_attr(color(
        theme
            .node_bkg
            .as_deref()
            .or(theme.primary_color.as_deref())
            .or(theme.main_bkg.as_deref()),
        "#ffffff",
    ));
    let stroke = escape_attr(color(
        theme
            .node_border
            .as_deref()
            .or(theme.primary_border_color.as_deref()),
        "#aeb8c2",
    ));
    out.push_str(r#"<g class="oriha-mermaid-sequence-participant">"#);
    render_rect(
        out,
        participant.rect,
        corner_radius(style, 8.0),
        &fill,
        &stroke,
        stroke_width(style, "1.4"),
    );
    render_text_center(out, &participant.text, participant.rect, theme);
    out.push_str("</g>");
}

pub(super) fn render_sequence_lifeline(
    out: &mut String,
    participant: &SequenceParticipantLayout,
    top: f32,
    bottom: f32,
    theme: &MermaidTheme,
    style: &DiagramStyle,
) {
    let stroke = escape_attr(color(theme.line_color.as_deref(), "#5f6b7a"));
    let stroke_width = stroke_width(style, "1.2");
    out.push_str(&format!(
        r#"<line class="oriha-mermaid-lifeline" x1="{}" y1="{}" x2="{}" y2="{}" stroke="{stroke}" stroke-width="{stroke_width}" stroke-dasharray="5 5"/>"#,
        format_number(participant.center_x),
        format_number(top),
        format_number(participant.center_x),
        format_number(bottom)
    ));
}

pub(super) fn render_sequence_message(
    out: &mut String,
    message: &SequenceMessageLayout,
    theme: &MermaidTheme,
    style: &DiagramStyle,
    arrow_id: &str,
) {
    let line = escape_attr(color(theme.line_color.as_deref(), "#5f6b7a"));
    let dash = if message.dashed {
        r#" stroke-dasharray="6 5""#
    } else {
        ""
    };
    let stroke_width = stroke_width(style, "1.6");
    out.push_str(&format!(
        r##"<line class="oriha-mermaid-message" x1="{}" y1="{}" x2="{}" y2="{}" stroke="{line}" stroke-width="{stroke_width}"{dash} marker-end="url(#{})"/>"##,
        format_number(message.from_x),
        format_number(message.y),
        format_number(message.to_x),
        format_number(message.y),
        escape_attr(arrow_id)
    ));
    let center = (message.from_x + message.to_x) / 2.0;
    let text = if let Some(number) = message.number {
        format!("{number}. {}", message.text.text)
    } else {
        message.text.text.clone()
    };
    render_plain_text(
        out,
        &text,
        center,
        message.y - 12.0,
        "middle",
        &message.text,
        theme,
    );
}

pub(super) fn render_rect(
    out: &mut String,
    rect: Rect,
    radius: f32,
    fill: &str,
    stroke: &str,
    stroke_width: &str,
) {
    out.push_str(&format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{fill}" stroke="{stroke}" stroke-width="{stroke_width}"/>"#,
        format_number(rect.x),
        format_number(rect.y),
        format_number(rect.width),
        format_number(rect.height),
        format_number(radius)
    ));
}

pub(super) fn render_text_center(
    out: &mut String,
    text: &MeasuredText,
    rect: Rect,
    theme: &MermaidTheme,
) {
    let line_height = text.font_size * 1.25;
    let total_height = line_height * text.lines.len().max(1) as f32;
    let first_y = rect.center_y() - total_height / 2.0 + text.font_size * 0.9;
    let fill = escape_attr(color(
        theme
            .primary_text_color
            .as_deref()
            .or(theme.text_color.as_deref()),
        "#1f2937",
    ));
    let weight = text.font_weight.as_deref().unwrap_or("400");
    out.push_str(&format!(
        r#"<text x="{}" y="{}" text-anchor="middle" fill="{fill}" font-family="{}" font-size="{}" font-weight="{}">"#,
        format_number(rect.center_x()),
        format_number(first_y),
        escape_attr(&theme.font_family),
        format_number(text.font_size),
        escape_attr(weight)
    ));
    for (index, line) in text.lines.iter().enumerate() {
        if index == 0 {
            out.push_str(&format!(
                r#"<tspan x="{}">{}</tspan>"#,
                format_number(rect.center_x()),
                escape_text(line)
            ));
        } else {
            out.push_str(&format!(
                r#"<tspan x="{}" dy="{}">{}</tspan>"#,
                format_number(rect.center_x()),
                format_number(line_height),
                escape_text(line)
            ));
        }
    }
    out.push_str("</text>");
}

pub(super) fn render_plain_text(
    out: &mut String,
    text: &str,
    x: f32,
    y: f32,
    anchor: &str,
    metrics: &MeasuredText,
    theme: &MermaidTheme,
) {
    let fill = escape_attr(color(theme.text_color.as_deref(), "#1f2937"));
    out.push_str(&format!(
        r#"<text x="{}" y="{}" text-anchor="{anchor}" fill="{fill}" font-family="{}" font-size="{}">{}</text>"#,
        format_number(x),
        format_number(y),
        escape_attr(&theme.font_family),
        format_number(metrics.font_size),
        escape_text(text)
    ));
}
