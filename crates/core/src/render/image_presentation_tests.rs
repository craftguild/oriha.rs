use super::*;
use std::sync::{atomic::AtomicUsize, Arc};

#[derive(Default)]
struct Counts {
    parse: AtomicUsize,
    present: AtomicUsize,
    finalize: AtomicUsize,
}
struct Prepared {
    counts: Arc<Counts>,
    presentation: ImagePresentation,
}
impl RenderHook for Prepared {
    fn after_document_parse(&self, _: &mut Document, _: &mut RenderState, _: &mut RenderContext) {
        self.counts.parse.fetch_add(1, Ordering::Relaxed);
    }
    fn image_presentation<'a>(
        &'a self,
        _: &ImageRenderRequest<'_>,
        _: &RenderContext<'_>,
    ) -> Option<&'a ImagePresentation> {
        self.counts.present.fetch_add(1, Ordering::Relaxed);
        Some(&self.presentation)
    }
    fn before_output_finalize(&self, _: &Document, _: &mut RenderState, _: &mut RenderContext) {
        self.counts.finalize.fetch_add(1, Ordering::Relaxed);
    }
    fn after_output_finalize(
        &self,
        _: &Document,
        _: &mut RenderOutput,
        _: &mut RenderState,
        _: &mut RenderContext,
    ) {
        self.counts.finalize.fetch_add(1, Ordering::Relaxed);
    }
}
fn engine(counts: Arc<Counts>, url: &str) -> OrihaEngine {
    OrihaEngine::default().add_hook(Prepared {
        counts,
        presentation: ImagePresentation::new(
            url,
            vec![ImageCandidate::new("/candidate", 640).unwrap()],
            Some("100vw".into()),
            Some((640, 480)),
            ImageLayout::Responsive,
        )
        .unwrap(),
    })
}
#[test]
fn real_parser_counters_and_hooks_prove_presentation_is_render_only_and_isolated() {
    crate::parser::PARSE_CALLS.with(|c| c.set((0, 0)));
    let a = Arc::new(Counts::default());
    let b = Arc::new(Counts::default());
    let producer = engine(a.clone(), "/a");
    let receiver = engine(b.clone(), "/b");
    let parsed = producer.parse(RenderInput {
        markdown: "![figure](/orig)\n\n# ![heading](/orig)\n\nref[^n]\n\n[^n]: ![foot](/orig)"
            .into(),
        ..Default::default()
    });
    let calls = crate::parser::PARSE_CALLS.with(|c| c.get());
    assert!(calls.0 > 0 && calls.1 > 0);
    assert_eq!(a.parse.load(Ordering::Relaxed), 1);
    assert_eq!(a.present.load(Ordering::Relaxed), 0);
    let before = format!("{:?}", parsed.document());
    let structured = serde_json::to_value(
        producer.derive_parsed_blocks(&parsed, crate::BlockOutputMode::Structured),
    )
    .unwrap();
    for _ in 0..3 {
        for (e, url) in [(&producer, "/a"), (&receiver, "/b")] {
            let result = e.derive_parsed_blocks(&parsed, crate::BlockOutputMode::Rendered);
            let crate::BlockOutput::Rendered(result) = result else {
                unreachable!()
            };
            assert_eq!(result.status, crate::BlockOutputStatus::Complete);
            let html: String = result.blocks.iter().filter_map(|b| b.html()).collect();
            assert_eq!(html.matches(&format!("src=\"{url}\"")).count(), 3);
            assert_eq!(e.hooks.len(), 1);
            assert_eq!(
                serde_json::to_value(
                    e.derive_parsed_blocks(&parsed, crate::BlockOutputMode::Structured)
                )
                .unwrap(),
                structured
            );
        }
        assert_eq!(a.finalize.load(Ordering::Relaxed), 0);
        assert_eq!(b.finalize.load(Ordering::Relaxed), 0);
    }
    assert_eq!(a.present.load(Ordering::Relaxed), 9);
    assert_eq!(b.present.load(Ordering::Relaxed), 9);
    producer.render_parsed(&parsed);
    receiver.render_parsed(&parsed);
    assert_eq!(a.present.load(Ordering::Relaxed), 12);
    assert_eq!(b.present.load(Ordering::Relaxed), 12);
    assert_eq!(a.finalize.load(Ordering::Relaxed), 2);
    assert_eq!(b.finalize.load(Ordering::Relaxed), 2);
    assert_eq!(a.parse.load(Ordering::Relaxed), 1);
    assert_eq!(b.parse.load(Ordering::Relaxed), 0);
    assert_eq!(crate::parser::PARSE_CALLS.with(|c| c.get()), calls);
    assert_eq!(format!("{:?}", parsed.document()), before);
    let no_override = OrihaEngine::default().render_parsed(&parsed);
    assert!(!no_override.html.contains("srcset"));
    assert!(no_override.html.contains("width: 300px"));
}
