use oriha_core::ir::{BlockKind, InlineKind};
use oriha_core::{parse, render_parsed, FeatureFlags, RenderInput, RenderMode};

fn input(source: &str, gfm: bool) -> RenderInput {
    RenderInput {
        markdown: source.to_string(),
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            gfm,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    }
}

#[test]
fn bare_namespaces_and_unknown_schemes_remain_text() {
    for source in [
        "Std::Lang::...",
        "Std::Lang::Type",
        "std::vector",
        "foo::bar",
        "A::B",
        "MyNamespace::Class",
        "ns_name::Type",
        "Foo:Bar",
        "note:value",
        "made-up-scheme://example.com/path",
        "made-up-scheme:hello@example.com",
        "mailto::Lang",
        "xmpp::Lang",
        "http::Lang",
        "http:example.com",
        "https:example.com",
        "http:",
        "http:.",
        "http://",
        "https://...",
        "xmpp:room",
        "xmpp://hello@example.com",
        "説明 Std::Lang::Type の説明",
    ] {
        let parsed = parse(input(source, true));
        let output = render_parsed(&parsed);
        assert!(
            output.diagnostics.is_empty(),
            "{source}: {:?}",
            output.diagnostics
        );
        assert!(!output.html.contains("<a "), "{source}: {}", output.html);
        let regions: Vec<_> = parsed.text_regions().collect();
        assert_eq!(regions.len(), 1, "{source}");
        assert_eq!(regions[0].text, source);
        assert_eq!(regions[0].mappings.len(), 1, "{source}");
        let expected_source = 0..source.len();
        assert_eq!(
            regions[0].mappings[0].mapping.source.as_slice(),
            std::slice::from_ref(&expected_source)
        );
        assert!(regions[0].mappings[0].mapping.exact);
    }
}

#[test]
fn recognized_bare_urls_and_email_protocols_keep_their_destinations() {
    for (source, destination) in [
        ("http://example.com", "http://example.com"),
        ("https://example.com/path", "https://example.com/path"),
        ("HTTPS://example.com/path", "HTTPS://example.com/path"),
        ("https://example.com/@tail", "https://example.com/@tail"),
        ("www.example.com", "http://www.example.com"),
        ("hello@example.com", "mailto:hello@example.com"),
        ("mailto:hello@example.com", "mailto:hello@example.com"),
        ("MAILTO:hello@example.com", "MAILTO:hello@example.com"),
        ("xmpp:hello@example.com/path", "xmpp:hello@example.com/path"),
    ] {
        let parsed = parse(input(source, true));
        let BlockKind::Paragraph(paragraph) = &parsed.document().blocks[0].kind else {
            panic!("expected paragraph: {source}");
        };
        let [node] = paragraph.children.as_slice() else {
            panic!("expected one link: {source}");
        };
        let InlineKind::Link {
            destination: actual,
            ..
        } = &node.kind
        else {
            panic!("expected link: {source}");
        };
        assert_eq!(actual, destination);
        assert!(render_parsed(&parsed).html.contains("<a href="));
        let region = parsed.text_regions().next().unwrap();
        assert_eq!(region.text, source);
        let expected_source = 0..source.len();
        assert_eq!(
            region.mappings[0].mapping.source.as_slice(),
            std::slice::from_ref(&expected_source)
        );

        let disabled = parse(input(source, false));
        assert!(!render_parsed(&disabled).html.contains("<a "), "{source}");
    }
}

#[test]
fn explicit_commonmark_links_still_allow_unregistered_schemes() {
    for gfm in [false, true] {
        for source in [
            "<Std::Lang::Type>",
            "<made-up-scheme://example.com/path>",
            "[reference](Std::Lang::Type)",
            "[reference](made-up-scheme://example.com/path)",
        ] {
            let parsed = parse(input(source, gfm));
            let output = render_parsed(&parsed);
            assert!(output.diagnostics.is_empty());
            assert_eq!(
                output.html.matches("<a ").count(),
                1,
                "{source}: {}",
                output.html
            );
        }
        let code = parse(input("`Std::Lang::Type`", gfm));
        let output = render_parsed(&code);
        assert!(!output.html.contains("<a "));
        assert!(output.html.contains("<code>Std::Lang::Type</code>"));
    }
}

#[test]
fn rejecting_a_namespace_does_not_hide_later_links_or_shift_source_ranges() {
    let prefix = "名前 Std::Lang::Type ";
    let url = "https://example.com/@tail";
    let source = format!("{prefix}{url}");
    let parsed = parse(input(&source, true));
    let output = render_parsed(&parsed);
    assert_eq!(output.html.matches("<a ").count(), 1);
    assert!(output.html.contains("Std::Lang::Type <a href="));
    let region = parsed.text_regions().next().unwrap();
    assert_eq!(region.text, source);
    let link_mapping = region
        .mappings
        .iter()
        .find(|m| matches!(&m.node.kind, InlineKind::Text(text) if text == url))
        .unwrap();
    assert_eq!(link_mapping.mapping.display, prefix.len()..source.len());
    let expected_source = prefix.len()..source.len();
    assert_eq!(
        link_mapping.mapping.source.as_slice(),
        std::slice::from_ref(&expected_source)
    );
}
