//! Layout-box flattening and positioned primitive projection.
use super::*;

impl LayoutBox {
    pub(super) fn empty(width: f32, ascent: f32, descent: f32) -> Self {
        Self {
            width,
            ascent,
            descent,
            italic_correction: 0.0,
            children: Vec::new(),
        }
    }

    pub(super) fn flatten_into(&self, target: &mut Vec<LayoutElement>, dx: f32, dy: f32) {
        for child in &self.children {
            child.child.flatten_into(target, dx + child.x, dy + child.y);
        }
    }
}

impl PositionedChild {
    pub(super) fn boxed(x: f32, y: f32, child: LayoutBox) -> Self {
        Self {
            x,
            y,
            child: LayoutChild::Box(Box::new(child)),
        }
    }

    pub(super) fn primitive(x: f32, y: f32, primitive: LayoutPrimitive) -> Self {
        Self {
            x,
            y,
            child: LayoutChild::Primitive(primitive),
        }
    }
}

impl LayoutChild {
    pub(super) fn flatten_into(&self, target: &mut Vec<LayoutElement>, dx: f32, dy: f32) {
        match self {
            Self::Box(layout) => layout.flatten_into(target, dx, dy),
            Self::Primitive(primitive) => primitive.flatten_into(target, dx, dy),
        }
    }
}

impl LayoutPrimitive {
    pub(super) fn flatten_into(&self, target: &mut Vec<LayoutElement>, dx: f32, dy: f32) {
        match self {
            Self::Text {
                text,
                font_size,
                font_style,
            } => target.push(LayoutElement::Text {
                text: text.clone(),
                x: dx,
                y: dy,
                font_size: *font_size,
                font_style: *font_style,
            }),
            Self::Line {
                x1,
                y1,
                x2,
                y2,
                stroke_width,
            } => target.push(LayoutElement::Line {
                x1: dx + x1,
                y1: dy + y1,
                x2: dx + x2,
                y2: dy + y2,
                stroke_width: *stroke_width,
            }),
        }
    }
}

impl From<LayoutBox> for LatexLayout {
    fn from(layout: LayoutBox) -> Self {
        let mut elements = Vec::new();
        layout.flatten_into(&mut elements, 0.0, 0.0);
        Self {
            width: layout.width,
            ascent: layout.ascent,
            descent: layout.descent,
            elements,
        }
    }
}
