use oriha_core::{
    render, AppearanceOptions, AssetKind, FeatureFlags, RenderInput, RenderMode, RenderOutput,
};

fn math_input(markdown: &str) -> RenderInput {
    math_input_with_mode(markdown, RenderMode::SemanticHtml)
}

fn math_input_with_mode(markdown: &str, mode: RenderMode) -> RenderInput {
    RenderInput {
        markdown: markdown.to_owned(),
        mode,
        features: FeatureFlags {
            math: true,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    }
}

fn math_disabled_input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.to_owned(),
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            math: false,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    }
}

fn math_input_with_body_font_size(markdown: &str, body_font_size_px: f32) -> RenderInput {
    RenderInput {
        appearance: AppearanceOptions {
            body_font_size_px: Some(body_font_size_px),
            ..AppearanceOptions::default()
        },
        ..math_input(markdown)
    }
}

fn stix_math_asset_count(output: &RenderOutput) -> usize {
    output
        .assets
        .iter()
        .filter(|asset| asset.id == oriha_latex::MATH_FONT_ASSET_ID)
        .count()
}

#[test]
fn inline_latex_renders_svg_when_enabled() {
    let output = render(math_input(
        "Inline math: $E = mc^2$, $\\alpha + \\beta = \\gamma$, $\\sqrt{x^2 + 1}$, and $\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}$.\n",
    ));

    assert!(
        output.html.contains(r#"class="oriha-math-inline""#),
        "{}",
        output.html
    );
    assert!(output.html.contains("<svg "), "{}", output.html);
    assert!(output.html.contains("&#x1d438;"), "{}", output.html);
    assert!(output.html.contains("&#x1d6fc;"), "{}", output.html);
    assert!(output.html.contains("&#x221a;"), "{}", output.html);
}

#[test]
fn latex_font_sizes_follow_body_font_size() {
    let inline = render(math_input_with_body_font_size("Inline $E = mc^2$.\n", 20.0));
    let display = render(math_input_with_body_font_size("$$\na + b = c\n$$\n", 20.0));

    assert!(
        inline.html.contains(r#"font-size="22.5""#),
        "{}",
        inline.html
    );
    assert!(
        display.html.contains(r#"font-size="27.5""#),
        "{}",
        display.html
    );
}

#[test]
fn math_render_registers_one_stix_math_asset() {
    let output = render(math_input(
        "Inline $E = mc^2$.\n\n$$\n\\int_0^1 x^2\\,dx = \\frac{1}{3}\n$$\n\n```latex\n\\alpha + \\beta = \\gamma\n```\n",
    ));

    assert_eq!(stix_math_asset_count(&output), 1, "{:?}", output.assets);
    let asset = output
        .assets
        .iter()
        .find(|asset| asset.id == oriha_latex::MATH_FONT_ASSET_ID)
        .expect("rendered math should register bundled STIX math font");
    assert!(matches!(asset.kind, AssetKind::Other), "{asset:?}");
    assert_eq!(asset.mime_type, oriha_latex::MATH_FONT_MIME_TYPE);
    assert_eq!(asset.content, oriha_latex::math_font_base64());
}

#[test]
fn no_math_render_has_no_stix_math_asset() {
    let output = render(math_input(
        "Plain paragraph with `code` and **strong** text.\n",
    ));

    assert_eq!(stix_math_asset_count(&output), 0, "{:?}", output.assets);
}

#[test]
fn styled_math_render_includes_one_stix_font_face() {
    let output = render(math_input_with_mode(
        "Inline $E = mc^2$.\n\n$$\na + b = c\n$$\n",
        RenderMode::StyledHtml,
    ));
    let css = output
        .css
        .as_ref()
        .expect("styled output should include CSS");

    assert_eq!(stix_math_asset_count(&output), 1, "{:?}", output.assets);
    assert_eq!(css.matches(oriha_latex::math_font_face_css()).count(), 1);
    assert_eq!(css.matches("@font-face").count(), 1);
    assert_eq!(css.matches(oriha_latex::MATH_FONT_FAMILY).count(), 1);
}

#[test]
fn semantic_math_render_keeps_css_none_and_registers_asset() {
    let output = render(math_input("Inline $E = mc^2$.\n"));

    assert!(output.css.is_none());
    assert_eq!(stix_math_asset_count(&output), 1, "{:?}", output.assets);
}

#[test]
fn display_latex_renders_svg_block_when_enabled() {
    let output = render(math_input("$$\n\\int_0^1 x^2\\,dx = \\frac{1}{3}\n$$\n"));

    assert!(
        output.html.contains(r#"class="oriha-math-block""#),
        "{}",
        output.html
    );
    assert!(output.html.contains("<svg "), "{}", output.html);
    assert!(!output.html.contains(r#"class="oriha-code-block""#));
}

#[test]
fn fenced_latex_renders_svg_block_when_enabled() {
    let output = render(math_input(
        "```latex\n\\begin{bmatrix}\n1 & 0 & 0 \\\\\n0 & \\cos\\theta & -\\sin\\theta\n\\end{bmatrix}\n```\n",
    ));

    assert!(
        output.html.contains(r#"class="oriha-math-block""#),
        "{}",
        output.html
    );
    assert!(
        output.html.contains(r#"data-oriha-code-language="latex""#),
        "{}",
        output.html
    );
    assert!(output.html.contains("<svg "), "{}", output.html);
}

#[test]
fn math_disabled_preserves_inline_and_fenced_latex_as_markdown() {
    let inline = render(math_disabled_input(
        "Inline math stays literal: $E = mc^2$.\n",
    ));
    assert!(inline.html.contains("$E = mc^2$"), "{}", inline.html);
    assert!(!inline.html.contains("oriha-math-inline"));

    let fenced = render(math_disabled_input(
        "```latex\n\\alpha + \\beta = \\gamma\n```\n",
    ));
    assert!(fenced.html.contains(r#"class="oriha-code-block""#));
    assert!(fenced.html.contains(r#"class="language-latex""#));
    assert!(!fenced.html.contains("oriha-math-block"));
    assert_eq!(stix_math_asset_count(&inline), 0, "{:?}", inline.assets);
    assert_eq!(stix_math_asset_count(&fenced), 0, "{:?}", fenced.assets);
}

#[test]
fn invalid_display_and_fenced_latex_render_error_figures() {
    let display = render(math_input("$$\n\\unknowncommand{x}\n$$\n"));
    assert!(
        display
            .html
            .contains(r#"class="oriha-math-block oriha-math--error""#),
        "{}",
        display.html
    );
    assert!(display
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "math.render_failed"));
    assert_eq!(stix_math_asset_count(&display), 0, "{:?}", display.assets);

    let fenced = render(math_input("```latex\n\\unknowncommand{x}\n```\n"));
    assert!(
        fenced
            .html
            .contains(r#"class="oriha-math-block oriha-math--error""#),
        "{}",
        fenced.html
    );
    assert!(fenced
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "math.render_failed"));
    assert_eq!(stix_math_asset_count(&fenced), 0, "{:?}", fenced.assets);
}

#[test]
fn invalid_inline_latex_falls_back_to_literal_source() {
    let output = render(math_input("Inline fallback: $\\unknowncommand{x}$.\n"));

    assert!(
        output.html.contains("$\\unknowncommand{x}$"),
        "{}",
        output.html
    );
    assert!(!output.html.contains("oriha-math-inline"));
    assert_eq!(stix_math_asset_count(&output), 0, "{:?}", output.assets);
}
