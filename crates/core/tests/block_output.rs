use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use oriha_core::ir::*;
use oriha_core::render::{RenderContext, RenderFragment, RenderState};
use oriha_core::source::{SourceDocument, SourceSpan};
use oriha_core::*;

fn input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.into(),
        ..Default::default()
    }
}
fn structured(output: BlockOutput) -> StructuredBlockOutput {
    match output {
        BlockOutput::Structured(o) => o,
        BlockOutput::Rendered(_) => panic!("wrong mode"),
    }
}
fn rendered(output: BlockOutput) -> RenderedBlockOutput {
    match output {
        BlockOutput::Rendered(o) => o,
        BlockOutput::Structured(_) => panic!("wrong mode"),
    }
}
fn json(output: &impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(output).unwrap()
}
fn node<'a>(v: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    v["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == id)
        .unwrap()
}
fn root(v: &serde_json::Value, index: usize) -> &serde_json::Value {
    node(v, v["rootIds"][index].as_str().unwrap())
}
fn child<'a>(
    v: &'a serde_json::Value,
    parent: &serde_json::Value,
    index: usize,
) -> &'a serde_json::Value {
    node(v, parent["childIds"][index].as_str().unwrap())
}
fn plain(v: &serde_json::Value, n: &serde_json::Value) -> String {
    match n["kind"].as_str().unwrap() {
        "text" | "emoji" => n["text"].as_str().unwrap().into(),
        "code" | "math" | "html" | "fencedCode" | "indentedCode" | "mathBlock" | "htmlBlock"
        | "foreignHtml" => n["source"].as_str().unwrap().into(),
        "softBreak" | "hardBreak" => "\n".into(),
        "footnoteReference" => format!("[^{}]", n["label"].as_str().unwrap()),
        "image" => n["altIds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| plain(v, node(v, id.as_str().unwrap())))
            .collect(),
        _ => n["childIds"]
            .as_array()
            .map(|ids| {
                ids.iter()
                    .map(|id| plain(v, node(v, id.as_str().unwrap())))
                    .collect()
            })
            .unwrap_or_default(),
    }
}
fn body_key(mode: BlockOutputMode) -> &'static str {
    if mode == BlockOutputMode::Structured {
        "nodes"
    } else {
        "blocks"
    }
}
fn inline(kind: InlineKind) -> InlineNode {
    kind.into()
}
fn block(id: &str, kind: BlockKind) -> BlockNode {
    BlockNode::new(id, kind, SourceSpan::new(0, 0))
}

struct Programmatic;
impl RenderHook for Programmatic {
    fn after_document_parse(&self, d: &mut Document, _: &mut RenderState, _: &mut RenderContext) {
        let text = || vec![inline(InlineKind::Text("日本 & decoded".into()))];
        let all = vec![
            inline(InlineKind::Annotation {
                annotation: AnnotationMetadata {
                    kind: "mention".into(),
                    id: "a".into(),
                    attributes: AnnotationAttributes {
                        classes: vec!["mark".into()],
                        data: BTreeMap::from([("key".into(), "value".into())]),
                    },
                },
                children: text(),
            }),
            inline(InlineKind::Text(" text 😀".into())),
            inline(InlineKind::Emoji {
                shortcode: "smile".into(),
                emoji: "😄".into(),
            }),
            inline(InlineKind::Code("let x = <3;".into())),
            inline(InlineKind::Math {
                source: "x^2".into(),
            }),
            inline(InlineKind::Html {
                raw: "<b>raw</b>".into(),
                tagfilter: true,
            }),
            inline(InlineKind::SoftBreak),
            inline(InlineKind::HardBreak),
            inline(InlineKind::Emphasis { children: text() }),
            inline(InlineKind::Strong {
                children: vec![inline(InlineKind::Strikethrough { children: text() })],
            }),
            inline(InlineKind::Link {
                destination: "https://example.test/a?b=c&d=e".into(),
                title: Some("Title".into()),
                children: text(),
            }),
            inline(InlineKind::Image {
                destination: "/a.png#width=32".into(),
                title: None,
                alt: vec![inline(InlineKind::Strong { children: text() })],
            }),
            inline(InlineKind::FootnoteReference {
                label: "note".into(),
            }),
        ];
        let code = CodeBlockNode {
            language: Some("rust".into()),
            info: "rust title".into(),
            body_span: SourceSpan::new(0, 4),
            closed: false,
            indent: 0,
        };
        d.blocks = vec![
            {
                let mut n = block("doc", BlockKind::Document);
                n.children.push(block("inside", BlockKind::ThematicBreak));
                n
            },
            block("p", BlockKind::Paragraph(ParagraphNode { children: all })),
            block(
                "h",
                BlockKind::Heading(HeadingNode {
                    level: 3,
                    inline_span: SourceSpan::new(0, 0),
                    children: text(),
                }),
            ),
            block("fence", BlockKind::FencedCode(code.clone())),
            {
                let mut n = block("indent", BlockKind::IndentedCode(code));
                n.attrs
                    .insert("normalized_code".into(), "normalized\ncode".into());
                n
            },
            block(
                "math",
                BlockKind::MathBlock(MathBlockNode {
                    source_span: SourceSpan::new(0, 4),
                    source: Some("y^3".into()),
                    display: true,
                    closed: true,
                }),
            ),
            block("hr", BlockKind::ThematicBreak),
            {
                let mut n = block("html", BlockKind::HtmlBlock);
                n.span = SourceSpan::new(0, 4);
                n
            },
            {
                let mut n = block("ref", BlockKind::LinkReferenceDefinition);
                n.span = SourceSpan::new(0, 4);
                n
            },
            {
                let mut n = block("quote", BlockKind::BlockQuote);
                n.children.push(block(
                    "quote-p",
                    BlockKind::Paragraph(ParagraphNode { children: text() }),
                ));
                n
            },
            {
                let mut n = block(
                    "list",
                    BlockKind::List(ListNode {
                        ordered: true,
                        loose: true,
                        marker: "003)".into(),
                    }),
                );
                let mut item = block(
                    "item",
                    BlockKind::ListItem(ListItemNode {
                        raw_prefix: "3) ".into(),
                        content_span: SourceSpan::new(0, 4),
                        task: Some(TaskListItemNode { checked: true }),
                    }),
                );
                item.children.push(block(
                    "item-p",
                    BlockKind::Paragraph(ParagraphNode { children: text() }),
                ));
                n.children.push(item);
                n
            },
            block(
                "table",
                BlockKind::GfmTable(GfmTableNode {
                    alignments: vec![
                        TableAlignment::Left,
                        TableAlignment::Center,
                        TableAlignment::Right,
                        TableAlignment::None,
                    ],
                    header: vec![TableCellNode::new("H", text())],
                    rows: vec![TableRowNode {
                        cells: vec![TableCellNode::new("C", text())],
                    }],
                }),
            ),
            block("blank", BlockKind::BlankLine),
            {
                let mut n = block(
                    "callout",
                    BlockKind::Callout(CalloutNode {
                        kind: "note".into(),
                        title: Some("Custom".into()),
                        body_span: SourceSpan::new(0, 0),
                    }),
                );
                n.children.push(block(
                    "callout-p",
                    BlockKind::Paragraph(ParagraphNode { children: text() }),
                ));
                n
            },
            block(
                "foreign",
                BlockKind::ForeignHtml(ForeignHtmlNode {
                    html: "<em>foreign</em>".into(),
                    origin: "consumer".into(),
                    trust_level: ForeignHtmlTrustLevel::Sanitized,
                }),
            ),
        ];
        d.link_references = BTreeMap::from([(
            "key".into(),
            LinkReference {
                destination: "/ref".into(),
                title: Some("Reference".into()),
            },
        )]);
        d.footnotes = vec![
            FootnoteDefinition {
                label: "note".into(),
                children: text(),
            },
            FootnoteDefinition {
                label: "unused".into(),
                children: text(),
            },
        ];
    }
}

#[test]
fn every_actual_kind_has_typed_data_and_self_contained_json() {
    let engine = OrihaEngine::default().add_hook(Programmatic);
    let parsed = engine.parse(input("body"));
    let o = structured(engine.derive_parsed_blocks(&parsed, BlockOutputMode::Structured));
    assert_eq!(o.status, BlockOutputStatus::Complete);
    let v = json(&o);
    assert_eq!(v["schemaVersion"], 2);
    assert_eq!(v["rootIds"].as_array().unwrap().len(), 15);
    o.validate(10_000, 512, 2_000_000).unwrap();
    let p = root(&v, 1);
    assert!(plain(&v, p).contains("日本 & decoded"));
    assert!(plain(&v, p).contains("[^note]"));
    assert_eq!(child(&v, p, 0)["data"]["key"], "value");
    assert_eq!(child(&v, p, 0)["annotationKind"], "mention");
    assert_eq!(child(&v, p, 2)["shortcode"], "smile");
    assert_eq!(child(&v, p, 3)["source"], "let x = <3;");
    assert_eq!(child(&v, p, 4)["source"], "x^2");
    assert_eq!(child(&v, p, 5)["tagfilter"], true);
    assert_eq!(child(&v, p, 6)["kind"], "softBreak");
    assert_eq!(child(&v, p, 7)["kind"], "hardBreak");
    assert_eq!(child(&v, child(&v, p, 9), 0)["kind"], "strikethrough");
    assert_eq!(
        child(&v, p, 10)["destination"],
        "https://example.test/a?b=c&d=e"
    );
    let image = child(&v, p, 11);
    assert_eq!(
        node(&v, image["altIds"][0].as_str().unwrap())["kind"],
        "strong"
    );
    assert_eq!(root(&v, 2)["level"], 3);
    assert_eq!(root(&v, 2)["anchor"], "日本-decoded");
    assert_eq!(root(&v, 3)["source"], "body");
    assert_eq!(root(&v, 3)["closed"], false);
    assert_eq!(root(&v, 3)["info"], "rust title");
    assert_eq!(root(&v, 4)["source"], "normalized\ncode");
    assert_eq!(root(&v, 5)["source"], "y^3");
    assert_eq!(root(&v, 7)["source"], "body");
    assert_eq!(root(&v, 8)["source"], "body");
    assert_eq!(root(&v, 10)["start"], "3");
    assert_eq!(child(&v, root(&v, 10), 0)["checked"], true);
    assert_eq!(
        root(&v, 11)["alignments"],
        json(&vec!["left", "center", "right", "none"])
    );
    let row = node(&v, root(&v, 11)["rowIds"][1].as_str().unwrap());
    let cell = node(&v, row["cellIds"][0].as_str().unwrap());
    assert_eq!(plain(&v, cell), "日本 & decoded");
    assert_eq!(root(&v, 12)["kind"], "blankLine");
    assert!(root(&v, 12).get("text").is_none());
    assert_eq!(root(&v, 13)["title"], "Custom");
    assert_eq!(root(&v, 14)["trustLevel"], "sanitized");
    assert_eq!(
        node(&v, v["referenceIds"][0].as_str().unwrap())["destination"],
        "/ref"
    );
    assert_eq!(
        node(&v, v["footnoteIds"][1].as_str().unwrap())["label"],
        "unused"
    );
    let roundtrip: StructuredBlockOutput = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(json(&roundtrip), v);
    let r = rendered(engine.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered));
    assert_eq!(r.status, BlockOutputStatus::Complete);
    assert_eq!(r.blocks.len(), 16); // 15 roots and the generated footnote section
    assert_eq!(json(&r)["blocks"][8]["nullReason"], "referenceDefinition");
    assert_eq!(json(&r)["blocks"][12]["html"], serde_json::Value::Null);
    assert_eq!(r.blocks[0].kind, HtmlBlockKind::Document);
    assert!(r.blocks[0].html().unwrap().contains("inside"));
    assert!(json(&r)["blocks"][0].get("children").is_none());
    let rv = json(&r);
    assert_eq!(
        json(&serde_json::from_value::<RenderedBlockOutput>(rv.clone()).unwrap()),
        rv
    );
}

#[test]
fn normalized_nested_sources_tables_images_and_annotations_survive() {
    let engine = OrihaEngine::default();
    let source = "> - ```rust\n>   日本 <&>\n>   ```\n>\n>   $$\n>   x^2\n>   $$\n\n| a\\|b | ![alt **太字**](/i) |\n| --- | --- |\n| &amp; | `code` |\n\n@ali**ce**\n";
    let parsed = engine.parse(input(source));
    let v = json(&structured(
        engine.derive_parsed_blocks(&parsed, BlockOutputMode::Structured),
    ));
    let nested = child(&v, child(&v, root(&v, 0), 0), 0);
    assert!(child(&v, nested, 0)["source"]
        .as_str()
        .unwrap()
        .contains("日本 <&>"));
    assert!(plain(&v, child(&v, nested, 1)).contains("x^2"));
    let table = root(&v, 1);
    let header = node(&v, table["rowIds"][0].as_str().unwrap());
    assert_eq!(
        plain(&v, node(&v, header["cellIds"][0].as_str().unwrap())),
        "a|b"
    );
    assert_eq!(
        plain(&v, node(&v, header["cellIds"][1].as_str().unwrap())),
        "alt 太字"
    );
    let row = node(&v, table["rowIds"][1].as_str().unwrap());
    assert_eq!(
        plain(&v, node(&v, row["cellIds"][0].as_str().unwrap())),
        "&"
    );
    let region = parsed.text_regions().find(|r| r.text == "@alice").unwrap();
    let annotated = parsed
        .apply_annotations(&[TextAnnotation {
            region: region.id,
            range: 0..6,
            metadata: AnnotationMetadata {
                kind: "mention".into(),
                id: "one".into(),
                attributes: Default::default(),
            },
        }])
        .unwrap();
    let av = json(&structured(
        engine.derive_parsed_blocks(&annotated, BlockOutputMode::Structured),
    ));
    assert_eq!(plain(&av, root(&av, 2)), "@alice");
    assert_eq!(child(&av, root(&av, 2), 0)["kind"], "annotation");
}

#[test]
fn rendered_roots_equal_semantic_document_body_and_share_context() {
    let engine = OrihaEngine::default();
    let mut i = input("# Same\n\n# Same\n\n[link][ref] and [^n].\n\n> quote **bold**\n\n- one\n- two\n\n[ref]: /target \"title\"\n[^n]: note\n");
    i.mode = RenderMode::SemanticHtml;
    let parsed = engine.parse(i);
    let r = rendered(engine.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered));
    assert_eq!(r.status, BlockOutputStatus::Complete);
    let assembled = r
        .blocks
        .iter()
        .filter_map(RenderedHtmlBlock::html)
        .collect::<String>();
    let doc = engine.render_parsed(&parsed);
    let body = doc
        .html
        .split_once('>')
        .unwrap()
        .1
        .strip_suffix("</article>")
        .unwrap();
    assert_eq!(assembled, body);
    assert!(assembled.contains("id=\"same-2\""));
    assert!(assembled.contains("href=\"/target\""));
    assert!(assembled.contains("href=\"#fn-n\""));
    assert_eq!(r.blocks.last().unwrap().kind, HtmlBlockKind::Footnotes);
    assert_eq!(r.blocks.last().unwrap().id, "oriha-generated-footnotes");
}

#[derive(Default)]
struct Counts {
    before_finalize: AtomicUsize,
    parse: AtomicUsize,
    render: AtomicUsize,
    finalize: AtomicUsize,
}
struct Instrument(Arc<Counts>);
impl RenderHook for Instrument {
    fn before_output_finalize(&self, _: &Document, _: &mut RenderState, _: &mut RenderContext) {
        self.0.before_finalize.fetch_add(1, Ordering::SeqCst);
    }
    fn after_document_parse(&self, d: &mut Document, _: &mut RenderState, _: &mut RenderContext) {
        self.0.parse.fetch_add(1, Ordering::SeqCst);
        if let BlockKind::Paragraph(p) = &mut d.blocks[0].kind {
            p.children = vec![inline(InlineKind::Text("retained hook IR".into()))];
        }
    }
    fn before_block_render(
        &self,
        b: &mut BlockNode,
        _: &mut RenderFragment,
        s: &mut RenderState,
        c: &RenderContext,
    ) {
        assert!(matches!(c.mode, RenderMode::SemanticHtml));
        self.0.render.fetch_add(1, Ordering::SeqCst);
        assert!(!s.attrs.contains_key("once"));
        s.attrs.insert("once".into(), "yes".into());
        b.id = "working-id".into();
    }
    fn after_output_finalize(
        &self,
        _: &Document,
        o: &mut RenderOutput,
        _: &mut RenderState,
        _: &mut RenderContext,
    ) {
        self.0.finalize.fetch_add(1, Ordering::SeqCst);
        o.html.push_str("FINAL");
    }
}
#[test]
fn hooks_parse_reuse_identity_and_repeated_derivation_are_isolated() {
    let counts = Arc::new(Counts::default());
    let engine = OrihaEngine::default().add_hook(Instrument(counts.clone()));
    let mut i = input("original markdown");
    i.mode = RenderMode::SemanticHtml;
    let parsed = engine.parse(i);
    for _ in 0..2 {
        let s = structured(engine.derive_parsed_blocks(&parsed, BlockOutputMode::Structured));
        assert_eq!(plain(&json(&s), root(&json(&s), 0)), "retained hook IR");
        let r = rendered(engine.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered));
        assert_eq!(r.blocks[0].id, "working-id");
        assert!(!r.blocks[0].html().unwrap().contains("FINAL"));
    }
    assert_eq!(counts.parse.load(Ordering::SeqCst), 1);
    assert_eq!(counts.render.load(Ordering::SeqCst), 2);
    assert_eq!(counts.finalize.load(Ordering::SeqCst), 0);
    assert_eq!(counts.before_finalize.load(Ordering::SeqCst), 0);
    assert_ne!(parsed.document().blocks[0].id, "working-id");
    assert!(engine.render_parsed(&parsed).html.ends_with("FINAL"));
    assert_eq!(counts.parse.load(Ordering::SeqCst), 1);
    assert_eq!(counts.finalize.load(Ordering::SeqCst), 1);
    assert_eq!(counts.before_finalize.load(Ordering::SeqCst), 1);
}

struct EmptyRenderer(bool);
impl BlockRenderer for EmptyRenderer {
    fn kind(&self) -> &'static str {
        "paragraph"
    }
    fn block_emission(&self, _: &BlockNode) -> BlockEmission {
        if self.0 {
            BlockEmission::Null(BlockNullReason::BlankLine)
        } else {
            BlockEmission::Html
        }
    }
    fn render(
        &self,
        _: &mut BlockNode,
        _: &SourceDocument,
        _: &mut RenderState,
        _: &RenderContext,
        _: &[Box<dyn BlockRenderer>],
        _: &mut String,
    ) {
    }
}
#[test]
fn custom_empty_html_and_explicit_null_have_distinct_wire_forms() {
    for null in [false, true] {
        let o = rendered(
            OrihaEngine::default()
                .register_renderer(EmptyRenderer(null))
                .derive_blocks(input("p"), BlockOutputMode::Rendered),
        );
        assert_eq!(o.status, BlockOutputStatus::Complete);
        if null {
            assert_eq!(json(&o)["blocks"][0]["html"], serde_json::Value::Null);
        } else {
            assert_eq!(json(&o)["blocks"][0]["html"], "");
            assert_eq!(o.blocks[0].html(), Some(""));
        }
    }
}

struct BadIds;
impl RenderHook for BadIds {
    fn after_document_parse(&self, d: &mut Document, _: &mut RenderState, _: &mut RenderContext) {
        d.blocks[1].id = d.blocks[0].id.clone();
    }
}
#[test]
fn incomplete_failed_and_invalid_documents_return_empty_results() {
    for mode in [BlockOutputMode::Structured, BlockOutputMode::Rendered] {
        let engine = OrihaEngine::default().with_limits(EngineLimits {
            max_blocks: 1,
            ..Default::default()
        });
        let o = json(&engine.derive_blocks(input("one\n\ntwo"), mode));
        assert_eq!(o["status"], "incomplete");
        assert_eq!(o[body_key(mode)], json(&Vec::<u8>::new()));
        let engine = OrihaEngine::default().with_limits(EngineLimits {
            max_source_bytes: 2,
            ..Default::default()
        });
        assert_eq!(
            json(&engine.derive_blocks(input("long"), mode))["status"],
            "failed"
        );
        let o = json(
            &OrihaEngine::default()
                .add_hook(BadIds)
                .derive_blocks(input("one\n\ntwo"), mode),
        );
        assert_eq!(o["status"], "failed");
        assert_eq!(o["diagnostics"][0]["code"], "blocks.invalid_identity");
        assert_eq!(o[body_key(mode)], json(&Vec::<u8>::new()));
    }
}

#[test]
fn aggregate_budget_counts_escapes_assets_metadata_and_envelope() {
    for mode in [BlockOutputMode::Structured, BlockOutputMode::Rendered] {
        let i = input("alpha & β\n\nbeta & γ");
        let complete = derive_blocks(i.clone(), mode);
        let exact = serde_json::to_vec(&complete).unwrap().len();
        for (limit, expected) in [(exact, "complete"), (exact - 1, "failed"), (0, "failed")] {
            let engine = OrihaEngine::default().with_limits(EngineLimits {
                max_output_bytes: limit,
                ..Default::default()
            });
            let o = engine.derive_blocks(i.clone(), mode);
            assert_eq!(
                json(&o)["status"],
                expected,
                "mode {mode:?}, limit {limit}, {}",
                json(&o)
            );
            if expected == "failed" {
                assert_eq!(json(&o)[body_key(mode)], json(&Vec::<u8>::new()));
            }
        }
    }
    let engine = OrihaEngine::default().with_limits(EngineLimits {
        max_output_bytes: 1024,
        ..Default::default()
    });
    let mut math = input("$x^2$");
    math.features.math = true;
    let r = rendered(engine.derive_blocks(math.clone(), BlockOutputMode::Rendered));
    assert_eq!(r.status, BlockOutputStatus::Failed);
    assert!(r.assets.is_empty());
    let r = rendered(derive_blocks(math, BlockOutputMode::Rendered));
    assert_eq!(r.status, BlockOutputStatus::Complete);
    assert_eq!(r.assets.len(), 1);
    assert_eq!(r.assets[0].mime_type, "font/otf");
}

struct PanicRender;
impl RenderHook for PanicRender {
    fn before_block_render(
        &self,
        _: &mut BlockNode,
        _: &mut RenderFragment,
        _: &mut RenderState,
        _: &RenderContext,
    ) {
        panic!("extension panic");
    }
}
#[test]
fn caught_panics_and_output_failures_release_working_state() {
    let engine = OrihaEngine::default().add_hook(PanicRender);
    let parsed = engine.parse(input("safe"));
    let r = rendered(engine.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered));
    assert_eq!(r.status, BlockOutputStatus::Failed);
    assert!(r.blocks.is_empty());
    assert_eq!(r.diagnostics[0].code, "blocks.panic");
    let s = structured(engine.derive_parsed_blocks(&parsed, BlockOutputMode::Structured));
    assert_eq!(s.status, BlockOutputStatus::Complete);
    assert_eq!(
        rendered(derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)).status,
        BlockOutputStatus::Complete
    );
    let limited = OrihaEngine::default().with_limits(EngineLimits {
        max_output_bytes: 512,
        ..Default::default()
    });
    assert_eq!(
        rendered(limited.derive_blocks(input(&"<&>".repeat(500)), BlockOutputMode::Rendered))
            .status,
        BlockOutputStatus::Failed
    );
    assert_eq!(
        rendered(derive_blocks(input("good"), BlockOutputMode::Rendered)).status,
        BlockOutputStatus::Complete
    );
}

#[test]
fn finite_concurrent_derivations_share_no_working_state() {
    let parsed = Arc::new(parse(input("# Same\n\n# Same\n\n[^x]\n\n[^x]: note")));
    let expected =
        serde_json::to_string(&derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)).unwrap();
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let p = parsed.clone();
            std::thread::spawn(move || {
                serde_json::to_string(&derive_parsed_blocks(&p, BlockOutputMode::Rendered)).unwrap()
            })
        })
        .collect();
    for h in handles {
        assert_eq!(h.join().unwrap(), expected);
    }
}
