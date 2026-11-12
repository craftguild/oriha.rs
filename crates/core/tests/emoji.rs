use oriha_core::{render, RenderInput, RenderMode};

#[test]
fn renders_known_emoji_shortcodes_as_native_emoji_text() {
    let output = render(RenderInput {
        markdown: "# :+1:\n\nParagraph :+1: :thumbs_up_light_skin_tone: [go :rocket:](https://example.com).\n\n![alt :+1:](https://example.com/image.png?w=10)\n\n`:+1:` and :not_a_real_shortcode:\n".to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default()
    });

    assert!(output.html.contains(r#"id="1""#), "{}", output.html);
    assert!(output.html.contains(">👍</h1>"), "{}", output.html);
    assert!(
        output
            .html
            .contains(r#"Paragraph 👍 👍🏻 <a href="https://example.com">go 🚀</a>."#),
        "{}",
        output.html
    );
    assert!(output.html.contains(r#"alt="alt 👍""#), "{}", output.html);
    assert!(output.html.contains("<code>:+1:</code>"), "{}", output.html);
    assert!(
        output.html.contains(":not_a_real_shortcode:"),
        "{}",
        output.html
    );
}

#[test]
fn escaped_emoji_shortcode_renders_literally() {
    let output = render(RenderInput {
        markdown: r"\:+1: and :+1:".to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default()
    });

    assert!(output.html.contains(":+1: and 👍"), "{}", output.html);
}
