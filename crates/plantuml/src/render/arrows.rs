//! Message endpoint arrow geometry and decoration.
use super::*;

pub(super) fn render_message_arrow_sides(
    svg: &mut String,
    message: &MessageLayout,
    start_side: ArrowSide,
    end_side: ArrowSide,
    color: &str,
    style: &PlantUmlStyle,
) {
    let direction = message_line_direction(message);
    render_arrow_side(
        svg,
        start_side,
        message.start_x,
        message.y,
        -direction,
        color,
        style,
    );
    render_arrow_side(
        svg,
        end_side,
        message.end_x,
        message.y,
        direction,
        color,
        style,
    );
}

pub(super) fn render_arrow_side(
    svg: &mut String,
    side: ArrowSide,
    anchor_x: f32,
    anchor_y: f32,
    direction: f32,
    color: &str,
    style: &PlantUmlStyle,
) {
    if side.cross {
        render_arrow_cross(svg, anchor_x, anchor_y, color, style);
    }
    if side.circle {
        render_arrow_circle(svg, anchor_x, anchor_y, color, style);
    }
    if let Some(head) = side.head {
        let tip_x = if side.circle {
            anchor_x - direction * ARROW_DECORATION_GAP
        } else {
            anchor_x
        };
        render_arrowhead_at(svg, head, tip_x, anchor_y, direction, color, style);
    }
}

pub(super) fn render_arrowhead_at(
    svg: &mut String,
    head: MessageArrowHead,
    tip_x: f32,
    tip_y: f32,
    direction: f32,
    color: &str,
    style: &PlantUmlStyle,
) {
    let stroke_width = style.stroke_width.svg_px();
    let head_length = style.stroke_width.arrowhead_length();
    let head_half_height = style.stroke_width.arrowhead_half_height();
    let base_x = tip_x - direction * head_length;
    let upper_y = tip_y - head_half_height;
    let lower_y = tip_y + head_half_height;

    match head {
        MessageArrowHead::Filled => {
            render_styled_arrowhead_at(svg, tip_x, tip_y, direction, color, style)
        }
        MessageArrowHead::Open => {
            write!(
                svg,
                r#"<path class="arrowhead arrowhead--open" d="M {},{} L {},{} L {},{}" fill="none" stroke="{}" stroke-width="{stroke_width}" stroke-linecap="round" stroke-linejoin="round"/>"#,
                format_number(base_x),
                format_number(upper_y),
                format_number(tip_x),
                format_number(tip_y),
                format_number(base_x),
                format_number(lower_y),
                escape_attr(color),
            )
            .expect("writing to String cannot fail");
        }
        MessageArrowHead::HalfTop | MessageArrowHead::HalfBottom => {
            let diagonal_y = match head {
                MessageArrowHead::HalfTop => tip_y + direction * head_half_height,
                MessageArrowHead::HalfBottom => tip_y - direction * head_half_height,
                MessageArrowHead::Filled | MessageArrowHead::Open => unreachable!(),
            };
            let class_name = arrow_head_name(head);
            write!(
                svg,
                r#"<polygon class="arrowhead arrowhead--{class_name}" points="{},{} {},{} {},{}" fill="{}" stroke="{}" stroke-width="{stroke_width}"/>"#,
                format_number(base_x),
                format_number(diagonal_y),
                format_number(tip_x),
                format_number(tip_y),
                format_number(base_x),
                format_number(tip_y),
                escape_attr(color),
                escape_attr(color),
            )
            .expect("writing to String cannot fail");
        }
    }
}

pub(super) fn render_styled_arrowhead_at(
    svg: &mut String,
    tip_x: f32,
    tip_y: f32,
    direction: f32,
    color: &str,
    style: &PlantUmlStyle,
) {
    let stroke_width = style.stroke_width.svg_px();
    let head_length = style.stroke_width.arrowhead_length();
    let head_half_height = style.stroke_width.arrowhead_half_height();
    let base_x = tip_x - direction * head_length;
    let upper_y = tip_y - head_half_height;
    let lower_y = tip_y + head_half_height;

    match style.arrowhead {
        ArrowheadStyle::Triangle => {
            write!(
                svg,
                r#"<polygon class="arrowhead arrowhead--triangle" points="{},{} {},{} {},{}" fill="{}" stroke="{}" stroke-width="{stroke_width}"/>"#,
                format_number(tip_x),
                format_number(tip_y),
                format_number(base_x),
                format_number(upper_y),
                format_number(base_x),
                format_number(lower_y),
                escape_attr(color),
                escape_attr(color),
            )
            .expect("writing to String cannot fail");
        }
        ArrowheadStyle::Chevron => {
            write!(
                svg,
                r#"<path class="arrowhead arrowhead--chevron" d="M {},{} L {},{} L {},{}" fill="none" stroke="{}" stroke-width="{stroke_width}" stroke-linecap="round" stroke-linejoin="round"/>"#,
                format_number(base_x),
                format_number(upper_y),
                format_number(tip_x),
                format_number(tip_y),
                format_number(base_x),
                format_number(lower_y),
                escape_attr(color),
            )
            .expect("writing to String cannot fail");
        }
        ArrowheadStyle::Kite => {
            let tail_x = tip_x - direction * (head_length * 0.55);
            write!(
                svg,
                r#"<path class="arrowhead arrowhead--kite" d="M {},{} L {},{} L {},{} L {},{} Z" fill="{}" stroke="{}" stroke-width="{stroke_width}" stroke-linejoin="round"/>"#,
                format_number(tip_x),
                format_number(tip_y),
                format_number(base_x),
                format_number(upper_y),
                format_number(tail_x),
                format_number(tip_y),
                format_number(base_x),
                format_number(lower_y),
                escape_attr(color),
                escape_attr(color),
            )
            .expect("writing to String cannot fail");
        }
    }
}

pub(super) fn render_arrow_circle(
    svg: &mut String,
    cx: f32,
    cy: f32,
    color: &str,
    style: &PlantUmlStyle,
) {
    write!(
        svg,
        r#"<circle class="arrowhead arrowhead--circle" cx="{}" cy="{}" r="{}" fill="{}" stroke="{}" stroke-width="{}"/>"#,
        format_number(cx),
        format_number(cy),
        format_number(ARROW_CIRCLE_RADIUS),
        escape_attr(color),
        escape_attr(color),
        style.stroke_width.svg_px(),
    )
    .expect("writing to String cannot fail");
}

pub(super) fn render_arrow_cross(
    svg: &mut String,
    cx: f32,
    cy: f32,
    color: &str,
    style: &PlantUmlStyle,
) {
    let size = ARROW_CROSS_SIZE;
    let stroke_width = style.stroke_width.svg_px().max(2);
    write!(
        svg,
        r#"<g class="arrowhead arrowhead--cross"><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{stroke_width}"/><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{stroke_width}"/></g>"#,
        format_number(cx - size),
        format_number(cy - size),
        format_number(cx + size),
        format_number(cy + size),
        escape_attr(color),
        format_number(cx - size),
        format_number(cy + size),
        format_number(cx + size),
        format_number(cy - size),
        escape_attr(color),
    )
    .expect("writing to String cannot fail");
}

pub(super) fn message_start_arrow_side(message: &MessageLayout) -> ArrowSide {
    let side = ArrowSide {
        head: message.arrow_features.start_head,
        circle: message.arrow_features.start_circle,
        cross: message.arrow_features.start_cross,
    };
    if side == ArrowSide::default()
        && message.arrow_features.bidirectional
        && !matches!(message.direction, MessageDirection::SelfMessage)
    {
        ArrowSide {
            head: Some(MessageArrowHead::Filled),
            ..ArrowSide::default()
        }
    } else {
        side
    }
}

pub(super) fn message_end_arrow_side(message: &MessageLayout) -> ArrowSide {
    let side = ArrowSide {
        head: message.arrow_features.end_head,
        circle: message.arrow_features.end_circle,
        cross: message.arrow_features.end_cross,
    };
    if side == ArrowSide::default() {
        ArrowSide {
            head: Some(MessageArrowHead::Filled),
            ..ArrowSide::default()
        }
    } else {
        side
    }
}

pub(super) fn message_line_direction(message: &MessageLayout) -> f32 {
    if message.end_x >= message.start_x {
        1.0
    } else {
        -1.0
    }
}

pub(super) fn arrow_side_line_inset(side: ArrowSide, style: &PlantUmlStyle) -> f32 {
    let mut inset: f32 = 0.0;
    if side.circle {
        inset = inset.max(ARROW_DECORATION_GAP + ARROW_CIRCLE_RADIUS);
    }
    if let Some(head) = side.head {
        let head_inset = match head {
            MessageArrowHead::Filled if matches!(style.arrowhead, ArrowheadStyle::Chevron) => 0.0,
            MessageArrowHead::Filled => ARROW_DECORATION_GAP,
            MessageArrowHead::Open | MessageArrowHead::HalfTop | MessageArrowHead::HalfBottom => {
                0.0
            }
        };
        inset = inset.max(head_inset);
    }
    if side.circle && side.head.is_some() {
        inset = inset.max(ARROW_DECORATION_GAP * 2.0);
    }
    inset
}
