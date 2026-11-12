//! Class box and relation SVG emission.
use super::*;

pub(super) fn render_svg(
    layout: &ClassDiagramLayout,
    options: &crate::PlantUmlOptions,
) -> crate::PlantUmlSvg {
    let width = layout.width.ceil() as u32;
    let height = layout.height.ceil() as u32;
    let mut svg = String::new();

    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" data-diagram-type="CLASS" viewBox="{}" width="{width}" height="{height}" font-family="{}" font-size="{}">"#,
        escape_attr(&layout.view_box),
        escape_attr(&options.font_family),
        options.resolved_font_size(),
    )
    .expect("writing to String cannot fail");
    svg.push_str(r##"<defs><marker id="class-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#475569"/></marker><marker id="class-triangle" viewBox="0 0 12 10" refX="11" refY="5" markerWidth="10" markerHeight="10" orient="auto-start-reverse"><path d="M 1 1 L 11 5 L 1 9 z" fill="#ffffff" stroke="#475569" stroke-width="1.5"/></marker></defs>"##);
    svg.push_str(r##"<rect width="100%" height="100%" fill="#ffffff"/>"##);

    for relation in &layout.relations {
        render_relation(&mut svg, relation);
    }
    for class in &layout.classes {
        render_class_box(&mut svg, class);
    }

    svg.push_str("</svg>");

    crate::PlantUmlSvg {
        svg,
        width,
        height,
        view_box: layout.view_box.clone(),
        kind: crate::PlantUmlDiagramKind::Class,
    }
}

pub(super) fn render_class_box(svg: &mut String, class: &ClassBoxLayout) {
    write!(
        svg,
        r##"<g class="class-box"><rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" rx="4" fill="#f8fafc" stroke="#475569" stroke-width="1.4"/>"##,
        class.x, class.y, class.width, class.height,
    )
    .expect("writing to String cannot fail");

    if !class.class.members.is_empty() {
        write!(
            svg,
            r##"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="#475569" stroke-width="1"/>"##,
            class.x,
            class.y + class.header_height,
            class.x + class.width,
            class.y + class.header_height,
        )
        .expect("writing to String cannot fail");
    }

    let center_x = class.x + class.width / 2.0;
    let mut text_y = class.y + BOX_PADDING_Y + 13.0;
    if let Some(stereotype) = class.class.stereotype() {
        write_center_text(svg, center_x, text_y, stereotype, "normal");
        text_y += HEADER_LINE_HEIGHT;
    }
    write_center_text(svg, center_x, text_y, &class.class.name, "600");

    let mut member_y = class.y + class.header_height + MEMBER_SECTION_GAP + 13.0;
    for member in &class.class.members {
        write_start_text(svg, class.x + BOX_PADDING_X, member_y, member);
        member_y += MEMBER_LINE_HEIGHT;
    }

    svg.push_str("</g>");
}

pub(super) fn render_relation(svg: &mut String, relation: &RelationLayout) {
    let (dash, marker_end) = match relation.relation.kind {
        RelationKind::Association => ("", ""),
        RelationKind::DirectedAssociation => ("", r#" marker-end="url(#class-arrow)""#),
        RelationKind::Inheritance => ("", r#" marker-end="url(#class-triangle)""#),
        RelationKind::Realization => (
            r#" stroke-dasharray="4 4""#,
            r#" marker-end="url(#class-triangle)""#,
        ),
    };
    write!(
        svg,
        r##"<line x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="#475569" stroke-width="1.4"{}{} />"##,
        relation.from_x,
        relation.from_y,
        relation.to_x,
        relation.to_y,
        dash,
        marker_end,
    )
    .expect("writing to String cannot fail");

    if let Some(label) = &relation.relation.label {
        write_center_text(svg, relation.label_x, relation.label_y, label, "normal");
    }
}

pub(super) fn write_center_text(svg: &mut String, x: f32, y: f32, text: &str, weight: &str) {
    write!(
        svg,
        r##"<text x="{x:.1}" y="{y:.1}" text-anchor="middle" fill="#0f172a" font-weight="{}">{}</text>"##,
        escape_attr(weight),
        escape_text(text),
    )
    .expect("writing to String cannot fail");
}

pub(super) fn write_start_text(svg: &mut String, x: f32, y: f32, text: &str) {
    write!(
        svg,
        r##"<text x="{x:.1}" y="{y:.1}" fill="#0f172a">{}</text>"##,
        escape_text(text),
    )
    .expect("writing to String cannot fail");
}

pub(super) fn escape_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub(super) fn escape_attr(value: &str) -> String {
    escape_text(value)
}
