//! Regressions from the independent ADR-0002 production review.
use oriha_core::ir::{BlockNode, Document};
use oriha_core::render::{RenderContext, RenderFragment, RenderState};
use oriha_core::*;

fn input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.into(),
        mode: RenderMode::SemanticHtml,
        ..Default::default()
    }
}
fn rendered(output: BlockOutput) -> RenderedBlockOutput {
    let BlockOutput::Rendered(output) = output else {
        panic!("expected rendered output")
    };
    output
}

#[test]
fn authored_inline_html_obeys_retained_flag_in_roots_and_footnotes() {
    let tags = [
        "<img src=x onerror=alert(1)>",
        "<img src=x onerror=alert(2)>",
        "<svg onload=alert(3)>",
        "<img src=x onerror=alert(4)>",
        "<img src=x onerror=alert(5)>",
        "<img src=x onerror=alert(6)>",
    ];
    let source = format!(
        "Before {} after **bold {}**\n\n# Heading {}\n\n> quote {}\n\n| header |\n| --- |\n| cell {} |\n\nA note[^n].\n\n[^n]: footnote *{}*\n",
        tags[0], tags[1], tags[2], tags[3], tags[4], tags[5],
    );
    for raw_html in [false, true] {
        let mut i = input(&source);
        i.features.raw_html = raw_html;
        let producer = OrihaEngine::default();
        let parsed = producer.parse(i);
        // The deriving engine's feature profile must not override retained input.
        let receiver = OrihaEngine::default().with_features(FeatureFlags {
            raw_html: !raw_html,
            ..Default::default()
        });
        let output = rendered(receiver.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered));
        assert_eq!(output.status, BlockOutputStatus::Complete);
        let fragments = output
            .blocks
            .iter()
            .filter_map(RenderedHtmlBlock::html)
            .collect::<String>();
        let ordinary = producer.render_parsed(&parsed);
        let body = ordinary
            .html
            .split_once('>')
            .unwrap()
            .1
            .strip_suffix("</article>")
            .unwrap();
        assert_eq!(fragments, body);
        assert!(fragments.contains("<strong>bold "));
        assert!(fragments.contains("<em>"));
        assert_eq!(output.blocks.last().unwrap().kind, HtmlBlockKind::Footnotes);
        for tag in tags {
            let escaped = tag.replace('<', "&lt;").replace('>', "&gt;");
            if raw_html {
                assert!(fragments.contains(tag), "{tag}: {fragments}");
            } else {
                assert!(!fragments.contains(tag), "active authored tag: {tag}");
                assert!(fragments.contains(&escaped), "{tag}: {fragments}");
            }
        }
        // Structured output retains authored data regardless of execution policy.
        let structured = serde_json::to_string(
            &producer.derive_parsed_blocks(&parsed, BlockOutputMode::Structured),
        )
        .unwrap();
        for tag in tags {
            assert!(structured.contains(tag));
        }
    }
}

#[test]
fn context_free_inline_renderer_obeys_raw_html_without_changing_tagfilter() {
    let engine = OrihaEngine::default();
    for raw_html in [false, true] {
        for gfm in [false, true] {
            let features = FeatureFlags {
                raw_html,
                gfm,
                ..Default::default()
            };
            let html = engine.inline_renderer.render(
                "Before **<img src=x onerror=alert(1)>** <script>alert(2)</script> after",
                &features,
                &mut RenderStats::default(),
            );
            let image = "<img src=x onerror=alert(1)>";
            assert!(html.contains("<strong>"));
            if raw_html {
                assert!(html.contains(image));
            } else {
                assert!(!html.contains(image));
                assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
            }
            if raw_html && !gfm {
                assert!(html.contains("<script>"));
            } else {
                assert!(!html.contains("<script>"));
                assert!(html.contains("&lt;script"));
            }
        }
    }
}

struct AddResources {
    assets: bool,
    diagnostics: bool,
}
impl AddResources {
    fn add(&self, id: &str, state: &mut RenderState) {
        if self.assets {
            state.assets.push(RenderAsset {
                id: id.into(),
                kind: AssetKind::Other,
                mime_type: "application/octet-stream".into(),
                content: "日本 \"quoted\" \\ newline\n".into(),
            });
        }
        if self.diagnostics {
            state.diagnostics.push(RenderDiagnostic {
                severity: Severity::Warning,
                code: "review.budget".into(),
                message: "日本 \"quoted\" \\ newline\n".into(),
                block_id: Some(id.into()),
                source_range: None,
                engine: serde_json::from_str("\"core\"").unwrap(),
            });
        }
    }
}
impl RenderHook for AddResources {
    fn after_document_parse(
        &self,
        _: &mut Document,
        state: &mut RenderState,
        _: &mut RenderContext,
    ) {
        self.add("retained", state);
    }
    fn after_block_render(
        &self,
        block: &mut BlockNode,
        _: &mut RenderFragment,
        state: &mut RenderState,
        _: &RenderContext,
    ) {
        self.add(&block.id, state);
    }
}
fn derive_with_resources(
    i: RenderInput,
    limit: usize,
    assets: bool,
    diagnostics: bool,
) -> RenderedBlockOutput {
    rendered(
        OrihaEngine::default()
            .with_limits(EngineLimits {
                max_output_bytes: limit,
                ..Default::default()
            })
            .add_hook(AddResources {
                assets,
                diagnostics,
            })
            .derive_blocks(i, BlockOutputMode::Rendered),
    )
}

#[test]
fn rendered_budget_accepts_exact_size_for_many_roots_and_resource_combinations() {
    for roots in [0, 1, 50, 100, 200, 500] {
        for (assets, diagnostics) in [(false, false), (true, false), (false, true), (true, true)] {
            let i = input(&"x\n\n".repeat(roots));
            let expected = derive_with_resources(i.clone(), 2_000_000, assets, diagnostics);
            assert_eq!(expected.status, BlockOutputStatus::Complete);
            assert_eq!(expected.blocks.len(), roots);
            let bytes = serde_json::to_vec(&expected).unwrap().len();
            if roots == 100 && !assets && !diagnostics {
                assert_eq!(bytes, 13_981);
            }
            let exact = derive_with_resources(i.clone(), bytes, assets, diagnostics);
            assert_eq!(
                exact.status,
                BlockOutputStatus::Complete,
                "roots={roots} assets={assets} diagnostics={diagnostics} bytes={bytes}: {:?}",
                exact.diagnostics
            );
            assert_eq!(
                serde_json::to_value(exact).unwrap(),
                serde_json::to_value(expected).unwrap()
            );
            let below = derive_with_resources(i, bytes - 1, assets, diagnostics);
            assert_eq!(below.status, BlockOutputStatus::Failed);
            assert!(below.blocks.is_empty());
            assert!(below.assets.is_empty());
            assert_eq!(below.diagnostics[0].code, "blocks.output_limit");
        }
    }
}

#[test]
fn rendered_budget_includes_generated_footnotes_and_deduplicated_math_font() {
    let mut i = input(&format!(
        "{}\n\nA note[^n].\n\n[^n]: $x$ and $y$\n",
        "paragraph\n\n".repeat(100)
    ));
    i.features.math = true;
    let expected = rendered(derive_blocks(i.clone(), BlockOutputMode::Rendered));
    assert_eq!(expected.status, BlockOutputStatus::Complete);
    assert_eq!(expected.assets.len(), 1);
    assert_eq!(
        expected.blocks.last().unwrap().kind,
        HtmlBlockKind::Footnotes
    );
    let bytes = serde_json::to_vec(&expected).unwrap().len();
    for (limit, status) in [
        (bytes, BlockOutputStatus::Complete),
        (bytes - 1, BlockOutputStatus::Failed),
    ] {
        let actual = rendered(
            OrihaEngine::default()
                .with_limits(EngineLimits {
                    max_output_bytes: limit,
                    ..Default::default()
                })
                .derive_blocks(i.clone(), BlockOutputMode::Rendered),
        );
        assert_eq!(actual.status, status);
        if status == BlockOutputStatus::Complete {
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                serde_json::to_value(&expected).unwrap()
            );
        } else {
            assert!(actual.blocks.is_empty());
            assert!(actual.assets.is_empty());
        }
    }
}
