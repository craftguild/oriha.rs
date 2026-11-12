//! Class diagram model, text measurements and layout.
mod parser;
#[allow(unused_imports)]
use parser::*;
mod render;
#[allow(unused_imports)]
use render::*;

use std::fmt::Write;

use crate::text_layout::{inline_text_measurement_request, InlineTextMeasurementRequest};

const CANVAS_PADDING: f32 = 24.0;
const BOX_GAP: f32 = 56.0;
const BOX_PADDING_X: f32 = 14.0;
const BOX_PADDING_Y: f32 = 10.0;
const BOX_MIN_WIDTH: f32 = 96.0;
const HEADER_LINE_HEIGHT: f32 = 18.0;
const MEMBER_LINE_HEIGHT: f32 = 18.0;
const MEMBER_SECTION_GAP: f32 = 8.0;
const RELATION_LABEL_GAP: f32 = 14.0;

pub(crate) fn render_class_source_svg_with_text_measurer(
    source: &str,
    options: &crate::PlantUmlOptions,
    measurer: &dyn crate::text_layout::TextMeasurer,
) -> Result<crate::PlantUmlSvg, crate::PlantUmlError> {
    let diagram = parse_class_diagram(source)?;
    let layout = layout_class_diagram(&diagram, measurer);
    Ok(render_svg(&layout, options))
}

pub(crate) fn collect_class_text_measurement_requests(
    source: &str,
    _options: &crate::PlantUmlOptions,
    base_style: &crate::text_layout::InlineTextMeasurementStyle,
) -> Result<Vec<crate::text_layout::InlineTextMeasurementRequest>, crate::PlantUmlError> {
    let diagram = parse_class_diagram(source)?;
    let mut requests = Vec::new();

    for class in &diagram.classes {
        push_measurement_request(&mut requests, &class.name, base_style);
        if let Some(stereotype) = class.stereotype() {
            push_measurement_request(&mut requests, stereotype, base_style);
        }
        for member in &class.members {
            push_measurement_request(&mut requests, member, base_style);
        }
    }

    for relation in &diagram.relations {
        if let Some(label) = &relation.label {
            push_measurement_request(&mut requests, label, base_style);
        }
    }

    Ok(requests)
}

pub(crate) fn is_class_statement(line: &str) -> bool {
    let line = strip_inline_comment(line).trim();
    if line.is_empty() || is_directive(line) {
        return false;
    }
    parse_class_declaration_header(line).is_some() || parse_relation_line(0, line).is_some()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassDiagram {
    classes: Vec<ClassBox>,
    relations: Vec<Relation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClassBox {
    name: String,
    kind: ClassKind,
    members: Vec<String>,
}

impl ClassBox {
    fn stereotype(&self) -> Option<&'static str> {
        match self.kind {
            ClassKind::Class => None,
            ClassKind::Interface => Some("<<interface>>"),
            ClassKind::AbstractClass => Some("<<abstract>>"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClassKind {
    Class,
    Interface,
    AbstractClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Relation {
    from: String,
    to: String,
    kind: RelationKind,
    label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RelationKind {
    Association,
    DirectedAssociation,
    Inheritance,
    Realization,
}

#[derive(Debug, Clone)]
struct ClassDiagramLayout {
    classes: Vec<ClassBoxLayout>,
    relations: Vec<RelationLayout>,
    width: f32,
    height: f32,
    view_box: String,
}

#[derive(Debug, Clone)]
struct ClassBoxLayout {
    class: ClassBox,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    header_height: f32,
}

#[derive(Debug, Clone)]
struct RelationLayout {
    relation: Relation,
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
    label_x: f32,
    label_y: f32,
}

#[derive(Debug)]
struct ParsedClassHeader {
    class: ClassBox,
    opens_block: bool,
    closes_block: bool,
    inline_members: Vec<String>,
}

fn layout_class_diagram(
    diagram: &ClassDiagram,
    measurer: &dyn crate::text_layout::TextMeasurer,
) -> ClassDiagramLayout {
    let mut x = CANVAS_PADDING;
    let y = CANVAS_PADDING;
    let mut classes = Vec::new();
    let mut max_height: f32 = 0.0;

    for class in &diagram.classes {
        let stereotype_width = class
            .stereotype()
            .map(|stereotype| measurer.measure_inline_text(stereotype))
            .unwrap_or(0.0);
        let name_width = measurer.measure_inline_text(&class.name);
        let member_width = class
            .members
            .iter()
            .map(|member| measurer.measure_inline_text(member))
            .fold(0.0, f32::max);

        let width = BOX_MIN_WIDTH
            .max(name_width.max(stereotype_width).max(member_width) + BOX_PADDING_X * 2.0);
        let header_lines = if class.stereotype().is_some() {
            2.0
        } else {
            1.0
        };
        let header_height = BOX_PADDING_Y * 2.0 + header_lines * HEADER_LINE_HEIGHT;
        let members_height = if class.members.is_empty() {
            0.0
        } else {
            MEMBER_SECTION_GAP + class.members.len() as f32 * MEMBER_LINE_HEIGHT + BOX_PADDING_Y
        };
        let height = header_height + members_height;

        classes.push(ClassBoxLayout {
            class: class.clone(),
            x,
            y,
            width,
            height,
            header_height,
        });
        x += width + BOX_GAP;
        max_height = max_height.max(height);
    }

    let width = if classes.is_empty() {
        CANVAS_PADDING * 2.0
    } else {
        x - BOX_GAP + CANVAS_PADDING
    };
    let relation_band_height = if diagram.relations.is_empty() {
        CANVAS_PADDING
    } else {
        72.0
    };
    let height = CANVAS_PADDING + max_height + relation_band_height;
    let relations = layout_relations(diagram, &classes);

    ClassDiagramLayout {
        classes,
        relations,
        width,
        height,
        view_box: format!("0 0 {:.0} {:.0}", width.ceil(), height.ceil()),
    }
}

fn layout_relations(diagram: &ClassDiagram, classes: &[ClassBoxLayout]) -> Vec<RelationLayout> {
    diagram
        .relations
        .iter()
        .filter_map(|relation| {
            let from = classes
                .iter()
                .find(|class| class.class.name == relation.from)?;
            let to = classes
                .iter()
                .find(|class| class.class.name == relation.to)?;
            let (from_x, to_x) = if from.x <= to.x {
                (from.x + from.width, to.x)
            } else {
                (from.x, to.x + to.width)
            };
            let from_y = from.y + from.height / 2.0;
            let to_y = to.y + to.height / 2.0;
            Some(RelationLayout {
                relation: relation.clone(),
                from_x,
                from_y,
                to_x,
                to_y,
                label_x: (from_x + to_x) / 2.0,
                label_y: from_y.max(to_y) + RELATION_LABEL_GAP,
            })
        })
        .collect()
}

fn push_measurement_request(
    requests: &mut Vec<InlineTextMeasurementRequest>,
    text: &str,
    base_style: &crate::text_layout::InlineTextMeasurementStyle,
) {
    requests.push(inline_text_measurement_request(text, base_style));
}
