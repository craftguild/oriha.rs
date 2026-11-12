//! Fraction rules, baselines and radical placement.
use super::*;

pub(super) fn layout_fraction(
    numerator: &[IrNode],
    denominator: &[IrNode],
    context: LayoutContext,
) -> LayoutBox {
    let child_context = context.fraction_child();
    let numerator = layout_nodes(numerator, child_context);
    let denominator = layout_nodes(denominator, child_context);
    let horizontal_pad = context
        .metrics
        .fraction_horizontal_padding(context.font_size);
    let width = f32::max(numerator.width, denominator.width) + horizontal_pad * 2.0;
    let axis_y = -context.metrics.axis_height(context.font_size);
    let rule_thickness = context.metrics.rule_thickness(context.font_size);
    let numerator_gap_min = context
        .metrics
        .fraction_numerator_gap(context.style, context.font_size);
    let denominator_gap_min = context
        .metrics
        .fraction_denominator_gap(context.style, context.font_size);
    let mut numerator_y = -context
        .metrics
        .fraction_numerator_shift(context.style, context.font_size);
    let mut denominator_y = context
        .metrics
        .fraction_denominator_shift(context.style, context.font_size);
    let mut children = Vec::new();

    let numerator_gap = axis_y - rule_thickness / 2.0 - (numerator_y + numerator.descent);
    if numerator_gap < numerator_gap_min {
        numerator_y -= numerator_gap_min - numerator_gap;
    }

    let denominator_gap = (denominator_y - denominator.ascent) - (axis_y + rule_thickness / 2.0);
    if denominator_gap < denominator_gap_min {
        denominator_y += denominator_gap_min - denominator_gap;
    }

    children.push(PositionedChild::boxed(
        (width - numerator.width) / 2.0,
        numerator_y,
        numerator.clone(),
    ));
    children.push(PositionedChild::boxed(
        (width - denominator.width) / 2.0,
        denominator_y,
        denominator.clone(),
    ));
    children.push(PositionedChild::primitive(
        0.0,
        0.0,
        LayoutPrimitive::Line {
            x1: horizontal_pad * 0.5,
            y1: axis_y,
            x2: width - horizontal_pad * 0.5,
            y2: axis_y,
            stroke_width: rule_thickness,
        },
    ));

    LayoutBox {
        width,
        ascent: f32::max(
            -numerator_y + numerator.ascent,
            -axis_y + rule_thickness / 2.0,
        ),
        descent: f32::max(
            denominator_y + denominator.descent,
            axis_y + rule_thickness / 2.0,
        ),
        italic_correction: 0.0,
        children,
    }
}

pub(super) fn layout_radical(body: &[IrNode], context: LayoutContext) -> LayoutBox {
    let radical = layout_text_run(
        "\u{221a}",
        context,
        TextRunKind::Operator(MathAtomClass::Open),
    );
    let body = layout_nodes(body, context);
    let body_gap = context.metrics.radical_body_gap(context.font_size);
    let body_x = radical.width + body_gap;
    let rule_thickness = context.metrics.radical_rule_thickness(context.font_size);
    let vertical_gap = context
        .metrics
        .radical_vertical_gap(context.style, context.font_size);
    let extra_ascender = context.metrics.radical_extra_ascender(context.font_size);
    let bar_y = -body.ascent - vertical_gap - rule_thickness / 2.0;
    let radical_top = context
        .metrics
        .glyph_y_max("\u{221a}", context.font_size)
        .unwrap_or(radical.ascent);
    let radical_y = bar_y + radical_top - rule_thickness / 2.0;
    let bar_x1 = f32::max(0.0, radical.width - body_gap);
    let width = body_x + body.width;
    let mut children = vec![
        PositionedChild::boxed(0.0, radical_y, radical.clone()),
        PositionedChild::boxed(body_x, 0.0, body.clone()),
    ];

    children.push(PositionedChild::primitive(
        0.0,
        0.0,
        LayoutPrimitive::Line {
            x1: bar_x1,
            y1: bar_y,
            x2: width,
            y2: bar_y,
            stroke_width: rule_thickness,
        },
    ));

    LayoutBox {
        width,
        ascent: f32::max(
            -radical_y + radical.ascent,
            body.ascent + vertical_gap + rule_thickness + extra_ascender,
        ),
        descent: f32::max(f32::max(0.0, radical_y + radical.descent), body.descent),
        italic_correction: 0.0,
        children,
    }
}
