#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiagramStyle {
    pub corner_radius: Option<DiagramCornerRadius>,
    pub arrowhead: Option<DiagramArrowheadStyle>,
    pub stroke_width: Option<DiagramStrokeWidth>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramCornerRadius {
    None,
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramArrowheadStyle {
    Triangle,
    Chevron,
    Kite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagramStrokeWidth {
    Thin,
    Regular,
    Bold,
}
