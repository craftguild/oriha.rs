use oriha_core::ir::*;
use oriha_core::render::{RenderContext, RenderState};
use oriha_core::source::SourceSpan;
use oriha_core::*;

fn output(parsed: &ParsedDocument, options: StructuredOptions) -> StructuredBlockOutput {
    let BlockOutput::Structured(value) = OrihaEngine::default().derive_parsed_blocks_with_options(
        parsed,
        BlockOutputMode::Structured,
        options,
    ) else {
        unreachable!()
    };
    value
}

fn input(source: &str) -> RenderInput {
    RenderInput {
        markdown: source.into(),
        features: FeatureFlags {
            math: true,
            mermaid: true,
            plantuml: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn all() -> StructuredOptions {
    StructuredOptions {
        syntax: true,
        math: true,
        mermaid: true,
        plantuml: true,
    }
}

#[test]
fn optional_preparation_is_on_the_same_nodes_and_does_not_mutate_parse_or_html() {
    let source = "# Same\n\n# Same\n\n```rust\nlet 日本 = 1;\n\n```\n\n$x^2$ and $y$\n\n$$\n\\frac{a}{b}\n$$\n\n```mermaid\nflowchart TD\nA[Start] --> B[Done]\n```\n\n```plantuml\n@startuml\nAlice -> Bob: Hello\n@enduml\n```\n";
    let parsed = parse(input(source));
    let before =
        serde_json::to_value(derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)).unwrap();
    let plain = output(&parsed, StructuredOptions::default());
    let prepared = output(&parsed, all());
    assert_eq!(
        prepared.status,
        BlockOutputStatus::Complete,
        "{:?}",
        prepared.diagnostics
    );
    prepared.validate(10_000, 512, 2_000_000).unwrap();
    assert!(plain.fonts.is_empty());
    assert!(plain.diagnostics.is_empty());
    assert!(
        prepared.diagnostics.is_empty(),
        "{:?}",
        prepared.diagnostics
    );
    assert_eq!(prepared.fonts.len(), 1);
    let font = &prepared.fonts[0];
    assert_eq!(font.id, oriha_latex::MATH_FONT_ASSET_ID);
    assert_eq!(font.family, oriha_latex::MATH_FONT_FAMILY);
    assert_eq!(font.mime_type, "font/otf");
    assert_eq!(font.encoding, FontEncoding::Base64);
    assert_eq!(font.content, oriha_latex::math_font_base64());
    let mut svgs = Vec::new();
    for node in &prepared.nodes {
        match &node.content {
            DocumentNodeKind::FencedCode { svg: Some(svg), .. }
            | DocumentNodeKind::Math { svg: Some(svg), .. }
            | DocumentNodeKind::MathBlock { svg: Some(svg), .. } => svgs.push(svg),
            _ => {}
        }
    }
    assert_eq!(svgs.len(), 5); // two inline, display math and both diagram engines
    for svg in &svgs {
        assert!(svg.markup.starts_with("<svg"));
        assert!(svg.width.unwrap() > 0.0);
        assert!(svg.height.unwrap() > 0.0);
    }
    assert_eq!(svgs.iter().filter(|svg| svg.baseline.is_some()).count(), 3);
    assert_eq!(
        svgs.iter().filter(|svg| !svg.font_ids.is_empty()).count(),
        3
    );
    let structural = |value: &StructuredBlockOutput| {
        let mut nodes = serde_json::to_value(&value.nodes).unwrap();
        for node in nodes.as_array_mut().unwrap() {
            let fields = node.as_object_mut().unwrap();
            fields.remove("svg");
            fields.remove("highlighting");
        }
        nodes
    };
    assert_eq!(plain.root_ids, prepared.root_ids);
    assert_eq!(structural(&plain), structural(&prepared));
    assert_eq!(
        serde_json::to_value(output(&parsed, all())).unwrap(),
        serde_json::to_value(prepared).unwrap()
    );
    assert_eq!(
        serde_json::to_value(derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)).unwrap(),
        before
    );
}

struct Code(&'static str);
impl RenderHook for Code {
    fn after_document_parse(
        &self,
        document: &mut Document,
        _: &mut RenderState,
        context: &mut RenderContext,
    ) {
        document.blocks = vec![BlockNode::new(
            "code",
            BlockKind::FencedCode(CodeBlockNode {
                language: Some(self.0.into()),
                info: self.0.into(),
                closed: true,
                indent: 0,
                body_span: SourceSpan::new(0, context.source.text().len()),
            }),
            SourceSpan::new(0, 0),
        )];
    }
}

#[test]
fn literal_tokens_reconstruct_unicode_whitespace_blank_and_terminal_lines() {
    for language in ["rust", "unknown-language"] {
        for source in [
            "",
            "日本😀",
            "let a = \"日本😀\";\n",
            "\n",
            "\n\n",
            " \t\n \nend",
            "a\r\nb\r\n",
        ] {
            let parsed = OrihaEngine::default()
                .add_hook(Code(language))
                .parse(input(source));
            let value = output(
                &parsed,
                StructuredOptions {
                    syntax: true,
                    ..Default::default()
                },
            );
            assert_eq!(value.status, BlockOutputStatus::Complete);
            let DocumentNodeKind::FencedCode {
                source: actual,
                highlighting: Some(highlighting),
                ..
            } = &value.nodes[0].content
            else {
                panic!("missing highlighting")
            };
            assert_eq!(actual, source);
            let rows: Vec<String> = highlighting
                .lines
                .iter()
                .map(|line| {
                    line.tokens
                        .iter()
                        .map(|token| token.text.as_str())
                        .collect()
                })
                .collect();
            assert_eq!(rows.join("\n"), source, "{language}: {source:?}");
            assert_eq!(rows.len(), source.split('\n').count());
            if language == "unknown-language" {
                assert!(highlighting
                    .lines
                    .iter()
                    .flat_map(|line| &line.tokens)
                    .all(|token| token.kind == TokenKind::Plain));
            }
        }
    }
}

#[test]
fn malformed_and_unsupported_rendering_is_local_but_aggregate_failure_is_not() {
    let source = "before\n\n$\\notARealCommand{x}$\n\n```mermaid\nnot a diagram\n```\n\n```plantuml\n@startmindmap\n* unsupported\n@endmindmap\n```\n\n$$\nunclosed";
    let parsed = parse(input(source));
    let value = output(&parsed, all());
    assert_eq!(
        value.status,
        BlockOutputStatus::Complete,
        "{:?}",
        value.diagnostics
    );
    assert_eq!(value.diagnostics.len(), 4);
    for diagnostic in &value.diagnostics {
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert!(diagnostic.message.len() < 100);
        let node = value
            .nodes
            .iter()
            .find(|node| Some(&node.id) == diagnostic.block_id.as_ref())
            .unwrap();
        match &node.content {
            DocumentNodeKind::Math { source, svg }
            | DocumentNodeKind::MathBlock { source, svg, .. }
            | DocumentNodeKind::FencedCode { source, svg, .. } => {
                assert!(!source.is_empty());
                assert!(svg.is_none());
            }
            _ => panic!("diagnostic must identify its source node"),
        }
    }
    let parsed = OrihaEngine::default()
        .with_limits(EngineLimits {
            max_output_bytes: 4096,
            ..Default::default()
        })
        .parse(input("$x$"));
    let value = output(&parsed, all());
    assert_eq!(value.status, BlockOutputStatus::Failed);
    assert!(value.nodes.is_empty());
    assert!(value.fonts.is_empty());
    assert!(value
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error));
    assert_eq!(
        output(&parsed, StructuredOptions::default()).status,
        BlockOutputStatus::Complete
    );
}

#[test]
fn fenced_indentation_preserves_source_terminal_newline() {
    let parsed = parse(input("  ```rust\n  α\n\n  ```"));
    let value = output(
        &parsed,
        StructuredOptions {
            syntax: true,
            ..Default::default()
        },
    );
    let DocumentNodeKind::FencedCode {
        source,
        highlighting: Some(highlighting),
        ..
    } = &value.nodes[0].content
    else {
        panic!("expected fence")
    };
    assert_eq!(source, "α\n\n");
    assert_eq!(highlighting.lines.len(), 3);
}
