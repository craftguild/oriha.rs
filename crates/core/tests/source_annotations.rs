// Expected source mappings deliberately contain singleton ranges.
#![allow(clippy::single_range_in_vec_init)]
use std::collections::BTreeMap;
use std::ops::Range;

use oriha_core::ir::{BlockKind, InlineKind};
use oriha_core::{
    parse, render, render_parsed, AnnotationAttributes, AnnotationError, AnnotationMetadata,
    OrihaEngine, ParsedDocument, RenderInput, TextAnnotation, TextProvenance, TextRegion,
};

fn parsed(source: &str) -> ParsedDocument {
    parse(RenderInput {
        markdown: source.to_string(),
        ..Default::default()
    })
}

fn annotation(region: &TextRegion<'_>, range: Range<usize>, id: &str) -> TextAnnotation {
    TextAnnotation {
        region: region.id,
        range,
        metadata: AnnotationMetadata {
            kind: "mention".into(),
            id: id.into(),
            attributes: AnnotationAttributes::default(),
        },
    }
}

// Project a visible subrange through public correspondence, retaining an entire
// transformed source extent when bytewise slicing is not defined.
fn source_ranges(region: &TextRegion<'_>, range: Range<usize>) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    for segment in &region.mappings {
        let m = &segment.mapping;
        let start = range.start.max(m.display.start);
        let end = range.end.min(m.display.end);
        if start >= end {
            continue;
        }
        let next = if m.exact {
            vec![
                m.source[0].start + start - m.display.start
                    ..m.source[0].start + end - m.display.start,
            ]
        } else {
            m.source.clone()
        };
        for r in next {
            if let Some(last) = ranges.last_mut() {
                let last: &mut Range<usize> = last;
                if *last == r {
                    continue;
                }
                if last.end == r.start {
                    last.end = r.end;
                    continue;
                }
            }
            ranges.push(r);
        }
    }
    ranges
}

#[test]
fn mention_provenance_is_recorded_at_parse_time() {
    let doc = parsed(r"@alice \@alice \\@alice &#64;alice @ali**ce**");
    let regions: Vec<_> = doc.text_regions().collect();
    assert_eq!(regions.len(), 1);
    let region = &regions[0];
    assert_eq!(region.text, "@alice @alice \\@alice @alice @alice");
    let at_signs: Vec<_> = region.text.match_indices('@').map(|(i, _)| i).collect();
    let origins: Vec<_> = at_signs
        .iter()
        .map(|i| {
            region
                .mappings
                .iter()
                .find(|s| s.mapping.display.contains(i))
                .unwrap()
                .mapping
                .provenance
        })
        .collect();
    assert_eq!(
        origins,
        [
            TextProvenance::Literal,
            TextProvenance::Escape,
            TextProvenance::Literal,
            TextProvenance::CharacterReference,
            TextProvenance::Literal
        ]
    );
    assert_eq!(
        source_ranges(region, at_signs[0]..at_signs[0] + 6),
        vec![0..6]
    );
    assert_eq!(
        source_ranges(region, at_signs[1]..at_signs[1] + 1),
        vec![7..9]
    );
    assert_eq!(
        source_ranges(region, at_signs[2]..at_signs[2] + 1),
        vec![17..18]
    );
    assert_eq!(
        source_ranges(region, at_signs[3]..at_signs[3] + 1),
        vec![24..29]
    );
    assert_eq!(
        source_ranges(region, at_signs[4]..at_signs[4] + 6),
        vec![35..39, 41..43]
    );
    let BlockKind::Paragraph(paragraph) = &doc.document().blocks[0].kind else {
        panic!()
    };
    let strong = paragraph
        .children
        .iter()
        .find(|n| matches!(n.kind, InlineKind::Strong { .. }))
        .unwrap();
    assert_eq!(strong.source_mapping[0].source, vec![39..45]);
}

#[test]
fn nested_normalization_composes_to_complete_original_input() {
    let doc = parsed("> - 日本語 @alice\r\n>   - 🙂 @ali**ce**\r\n");
    let regions: Vec<_> = doc.text_regions().collect();
    assert_eq!(
        regions.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
        ["日本語 @alice", "🙂 @alice"]
    );
    assert_eq!(source_ranges(&regions[0], 0..9), vec![4..13]);
    assert_eq!(source_ranges(&regions[0], 10..16), vec![14..20]);
    assert_eq!(source_ranges(&regions[1], 0..4), vec![28..32]);
    assert_eq!(source_ranges(&regions[1], 5..11), vec![33..37, 39..41]);

    let callout = parsed("> [!NOTE]\r\n> \t@alice\r\n");
    let region = callout.text_regions().next().unwrap();
    assert_eq!(region.text, "@alice");
    assert_eq!(source_ranges(&region, 0..6), vec![14..20]);
    let fenced = parsed(":::note\r\n@alice\r\n:::\r\n");
    assert_eq!(
        source_ranges(&fenced.text_regions().next().unwrap(), 0..6),
        vec![9..15]
    );
}

#[test]
fn tables_footnotes_tabs_and_entities_retain_actual_ranges() {
    let table = parsed("| 日本語 \\| @alice | @alice |\r\n| --- | --- |\r\n| 🙂 | &ngE; |\r\n");
    let regions: Vec<_> = table.text_regions().collect();
    assert_eq!(
        regions.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
        ["日本語 | @alice", "@alice", "🙂", "≧̸"]
    );
    assert_eq!(source_ranges(&regions[0], 10..11), vec![12..14]);
    assert_eq!(
        regions[0]
            .mappings
            .iter()
            .find(|m| m.mapping.display.contains(&10))
            .unwrap()
            .mapping
            .provenance,
        TextProvenance::Escape
    );
    assert_eq!(source_ranges(&regions[0], 12..18), vec![15..21]);
    assert_eq!(source_ranges(&regions[1], 0..6), vec![24..30]);
    assert_eq!(source_ranges(&regions[2], 0..4), vec![51..55]);
    assert_eq!(source_ranges(&regions[3], 0..3), vec![58..63]);
    assert_eq!(source_ranges(&regions[3], 3..5), vec![58..63]);
    let decorated = table
        .apply_annotations(&[annotation(&regions[3], 0..3, "entity-part")])
        .unwrap();
    assert!(render_parsed(&decorated).html.contains("entity-part"));

    let footnote = parsed("text[^n]\r\n\r\n[^n]: @alice\r\n    日本語\r\n");
    let regions: Vec<_> = footnote.text_regions().collect();
    assert_eq!(
        regions.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
        ["text", "@alice", "日本語"]
    );
    assert_eq!(source_ranges(&regions[1], 0..6), vec![18..24]);
    assert_eq!(source_ranges(&regions[2], 0..9), vec![30..39]);
    let annotated = footnote
        .apply_annotations(&[annotation(&regions[1], 0..6, "footnote")])
        .unwrap();
    assert!(render_parsed(&annotated)
        .html
        .contains("data-oriha-annotation-id=\"footnote\""));

    let tabs = parsed("a\t日本語\r\n\tb\n");
    let regions: Vec<_> = tabs.text_regions().collect();
    assert_eq!(regions[0].text, "a\t日本語");
    assert_eq!(source_ranges(&regions[0], 1..2), vec![1..2]);
    assert_eq!(source_ranges(&regions[1], 0..1), vec![14..15]);
}

#[test]
fn regions_obey_structure_and_exclusion_boundaries() {
    let doc = parse(RenderInput {
        markdown: "@ali**ce** [label](https://destination/@hidden \"@title\") ~~end~~\nnext `@code` tail $x$ ![@alt](url) :rocket: [^n] <span data-user=\"@attr\">@inside</span>\n\n# @heading\n\n[^n]: @footnote\n".into(),
        features: oriha_core::FeatureFlags { math: true, ..Default::default() }, ..Default::default()
    });
    let regions: Vec<_> = doc.text_regions().collect();
    assert_eq!(
        regions.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
        [
            "@alice label end",
            "next ",
            " tail ",
            " ",
            " ",
            " ",
            " ",
            "@inside",
            "@heading",
            "@footnote"
        ]
    );
    assert!(regions.iter().all(|r| !r.text.contains("hidden")
        && !r.text.contains("@alt")
        && !r.text.contains("@attr")));
    let html = parsed("before<br>after");
    assert_eq!(
        html.text_regions().map(|r| r.text).collect::<Vec<_>>(),
        ["before", "after"]
    );
    let breaks = parsed("@ali  \nce\nnext\n\n:::note Generated title\nbody\n:::\n");
    assert_eq!(
        breaks.text_regions().map(|r| r.text).collect::<Vec<_>>(),
        ["@ali", "ce", "next", "body"]
    );
}

#[test]
fn annotations_preserve_formatting_links_anchors_stats_and_original() {
    let doc = parsed("# @ali**ce**\n\n[@ali**ce**](https://example.com) and @alice\n");
    let original = render_parsed(&doc);
    let regions: Vec<_> = doc.text_regions().collect();
    let annotated = doc
        .apply_annotations(&[
            annotation(&regions[0], 0..6, "heading"),
            annotation(&regions[1], 0..6, "linked"),
        ])
        .unwrap();
    let rendered = render_parsed(&annotated);
    assert_eq!(
        rendered
            .html
            .matches("data-oriha-annotation-id=\"heading\"")
            .count(),
        2
    );
    assert_eq!(
        rendered
            .html
            .matches("data-oriha-annotation-id=\"linked\"")
            .count(),
        2
    );
    assert!(rendered.html.contains("<strong><span data-oriha-annotation-kind=\"mention\" data-oriha-annotation-id=\"linked\">ce</span></strong></a>"));
    assert_eq!(rendered.stats, original.stats);
    assert_eq!(
        serde_json::to_value(&rendered.source_map).unwrap(),
        serde_json::to_value(&original.source_map).unwrap()
    );
    assert_eq!(rendered.css, original.css);
    assert!(rendered.html.contains("id=\"alice\""));
    assert_eq!(
        serde_json::to_value(render_parsed(&doc)).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    assert_eq!(
        serde_json::to_value(render_parsed(&annotated)).unwrap(),
        serde_json::to_value(&rendered).unwrap()
    );
}

#[test]
fn annotation_validation_is_atomic_and_document_scoped() {
    let doc = parsed("日本語 @alice and @bob");
    let region = doc.text_regions().next().unwrap();
    let before = render_parsed(&doc).html;
    let valid = annotation(&region, 10..16, "alice");
    for range in [0..0, 1..3, 0..100, Range { start: 20, end: 4 }] {
        let invalid = annotation(&region, range, "bad");
        assert_eq!(
            doc.apply_annotations(&[valid.clone(), invalid]).err(),
            Some(AnnotationError::InvalidRange)
        );
        assert_eq!(render_parsed(&doc).html, before);
    }
    assert_eq!(
        doc.apply_annotations(&[valid.clone(), annotation(&region, 12..17, "overlap")])
            .err(),
        Some(AnnotationError::Overlap)
    );
    let clone = doc.clone();
    assert_eq!(
        clone.apply_annotations(std::slice::from_ref(&valid)).err(),
        Some(AnnotationError::ForeignDocument)
    );
    let other = parsed("日本語 @alice and @bob");
    assert_eq!(
        other.apply_annotations(std::slice::from_ref(&valid)).err(),
        Some(AnnotationError::ForeignDocument)
    );
    let adjacent = doc
        .apply_annotations(&[valid, annotation(&region, 16..20, "adjacent")])
        .unwrap();
    let region = adjacent.text_regions().next().unwrap();
    assert_eq!(
        adjacent
            .apply_annotations(&[annotation(&region, 10..16, "again")])
            .err(),
        Some(AnnotationError::Overlap)
    );
    assert!(adjacent
        .apply_annotations(&[annotation(&region, 20..24, "bob")])
        .is_ok());
}

#[test]
fn typed_attributes_and_text_are_escaped_and_raw_html_remains_separate() {
    let doc = parsed("&lt;&amp;&quot; <span data-oriha-annotation-id=\"forged\">source</span>");
    let region = doc.text_regions().next().unwrap();
    assert_eq!(region.text, "<&\" ");
    let mut a = annotation(&region, 0..3, "id\"<&");
    a.metadata.kind = "kind\"<&".into();
    a.metadata.attributes = AnnotationAttributes {
        classes: vec!["token\"<&".into()],
        data: BTreeMap::from([("recipient".into(), "\"<&".into())]),
    };
    let annotated = doc.apply_annotations(std::slice::from_ref(&a)).unwrap();
    let html = render_parsed(&annotated).html;
    assert!(html.contains("data-oriha-annotation-id=\"id&quot;&lt;&amp;\""));
    assert!(html.contains("data-recipient=\"&quot;&lt;&amp;\""));
    assert!(html.contains(">&lt;&amp;\"</span>"));
    let BlockKind::Paragraph(paragraph) = &doc.document().blocks[0].kind else {
        panic!()
    };
    assert!(paragraph
        .children
        .iter()
        .all(|n| !matches!(n.kind, InlineKind::Annotation { .. })));
    for key in ["onclick", "bad\" onmouseover", "oriha-annotation-id"] {
        let mut invalid = a.clone();
        // A plain `onclick` suffix is safely a data attribute, never an event handler.
        invalid.metadata.attributes.data = BTreeMap::from([(key.into(), "value".into())]);
        if key == "onclick" {
            assert!(doc.apply_annotations(&[invalid]).is_ok());
        } else {
            assert_eq!(
                doc.apply_annotations(&[invalid]).err(),
                Some(AnnotationError::InvalidAttributes)
            );
        }
    }
}

#[test]
fn emoji_shortcodes_are_mapped_and_unannotated_api_output_agrees() {
    let input = RenderInput {
        markdown: "🙂 :rocket: @alice".into(),
        ..Default::default()
    };
    let doc = parse(input.clone());
    let BlockKind::Paragraph(paragraph) = &doc.document().blocks[0].kind else {
        panic!()
    };
    let emoji = paragraph
        .children
        .iter()
        .find(|n| matches!(n.kind, InlineKind::Emoji { .. }))
        .unwrap();
    assert_eq!(emoji.source_mapping[0].source, vec![5..13]);
    assert_eq!(
        emoji.source_mapping[0].provenance,
        TextProvenance::Normalized
    );
    assert_eq!(
        serde_json::to_value(render_parsed(&doc)).unwrap(),
        serde_json::to_value(render(input)).unwrap()
    );
    assert_eq!(doc.source(), "🙂 :rocket: @alice");
    let limited = OrihaEngine::default()
        .with_limits(oriha_core::EngineLimits {
            max_source_bytes: 2,
            ..Default::default()
        })
        .parse(RenderInput {
            markdown: "oversize".into(),
            ..Default::default()
        });
    assert!(limited.document().blocks.is_empty());
    assert_eq!(
        limited.diagnostics()[0].code,
        "engine.max_source_bytes_exceeded"
    );
    assert_eq!(
        render_parsed(&limited).diagnostics[0].code,
        "engine.max_source_bytes_exceeded"
    );
}

#[test]
fn source_mapping_invariants_hold_across_the_markdown_spec_fixtures() {
    for fixture in [
        include_str!("fixtures/commonmark_0_31_2_spec.json"),
        include_str!("fixtures/gfm_0_29_extension_spec.json"),
        include_str!("fixtures/callout_extension_spec.json"),
    ] {
        let examples: serde_json::Value = serde_json::from_str(fixture).unwrap();
        for example in examples.as_array().unwrap() {
            let source = example["markdown"].as_str().unwrap();
            let doc = parsed(source);
            for region in doc.text_regions() {
                let mut cursor = 0;
                for segment in &region.mappings {
                    let mapping = &segment.mapping;
                    assert_eq!(mapping.display.start, cursor, "{example:?}");
                    assert!(mapping.display.start < mapping.display.end);
                    assert!(region.text.is_char_boundary(mapping.display.start));
                    assert!(region.text.is_char_boundary(mapping.display.end));
                    for range in &mapping.source {
                        assert!(
                            source.is_char_boundary(range.start)
                                && source.is_char_boundary(range.end),
                            "{range:?} in {example:?}"
                        );
                        assert!(range.start < range.end && range.end <= source.len());
                    }
                    assert_ne!(
                        mapping.provenance,
                        TextProvenance::Generated,
                        "parser lost correspondence: {example:?}"
                    );
                    if mapping.exact {
                        assert_eq!(mapping.source.len(), 1);
                        assert_eq!(
                            &region.text[mapping.display.clone()],
                            &source[mapping.source[0].clone()],
                            "{example:?}"
                        );
                    }
                    cursor = mapping.display.end;
                }
                assert_eq!(cursor, region.text.len(), "{example:?}");
            }
        }
    }
}
