use oriha_core::*;
use serde_json::json;

const FENCED: &str = "```rust title=example\nlet x = \"<script>&\";\n// second line\n```";
const INDENTED: &str = "    <script>&\n    second line\n";

fn input(markdown: &str, mode: RenderMode, options: CodeBlockOptions) -> RenderInput {
    RenderInput {
        markdown: markdown.into(),
        mode,
        render_options: RenderOptions {
            code_block: options,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn check(html: &str, options: CodeBlockOptions) {
    assert_eq!(
        html.contains("oriha-code-block-language"),
        options.show_language
    );
    assert_eq!(
        html.contains("oriha-code-copy-button"),
        options.show_copy_button
    );
    assert_eq!(
        html.contains("data-oriha-code-copy="),
        options.show_copy_button
    );
    assert_eq!(
        html.contains("oriha-code-gutter"),
        options.show_line_numbers
    );
    assert_eq!(
        html.contains("oriha-code-block-header"),
        options.show_language || options.show_copy_button
    );
    assert_eq!(
        html.matches("oriha-code-line-number").count(),
        if options.show_line_numbers { 2 } else { 0 }
    );
    assert!(html.contains("&lt;script&gt;&amp;"));
    assert!(!html.contains("<script>"));
    assert!(html.contains("oriha-code-scroll"));
    assert!(html.contains("oriha-code-lines"));
}

#[test]
fn all_combinations_work_for_both_html_modes_and_retained_block_derivation() {
    for markdown in [FENCED, INDENTED] {
        for mode in [RenderMode::StyledHtml, RenderMode::SemanticHtml] {
            let engine = OrihaEngine::default();
            for bits in 0..8 {
                let options = CodeBlockOptions {
                    show_language: bits & 1 != 0,
                    show_copy_button: bits & 2 != 0,
                    show_line_numbers: bits & 4 != 0,
                };
                let input = input(markdown, mode, options);
                let parsed = engine.parse(input.clone());
                let direct = engine.render(input);
                check(&direct.html, options);
                assert_eq!(direct.css.is_some(), matches!(mode, RenderMode::StyledHtml));
                if let Some(css) = &direct.css {
                    assert_eq!(
                        css.contains("grid-template-columns: minmax(0, 1fr);"),
                        !options.show_line_numbers
                    );
                    assert_eq!(
                        css.contains(".oriha-code-block-header { justify-content: flex-end; }"),
                        !options.show_language && options.show_copy_button
                    );
                }
                assert_eq!(direct.html, engine.render_parsed(&parsed).html);
                if markdown == FENCED {
                    assert!(direct.html.contains("language-rust"));
                    assert!(direct.html.contains("data-oriha-code-language=\"rust\""));
                    assert!(direct.html.contains("token keyword"));
                }
                let BlockOutput::Rendered(output) =
                    engine.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)
                else {
                    panic!()
                };
                assert_eq!(output.status, BlockOutputStatus::Complete);
                let RenderedBlockContent::Html(html) = &output.blocks[0].content else {
                    panic!()
                };
                check(html, options);
            }
        }
    }
}

#[test]
fn missing_and_partial_json_fields_preserve_enabled_defaults() {
    let expected = render(RenderInput {
        markdown: FENCED.into(),
        ..Default::default()
    })
    .html;
    for options in [
        json!({}),
        json!({"codeBlock": {}}),
        json!({"codeBlock": {"showLanguage": true}}),
    ] {
        let value = json!({"markdown": FENCED, "renderOptions": options});
        let decoded: RenderInput = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(
            decoded.render_options.code_block,
            CodeBlockOptions::default()
        );
        let output: serde_json::Value =
            serde_json::from_str(&render_json(&value.to_string())).unwrap();
        assert_eq!(output["html"], expected);
    }
    let value =
        json!({"markdown": FENCED, "renderOptions": {"codeBlock": {"showCopyButton": false}}});
    let decoded: RenderInput = serde_json::from_value(value).unwrap();
    let options = decoded.render_options.code_block;
    assert_eq!(
        options,
        CodeBlockOptions {
            show_copy_button: false,
            ..Default::default()
        }
    );
    check(&render(decoded).html, options);
    assert_eq!(
        serde_json::to_value(options).unwrap(),
        json!({"showLanguage": true, "showCopyButton": false, "showLineNumbers": true})
    );
    assert!(
        serde_json::from_value::<CodeBlockOptions>(json!({"showCopyButton": "false"})).is_err()
    );
}

#[test]
fn structured_code_metadata_is_independent_of_presentation() {
    let enabled = input(
        FENCED,
        RenderMode::SemanticHtml,
        CodeBlockOptions::default(),
    );
    let disabled = input(
        FENCED,
        RenderMode::SemanticHtml,
        CodeBlockOptions {
            show_language: false,
            show_copy_button: false,
            show_line_numbers: false,
        },
    );
    let first = derive_blocks(enabled, BlockOutputMode::Structured);
    let second = derive_blocks(disabled, BlockOutputMode::Structured);
    assert_eq!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(second).unwrap()
    );
}

#[test]
fn prepared_json_roundtrip_retains_nondefault_options() {
    let options = CodeBlockOptions {
        show_copy_button: false,
        show_line_numbers: false,
        ..Default::default()
    };
    for mode in [RenderMode::StyledHtml, RenderMode::SemanticHtml] {
        let input = input(FENCED, mode, options);
        let value = serde_json::to_string(&input).unwrap();
        let prepared = prepare_markdown_layout_json(&value);
        let output: serde_json::Value = serde_json::from_str(&render_prepared_markdown_json(
            &prepared,
            "{\"results\":[]}",
        ))
        .unwrap();
        check(output["html"].as_str().unwrap(), options);
        assert_eq!(output["html"], render(input).html);
    }
}

#[test]
fn retained_hook_settings_control_both_markup_and_styled_layout() {
    struct HideUi;
    impl RenderHook for HideUi {
        fn after_document_parse(
            &self,
            _: &mut oriha_core::ir::Document,
            _: &mut oriha_core::render::RenderState,
            context: &mut oriha_core::render::RenderContext,
        ) {
            context.code_block = CodeBlockOptions {
                show_language: false,
                show_copy_button: false,
                show_line_numbers: false,
            };
        }
    }
    let engine = OrihaEngine::default().add_hook(HideUi);
    let parsed = engine.parse(input(
        FENCED,
        RenderMode::StyledHtml,
        CodeBlockOptions::default(),
    ));
    let output = engine.render_parsed(&parsed);
    check(
        &output.html,
        CodeBlockOptions {
            show_language: false,
            show_copy_button: false,
            show_line_numbers: false,
        },
    );
    assert!(output
        .css
        .unwrap()
        .contains("grid-template-columns: minmax(0, 1fr);"));
    let BlockOutput::Rendered(output) =
        engine.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)
    else {
        panic!()
    };
    let RenderedBlockContent::Html(html) = &output.blocks[0].content else {
        panic!()
    };
    assert!(!html.contains("oriha-code-block-header"));
    assert!(!html.contains("oriha-code-gutter"));
}

#[test]
fn custom_output_limits_still_apply_with_optional_code_ui() {
    for options in [
        CodeBlockOptions::default(),
        CodeBlockOptions {
            show_language: false,
            show_copy_button: false,
            show_line_numbers: false,
        },
    ] {
        let engine = OrihaEngine::default().with_limits(EngineLimits {
            max_output_bytes: 500,
            ..Default::default()
        });
        let output = engine.derive_blocks(
            input(
                &format!("```rust\n{}\n```", "let x = 1;\n".repeat(100)),
                RenderMode::SemanticHtml,
                options,
            ),
            BlockOutputMode::Rendered,
        );
        let BlockOutput::Rendered(output) = output else {
            panic!()
        };
        assert_ne!(output.status, BlockOutputStatus::Complete);
        assert!(output.blocks.is_empty());
        assert!(output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "blocks.output_limit"));
    }
}
