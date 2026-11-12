//! Math text runs, italic identifiers and fallback glyph advances.
use super::*;

pub(super) fn layout_text_run(text: &str, context: LayoutContext, kind: TextRunKind) -> LayoutBox {
    let run = prepare_text_run(text, kind);
    let width = context
        .metrics
        .text_width(&run.measurement_text, context.font_size, kind);
    let italic_correction =
        context
            .metrics
            .italic_correction(&run.measurement_text, context.font_size, kind);

    LayoutBox {
        width,
        ascent: context.metrics.default_ascent(context.font_size),
        descent: context.metrics.default_descent(context.font_size),
        italic_correction,
        children: vec![PositionedChild::primitive(
            0.0,
            0.0,
            LayoutPrimitive::Text {
                text: run.display_text,
                font_size: context.font_size,
                font_style: run.font_style,
            },
        )],
    }
}

pub(super) fn prepare_text_run(text: &str, kind: TextRunKind) -> PreparedTextRun {
    match kind {
        TextRunKind::Identifier => {
            let display_text = math_italic_identifier_text(text);
            PreparedTextRun {
                measurement_text: display_text.clone(),
                display_text,
                font_style: TextStyle::Normal,
            }
        }
        TextRunKind::Number | TextRunKind::Operator(_) | TextRunKind::TextOperator => {
            PreparedTextRun {
                display_text: text.to_owned(),
                measurement_text: text.to_owned(),
                font_style: TextStyle::Normal,
            }
        }
    }
}

pub(super) fn math_italic_identifier_text(text: &str) -> String {
    text.chars().map(math_italic_identifier_char).collect()
}

pub(super) fn math_italic_identifier_char(ch: char) -> char {
    if ch.is_ascii_uppercase() {
        return char::from_u32(0x1d434 + ch as u32 - 'A' as u32)
            .expect("mathematical italic uppercase codepoint should be valid");
    }

    if ch.is_ascii_lowercase() {
        if ch == 'h' {
            return '\u{210e}';
        }

        return char::from_u32(0x1d44e + ch as u32 - 'a' as u32)
            .expect("mathematical italic lowercase codepoint should be valid");
    }

    match ch {
        '\u{03b1}' => '\u{1d6fc}',
        '\u{03b2}' => '\u{1d6fd}',
        '\u{03b3}' => '\u{1d6fe}',
        '\u{03b4}' => '\u{1d6ff}',
        '\u{03b8}' => '\u{1d703}',
        '\u{03bb}' => '\u{1d706}',
        '\u{03bc}' => '\u{1d707}',
        '\u{03c0}' => '\u{1d70b}',
        _ => ch,
    }
}

pub(super) fn fallback_advance_em(ch: char, kind: TextRunKind) -> f32 {
    if ch.is_ascii_digit() {
        return 495.0 / FALLBACK_UNITS_PER_EM;
    }

    match kind {
        TextRunKind::Identifier => 560.0 / FALLBACK_UNITS_PER_EM,
        TextRunKind::Number => 495.0 / FALLBACK_UNITS_PER_EM,
        TextRunKind::TextOperator => 500.0 / FALLBACK_UNITS_PER_EM,
        TextRunKind::Operator(MathAtomClass::Open | MathAtomClass::Close) => {
            350.0 / FALLBACK_UNITS_PER_EM
        }
        TextRunKind::Operator(MathAtomClass::Punct) => 245.0 / FALLBACK_UNITS_PER_EM,
        TextRunKind::Operator(MathAtomClass::Bin | MathAtomClass::Rel) => {
            720.0 / FALLBACK_UNITS_PER_EM
        }
        TextRunKind::Operator(MathAtomClass::Op) => 700.0 / FALLBACK_UNITS_PER_EM,
        TextRunKind::Operator(MathAtomClass::Ord | MathAtomClass::Inner) => {
            560.0 / FALLBACK_UNITS_PER_EM
        }
    }
}
