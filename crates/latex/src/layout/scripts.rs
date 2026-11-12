//! Subscript and superscript placement against math metrics.
use super::*;

pub(super) fn layout_script(
    base: &IrNode,
    subscript: Option<&[IrNode]>,
    superscript: Option<&[IrNode]>,
    context: LayoutContext,
) -> LayoutBox {
    let base = layout_node(base, context);
    let script_context = context.script_child();
    let subscript = subscript.map(|nodes| layout_nodes(nodes, script_context));
    let superscript = superscript.map(|nodes| layout_nodes(nodes, script_context));
    let script_gap = context.metrics.script_gap(context.font_size);
    let sup_x = base.width + script_gap + base.italic_correction;
    let sub_x = base.width + script_gap;
    let mut width = base.width;
    let mut ascent = base.ascent;
    let mut descent = base.descent;
    let mut children = vec![PositionedChild::boxed(0.0, 0.0, base.clone())];

    let superscript_y = superscript.as_ref().map(|superscript| {
        -context
            .metrics
            .superscript_shift(&base, superscript, context.font_size)
    });
    let mut subscript_y = subscript.as_ref().map(|subscript| {
        context
            .metrics
            .subscript_shift(&base, subscript, context.font_size)
    });

    if let (Some(superscript), Some(subscript), Some(sup_y), Some(sub_y)) =
        (&superscript, &subscript, superscript_y, subscript_y)
    {
        let minimum_gap = context.metrics.sub_superscript_gap(context.font_size);
        let sup_bottom = sup_y + superscript.descent;
        let sub_top = sub_y - subscript.ascent;
        let actual_gap = sub_top - sup_bottom;
        if actual_gap < minimum_gap {
            subscript_y = Some(sub_y + minimum_gap - actual_gap);
        }
    }

    if let (Some(superscript), Some(y)) = (&superscript, superscript_y) {
        children.push(PositionedChild::boxed(sup_x, y, superscript.clone()));
        ascent = f32::max(ascent, -y + superscript.ascent);
        width = f32::max(width, sup_x + superscript.width);
    }

    if let (Some(subscript), Some(y)) = (&subscript, subscript_y) {
        children.push(PositionedChild::boxed(sub_x, y, subscript.clone()));
        descent = f32::max(descent, y + subscript.descent);
        width = f32::max(width, sub_x + subscript.width);
    }

    LayoutBox {
        width,
        ascent,
        descent,
        italic_correction: 0.0,
        children,
    }
}
