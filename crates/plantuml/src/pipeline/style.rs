//! Public diagram-style conversion and font-size normalization.
use super::*;

pub(super) fn resolve_body_font_size_px(value: Option<f32>) -> f32 {
    const DEFAULT_BODY_FONT_SIZE_PX: f32 = 16.0;
    value
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(DEFAULT_BODY_FONT_SIZE_PX)
}

pub(super) fn sequence_style_from_diagram_style(style: DiagramStyle) -> PlantUmlStyle {
    let default = PlantUmlStyle::default();
    PlantUmlStyle {
        corner_radius: style
            .corner_radius
            .map(render_corner_radius)
            .unwrap_or(default.corner_radius),
        arrowhead: style
            .arrowhead
            .map(render_arrowhead_style)
            .unwrap_or(default.arrowhead),
        stroke_width: style
            .stroke_width
            .map(render_stroke_width)
            .unwrap_or(default.stroke_width),
    }
}

pub(super) fn diagram_style_from_sequence_style(style: &PlantUmlStyle) -> DiagramStyle {
    DiagramStyle {
        corner_radius: Some(diagram_corner_radius(style.corner_radius)),
        arrowhead: Some(diagram_arrowhead_style(style.arrowhead)),
        stroke_width: Some(diagram_stroke_width(style.stroke_width)),
    }
}

pub(super) fn render_corner_radius(value: DiagramCornerRadius) -> crate::render::CornerRadius {
    match value {
        DiagramCornerRadius::None => crate::render::CornerRadius::None,
        DiagramCornerRadius::Xs => crate::render::CornerRadius::Xs,
        DiagramCornerRadius::Sm => crate::render::CornerRadius::Sm,
        DiagramCornerRadius::Md => crate::render::CornerRadius::Md,
        DiagramCornerRadius::Lg => crate::render::CornerRadius::Lg,
        DiagramCornerRadius::Xl => crate::render::CornerRadius::Xl,
    }
}

pub(super) fn diagram_corner_radius(value: crate::render::CornerRadius) -> DiagramCornerRadius {
    match value {
        crate::render::CornerRadius::None => DiagramCornerRadius::None,
        crate::render::CornerRadius::Xs => DiagramCornerRadius::Xs,
        crate::render::CornerRadius::Sm => DiagramCornerRadius::Sm,
        crate::render::CornerRadius::Md => DiagramCornerRadius::Md,
        crate::render::CornerRadius::Lg => DiagramCornerRadius::Lg,
        crate::render::CornerRadius::Xl => DiagramCornerRadius::Xl,
    }
}

pub(super) fn render_arrowhead_style(
    value: DiagramArrowheadStyle,
) -> crate::render::ArrowheadStyle {
    match value {
        DiagramArrowheadStyle::Triangle => crate::render::ArrowheadStyle::Triangle,
        DiagramArrowheadStyle::Chevron => crate::render::ArrowheadStyle::Chevron,
        DiagramArrowheadStyle::Kite => crate::render::ArrowheadStyle::Kite,
    }
}

pub(super) fn diagram_arrowhead_style(
    value: crate::render::ArrowheadStyle,
) -> DiagramArrowheadStyle {
    match value {
        crate::render::ArrowheadStyle::Triangle => DiagramArrowheadStyle::Triangle,
        crate::render::ArrowheadStyle::Chevron => DiagramArrowheadStyle::Chevron,
        crate::render::ArrowheadStyle::Kite => DiagramArrowheadStyle::Kite,
    }
}

pub(super) fn render_stroke_width(value: DiagramStrokeWidth) -> crate::render::StrokeWidth {
    match value {
        DiagramStrokeWidth::Thin => crate::render::StrokeWidth::Thin,
        DiagramStrokeWidth::Regular => crate::render::StrokeWidth::Regular,
        DiagramStrokeWidth::Bold => crate::render::StrokeWidth::Bold,
    }
}

pub(super) fn diagram_stroke_width(value: crate::render::StrokeWidth) -> DiagramStrokeWidth {
    match value {
        crate::render::StrokeWidth::Thin => DiagramStrokeWidth::Thin,
        crate::render::StrokeWidth::Regular => DiagramStrokeWidth::Regular,
        crate::render::StrokeWidth::Bold => DiagramStrokeWidth::Bold,
    }
}
