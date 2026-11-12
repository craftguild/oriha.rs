#![allow(clippy::single_range_in_vec_init)]

use oriha_core::ir::{BlockKind, Document, InlineKind, InlineNode};
use oriha_core::render::{RenderContext, RenderHook, RenderState};
use oriha_core::{
    parse, render_parsed, AnnotationAttributes, AnnotationMetadata, OrihaEngine, RenderInput,
    TextAnnotation, TextMapping, TextProvenance,
};

fn input(source: &str) -> RenderInput {
    RenderInput {
        markdown: source.into(),
        ..Default::default()
    }
}

#[test]
fn unused_footnotes_do_not_expose_visible_regions() {
    let doc = parse(input("ordinary prose\n\n[^unused]: @hidden\n"));
    assert!(!render_parsed(&doc).html.contains("@hidden"));
    assert!(doc
        .text_regions()
        .all(|region| !region.text.contains("@hidden")));
}

#[test]
fn footnote_regions_and_annotation_targets_follow_rendered_reference_order() {
    let doc = parse(input(
        "main[^b] then[^a] and again[^b]\n\n[^unused]: @hidden\n\n[^a]: @alpha\n\n[^b]: @beta\n",
    ));
    let regions: Vec<_> = doc
        .text_regions()
        .filter(|r| r.text.contains('@'))
        .collect();
    assert_eq!(
        regions.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
        ["@beta", "@alpha"]
    );
    let annotations: Vec<_> = regions
        .iter()
        .enumerate()
        .map(|(i, region)| TextAnnotation {
            region: region.id,
            range: 0..region.text.len(),
            metadata: AnnotationMetadata {
                kind: "mention".into(),
                id: format!("note-{i}"),
                attributes: AnnotationAttributes::default(),
            },
        })
        .collect();
    let annotated = doc.apply_annotations(&annotations).unwrap();
    let html = render_parsed(&annotated).html;
    assert!(html.contains("data-oriha-annotation-id=\"note-0\">@beta</span>"));
    assert!(html.contains("data-oriha-annotation-id=\"note-1\">@alpha</span>"));
    assert!(!html.contains("@hidden"));
    assert!(html.find("@beta").unwrap() < html.find("@alpha").unwrap());
    assert!(!render_parsed(&doc)
        .html
        .contains("data-oriha-annotation-id"));
}

#[test]
fn nested_only_and_image_alt_footnotes_match_existing_render_visibility() {
    // The renderer snapshots main-flow references before rendering note bodies.
    // A nested reference alone does not currently emit its definition's body.
    let doc = parse(input(
        "main[^outer] ![alt[^image]](url) `[^code]`\n\n[^outer]: @outer [^inner]\n\n[^inner]: @nested_only\n\n[^image]: @image_only\n\n[^code]: @code_only\n",
    ));
    let html = render_parsed(&doc).html;
    assert!(html.contains("@outer"));
    for hidden in ["@nested_only", "@image_only", "@code_only"] {
        assert!(!html.contains(hidden));
        assert!(doc.text_regions().all(|r| !r.text.contains(hidden)));
    }
    let doc = parse(input(
        "main[^outer] also[^inner]\n\n[^outer]: @outer [^inner]\n\n[^inner]: @visible_inner\n",
    ));
    assert!(doc
        .text_regions()
        .any(|r| r.text.contains("@visible_inner")));
    assert!(render_parsed(&doc).html.contains("@visible_inner"));
}

fn first_nodes(document: &Document) -> &[InlineNode] {
    let BlockKind::Paragraph(p) = &document.blocks[0].kind else {
        panic!("paragraph")
    };
    &p.children
}

struct EditValue {
    emoji: bool,
    value: &'static str,
    mapping: Option<Vec<TextMapping>>,
}
impl RenderHook for EditValue {
    fn after_document_parse(&self, doc: &mut Document, _: &mut RenderState, _: &mut RenderContext) {
        let BlockKind::Paragraph(p) = &mut doc.blocks[0].kind else {
            panic!("paragraph")
        };
        for node in &mut p.children {
            match &mut node.kind {
                InlineKind::Code(s) if !self.emoji => *s = self.value.into(),
                InlineKind::Emoji { emoji, .. } if self.emoji => *emoji = self.value.into(),
                _ => continue,
            }
            if let Some(mapping) = &self.mapping {
                node.source_mapping = mapping.clone();
            }
        }
    }
}

fn assert_generated_edit(source: &str, emoji: bool, value: &'static str) {
    let doc = OrihaEngine::default()
        .add_hook(EditValue {
            emoji,
            value,
            mapping: None,
        })
        .parse(input(source));
    let node = &first_nodes(doc.document())[0];
    if value.is_empty() {
        assert!(node.source_mapping.is_empty());
        return;
    }
    assert_eq!(node.source_mapping.len(), 1);
    let mapping = &node.source_mapping[0];
    assert_eq!(mapping.display, 0..value.len());
    assert_eq!(mapping.provenance, TextProvenance::Generated);
    assert!(mapping.source.is_empty());
    assert!(!mapping.exact);
}

#[test]
fn code_hook_edits_without_correspondence_are_generated() {
    for value in ["changed code content", "diff", ""] {
        assert_generated_edit("`code`", false, value);
    }
}

#[test]
fn emoji_hook_edits_without_correspondence_are_generated() {
    for value in ["changed emoji content", "🙂", ""] {
        assert_generated_edit(":rocket:", true, value);
    }
}

#[test]
fn valid_explicit_replacement_mapping_survives_code_and_emoji_edits() {
    for (source, emoji, value, range) in [
        ("`code` tail", false, "tail", 7..11),
        (":rocket: 🙂", true, "🙂", 9..13),
    ] {
        let mapping = vec![TextMapping::literal(0..value.len(), range)];
        let doc = OrihaEngine::default()
            .add_hook(EditValue {
                emoji,
                value,
                mapping: Some(mapping.clone()),
            })
            .parse(input(source));
        assert_eq!(first_nodes(doc.document())[0].source_mapping, mapping);
    }
}

#[test]
fn invalid_explicit_mapping_is_not_retained_for_code_or_emoji() {
    for (source, emoji) in [("`code`", false), (":rocket:", true)] {
        let doc = OrihaEngine::default()
            .add_hook(EditValue {
                emoji,
                value: "changed",
                mapping: Some(vec![TextMapping::literal(0..7, 100..107)]),
            })
            .parse(input(source));
        assert_eq!(
            first_nodes(doc.document())[0].source_mapping[0],
            TextMapping::generated(0..7)
        );
    }
}

struct Noop;
impl RenderHook for Noop {}

#[test]
fn many_node_noop_hook_preserves_original_text_code_and_emoji_mappings() {
    let source = "repeated `code` :rocket: **日本語** &amp;\n\n".repeat(1500);
    let plain = parse(input(&source));
    let hooked = OrihaEngine::default().add_hook(Noop).parse(input(&source));
    assert_eq!(
        format!("{:?}", plain.document()),
        format!("{:?}", hooked.document())
    );
}

#[test]
fn large_multiline_paragraph_preserves_late_unicode_and_entity_ranges() {
    let source = format!("{}日本語 &#64;tail\r\n", "simple words\r\n".repeat(2000));
    let doc = parse(input(&source));
    let region = doc
        .text_regions()
        .find(|r| r.text.contains("日本語"))
        .unwrap();
    let mapping = &region.mappings[0].mapping;
    assert_eq!(mapping.source[0].start, "simple words\r\n".len() * 2000);
    for part in &region.mappings {
        for range in &part.mapping.source {
            assert!(source.is_char_boundary(range.start) && source.is_char_boundary(range.end));
        }
    }
    let at = region.text.find('@').unwrap();
    let at_mapping = &region
        .mappings
        .iter()
        .find(|m| m.mapping.display.contains(&at))
        .unwrap()
        .mapping;
    assert_eq!(at_mapping.provenance, TextProvenance::CharacterReference);
    assert_eq!(&source[at_mapping.source[0].clone()], "&#64;");
}
