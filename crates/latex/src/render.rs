use crate::layout::{LatexLayout, LayoutElement, TextStyle};
use crate::LatexRenderMode;

const FONT_STACK: &str = "'ORIHA STIX Two Math', 'STIX Two Math', 'Cambria Math', serif";

pub(crate) fn render_svg(layout: &LatexLayout, mode: LatexRenderMode) -> String {
    let padding = match mode {
        LatexRenderMode::Inline => 2.0,
        LatexRenderMode::Display => 6.0,
    };
    let width = f32::max(1.0, layout.width + padding * 2.0);
    let height = f32::max(1.0, layout.ascent + layout.descent + padding * 2.0);
    let baseline = padding + layout.ascent;
    let mut svg = String::new();

    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" data-latex-mode=\"{}\" role=\"img\">",
        number(width),
        number(height),
        number(width),
        number(height),
        match mode {
            LatexRenderMode::Inline => "inline",
            LatexRenderMode::Display => "display",
        }
    ));
    svg.push_str("<title>LaTeX math</title>");

    for element in &layout.elements {
        match element {
            LayoutElement::Text {
                text,
                x,
                y,
                font_size,
                font_style,
            } => {
                let font_style = match font_style {
                    TextStyle::Normal => "",
                    TextStyle::Italic => " font-style=\"italic\"",
                };
                svg.push_str(&format!(
                    "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\"{} fill=\"#111827\">{}</text>",
                    number(padding + x),
                    number(baseline + y),
                    FONT_STACK,
                    number(*font_size),
                    font_style,
                    escape_text(text)
                ));
            }
            LayoutElement::Line {
                x1,
                y1,
                x2,
                y2,
                stroke_width,
            } => {
                svg.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#111827\" stroke-width=\"{}\" stroke-linecap=\"butt\" />",
                    number(padding + x1),
                    number(baseline + y1),
                    number(padding + x2),
                    number(baseline + y2),
                    number(*stroke_width)
                ));
            }
        }
    }

    svg.push_str("</svg>");
    svg
}

fn escape_text(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            ch if ch.is_ascii() => escaped.push(ch),
            ch => escaped.push_str(&format!("&#x{:x};", ch as u32)),
        }
    }
    escaped
}

fn number(value: f32) -> String {
    let mut text = format!("{value:.2}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" {
        "0".to_owned()
    } else {
        text
    }
}
