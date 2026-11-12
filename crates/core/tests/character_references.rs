#![allow(clippy::single_range_in_vec_init)]

use oriha_core::ir::{BlockKind, InlineKind};
use oriha_core::{
    parse, render_parsed, AnnotationAttributes, AnnotationMetadata, FeatureFlags, ParseStatus,
    ParsedDocument, RenderInput, RenderMode, TextAnnotation, TextProvenance,
};

fn parsed(source: &str) -> ParsedDocument {
    parse(RenderInput {
        markdown: source.into(),
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            gfm: false,
            callouts: false,
            math: false,
            mermaid: false,
            plantuml: false,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    })
}

#[test]
fn every_named_reference_keeps_original_extent_through_parse_and_render() {
    let vocabulary: serde_json::Value =
        serde_json::from_str(include_str!("../entity-data/whatwg-entities.json")).unwrap();
    let mut count = 0;
    for (name, value) in vocabulary.as_object().unwrap() {
        if !name.ends_with(';') {
            continue;
        }
        count += 1;
        let decoded = value["characters"].as_str().unwrap();
        let source = format!("p{name}q");
        let doc = parsed(&source);
        assert_eq!(doc.parse_status(), ParseStatus::Complete, "{name}");
        assert_eq!(doc.source(), source);
        let region = doc.text_regions().next().unwrap();
        assert_eq!(region.text, format!("p{decoded}q"), "{name}");
        let mapping = &region
            .mappings
            .iter()
            .find(|m| m.mapping.provenance == TextProvenance::CharacterReference)
            .unwrap()
            .mapping;
        assert_eq!(mapping.display, 1..1 + decoded.len(), "{name}");
        assert_eq!(mapping.source, vec![1..1 + name.len()], "{name}");
        assert!(!mapping.exact, "{name}");
        let escaped = decoded
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        assert!(
            render_parsed(&doc).html.contains(&format!("p{escaped}q")),
            "{name}"
        );
    }
    assert_eq!(count, 2125);
}

#[test]
fn numeric_grammar_unknown_names_and_missing_semicolons_are_literal() {
    for source in [
        "&#+64;bob",
        "&#x+40;bob",
        "&#-64;bob",
        "&#x-40;bob",
        "&#000000064;bob",
        "&#x00000040;bob",
        "&#00000064;bob",
        "&#x0000040;bob",
        "&#;bob",
        "&#x;bob",
        "&#６４;bob",
        "&#64bob",
        "&#x40bob",
        "&COMMAT;bob",
        "&notanentity;bob",
        "&commat bob",
        "&commat",
    ] {
        let doc = parsed(source);
        let region = doc.text_regions().next().unwrap();
        assert_eq!(region.text, source);
        assert!(region
            .mappings
            .iter()
            .all(|m| m.mapping.provenance == TextProvenance::Literal));
        assert!(render_parsed(&doc)
            .html
            .contains(&source.replace('&', "&amp;")));
    }
    for (source, visible) in [
        ("&#0000064;bob", "@bob"),
        ("&#X000040;bob", "@bob"),
        ("&#0;", "\u{fffd}"),
        ("&#xD800;", "\u{fffd}"),
        ("&#1114112;", "\u{fffd}"),
        ("&#xFFFFFF;", "\u{fffd}"),
        ("&#128;", "\u{80}"),
    ] {
        let doc = parsed(source);
        assert_eq!(doc.text_regions().next().unwrap().text, visible);
    }
}

#[test]
fn partial_annotations_on_two_codepoint_expansion_keep_the_whole_source() {
    let source = "日本語 &NotEqualTilde; &commat;ali**ce**";
    let doc = parsed(source);
    let region = doc.text_regions().next().unwrap();
    let start = region.text.find('≂').unwrap();
    let entity_start = source.find('&').unwrap();
    let entity_range = entity_start..entity_start + "&NotEqualTilde;".len();
    let annotation = TextAnnotation {
        region: region.id,
        range: start + '≂'.len_utf8()..start + "≂\u{338}".len(),
        metadata: AnnotationMetadata {
            kind: "test".into(),
            id: "part".into(),
            attributes: AnnotationAttributes::default(),
        },
    };
    let decorated = doc.apply_annotations(&[annotation]).unwrap();
    assert_eq!(decorated.source(), source);
    let decorated_region = decorated.text_regions().next().unwrap();
    for mapping in decorated_region.mappings.iter().filter(|m| {
        m.mapping.provenance == TextProvenance::CharacterReference
            && m.mapping.display.start < start + "≂\u{338}".len()
    }) {
        assert_eq!(mapping.mapping.source, vec![entity_range.clone()]);
        assert!(!mapping.mapping.exact);
    }
    assert!(render_parsed(&decorated)
        .html
        .contains("data-oriha-annotation-id=\"part\">\u{338}</span>"));
    assert!(!render_parsed(&doc)
        .html
        .contains("data-oriha-annotation-id"));
}

#[test]
fn decoding_never_introduces_markdown_or_reinterprets_escaped_references() {
    for source in [
        "&ast;x&ast;",
        "&lowbar;x&lowbar;",
        "&grave;x&grave;",
        "&lt;b&gt;x&lt;/b&gt;",
    ] {
        let doc = parsed(source);
        let BlockKind::Paragraph(p) = &doc.document().blocks[0].kind else {
            panic!()
        };
        assert!(p
            .children
            .iter()
            .all(|n| matches!(n.kind, InlineKind::Text(_))));
    }
    for source in [
        r"\&commat;bob",
        r"[\&commat;bob][ref]

[ref]: /url",
    ] {
        let doc = parsed(source);
        let region = doc.text_regions().next().unwrap();
        assert_eq!(region.text, "&commat;bob");
        assert_eq!(
            region.mappings[0].mapping.provenance,
            TextProvenance::Escape
        );
    }
    assert_eq!(
        parsed("&amp;commat;bob")
            .text_regions()
            .next()
            .unwrap()
            .text,
        "&commat;bob"
    );
    for source in [
        "`&commat;bob`",
        "    &commat;bob\n",
        "```\n&commat;bob\n```\n",
    ] {
        assert!(render_parsed(&parsed(source))
            .html
            .contains("&amp;commat;bob"));
    }
}

#[test]
fn link_metadata_is_decoded_once_and_autolinks_remain_raw() {
    for (source, destination, title) in [
        ("[x](/&commat; \"&NotEqualTilde;\")", "/@", Some("≂\u{338}")),
        ("[x](/&amp;commat;)", "/&commat;", None),
        (r"[x](/\&commat;)", "/&commat;", None),
        ("[x]\n\n[x]: /&amp;commat;", "/&commat;", None),
        ("[x]\n\n[x]: /\\&commat;", "/&commat;", None),
        (
            "<https://example.com/&commat;>",
            "https://example.com/&commat;",
            None,
        ),
        (
            "<https://example.com/&#64;>",
            "https://example.com/&#64;",
            None,
        ),
    ] {
        let doc = parsed(source);
        let BlockKind::Paragraph(p) = &doc.document().blocks[0].kind else {
            panic!()
        };
        let InlineKind::Link {
            destination: actual,
            title: actual_title,
            ..
        } = &p.children[0].kind
        else {
            panic!("{source}")
        };
        assert_eq!(actual, destination, "{source}");
        assert_eq!(actual_title.as_deref(), title, "{source}");
        assert!(
            render_parsed(&doc)
                .html
                .contains(&format!("href=\"{}\"", destination.replace('&', "&amp;"))),
            "{source}"
        );
    }
    let html = render_parsed(&parsed("```f&amp;amp;\nx\n```\n")).html;
    assert!(html.contains("language-f&amp;amp;"), "{html}");
}

#[test]
fn consumer_predecessor_boundaries_see_decoded_text_and_at_sign_provenance() {
    // Repo-owned cases cover decoded predecessor and at-sign provenance.
    for (source, visible, eligible) in [
        ("&commat;bob", "@bob", true),
        ("&commat;@bob", "@@bob", false),
        ("&colon;@bob", ":@bob", false),
        ("&sol;@bob", "/@bob", false),
        ("&period;@bob", ".@bob", false),
        ("&lowbar;@bob", "_@bob", false),
        ("&percnt;@bob", "%@bob", false),
        ("&#+64;bob", "&#+64;bob", false),
    ] {
        let doc = parsed(source);
        let region = doc.text_regions().next().unwrap();
        assert_eq!(region.text, visible);
        let found = visible.char_indices().any(|(i, c)| {
            c == '@'
                && visible[i + 1..].starts_with("bob")
                && visible[..i]
                    .chars()
                    .next_back()
                    .is_none_or(|p| !p.is_alphanumeric() && !"_-/.:@%".contains(p))
                && region.mappings.iter().any(|m| {
                    m.mapping.display.contains(&i)
                        && matches!(
                            m.mapping.provenance,
                            TextProvenance::Literal | TextProvenance::CharacterReference
                        )
                })
        });
        assert_eq!(found, eligible, "{source}");
    }
}
