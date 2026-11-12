//! Math layout model and recursive node coordination.
mod text;
#[allow(unused_imports)]
use text::*;
mod fractions;
#[allow(unused_imports)]
use fractions::*;
mod scripts;
#[allow(unused_imports)]
use scripts::*;
mod grids;
#[allow(unused_imports)]
use grids::*;
mod metrics;
#[allow(unused_imports)]
use metrics::*;
mod boxes;
#[allow(unused_imports)]
use boxes::*;

use crate::ast::MathAtomClass;
use crate::ir::{GridKind, IrNode, LatexIr};
use crate::opentype::{stix_math_metrics, OpenTypeMathFont};
use crate::LatexRenderMode;

#[derive(Debug, Clone)]
pub(crate) struct LatexLayout {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub elements: Vec<LayoutElement>,
}

#[derive(Debug, Clone)]
pub(crate) enum LayoutElement {
    Text {
        text: String,
        x: f32,
        y: f32,
        font_size: f32,
        font_style: TextStyle,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        stroke_width: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextStyle {
    Normal,
    #[allow(dead_code)]
    Italic,
}

#[derive(Debug, Clone)]
struct LayoutBox {
    width: f32,
    ascent: f32,
    descent: f32,
    italic_correction: f32,
    children: Vec<PositionedChild>,
}

#[derive(Debug, Clone)]
struct PositionedChild {
    x: f32,
    y: f32,
    child: LayoutChild,
}

#[derive(Debug, Clone)]
enum LayoutChild {
    Box(Box<LayoutBox>),
    Primitive(LayoutPrimitive),
}

#[derive(Debug, Clone)]
enum LayoutPrimitive {
    Text {
        text: String,
        font_size: f32,
        font_style: TextStyle,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        stroke_width: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathStyle {
    Display,
    Text,
    Script,
    ScriptScript,
}

#[derive(Debug, Clone, Copy)]
struct LayoutContext {
    style: MathStyle,
    font_size: f32,
    metrics: MathMetrics,
}

#[derive(Debug, Clone, Copy)]
struct MathMetrics {
    font: &'static OpenTypeMathFont,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextRunKind {
    Identifier,
    Number,
    Operator(MathAtomClass),
    TextOperator,
}

#[derive(Debug, Clone)]
struct PreparedTextRun {
    display_text: String,
    measurement_text: String,
    font_style: TextStyle,
}

const TEXT_FONT_SIZE: f32 = 18.0;
const DISPLAY_FONT_SIZE: f32 = 22.0;
const DEFAULT_BODY_FONT_SIZE: f32 = 16.0;
const TEXT_FONT_SIZE_RATIO: f32 = TEXT_FONT_SIZE / DEFAULT_BODY_FONT_SIZE;
const DISPLAY_FONT_SIZE_RATIO: f32 = DISPLAY_FONT_SIZE / DEFAULT_BODY_FONT_SIZE;
const STIX_FRACTION_HORIZONTAL_PADDING_UNITS: f32 = 180.0;
const STIX_RADICAL_BODY_GAP_UNITS: f32 = 80.0;
const FALLBACK_UNITS_PER_EM: f32 = 1000.0;

pub(crate) fn layout_latex(
    ir: &LatexIr,
    mode: LatexRenderMode,
    body_font_size_px: Option<f32>,
) -> LatexLayout {
    let context = LayoutContext {
        style: match mode {
            LatexRenderMode::Inline => MathStyle::Text,
            LatexRenderMode::Display => MathStyle::Display,
        },
        font_size: latex_font_size_for_body(mode, body_font_size_px),
        metrics: MathMetrics::stix(),
    };
    let root = layout_nodes(&ir.nodes, context);

    LatexLayout::from(root)
}

fn latex_font_size_for_body(mode: LatexRenderMode, body_font_size_px: Option<f32>) -> f32 {
    let body_font_size_px = body_font_size_px
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(DEFAULT_BODY_FONT_SIZE);
    match mode {
        LatexRenderMode::Inline => body_font_size_px * TEXT_FONT_SIZE_RATIO,
        LatexRenderMode::Display => body_font_size_px * DISPLAY_FONT_SIZE_RATIO,
    }
}

fn layout_nodes(nodes: &[IrNode], context: LayoutContext) -> LayoutBox {
    let mut width = 0.0;
    let mut ascent = context.metrics.default_ascent(context.font_size);
    let mut descent = context.metrics.default_descent(context.font_size);
    let mut children = Vec::new();
    let mut previous_class = None;

    for node in nodes {
        let current_class = node
            .atom_class()
            .map(|class| effective_atom_class(previous_class, class));
        if let (Some(left), Some(right)) = (previous_class, current_class) {
            width += context
                .metrics
                .atom_spacing(left, right, context.style, context.font_size);
        }

        let node_box = layout_node(node, context);
        ascent = f32::max(ascent, node_box.ascent);
        descent = f32::max(descent, node_box.descent);
        children.push(PositionedChild::boxed(width, 0.0, node_box.clone()));
        width += node_box.width;
        previous_class = current_class;
    }

    LayoutBox {
        width,
        ascent,
        descent,
        italic_correction: 0.0,
        children,
    }
}

fn layout_node(node: &IrNode, context: LayoutContext) -> LayoutBox {
    match node {
        IrNode::Identifier(identifier) => {
            layout_text_run(identifier, context, TextRunKind::Identifier)
        }
        IrNode::Number(number) => layout_text_run(number, context, TextRunKind::Number),
        IrNode::Operator { text, class } => {
            layout_text_run(text, context, TextRunKind::Operator(*class))
        }
        IrNode::TextOperator(operator) => {
            layout_text_run(operator, context, TextRunKind::TextOperator)
        }
        IrNode::Space(spacing) => LayoutBox::empty(
            context.font_size * spacing.em,
            context.metrics.default_ascent(context.font_size),
            context.metrics.default_descent(context.font_size),
        ),
        IrNode::Fraction {
            numerator,
            denominator,
        } => layout_fraction(numerator, denominator, context),
        IrNode::Radical { body } => layout_radical(body, context),
        IrNode::Script {
            base,
            subscript,
            superscript,
        } => layout_script(base, subscript.as_deref(), superscript.as_deref(), context),
        IrNode::Grid { kind, rows } => layout_grid(*kind, rows, context),
    }
}

fn line(x1: f32, y1: f32, x2: f32, y2: f32, stroke_width: f32) -> PositionedChild {
    PositionedChild::primitive(
        0.0,
        0.0,
        LayoutPrimitive::Line {
            x1,
            y1,
            x2,
            y2,
            stroke_width,
        },
    )
}

fn effective_atom_class(previous: Option<MathAtomClass>, current: MathAtomClass) -> MathAtomClass {
    if current != MathAtomClass::Bin {
        return current;
    }

    match previous {
        None
        | Some(MathAtomClass::Bin)
        | Some(MathAtomClass::Rel)
        | Some(MathAtomClass::Open)
        | Some(MathAtomClass::Punct) => MathAtomClass::Ord,
        _ => current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ir, parser};

    fn layout_source(source: &str, mode: LatexRenderMode) -> LatexLayout {
        let ast = parser::parse_latex(source).expect("source should parse");
        let ir = ir::lower_latex(&ast);
        layout_latex(&ir, mode, None)
    }

    fn assert_close(actual: f32, expected: f32) {
        let delta = (actual - expected).abs();
        assert!(
            delta < 0.03,
            "expected {actual:.3} to be close to {expected:.3}, delta {delta:.3}"
        );
    }

    fn text_element(layout: &LatexLayout, expected: &str) -> (f32, f32, f32, TextStyle) {
        layout
            .elements
            .iter()
            .find_map(|element| match element {
                LayoutElement::Text {
                    text,
                    x,
                    y,
                    font_size,
                    font_style,
                    ..
                } if text == expected => Some((*x, *y, *font_size, *font_style)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing text element {expected:?}: {layout:?}"))
    }

    fn line_element(layout: &LatexLayout) -> (f32, f32, f32, f32, f32) {
        layout
            .elements
            .iter()
            .find_map(|element| match element {
                LayoutElement::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    stroke_width,
                } => Some((*x1, *y1, *x2, *y2, *stroke_width)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing line element: {layout:?}"))
    }

    fn line_elements(layout: &LatexLayout) -> Vec<(f32, f32, f32, f32, f32)> {
        layout
            .elements
            .iter()
            .filter_map(|element| match element {
                LayoutElement::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    stroke_width,
                } => Some((*x1, *y1, *x2, *y2, *stroke_width)),
                _ => None,
            })
            .collect()
    }

    fn glyph_advance_px(metrics: MathMetrics, ch: char, font_size: f32) -> f32 {
        let glyph_id = metrics
            .font
            .glyph_id(ch)
            .unwrap_or_else(|| panic!("missing glyph id for {ch:?}"));
        let advance = metrics
            .font
            .glyph_advance_by_id(glyph_id)
            .unwrap_or_else(|| panic!("missing glyph advance for {ch:?}"));

        advance as f32 * font_size / metrics.font.units_per_em() as f32
    }

    fn glyph_italic_correction_px(metrics: MathMetrics, ch: char, font_size: f32) -> f32 {
        metrics
            .font
            .glyph_id(ch)
            .and_then(|glyph_id| metrics.font.glyph_italic_correction(glyph_id))
            .map(|correction| metrics.font_units_to_px(correction, font_size))
            .unwrap_or(0.0)
    }

    fn assert_superscript_geometry(source: &str, base_text: &str) {
        let layout = layout_source(source, LatexRenderMode::Inline);
        let (base_x, base_y, base_size, base_style) = text_element(&layout, base_text);
        let (sup_x, sup_y, sup_size, sup_style) = text_element(&layout, "2");
        let metrics = MathMetrics::stix();
        let base_char = base_text
            .chars()
            .next()
            .expect("base text should contain one glyph");
        let expected_sup_x = glyph_advance_px(metrics, base_char, TEXT_FONT_SIZE)
            + metrics.script_gap(TEXT_FONT_SIZE)
            + glyph_italic_correction_px(metrics, base_char, TEXT_FONT_SIZE);

        assert_close(base_x, 0.0);
        assert_close(base_y, 0.0);
        assert_close(base_size, TEXT_FONT_SIZE);
        assert_eq!(base_style, TextStyle::Normal);
        assert_close(sup_x, expected_sup_x);
        assert_close(sup_y, -9.58);
        assert_close(sup_size, 12.6);
        assert_eq!(sup_style, TextStyle::Normal);
        assert!(layout.width > sup_x, "{layout:?}");
    }

    #[test]
    fn uses_stix_metrics_for_text_and_math_constants() {
        let metrics = MathMetrics::stix();

        assert_close(metrics.default_ascent(TEXT_FONT_SIZE), 13.72);
        assert_close(metrics.default_descent(TEXT_FONT_SIZE), 4.28);
        assert_close(metrics.axis_height(TEXT_FONT_SIZE), 4.64);
        assert_close(metrics.rule_thickness(TEXT_FONT_SIZE), 1.22);
        assert_close(
            metrics.text_width("12345", TEXT_FONT_SIZE, TextRunKind::Number),
            44.55,
        );
        assert_close(
            metrics.text_width("sin", TEXT_FONT_SIZE, TextRunKind::TextOperator),
            22.61,
        );
        assert_close(
            metrics.text_width(
                "+",
                TEXT_FONT_SIZE,
                TextRunKind::Operator(MathAtomClass::Bin),
            ),
            12.96,
        );
    }

    #[test]
    fn lays_out_paired_scripts_with_stix_script_scale_and_gap() {
        let layout = layout_source(r"x_1^2", LatexRenderMode::Inline);
        let metrics = MathMetrics::stix();
        let math_x = '\u{1d465}';
        let (x_x, x_y, x_size, x_style) = text_element(&layout, "\u{1d465}");
        let (sub_x, sub_y, sub_size, sub_style) = text_element(&layout, "1");
        let (sup_x, sup_y, sup_size, sup_style) = text_element(&layout, "2");
        let expected_sub_x =
            glyph_advance_px(metrics, math_x, TEXT_FONT_SIZE) + metrics.script_gap(TEXT_FONT_SIZE);
        let expected_sup_x =
            expected_sub_x + glyph_italic_correction_px(metrics, math_x, TEXT_FONT_SIZE);

        assert_close(x_x, 0.0);
        assert_close(x_y, 0.0);
        assert_close(x_size, 18.0);
        assert_eq!(x_style, TextStyle::Normal);
        assert_close(sub_size, 12.6);
        assert_close(sup_size, 12.6);
        assert_eq!(sub_style, TextStyle::Normal);
        assert_eq!(sup_style, TextStyle::Normal);
        assert_close(sub_x, expected_sub_x);
        assert_close(sup_x, expected_sup_x);
        assert_close(sub_y, 7.16);
        assert_close(sup_y, -9.58);
        assert!(layout.ascent > 19.0, "{layout:?}");
        assert!(layout.descent > 10.0, "{layout:?}");
    }

    #[test]
    fn lays_out_latin_identifier_scripts_with_math_italic_glyphs() {
        assert_superscript_geometry(r"x^2", "\u{1d465}");
        assert_superscript_geometry(r"f^2", "\u{1d453}");
    }

    #[test]
    fn lays_out_greek_identifier_scripts_with_math_italic_glyphs() {
        assert_eq!(
            math_italic_identifier_text("\u{03b1}\u{03b2}\u{03b3}\u{03b4}"),
            "\u{1d6fc}\u{1d6fd}\u{1d6fe}\u{1d6ff}"
        );
        assert_eq!(
            math_italic_identifier_text("\u{03c0}\u{03b8}\u{03bb}\u{03bc}"),
            "\u{1d70b}\u{1d703}\u{1d706}\u{1d707}"
        );
        assert_superscript_geometry(r"\theta^2", "\u{1d703}");
    }

    #[test]
    fn keeps_integral_and_numeric_scripts_upright() {
        let layout = layout_source(r"\int_0^1", LatexRenderMode::Inline);
        let metrics = MathMetrics::stix();
        let integral = '\u{222b}';
        let (int_x, int_y, int_size, int_style) = text_element(&layout, "\u{222b}");
        let (sub_x, _, sub_size, sub_style) = text_element(&layout, "0");
        let (sup_x, _, sup_size, sup_style) = text_element(&layout, "1");
        let expected_sub_x = glyph_advance_px(metrics, integral, TEXT_FONT_SIZE)
            + metrics.script_gap(TEXT_FONT_SIZE);
        let expected_sup_x =
            expected_sub_x + glyph_italic_correction_px(metrics, integral, TEXT_FONT_SIZE);

        assert_close(int_x, 0.0);
        assert_close(int_y, 0.0);
        assert_close(int_size, TEXT_FONT_SIZE);
        assert_eq!(int_style, TextStyle::Normal);
        assert_close(sub_x, expected_sub_x);
        assert_close(sup_x, expected_sup_x);
        assert_close(sub_size, 12.6);
        assert_close(sup_size, 12.6);
        assert_eq!(sub_style, TextStyle::Normal);
        assert_eq!(sup_style, TextStyle::Normal);
    }

    #[test]
    fn keeps_scientific_notation_digits_and_signs_upright() {
        let layout = layout_source(r"10^{-3}", LatexRenderMode::Inline);
        let metrics = MathMetrics::stix();
        let (base_x, _, base_size, base_style) = text_element(&layout, "10");
        let (minus_x, _, minus_size, minus_style) = text_element(&layout, "-");
        let (three_x, _, three_size, three_style) = text_element(&layout, "3");
        let expected_script_x = metrics.text_width("10", TEXT_FONT_SIZE, TextRunKind::Number)
            + metrics.script_gap(TEXT_FONT_SIZE);

        assert_close(base_x, 0.0);
        assert_close(base_size, TEXT_FONT_SIZE);
        assert_eq!(base_style, TextStyle::Normal);
        assert_close(minus_x, expected_script_x);
        assert!(three_x > minus_x, "{layout:?}");
        assert_close(minus_size, 12.6);
        assert_close(three_size, 12.6);
        assert_eq!(minus_style, TextStyle::Normal);
        assert_eq!(three_style, TextStyle::Normal);
    }

    #[test]
    fn keeps_text_operators_upright_while_mapping_following_identifier() {
        let layout = layout_source(r"\sin x", LatexRenderMode::Inline);
        let metrics = MathMetrics::stix();
        let (sin_x, _, sin_size, sin_style) = text_element(&layout, "sin");
        let (x_x, _, x_size, x_style) = text_element(&layout, "\u{1d465}");
        let expected_x = metrics.text_width("sin", TEXT_FONT_SIZE, TextRunKind::TextOperator)
            + TEXT_FONT_SIZE * 0.28;

        assert_close(sin_x, 0.0);
        assert_close(sin_size, TEXT_FONT_SIZE);
        assert_eq!(sin_style, TextStyle::Normal);
        assert_close(x_x, expected_x);
        assert_close(x_size, TEXT_FONT_SIZE);
        assert_eq!(x_style, TextStyle::Normal);
    }

    #[test]
    fn lays_out_inline_fraction_around_the_stix_math_axis() {
        let layout = layout_source(r"\frac{1}{3}", LatexRenderMode::Inline);
        let (_, numerator_y, numerator_size, _) = text_element(&layout, "1");
        let (_, denominator_y, denominator_size, _) = text_element(&layout, "3");
        let (_, rule_y1, _, rule_y2, stroke_width) = line_element(&layout);

        assert_close(numerator_size, 12.6);
        assert_close(denominator_size, 12.6);
        assert_close(rule_y1, -4.64);
        assert_close(rule_y2, -4.64);
        assert_close(stroke_width, 1.22);
        assert!(numerator_y < rule_y1, "{layout:?}");
        assert!(denominator_y > rule_y1, "{layout:?}");
    }

    #[test]
    fn display_fraction_uses_text_style_children() {
        let layout = layout_source(r"\frac{1}{3}", LatexRenderMode::Display);
        let (_, numerator_y, numerator_size, _) = text_element(&layout, "1");
        let (_, denominator_y, denominator_size, _) = text_element(&layout, "3");
        let (_, rule_y1, _, rule_y2, stroke_width) = line_element(&layout);

        assert_close(numerator_size, 18.0);
        assert_close(denominator_size, 18.0);
        assert_close(rule_y1, -5.68);
        assert_close(rule_y2, -5.68);
        assert_close(stroke_width, 1.5);
        assert!(numerator_y < rule_y1, "{layout:?}");
        assert!(denominator_y > rule_y1, "{layout:?}");
    }

    #[test]
    fn lays_out_inline_radical_with_stix_radical_constants() {
        let layout = layout_source(r"\sqrt{x}", LatexRenderMode::Inline);
        let metrics = MathMetrics::stix();
        let (radical_x, radical_y, radical_size, radical_style) = text_element(&layout, "\u{221a}");
        let (body_x, body_y, body_size, body_style) = text_element(&layout, "\u{1d465}");
        let (bar_x1, bar_y1, bar_x2, bar_y2, stroke_width) = line_element(&layout);
        let radical_width = glyph_advance_px(metrics, '\u{221a}', TEXT_FONT_SIZE);
        let body_gap = metrics.radical_body_gap(TEXT_FONT_SIZE);
        let expected_body_x = radical_width + body_gap;
        let expected_bar_x1 = radical_width - body_gap;
        let expected_bar_y = -metrics.default_ascent(TEXT_FONT_SIZE)
            - metrics.radical_vertical_gap(MathStyle::Text, TEXT_FONT_SIZE)
            - metrics.radical_rule_thickness(TEXT_FONT_SIZE) / 2.0;
        let expected_radical_y = expected_bar_y
            + metrics
                .glyph_y_max("\u{221a}", TEXT_FONT_SIZE)
                .expect("sqrt glyph should expose a CFF yMax")
            - metrics.radical_rule_thickness(TEXT_FONT_SIZE) / 2.0;

        assert_close(radical_x, 0.0);
        assert_close(radical_y, expected_radical_y);
        assert!(radical_y > 0.0, "{layout:?}");
        assert_close(radical_size, TEXT_FONT_SIZE);
        assert_eq!(radical_style, TextStyle::Normal);
        assert_close(body_x, expected_body_x);
        assert_close(body_y, 0.0);
        assert_close(body_size, TEXT_FONT_SIZE);
        assert_eq!(body_style, TextStyle::Normal);
        assert_close(bar_x1, expected_bar_x1);
        assert!(bar_x1 < body_x, "{layout:?}");
        assert_close(bar_y1, expected_bar_y);
        assert_close(bar_y2, expected_bar_y);
        assert!(bar_x2 > body_x, "{layout:?}");
        assert_close(stroke_width, metrics.radical_rule_thickness(TEXT_FONT_SIZE));
        assert!(layout.ascent > metrics.default_ascent(TEXT_FONT_SIZE));
    }

    #[test]
    fn display_radical_uses_display_vertical_gap() {
        let layout = layout_source(r"\sqrt{x}", LatexRenderMode::Display);
        let metrics = MathMetrics::stix();
        let (_, bar_y1, _, _, stroke_width) = line_element(&layout);
        let expected_bar_y = -metrics.default_ascent(DISPLAY_FONT_SIZE)
            - metrics.radical_vertical_gap(MathStyle::Display, DISPLAY_FONT_SIZE)
            - metrics.radical_rule_thickness(DISPLAY_FONT_SIZE) / 2.0;

        assert_close(bar_y1, expected_bar_y);
        assert_close(
            stroke_width,
            metrics.radical_rule_thickness(DISPLAY_FONT_SIZE),
        );
    }

    #[test]
    fn radical_over_fraction_keeps_fraction_rule_and_overbar() {
        let layout = layout_source(r"\sqrt{\frac{1}{3}}", LatexRenderMode::Inline);
        let lines = line_elements(&layout);

        assert_eq!(lines.len(), 2, "{layout:?}");
        let overbar = lines
            .iter()
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .expect("radical layout should include an overbar");
        let fraction_rule = lines
            .iter()
            .max_by(|left, right| left.1.total_cmp(&right.1))
            .expect("fraction layout should include a rule");

        assert!(overbar.1 < fraction_rule.1, "{layout:?}");
        assert_close(overbar.3, overbar.1);
        assert!(overbar.0 > 0.0, "{layout:?}");
        assert_close(overbar.2, layout.width);
        assert!(layout.ascent > 20.0, "{layout:?}");
    }

    #[test]
    fn display_matrix_cells_use_text_style_size() {
        let layout = layout_source(
            r"\begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix}",
            LatexRenderMode::Display,
        );
        let text_sizes: Vec<f32> = layout
            .elements
            .iter()
            .filter_map(|element| match element {
                LayoutElement::Text { font_size, .. } => Some(*font_size),
                _ => None,
            })
            .collect();

        assert_eq!(text_sizes.len(), 4, "{layout:?}");
        for font_size in text_sizes {
            assert_close(font_size, 18.0);
        }
    }
}
