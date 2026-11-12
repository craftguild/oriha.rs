use oriha_core::render::{RenderContext, RenderState};
use oriha_core::*;
use std::sync::{Arc, Mutex};

fn input(markdown: &str) -> RenderInput {
    RenderInput {
        markdown: markdown.into(),
        mode: RenderMode::SemanticHtml,
        ..Default::default()
    }
}
fn prepared(source: &str) -> ImagePresentation {
    ImagePresentation::new(
        source,
        vec![
            ImageCandidate::new("/large.png", 960).unwrap(),
            ImageCandidate::new("/small.png", 320).unwrap(),
        ],
        Some("(max-width: 40em) 100vw, 40em".into()),
        Some((960, 640)),
        ImageLayout::Responsive,
    )
    .unwrap()
}
struct Prepared(ImagePresentation);
impl RenderHook for Prepared {
    fn image_presentation<'a>(
        &'a self,
        _: &ImageRenderRequest<'_>,
        _: &RenderContext<'_>,
    ) -> Option<&'a ImagePresentation> {
        Some(&self.0)
    }
}
fn blocks(engine: &OrihaEngine, markdown: &str) -> RenderedBlockOutput {
    let BlockOutput::Rendered(o) = engine.derive_blocks(input(markdown), BlockOutputMode::Rendered)
    else {
        unreachable!()
    };
    assert_eq!(o.status, BlockOutputStatus::Complete, "{:?}", o.diagnostics);
    o
}
fn join(o: &RenderedBlockOutput) -> String {
    o.blocks.iter().filter_map(|b| b.html()).collect()
}
fn body(html: &str) -> &str {
    html.split_once('>')
        .unwrap()
        .1
        .strip_suffix("</article>")
        .unwrap()
}

#[test]
fn all_placements_agree_in_direct_and_retained_document_and_block_apis() {
    let markdown = "![figure **alt**](/original.png \"caption & title\")\n\nText ![inline](/original.png \"tooltip\") and [![linked](/original.png)](/target).\n\n# Heading ![heading](/original.png)\n\n> ![quote](/original.png)\n\n- ![list](/original.png)\n\n> [!NOTE]\n> ![callout](/original.png)\n\n| ![header](/original.png) | value |\n| --- | --- |\n| ![cell](/original.png) | other |\n\nref[^n]\n\n[^n]: note ![footnote](/original.png \"foot title\")";
    let engine = OrihaEngine::default().add_hook(Prepared(prepared("/fallback.png")));
    let parsed = engine.parse(input(markdown));
    let before = format!("{:?}", parsed.document());
    let direct = blocks(&engine, markdown);
    let BlockOutput::Rendered(retained) =
        engine.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)
    else {
        unreachable!()
    };
    assert_eq!(
        serde_json::to_value(&retained).unwrap(),
        serde_json::to_value(&direct).unwrap()
    );
    let html = join(&direct);
    assert_eq!(body(&engine.render(input(markdown)).html), html);
    assert_eq!(body(&engine.render_parsed(&parsed).html), html);
    assert_eq!(html.matches("srcset=\"").count(), 10, "{html}");
    assert_eq!(html.matches("width=\"960\" height=\"640\"").count(), 10);
    for marker in [
        "<figure id=",
        "oriha-image--captioned",
        "<figcaption",
        "caption &amp; title",
        "alt=\"figure alt\"",
        "title=\"tooltip\"",
        "<a href=\"/target\"><img",
        "data-oriha-heading-level=\"1\"",
        "oriha-blockquote",
        "<ul",
        "oriha-callout",
        "<th",
        "<td",
        "oriha-footnotes",
        "title=\"foot title\"",
    ] {
        assert!(html.contains(marker), "{marker}: {html}");
    }
    assert!(!html.contains("width: 300px"));
    assert!(html.contains("height: auto; max-width: min(100%, 960px); max-height: 640px;"));
    assert!(!html.contains("title=\"caption"));
    for block in &parsed.document().blocks {
        assert_eq!(
            direct.blocks.iter().find(|b| b.id == block.id).unwrap().id,
            block.id
        );
    }
    let baseline = OrihaEngine::default().render(input(markdown));
    assert_eq!(
        serde_json::to_value(engine.render_parsed(&parsed).source_map).unwrap(),
        serde_json::to_value(baseline.source_map).unwrap()
    );
    assert_eq!(format!("{:?}", parsed.document()), before);
}

#[test]
fn explicit_display_bounds_do_not_replace_intrinsic_dimensions() {
    let engine = OrihaEngine::default().add_hook(Prepared(prepared("/fallback")));
    for markdown in [
        "![x](/original?w=240&h=160)",
        "text ![x](/original#width=240&height=160)",
    ] {
        let html = join(&blocks(&engine, markdown));
        assert!(html.contains("width=\"960\" height=\"640\""), "{html}");
        assert!(
            html.contains("max-width: min(100%, 240px); max-height: 160px;"),
            "{html}"
        );
        assert!(!html.contains("300px"));
    }
    let fixed = ImagePresentation::new("/fixed", vec![], None, None, ImageLayout::Fixed).unwrap();
    assert!(join(&blocks(
        &OrihaEngine::default().add_hook(Prepared(fixed)),
        "![x](/orig)"
    ))
    .contains("width: 300px"));
}

#[test]
fn malformed_construction_is_rejected_without_silent_repairs() {
    assert_eq!(
        ImageCandidate::new("", 1).unwrap_err(),
        ImagePresentationError::EmptyCandidateUrl
    );
    assert_eq!(
        ImageCandidate::new("/x", 0).unwrap_err(),
        ImagePresentationError::ZeroCandidateWidth
    );
    let candidate = ImageCandidate::new("/x", 300).unwrap();
    assert_eq!(candidate.url(), "/x");
    assert_eq!(candidate.width(), 300);
    let make = |c, s, d| ImagePresentation::new("/x", c, s, d, ImageLayout::Responsive);
    assert_eq!(
        make(
            vec![candidate.clone(), candidate.clone()],
            Some("100vw".into()),
            None
        )
        .unwrap_err(),
        ImagePresentationError::DuplicateCandidateWidth(300)
    );
    for sizes in [None, Some("".into()), Some(" \t\n".into())] {
        assert_eq!(
            make(vec![candidate.clone()], sizes, None).unwrap_err(),
            ImagePresentationError::MissingSizes
        );
    }
    for dimensions in [(0, 0), (1, 0), (0, 1)] {
        assert_eq!(
            make(vec![], None, Some(dimensions)).unwrap_err(),
            ImagePresentationError::InvalidIntrinsicDimensions
        );
    }
    assert!(make(vec![], None, None).is_ok());
    assert!(make(vec![candidate], Some("100vw".into()), Some((u32::MAX, 1))).is_ok());
}

struct DefaultNoOverride;
impl RenderHook for DefaultNoOverride {}

#[test]
fn default_method_preserves_styled_html_css_and_serialized_outputs() {
    let baseline = OrihaEngine::default();
    let hooked = OrihaEngine::default().add_hook(DefaultNoOverride);
    let mut input =
        input("![caption](/a?w=123&h=45 \"caption\")\n\ntext ![inline](/b)\n\n# ![heading](/c)");
    input.mode = RenderMode::StyledHtml;
    for limit in [1, 2048, EngineLimits::default().max_output_bytes] {
        let a = OrihaEngine::default().with_limits(EngineLimits {
            max_output_bytes: limit,
            ..Default::default()
        });
        let b = OrihaEngine::default()
            .add_hook(DefaultNoOverride)
            .with_limits(EngineLimits {
                max_output_bytes: limit,
                ..Default::default()
            });
        assert_eq!(
            serde_json::to_value(a.render(input.clone())).unwrap(),
            serde_json::to_value(b.render(input.clone())).unwrap()
        );
    }
    let a = baseline.parse(input.clone());
    let b = hooked.parse(input.clone());
    assert_eq!(
        serde_json::to_value(baseline.render(input.clone())).unwrap(),
        serde_json::to_value(hooked.render(input)).unwrap()
    );
    for mode in [BlockOutputMode::Structured, BlockOutputMode::Rendered] {
        assert_eq!(
            serde_json::to_value(baseline.derive_parsed_blocks(&a, mode)).unwrap(),
            serde_json::to_value(hooked.derive_parsed_blocks(&b, mode)).unwrap()
        );
    }
}

struct Observe {
    name: &'static str,
    presentation: Option<ImagePresentation>,
    calls: Arc<Mutex<Vec<String>>>,
}
impl RenderHook for Observe {
    fn image_presentation<'a>(
        &'a self,
        image: &ImageRenderRequest<'_>,
        context: &RenderContext<'_>,
    ) -> Option<&'a ImagePresentation> {
        assert_eq!(image.destination, "/parsed");
        assert_eq!(image.title, Some("parsed title"));
        assert!(!image.alt.is_empty());
        assert!(matches!(context.mode, RenderMode::SemanticHtml));
        self.calls
            .lock()
            .unwrap()
            .push(format!("{}:{:?}", self.name, image.placement));
        self.presentation.as_ref()
    }
}
struct ParseRewrite;
impl RenderHook for ParseRewrite {
    fn after_document_parse(
        &self,
        d: &mut oriha_core::ir::Document,
        _: &mut RenderState,
        _: &mut RenderContext,
    ) {
        let oriha_core::ir::BlockKind::Paragraph(p) = &mut d.blocks[0].kind else {
            unreachable!()
        };
        for node in &mut p.children {
            if let oriha_core::ir::InlineKind::Image {
                destination, title, ..
            } = &mut node.kind
            {
                *destination = "/parsed".into();
                *title = Some("parsed title".into());
            }
        }
    }
}
#[test]
fn first_override_wins_after_parse_hooks_and_none_preserves_legacy_output() {
    for markdown in ["![alt](/before)", "text ![alt](/before)"] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let observe = |name, presentation| Observe {
            name,
            presentation,
            calls: calls.clone(),
        };
        let engine = OrihaEngine::default()
            .add_hook(ParseRewrite)
            .add_hook(observe("none", None))
            .add_hook(observe("first", Some(prepared("/first"))))
            .add_hook(observe("last", Some(prepared("/last"))));
        let html = join(&blocks(&engine, markdown));
        assert!(html.contains("src=\"/first\""));
        assert!(!html.contains("/last"));
        let placement = if markdown.starts_with("text") {
            "Inline"
        } else {
            "Standalone"
        };
        assert_eq!(
            *calls.lock().unwrap(),
            [format!("none:{placement}"), format!("first:{placement}")]
        );
        let none = OrihaEngine::default()
            .add_hook(ParseRewrite)
            .add_hook(observe("none", None));
        let baseline = OrihaEngine::default().add_hook(ParseRewrite);
        assert_eq!(
            serde_json::to_value(none.render(input(markdown))).unwrap(),
            serde_json::to_value(baseline.render(input(markdown))).unwrap()
        );
        assert_eq!(
            serde_json::to_value(blocks(&none, markdown)).unwrap(),
            serde_json::to_value(blocks(&baseline, markdown)).unwrap()
        );
    }
}

#[test]
fn responsive_display_intersects_intrinsic_and_authored_bounds() {
    for (destination, width, height) in [
        ("/original", 7, 11),
        ("/original?width=999&height=999", 7, 11),
        ("/original?width=3&height=4", 3, 4),
    ] {
        let presentation = ImagePresentation::new(
            "/fallback",
            vec![ImageCandidate::new("/tiny", 7).unwrap()],
            Some("100vw".into()),
            Some((7, 11)),
            ImageLayout::Responsive,
        )
        .unwrap();
        let engine = OrihaEngine::default().add_hook(Prepared(presentation));
        let markdown = format!("![tiny]({destination})\n\ninline ![tiny]({destination})");
        let html = join(&blocks(&engine, &markdown));
        assert_eq!(html.matches("width=\"7\" height=\"11\"").count(), 2);
        assert_eq!(html.matches(&format!("width: auto; height: auto; max-width: min(100%, {width}px); max-height: {height}px;")).count(), 2, "{html}");
    }
    let unknown =
        ImagePresentation::new("/original", vec![], None, None, ImageLayout::Responsive).unwrap();
    let html = join(&blocks(
        &OrihaEngine::default().add_hook(Prepared(unknown)),
        "![unknown](/original)",
    ));
    assert!(html.contains("width: auto; height: auto; max-width: 100%;"));
    assert!(!html.contains("max-height:"));
}
