use oriha_core::render::{RenderContext, RenderState};
use oriha_core::{
    ir::Document, EngineLimits, OrihaEngine, ParseStatus, ParsedDocument, RenderHook, RenderInput,
    Severity,
};

fn engine(source: usize, blocks: usize, depth: usize) -> OrihaEngine {
    OrihaEngine::default().with_limits(EngineLimits {
        max_source_bytes: source,
        max_blocks: blocks,
        max_nesting_depth: depth,
        max_output_bytes: 262144,
    })
}
fn parse(engine: &OrihaEngine, source: &str) -> ParsedDocument {
    engine.parse(RenderInput {
        markdown: source.into(),
        ..Default::default()
    })
}
fn visible(parsed: &ParsedDocument) -> Vec<String> {
    parsed.text_regions().map(|r| r.text).collect()
}

#[test]
fn source_and_block_limits_have_distinct_machine_readable_outcomes() {
    let e = engine(4, 2, 64);
    for text in ["xxx", "xxxx"] {
        assert_eq!(parse(&e, text).parse_status(), ParseStatus::Complete);
    }
    let failed = parse(&e, "xxxxx");
    assert_eq!(failed.parse_status(), ParseStatus::Failed);
    assert!(failed
        .diagnostics()
        .iter()
        .any(|d| d.code == "engine.max_source_bytes_exceeded"));
    let e = engine(65536, 2, 64);
    for count in [1, 2, 3] {
        let parsed = parse(&e, &"# @tail\n\n".repeat(count));
        assert_eq!(
            parsed.parse_status(),
            if count <= 2 {
                ParseStatus::Complete
            } else {
                ParseStatus::Incomplete
            }
        );
        assert_eq!(visible(&parsed).len(), count.min(2));
        if count == 3 {
            assert!(parsed
                .diagnostics()
                .iter()
                .any(|d| d.code == "engine.max_blocks_exceeded"
                    && matches!(d.severity, Severity::Warning)));
        }
    }
    // Blank lines and already collected definitions cannot cause false truncation.
    assert_eq!(
        parse(&e, "# one\n\n# two\n\n[ref]: /url\n\n[^unused]: @hidden\n").parse_status(),
        ParseStatus::Complete
    );
    assert_eq!(
        parse(&engine(65536, 0, 0), "\n[^unused]: @hidden\n").parse_status(),
        ParseStatus::Complete
    );
}

#[test]
fn nested_container_depth_is_inherited_and_omissions_reach_the_outer_result() {
    for (source, crossings) in [
        ("> @tail\n", 1),
        ("- @tail\n", 1),
        ("> > @tail\n", 2),
        ("- outer\n  - @tail\n", 2),
        ("> - @tail\n", 2),
        ("- > @tail\n", 2),
        (":::note\n@tail\n:::\n", 1),
        ("> [!NOTE]\n> @tail\n", 1),
        ("> [!NOTE]\n> > @tail\n", 2),
        (":::note\n> - @tail\n:::\n", 3),
    ] {
        for depth in 0..=crossings + 1 {
            let parsed = parse(&engine(65536, 4096, depth), source);
            let complete = depth >= crossings;
            assert_eq!(
                parsed.parse_status(),
                if complete {
                    ParseStatus::Complete
                } else {
                    ParseStatus::Incomplete
                },
                "depth={depth}, {source:?}"
            );
            assert_eq!(
                visible(&parsed).iter().any(|s| s.contains("@tail")),
                complete,
                "depth={depth}, {source:?}"
            );
            assert_eq!(
                parsed
                    .diagnostics()
                    .iter()
                    .any(|d| d.code == "engine.max_nesting_depth_exceeded"),
                !complete,
                "depth={depth}, {source:?}"
            );
        }
    }
    // Blocks remain per invocation, not an implicit global descendant budget.
    let e = engine(65536, 1, 64);
    assert_eq!(
        parse(&e, "> @first\n>\n> @omitted\n").parse_status(),
        ParseStatus::Incomplete
    );
    assert_eq!(
        parse(&e, "- @first\n\n  @omitted\n").parse_status(),
        ParseStatus::Incomplete
    );
    assert_eq!(
        parse(&e, ":::note\n@first\n\n@omitted\n:::\n").parse_status(),
        ParseStatus::Incomplete
    );
    assert_eq!(
        parse(&e, "- @one\n- @two\n").parse_status(),
        ParseStatus::Complete
    );
    for depth in [63, 64, 65] {
        let parsed = parse(&engine(65536, 4096, 64), &("> ".repeat(depth) + "@tail\n"));
        assert_eq!(
            parsed.parse_status(),
            if depth <= 64 {
                ParseStatus::Complete
            } else {
                ParseStatus::Incomplete
            }
        );
        assert_eq!(
            visible(&parsed).iter().any(|r| r.contains("@tail")),
            depth <= 64
        );
    }
}

struct ClearDiagnostics;
impl RenderHook for ClearDiagnostics {
    fn after_document_parse(
        &self,
        doc: &mut Document,
        state: &mut RenderState,
        _: &mut RenderContext,
    ) {
        doc.diagnostics.clear();
        state.diagnostics.clear();
    }
}
struct Panic;
impl RenderHook for Panic {
    fn before_document_parse(
        &self,
        _: &mut RenderInput,
        _: &mut RenderState,
        _: &mut RenderContext,
    ) {
        panic!("test parsing failure");
    }
}

#[test]
fn status_survives_cloning_annotation_hooks_and_repeat_rendering() {
    let e = engine(65536, 1, 0);
    let parsed = parse(&e, "> @tail\n");
    let diagnostics = serde_json::to_value(parsed.diagnostics()).unwrap();
    for doc in [parsed.clone(), parsed.apply_annotations(&[]).unwrap()] {
        assert_eq!(doc.parse_status(), ParseStatus::Incomplete);
        let first = e.render_parsed(&doc);
        let second = e.render_parsed(&doc);
        assert_eq!(first.html, second.html);
        assert_eq!(
            serde_json::to_value(&first.diagnostics).unwrap(),
            diagnostics
        );
        assert_eq!(
            serde_json::to_value(&second.diagnostics).unwrap(),
            diagnostics
        );
        assert_eq!(
            serde_json::to_value(doc.diagnostics()).unwrap(),
            diagnostics
        );
        assert_eq!(doc.parse_status(), ParseStatus::Incomplete);
    }
    let cleared = parse(&e.add_hook(ClearDiagnostics), "> @tail\n");
    assert!(cleared.diagnostics().is_empty());
    assert_eq!(cleared.parse_status(), ParseStatus::Incomplete);
    let failed = parse(&OrihaEngine::default().add_hook(Panic), "@tail");
    assert_eq!(failed.parse_status(), ParseStatus::Failed);
    assert!(failed
        .diagnostics()
        .iter()
        .any(|d| d.code == "engine.panic"));
    assert_eq!(failed.clone().parse_status(), ParseStatus::Failed);
    assert_eq!(
        failed.apply_annotations(&[]).unwrap().parse_status(),
        ParseStatus::Failed
    );
}

#[test]
fn visibility_recovery_warnings_and_render_limits_are_separate_from_parsing() {
    let e = engine(65536, 4096, 0);
    for source in ["> \n", "- \n\n", ":::note\n:::\n"] {
        let empty = parse(&e, source);
        assert_eq!(empty.parse_status(), ParseStatus::Complete, "{source}");
        assert!(visible(&empty).is_empty());
    }
    for source in [
        "`@hidden`",
        "![hello @hidden](/image)",
        "[^unused]: @hidden",
        "```\n@code",
        "main[^used]\n\n[^used]: @body\n    continuation\n",
    ] {
        assert_eq!(
            parse(&e, source).parse_status(),
            ParseStatus::Complete,
            "{source}"
        );
    }
    assert_eq!(
        visible(&parse(&e, "main[^used]\n\n[^used]: @body\n")),
        ["main", "@body"]
    );
    assert_eq!(
        parse(&engine(65536, 0, 0), "main[^used]\n\n[^used]: @body\n").parse_status(),
        ParseStatus::Incomplete
    );
    let warning = parse(&e, "```\n@code");
    assert!(warning
        .diagnostics()
        .iter()
        .any(|d| matches!(d.severity, Severity::Warning)));
    let tiny_output = OrihaEngine::default().with_limits(EngineLimits {
        max_output_bytes: 1,
        ..EngineLimits::default()
    });
    let full = parse(&tiny_output, "@tail");
    assert_eq!(full.parse_status(), ParseStatus::Complete);
    assert!(tiny_output
        .render_parsed(&full)
        .diagnostics
        .iter()
        .any(|d| d.code == "engine.max_output_bytes_exceeded"));
}

#[test]
fn adoption_scope_fixtures_are_executable_consumer_lexer_inputs() {
    let e = OrihaEngine::default();
    for (source, expected) in [
        ("![hello @bob](https://x/image)", vec![]),
        ("main[^used]\n\n[^used]: @bob", vec!["@bob"]),
        ("word:rocket:@bob", vec!["@bob"]),
    ] {
        let parsed = parse(&e, source);
        assert_eq!(parsed.parse_status(), ParseStatus::Complete);
        let candidates: Vec<_> = parsed
            .text_regions()
            .flat_map(|r| {
                r.text
                    .split_whitespace()
                    .filter(|s| s.starts_with('@'))
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(candidates, expected, "{source}");
    }
}
