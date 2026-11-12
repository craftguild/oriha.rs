use std::fmt;

mod ast;
pub mod font;
mod ir;
mod layout;
mod opentype;
mod parser;
mod render;

pub use font::{
    math_font_base64, math_font_bytes, math_font_face_css, MATH_FONT_ASSET_ID, MATH_FONT_FAMILY,
    MATH_FONT_FORMAT, MATH_FONT_MIME_TYPE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedLatex {
    pub svg: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatexRenderMode {
    Inline,
    Display,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LatexRenderError {
    EmptySource,
    Parse { message: String, position: usize },
    UnsupportedCommand(String),
    UnsupportedEnvironment(String),
}

impl fmt::Display for LatexRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySource => f.write_str("LaTeX source did not contain math"),
            Self::Parse { message, position } => {
                write!(f, "LaTeX parse error at character {position}: {message}")
            }
            Self::UnsupportedCommand(command) => {
                write!(f, "unsupported LaTeX command: \\{command}")
            }
            Self::UnsupportedEnvironment(environment) => {
                write!(f, "unsupported LaTeX environment: {environment}")
            }
        }
    }
}

impl std::error::Error for LatexRenderError {}

pub fn render_latex_svg(
    source: &str,
    mode: LatexRenderMode,
    body_font_size_px: Option<f32>,
) -> Result<RenderedLatex, LatexRenderError> {
    let ast = parser::parse_latex(source)?;
    let ir = ir::lower_latex(&ast);
    let layout = layout::layout_latex(&ir, mode, body_font_size_px);
    let svg = render::render_svg(&layout, mode);

    Ok(RenderedLatex { svg })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{MathAtomClass, MathNode};
    use crate::font::{
        math_font_base64, math_font_bytes, math_font_face_css, MATH_FONT_ASSET_ID,
        MATH_FONT_FAMILY, MATH_FONT_FORMAT, MATH_FONT_MIME_TYPE,
    };
    use crate::ir::IrNode;

    fn render_sample(source: &str, mode: LatexRenderMode) -> String {
        render_latex_svg(source, mode, None)
            .expect("sample should render")
            .svg
    }

    #[test]
    fn renders_expo_inline_samples() {
        let energy = render_sample(r"$E = mc^2$", LatexRenderMode::Inline);
        assert!(energy.starts_with("<svg "));
        assert!(energy.contains("&#x1d438;"));
        assert!(energy.contains("&#x1d45a;"));
        assert!(energy.contains("&#x1d450;"));
        assert!(energy.contains(">2<"));
        assert!(energy.contains("data-latex-mode=\"inline\""));
        assert!(energy.contains(
            "font-family=\"'ORIHA STIX Two Math', 'STIX Two Math', 'Cambria Math', serif\""
        ));

        let greek = render_sample(r"\alpha + \beta = \gamma", LatexRenderMode::Inline);
        assert!(greek.contains("&#x1d6fc;"));
        assert!(greek.contains("&#x1d6fd;"));
        assert!(greek.contains("&#x1d6fe;"));

        let summation = render_sample(
            r"\sum_{i=1}^{n} i = \frac{n(n+1)}{2}",
            LatexRenderMode::Inline,
        );
        assert!(summation.contains("&#x2211;"));
        assert!(summation.contains("<line "));

        let radical = render_sample(r"\sqrt{x^2 + 1}", LatexRenderMode::Inline);
        assert!(radical.contains("&#x221a;"));
        assert!(radical.contains("&#x1d465;"));
        assert!(radical.contains("<line "));
    }

    #[test]
    fn body_font_size_scales_rendered_math_font_size() {
        let inline = render_latex_svg("E = mc^2", LatexRenderMode::Inline, Some(20.0))
            .expect("inline math should render")
            .svg;
        let display = render_latex_svg("a + b = c", LatexRenderMode::Display, Some(20.0))
            .expect("display math should render")
            .svg;

        assert!(inline.contains(r#"font-size="22.5""#), "{inline}");
        assert!(display.contains(r#"font-size="27.5""#), "{display}");
    }

    #[test]
    fn missing_or_invalid_body_font_size_uses_default_math_font_size() {
        let none = render_latex_svg("E = mc^2", LatexRenderMode::Inline, None)
            .expect("inline math should render")
            .svg;
        let invalid = render_latex_svg("E = mc^2", LatexRenderMode::Inline, Some(f32::NAN))
            .expect("inline math should render")
            .svg;

        assert!(none.contains(r#"font-size="18""#), "{none}");
        assert!(invalid.contains(r#"font-size="18""#), "{invalid}");
    }

    #[test]
    fn renders_expo_display_samples() {
        let integral = render_sample(r"\int_0^1 x^2\,dx = \frac{1}{3}", LatexRenderMode::Display);
        assert!(integral.contains("&#x222b;"));
        assert!(integral.contains("data-latex-mode=\"display\""));

        let aligned = render_sample(
            r"\begin{aligned} a &= b + c \\ d &= e + f \end{aligned}",
            LatexRenderMode::Display,
        );
        assert!(aligned.contains("&#x1d44e;"));
        assert!(aligned.contains("&#x1d451;"));
        assert!(aligned.contains(">=<"));

        let matrix = render_sample(
            r"\begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix}",
            LatexRenderMode::Display,
        );
        assert!(matrix.contains(">1<"));
        assert!(matrix.contains("<line "));
    }

    #[test]
    fn reports_unsupported_environment_and_malformed_sqrt_degree() {
        let environment =
            render_latex_svg(r"\begin{cases}x\end{cases}", LatexRenderMode::Display, None)
                .unwrap_err();
        assert_eq!(
            environment,
            LatexRenderError::UnsupportedEnvironment("cases".to_owned())
        );

        let degree = render_latex_svg(r"\sqrt[3]{x}", LatexRenderMode::Inline, None).unwrap_err();
        assert!(matches!(degree, LatexRenderError::Parse { .. }));
    }

    #[test]
    fn parser_keeps_script_pair_with_number_runs() {
        let ast = parser::parse_latex(r"x_1^2 + 10^{-3}").expect("source should parse");

        let MathNode::Script {
            base,
            subscript,
            superscript,
        } = &ast.nodes[0]
        else {
            panic!("x_1^2 should parse as a paired script node: {ast:?}");
        };
        assert_eq!(**base, MathNode::Identifier("x".to_owned()));
        assert_eq!(
            subscript.as_ref().expect("subscript should exist"),
            &vec![MathNode::Number("1".to_owned())]
        );
        assert_eq!(
            superscript.as_ref().expect("superscript should exist"),
            &vec![MathNode::Number("2".to_owned())]
        );

        assert!(
            ast.nodes.iter().any(|node| matches!(
                node,
                MathNode::Script { base, .. }
                    if matches!(base.as_ref(), MathNode::Number(number) if number == "10")
            )),
            "10 should stay as a single number run before superscript layout: {ast:?}"
        );
    }

    #[test]
    fn parser_classifies_math_atom_kinds() {
        let ast = parser::parse_latex(r"a+b=c").expect("source should parse");
        assert_eq!(ast.nodes[0], MathNode::Identifier("a".to_owned()));
        assert_eq!(
            ast.nodes[1],
            MathNode::Operator {
                text: "+".to_owned(),
                class: MathAtomClass::Bin,
            }
        );
        assert_eq!(
            ast.nodes[3],
            MathNode::Operator {
                text: "=".to_owned(),
                class: MathAtomClass::Rel,
            }
        );

        let trig = parser::parse_latex(r"\sin x + \cos\theta").expect("source should parse");
        assert!(matches!(trig.nodes[0], MathNode::TextOperator(ref text) if text == "sin"));
        assert!(trig
            .nodes
            .iter()
            .any(|node| matches!(node, MathNode::TextOperator(text) if text == "cos")));
        assert!(trig
            .nodes
            .iter()
            .any(|node| matches!(node, MathNode::Identifier(text) if text == "\u{03b8}")));

        let radical = parser::parse_latex(r"\sqrt{x^2 + 1}").expect("source should parse");
        assert!(matches!(radical.nodes[0], MathNode::Radical { .. }));
    }

    #[test]
    fn lower_latex_preserves_semantic_math_atoms() {
        let ast = parser::parse_latex(r"12345 + \sin x + \sqrt{x}").expect("source should parse");
        let ir = ir::lower_latex(&ast);

        assert_eq!(ir.nodes[0], IrNode::Number("12345".to_owned()));
        assert_eq!(
            ir.nodes[2],
            IrNode::Operator {
                text: "+".to_owned(),
                class: MathAtomClass::Bin,
            }
        );
        assert!(ir
            .nodes
            .iter()
            .any(|node| matches!(node, IrNode::TextOperator(text) if text == "sin")));
        assert!(ir
            .nodes
            .iter()
            .any(|node| matches!(node, IrNode::Radical { .. })));
    }

    #[test]
    fn renderer_marks_identifiers_as_math_italic_glyphs_but_numbers_upright() {
        let svg = render_sample(r"x_1^2 + 12345", LatexRenderMode::Inline);

        assert!(svg.contains("fill=\"#111827\">&#x1d465;</text>"));
        assert!(!svg.contains("font-style=\"italic\""));
        assert!(svg.contains(">12345</text>"));
        assert!(!svg.contains("font-style=\"italic\" fill=\"#111827\">12345</text>"));
    }

    #[test]
    fn exposes_bundled_math_font_api_and_css() {
        assert_eq!(MATH_FONT_FAMILY, "ORIHA STIX Two Math");
        assert_eq!(MATH_FONT_ASSET_ID, "oriha-stix-two-math");
        assert_eq!(MATH_FONT_MIME_TYPE, "font/otf");
        assert_eq!(MATH_FONT_FORMAT, "opentype");
        assert_eq!(math_font_bytes().len(), 838_652);

        let encoded = math_font_base64();
        assert!(encoded.starts_with("T1RUTwAOAIAAAwBgQ0ZGIBNU760AAF"));
        assert!(!encoded.contains('\n'));

        let css = math_font_face_css();
        assert!(css.contains("@font-face"));
        assert!(css.contains("font-family: \"ORIHA STIX Two Math\""));
        assert!(css.contains("src: url(\"data:font/otf;base64,"));
        assert!(css.contains("\") format(\"opentype\")"));
        assert!(css.contains("font-weight: 400"));
        assert!(css.contains("font-style: normal"));
        assert!(css.contains("font-display: block"));
        assert!(css.contains(encoded));
    }
}
