//! Matrix, cases and delimiter layout.
use super::*;

pub(super) fn layout_grid(
    kind: GridKind,
    rows: &[Vec<Vec<IrNode>>],
    context: LayoutContext,
) -> LayoutBox {
    let cell_context = context.grid_cell();
    let cell_layouts: Vec<Vec<LayoutBox>> = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|cell| layout_nodes(cell, cell_context))
                .collect()
        })
        .collect();
    let column_count = cell_layouts.iter().map(Vec::len).max().unwrap_or(0);

    if column_count == 0 {
        return LayoutBox::empty(
            0.0,
            context.metrics.default_ascent(context.font_size),
            context.metrics.default_descent(context.font_size),
        );
    }

    let mut column_widths = vec![0.0; column_count];
    for row in &cell_layouts {
        for (index, cell) in row.iter().enumerate() {
            column_widths[index] = f32::max(column_widths[index], cell.width);
        }
    }

    let row_gap = context.font_size * 0.36;
    let column_gap = match kind {
        GridKind::Aligned => context.font_size * 0.5,
        GridKind::BMatrix => context.font_size * 0.72,
    };
    let content_width =
        column_widths.iter().sum::<f32>() + column_gap * column_count.saturating_sub(1) as f32;
    let row_metrics: Vec<(f32, f32)> = cell_layouts
        .iter()
        .map(|row| {
            row.iter().fold(
                (
                    context.metrics.default_ascent(cell_context.font_size),
                    context.metrics.default_descent(cell_context.font_size),
                ),
                |acc, cell| (f32::max(acc.0, cell.ascent), f32::max(acc.1, cell.descent)),
            )
        })
        .collect();
    let content_height = row_metrics
        .iter()
        .map(|(ascent, descent)| ascent + descent)
        .sum::<f32>()
        + row_gap * row_metrics.len().saturating_sub(1) as f32;
    let ascent = content_height / 2.0;
    let descent = content_height - ascent;
    let bracket_extra = if kind == GridKind::BMatrix {
        context.font_size * 0.72
    } else {
        0.0
    };
    let mut children = Vec::new();
    let mut y = -ascent;

    for (row_index, row) in cell_layouts.iter().enumerate() {
        let (row_ascent, row_descent) = row_metrics[row_index];
        let baseline = y + row_ascent;
        let mut x = bracket_extra;

        for (column_index, column_width) in
            column_widths.iter().copied().enumerate().take(column_count)
        {
            if let Some(cell) = row.get(column_index) {
                let cell_x = match kind {
                    GridKind::Aligned if column_index % 2 == 0 && column_count > 1 => {
                        x + column_width - cell.width
                    }
                    GridKind::BMatrix => x + (column_width - cell.width) / 2.0,
                    _ => x,
                };
                children.push(PositionedChild::boxed(cell_x, baseline, cell.clone()));
            }
            x += column_width + column_gap;
        }

        y += row_ascent + row_descent + row_gap;
    }

    if kind == GridKind::BMatrix {
        add_brackets(
            &mut children,
            content_width,
            ascent,
            descent,
            context.font_size,
        );
    }

    LayoutBox {
        width: content_width + bracket_extra * 2.0,
        ascent,
        descent,
        italic_correction: 0.0,
        children,
    }
}

pub(super) fn add_brackets(
    children: &mut Vec<PositionedChild>,
    content_width: f32,
    ascent: f32,
    descent: f32,
    font_size: f32,
) {
    let bracket_x = font_size * 0.18;
    let bracket_tick = font_size * 0.28;
    let content_offset = font_size * 0.72;
    let right_x = content_offset + content_width + font_size * 0.54;
    let top = -ascent;
    let bottom = descent;
    let stroke_width = f32::max(1.0, font_size * 0.055);

    children.push(line(bracket_x, top, bracket_x, bottom, stroke_width));
    children.push(line(
        bracket_x,
        top,
        bracket_x + bracket_tick,
        top,
        stroke_width,
    ));
    children.push(line(
        bracket_x,
        bottom,
        bracket_x + bracket_tick,
        bottom,
        stroke_width,
    ));
    children.push(line(right_x, top, right_x, bottom, stroke_width));
    children.push(line(
        right_x - bracket_tick,
        top,
        right_x,
        top,
        stroke_width,
    ));
    children.push(line(
        right_x - bracket_tick,
        bottom,
        right_x,
        bottom,
        stroke_width,
    ));
}
