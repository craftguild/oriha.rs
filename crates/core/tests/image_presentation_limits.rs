use oriha_core::render::RenderContext;
use oriha_core::*;

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
fn presentation(source: &str, urls: &[String], sizes: &str) -> ImagePresentation {
    ImagePresentation::new(
        source,
        urls.iter()
            .enumerate()
            .map(|(i, u)| ImageCandidate::new(u, i as u32 + 1).unwrap())
            .collect(),
        Some(sizes.into()),
        Some((640, 480)),
        ImageLayout::Responsive,
    )
    .unwrap()
}
fn engine(limit: usize, p: ImagePresentation) -> OrihaEngine {
    OrihaEngine::default()
        .with_limits(EngineLimits {
            max_output_bytes: limit,
            ..Default::default()
        })
        .add_hook(Prepared(p))
}
fn input() -> RenderInput {
    RenderInput {
        markdown: "![alt](/orig)\n\ntext ![inline](/orig)".into(),
        mode: RenderMode::SemanticHtml,
        ..Default::default()
    }
}
#[test]
fn exact_document_and_aggregate_block_limits_include_expanded_attributes() {
    let p = presentation(
        "/日本語?a=1&b=2",
        &["/a,b c.png".into(), "/x%20y".into()],
        "100vw & \"",
    );
    let generous = engine(100_000, p.clone());
    let expected = generous.derive_blocks(input(), BlockOutputMode::Rendered);
    let exact = serde_json::to_vec(&expected).unwrap().len();
    for (limit, status) in [
        (exact, BlockOutputStatus::Complete),
        (exact - 1, BlockOutputStatus::Failed),
        (1, BlockOutputStatus::Failed),
    ] {
        let e = engine(limit, p.clone());
        let parsed = e.parse(input());
        for output in [
            e.derive_blocks(input(), BlockOutputMode::Rendered),
            e.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered),
        ] {
            let BlockOutput::Rendered(ref b) = output else {
                unreachable!()
            };
            assert_eq!(b.status, status, "limit {limit}: {b:?}");
            if status == BlockOutputStatus::Complete {
                assert_eq!(
                    serde_json::to_value(output).unwrap(),
                    serde_json::to_value(&expected).unwrap()
                );
            } else {
                assert!(b.blocks.is_empty());
                assert!(b.assets.is_empty());
            }
        }
    }
    let expected = generous.render(input());
    for (limit, complete) in [
        (expected.html.len(), true),
        (expected.html.len() - 1, false),
        (1, false),
    ] {
        let e = engine(limit, p.clone());
        for o in [e.render(input()), e.render_parsed(&e.parse(input()))] {
            assert!(o.html.len() <= limit);
            assert_eq!(
                !o.diagnostics.iter().any(|d| d.severity == Severity::Error),
                complete
            );
            if complete {
                assert_eq!(o.html, expected.html);
            }
        }
    }
}

#[test]
fn oversized_sources_candidates_sizes_and_aggregate_candidates_fail_boundedly() {
    let huge = "&".repeat(32_768);
    let ps = [
        presentation(&huge, &["/ok".into()], "100vw"),
        presentation("/ok", std::slice::from_ref(&huge), "100vw"),
        presentation("/ok", &["/ok".into()], &huge),
        presentation(
            "/ok",
            &(0..200)
                .map(|i| format!("/candidate-{i}"))
                .collect::<Vec<_>>(),
            "100vw",
        ),
        // Escaping expansion must be checked even when the raw value fits.
        presentation("/ok", &["/ok".into()], &"&".repeat(500)),
    ];
    for p in ps {
        let e = engine(2048, p);
        let parsed = e.parse(input());
        for output in [
            e.derive_blocks(input(), BlockOutputMode::Rendered),
            e.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered),
        ] {
            let BlockOutput::Rendered(b) = output else {
                unreachable!()
            };
            assert_eq!(b.status, BlockOutputStatus::Failed);
            assert!(b.blocks.is_empty());
            assert!(serde_json::to_vec(&b).unwrap().len() <= 2048);
        }
        for o in [e.render(input()), e.render_parsed(&parsed)] {
            assert!(o.html.len() <= 2048);
            assert!(o
                .diagnostics
                .iter()
                .any(|d| d.code == "engine.max_output_bytes_exceeded"));
        }
        // The failed scope cannot leak a limit or error into another derivation.
        let o = OrihaEngine::default().derive_parsed_blocks(&parsed, BlockOutputMode::Rendered);
        let BlockOutput::Rendered(b) = o else {
            unreachable!()
        };
        assert_eq!(b.status, BlockOutputStatus::Complete);
        assert!(b.blocks[0].html().unwrap().contains("width: 300px"));
    }
}
