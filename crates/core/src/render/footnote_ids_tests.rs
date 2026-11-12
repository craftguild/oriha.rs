use super::*;
use crate::{BlockOutput, BlockOutputMode};
use std::collections::BTreeSet;

fn ids(html: &str) -> Vec<String> {
    html.split(" id=\"")
        .skip(1)
        .map(|part| part.split('"').next().unwrap().to_string())
        .collect()
}

fn check(parsed: &ParsedDocument) -> String {
    let engine = OrihaEngine::default();
    let BlockOutput::Rendered(output) =
        engine.derive_parsed_blocks(parsed, BlockOutputMode::Rendered)
    else {
        panic!("rendered")
    };
    assert_eq!(output.status, crate::BlockOutputStatus::Complete);
    let html = output
        .blocks
        .iter()
        .filter_map(|b| b.html())
        .collect::<String>();
    let anchors = builtin_html_anchors(parsed);
    assert_eq!(anchors, ids(&html));
    assert_eq!(
        anchors.len(),
        anchors.iter().collect::<BTreeSet<_>>().len(),
        "{html}"
    );
    let ordinary = engine.render_parsed(parsed).html;
    let ordinary_ids = ids(&ordinary);
    assert_eq!(ordinary_ids, anchors);

    let references: Vec<_> = html
        .split("<sup id=\"")
        .skip(1)
        .map(|part| {
            let id = part.split('"').next().unwrap();
            let destination = part
                .split("href=\"#")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap();
            let number = part
                .split("data-oriha-footnote-ref=\"true\">")
                .nth(1)
                .unwrap()
                .split('<')
                .next()
                .unwrap();
            (id, destination, number)
        })
        .collect();
    for part in html.split("<li id=\"").skip(1) {
        let entry = part.split("</li>").next().unwrap();
        if !entry.contains("oriha-footnotes__item") {
            continue;
        }
        let id = entry.split('"').next().unwrap();
        let number = entry
            .split("value=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let backref = entry
            .rsplit("href=\"#")
            .next()
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let first = references.iter().find(|r| r.2 == number).unwrap();
        assert_eq!(backref, first.0, "{entry}");
        assert!(references
            .iter()
            .filter(|r| r.2 == number)
            .all(|r| r.1 == id));
    }
    for (_, destination, _) in &references {
        assert!(
            anchors.iter().any(|id| id == destination),
            "missing {destination}"
        );
    }
    let title = html
        .split("aria-labelledby=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    assert!(anchors.iter().any(|id| id == title));
    html
}

fn parse(markdown: &str) -> ParsedDocument {
    OrihaEngine::default().parse(RenderInput {
        markdown: markdown.into(),
        mode: RenderMode::SemanticHtml,
        ..Default::default()
    })
}

#[test]
fn distinct_colliding_labels_keep_their_own_links_and_backlinks() {
    for labels in [
        vec!["a b", "a-b"],
        vec!["a-b", "a b"],
        vec!["!!!", "???", "note", "😀"],
        vec!["日 本", "日-本", "日.本"],
    ] {
        let mut source = labels
            .iter()
            .map(|label| format!("ref[^{label}] "))
            .collect::<String>();
        source.push_str(&format!("again[^{}]\n\n", labels[0]));
        for (index, label) in labels.iter().enumerate() {
            source.push_str(&format!("[^{label}]: Definition {index}\n"));
        }
        let html = check(&parse(&source));
        assert_eq!(html.matches("oriha-footnotes__item").count(), labels.len());
    }
}

#[test]
fn reference_counts_cannot_take_another_labels_first_reference_id() {
    for body in [
        "[^a] [^a] [^a-2] [^a-2] [^a]",
        "[^a-2] [^a] [^a] [^a-2] [^a]",
    ] {
        let html = check(&parse(&format!("{body}\n\n[^a]: A\n[^a-2]: A two\n")));
        assert!(html.contains("<li id=\"fn-a\""));
        assert!(html.contains("<li id=\"fn-a-2\""));
    }
}

#[test]
fn later_headings_and_footnotes_title_share_one_reserved_namespace() {
    let html = check(&parse("refs[^a] [^a b] [^a-b] [^a]\n\n# fn a-b\n\n# fnref a\n\n# oriha footnotes label\n\n[^a]: A\n[^a b]: Space\n[^a-b]: Dash\n"));
    assert!(html.contains("<h1 id=\"fn-a-b\""));
    assert!(html.contains("<h1 id=\"fnref-a\""));
    assert!(html.contains("aria-labelledby=\"oriha-footnotes-label--footnotes\""));
}

#[test]
fn noncolliding_ids_unused_definitions_and_definition_precedence_are_preserved() {
    let html = check(&parse("ref[^note] again[^note] and[^a b]\n\n[^note]: first source definition\n[^note]: ignored duplicate definition\n[^a b]: Used\n[^a-b]: Unused\n"));
    for id in [
        "fn-note",
        "fnref-note",
        "fnref-note-2",
        "fn-a-b",
        "fnref-a-b",
        "oriha-footnotes-label",
    ] {
        assert!(ids(&html).iter().any(|v| v == id));
    }
    assert!(html.contains("first source definition"));
    assert!(!html.contains("ignored duplicate definition"));
    assert!(!html.contains("Unused"));
}

#[test]
fn application_block_ids_cannot_occupy_the_fallback_identity() {
    struct ExtraBlock;
    impl RenderHook for ExtraBlock {
        fn after_document_parse(
            &self,
            doc: &mut Document,
            _: &mut RenderState,
            _: &mut RenderContext,
        ) {
            doc.blocks.push(BlockNode::new(
                "fn-a-b--612d62",
                BlockKind::ThematicBreak,
                crate::source::SourceSpan::new(0, 0),
            ));
        }
    }
    let engine = OrihaEngine::default().add_hook(ExtraBlock);
    let parsed = engine.parse(RenderInput {
        markdown: "[^a b] [^a-b]\n\n[^a b]: Space\n[^a-b]: Dash\n".into(),
        mode: RenderMode::SemanticHtml,
        ..Default::default()
    });
    let html = check(&parsed);
    assert!(html.contains("<li id=\"fn-a-b--612d62-2\""));
}
