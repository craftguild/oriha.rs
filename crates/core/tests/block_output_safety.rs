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
fn value(o: BlockOutput) -> serde_json::Value {
    serde_json::to_value(o).unwrap()
}
fn diagnostic(severity: Severity, message: &str) -> RenderDiagnostic {
    RenderDiagnostic {
        severity,
        code: "consumer.test".into(),
        message: message.into(),
        source_range: None,
        block_id: None,
        engine: serde_json::from_str("\"core\"").unwrap(),
    }
}
struct Modify(u8);
impl RenderHook for Modify {
    fn after_document_parse(&self, d: &mut Document, s: &mut RenderState, _: &mut RenderContext) {
        match self.0 {
            0 => d.blocks[0].id.clear(),
            1 => d.blocks[0].id = "oriha-generated-footnotes".into(),
            2 => s
                .diagnostics
                .push(diagnostic(Severity::Error, "actionable error")),
            3 => s.diagnostics.push(diagnostic(
                Severity::Warning,
                &"large diagnostic".repeat(1000),
            )),
            4 => d.blocks[0]
                .diagnostics
                .push(diagnostic(Severity::Error, "block error")),
            5 => panic!("parse failure"),
            6 => {
                let mut child =
                    BlockNode::new("bottom", BlockKind::ThematicBreak, SourceSpan::new(0, 0));
                for i in 0..260 {
                    let mut parent = BlockNode::new(
                        format!("container-{i}"),
                        BlockKind::Document,
                        SourceSpan::new(0, 0),
                    );
                    parent.children.push(child);
                    child = parent;
                }
                d.blocks = vec![child];
            }
            7 => d.blocks.push(BlockNode::new(
                "added",
                BlockKind::ThematicBreak,
                SourceSpan::new(0, 0),
            )),
            8 => s.assets.push(RenderAsset {
                id: "large".into(),
                kind: AssetKind::Other,
                mime_type: "application/octet-stream".into(),
                content: "x".repeat(10000),
            }),
            _ => unreachable!(),
        }
    }
}
#[test]
fn failures_and_diagnostics_are_bounded_and_never_partial_success() {
    for mode in [BlockOutputMode::Structured, BlockOutputMode::Rendered] {
        for modification in [0, 2, 3, 4, 5, 6, 7] {
            let e = OrihaEngine::default()
                .with_limits(EngineLimits {
                    max_blocks: 1,
                    max_output_bytes: 1024,
                    ..Default::default()
                })
                .add_hook(Modify(modification));
            let o = value(e.derive_blocks(input("one"), mode));
            assert_eq!(o["status"], "failed", "modification {modification}: {o}");
            assert!(o["blocks"].as_array().unwrap().is_empty());
            assert!(serde_json::to_vec(&o).unwrap().len() <= 1024);
            assert_eq!(o["diagnostics"].as_array().unwrap().len(), 1);
        }
    }
    let o = value(
        OrihaEngine::default()
            .add_hook(Modify(8))
            .with_limits(EngineLimits {
                max_output_bytes: 1024,
                ..Default::default()
            })
            .derive_blocks(input("one"), BlockOutputMode::Rendered),
    );
    assert_eq!(o["status"], "failed");
    assert!(o["assets"].as_array().unwrap().is_empty());
}
#[test]
fn generated_footnotes_avoid_reserved_identity_collisions() {
    let o = value(
        OrihaEngine::default()
            .add_hook(Modify(1))
            .derive_blocks(input("a[^n]\n\n[^n]: note"), BlockOutputMode::Rendered),
    );
    assert_eq!(o["status"], "complete");
    assert_eq!(o["blocks"][1]["id"], "oriha-generated-footnotes-1");
}
struct DuplicateAfter;
impl RenderHook for DuplicateAfter {
    fn after_block_render(
        &self,
        b: &mut BlockNode,
        _: &mut RenderFragment,
        _: &mut RenderState,
        _: &RenderContext,
    ) {
        b.id = "same".into();
    }
}
#[test]
fn identities_are_validated_again_after_render_hooks() {
    let e = OrihaEngine::default().add_hook(DuplicateAfter);
    let p = e.parse(input("first\n\nsecond"));
    let o = value(e.derive_parsed_blocks(&p, BlockOutputMode::Rendered));
    assert_eq!(o["status"], "failed");
    assert_eq!(o["diagnostics"][0]["code"], "blocks.invalid_identity");
    assert_eq!(
        value(e.derive_parsed_blocks(&p, BlockOutputMode::Structured))["status"],
        "complete"
    );
}
struct LegacyRenderer;
impl BlockRenderer for LegacyRenderer {
    fn kind(&self) -> &'static str {
        "paragraph"
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
fn existing_custom_renderer_needs_no_emission_method() {
    let o = value(
        OrihaEngine::default()
            .register_renderer(LegacyRenderer)
            .derive_blocks(input("p"), BlockOutputMode::Rendered),
    );
    assert_eq!(o["status"], "complete");
    assert_eq!(o["blocks"][0]["html"], "");
    assert!(o["blocks"][0]["nullReason"].is_null());
}
#[test]
fn retained_features_diagrams_math_and_decorations_follow_the_contract() {
    let mut i = input("```mermaid\nflowchart TD\nA[Start] --> B[Done]\n```\n\n```plantuml\n@startuml\nAlice -> Bob: Hello\n@enduml\n```\n\n$$\nx^2\n$$\n\n$x$ and $y$\n\n<div>unsafe</div>\n");
    i.features = FeatureFlags {
        mermaid: true,
        plantuml: true,
        math: true,
        ..Default::default()
    };
    i.render_options.watermark = Some(WatermarkOptions {
        text: "DOCUMENT ONLY".into(),
        ..Default::default()
    });
    let producer = OrihaEngine::default();
    let p = producer.parse(i);
    let receiver = OrihaEngine::default();
    let o = value(receiver.derive_parsed_blocks(&p, BlockOutputMode::Rendered));
    assert_eq!(o["status"], "complete", "{o}");
    assert!(o["blocks"][0]["html"]
        .as_str()
        .unwrap()
        .contains("oriha-mermaid"));
    assert!(o["blocks"][1]["html"]
        .as_str()
        .unwrap()
        .contains("oriha-plantuml"));
    assert!(o["blocks"][2]["html"]
        .as_str()
        .unwrap()
        .contains("oriha-math-block"));
    assert_eq!(o["assets"].as_array().unwrap().len(), 1);
    assert!(o["blocks"][4]["html"]
        .as_str()
        .unwrap()
        .contains("&lt;div&gt;"));
    assert!(o.get("css").is_none());
    assert!(!serde_json::to_string(&o).unwrap().contains("DOCUMENT ONLY"));
    let ordinary = producer.render_parsed(&p);
    assert!(ordinary.css.is_some());
    assert!(ordinary.html.contains("DOCUMENT ONLY"));
    let disabled = value(derive_blocks(
        input("```mermaid\nflowchart TD\nA --> B\n```"),
        BlockOutputMode::Rendered,
    ));
    assert!(disabled["blocks"][0]["html"]
        .as_str()
        .unwrap()
        .contains("oriha-code-block"));
    let s = value(receiver.derive_parsed_blocks(&p, BlockOutputMode::Structured));
    assert_eq!(s["blocks"][0]["kind"], "fencedCode");
    assert!(s["blocks"][0]["text"]
        .as_str()
        .unwrap()
        .contains("flowchart TD"));
}
#[test]
fn empty_text_structural_nodes_and_recovery_remain_meaningful() {
    for source in [
        "",
        "```\n```",
        "$$\nunclosed",
        "\0 malformed [link](",
        "> > > deep",
    ] {
        let mut i = input(source);
        i.features.math = true;
        for mode in [BlockOutputMode::Structured, BlockOutputMode::Rendered] {
            let o = value(derive_blocks(i.clone(), mode));
            assert_eq!(o["status"], "complete", "{source}: {o}");
        }
    }
    let o = value(derive_blocks(
        input("```\n```"),
        BlockOutputMode::Structured,
    ));
    assert_eq!(o["blocks"][0]["text"], "");
    let engine = OrihaEngine::default().with_limits(EngineLimits {
        max_nesting_depth: 1,
        ..Default::default()
    });
    assert_eq!(
        value(engine.derive_blocks(input("> > deep"), BlockOutputMode::Structured))["status"],
        "incomplete"
    );
}

struct DuplicateNotes;
impl RenderHook for DuplicateNotes {
    fn after_document_parse(&self, d: &mut Document, _: &mut RenderState, _: &mut RenderContext) {
        d.footnotes = ["first", "last"]
            .into_iter()
            .map(|s| FootnoteDefinition {
                label: "n".into(),
                children: vec![InlineKind::Text(s.into()).into()],
            })
            .collect();
    }
}
#[test]
fn duplicate_footnotes_keep_ir_order_and_existing_resolution_precedence() {
    let e = OrihaEngine::default().add_hook(DuplicateNotes);
    let p = e.parse(input("ref[^n]\n\n[^n]: original"));
    let s = value(e.derive_parsed_blocks(&p, BlockOutputMode::Structured));
    assert_eq!(s["footnotes"][0]["richText"][0]["text"], "first");
    assert_eq!(s["footnotes"][1]["richText"][0]["text"], "last");
    let r = value(e.derive_parsed_blocks(&p, BlockOutputMode::Rendered));
    let section = r["blocks"][1]["html"].as_str().unwrap();
    assert!(section.contains("last"));
    assert!(!section.contains("first"));
}

#[test]
fn owned_nullable_structured_positions_survive_wire_roundtrips() {
    let BlockOutput::Structured(mut output) =
        derive_blocks(input("**text**"), BlockOutputMode::Structured)
    else {
        unreachable!()
    };
    output.blocks.push(None);
    if let StructuredBlockProperties::Paragraph { rich_text, .. } =
        &mut output.blocks[0].as_mut().unwrap().content
    {
        rich_text.push(None);
        rich_text[0].as_mut().unwrap().children.push(None);
    }
    output.blocks[0].as_mut().unwrap().children.push(None);
    let wire = serde_json::to_value(&output).unwrap();
    assert!(wire["blocks"][1].is_null());
    assert!(wire["blocks"][0]["properties"]["richText"][1].is_null());
    assert!(wire["blocks"][0]["properties"]["richText"][0]["children"][1].is_null());
    assert!(wire["blocks"][0]["children"][0].is_null());
    let roundtrip: StructuredBlockOutput = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(roundtrip).unwrap(), wire);
}

#[test]
fn nested_inline_source_fails_safely_in_a_real_process() {
    const CHILD: &str = "ORIHA_BLOCK_INLINE_DEPTH_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let sources = [
            format!("{}x{}", "![".repeat(300), "](/image)".repeat(300)),
            format!("{}x{}", "[".repeat(300), "](/link)".repeat(300)),
            format!("{}x", "> ".repeat(300)),
            format!("{}x{}", "**![".repeat(80), "](/image)**".repeat(80)),
        ];
        for (markdown, expected) in
            sources
                .into_iter()
                .zip(["failed", "failed", "incomplete", "failed"])
        {
            for mode in [BlockOutputMode::Structured, BlockOutputMode::Rendered] {
                let o = value(derive_blocks(input(&markdown), mode));
                assert_eq!(o["status"], expected);
                assert!(o["blocks"].as_array().unwrap().is_empty());
            }
        }
        let ordinary = format!("{}x{}", "![".repeat(16), "](/image)".repeat(16));
        for mode in [BlockOutputMode::Structured, BlockOutputMode::Rendered] {
            assert_eq!(
                value(derive_blocks(input(&ordinary), mode))["status"],
                "complete"
            );
            assert_eq!(
                value(derive_blocks(input("after failure"), mode))["status"],
                "complete"
            );
        }
        return;
    }
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "nested_inline_source_fails_safely_in_a_real_process",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "child status: {}; stderr: {}",
        child.status,
        String::from_utf8_lossy(&child.stderr)
    );
}
