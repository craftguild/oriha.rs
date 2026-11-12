//! Flowchart SVG nodes, edges, markers and labels.
use super::*;

pub(super) fn render_flowchart_svg(
    diagram_id: &str,
    layout: &FlowchartLayout,
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

    out.push_str(r#"<g class="oriha-mermaid-flowchart">"#);
    for edge in &layout.edges {
        render_flow_edge(&mut out, edge, layout.direction, theme, style, &arrow_id);
    }
    for node in &layout.nodes {
        render_flow_node(&mut out, node, theme, style);
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

pub(super) fn render_flow_node(
    out: &mut String,
    node: &FlowNodeLayout,
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
    let stroke_width = stroke_width(style, "1.4");
    out.push_str(r#"<g class="oriha-mermaid-node">"#);
    match node.shape {
        FlowNodeShape::Diamond => {
            let cx = node.rect.center_x();
            let cy = node.rect.center_y();
            out.push_str(&format!(
                r#"<path d="M {} {} L {} {} L {} {} L {} {} Z" fill="{fill}" stroke="{stroke}" stroke-width="{}"/>"#,
                format_number(cx),
                format_number(node.rect.y),
                format_number(node.rect.x + node.rect.width),
                format_number(cy),
                format_number(cx),
                format_number(node.rect.y + node.rect.height),
                format_number(node.rect.x),
                format_number(cy),
                stroke_width
            ));
        }
        FlowNodeShape::Round => render_rect(out, node.rect, 18.0, &fill, &stroke, stroke_width),
        FlowNodeShape::Stadium => render_rect(
            out,
            node.rect,
            node.rect.height / 2.0,
            &fill,
            &stroke,
            stroke_width,
        ),
        FlowNodeShape::Rectangle => render_rect(
            out,
            node.rect,
            corner_radius(style, 8.0),
            &fill,
            &stroke,
            stroke_width,
        ),
    }
    render_text_center(out, &node.text, node.rect, theme);
    out.push_str("</g>");
}

pub(super) fn render_flow_edge(
    out: &mut String,
    edge: &FlowEdgeLayout,
    _direction: FlowDirection,
    theme: &MermaidTheme,
    style: &DiagramStyle,
    arrow_id: &str,
) {
    render_polyline_path(out, &edge.points, false, theme, style, arrow_id);
    if let Some(label) = &edge.label {
        render_edge_label(out, edge.points.as_slice(), label, theme, style);
    }
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

pub(super) fn render_polyline_path(
    out: &mut String,
    points: &[Point],
    dashed: bool,
    theme: &MermaidTheme,
    style: &DiagramStyle,
    arrow_id: &str,
) {
    if points.len() < 2 {
        return;
    }
    let line = escape_attr(color(theme.line_color.as_deref(), "#5f6b7a"));
    let dash = if dashed {
        r#" stroke-dasharray="6 5""#
    } else {
        ""
    };
    let stroke_width = stroke_width(style, "1.6");
    let mut path = String::new();
    for (index, point) in points.iter().enumerate() {
        if index == 0 {
            path.push_str(&format!(
                "M {} {}",
                format_number(point.x),
                format_number(point.y)
            ));
        } else {
            path.push_str(&format!(
                " L {} {}",
                format_number(point.x),
                format_number(point.y)
            ));
        }
    }
    out.push_str(&format!(
        r##"<path class="oriha-mermaid-edge" d="{path}" fill="none" stroke="{line}" stroke-width="{stroke_width}"{dash} marker-end="url(#{})"/>"##,
        escape_attr(arrow_id)
    ));
}

pub(super) fn render_edge_label(
    out: &mut String,
    points: &[Point],
    label: &MeasuredText,
    theme: &MermaidTheme,
    style: &DiagramStyle,
) {
    let center = midpoint(points);
    let padding_x = 7.0;
    let padding_y = 4.0;
    let rect = Rect {
        x: center.x - label.width / 2.0 - padding_x,
        y: center.y - label.height / 2.0 - padding_y,
        width: label.width + padding_x * 2.0,
        height: label.height + padding_y * 2.0,
    };
    let fill = escape_attr(color(
        theme
            .edge_label_background
            .as_deref()
            .or(theme.background.as_deref()),
        "#ffffff",
    ));
    let radius = format_number(corner_radius(style, 6.0));
    out.push_str(&format!(
        r#"<g class="oriha-mermaid-edge-label"><rect x="{}" y="{}" width="{}" height="{}" rx="{radius}" fill="{fill}" opacity="0.94"/>"#,
        format_number(rect.x),
        format_number(rect.y),
        format_number(rect.width),
        format_number(rect.height)
    ));
    render_text_center(out, label, rect, theme);
    out.push_str("</g>");
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

pub(super) fn midpoint(points: &[Point]) -> Point {
    if points.is_empty() {
        return Point { x: 0.0, y: 0.0 };
    }
    let middle = points.len() / 2;
    if points.len().is_multiple_of(2) {
        let a = points[middle - 1];
        let b = points[middle];
        Point {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        }
    } else {
        points[middle]
    }
}
