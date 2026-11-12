//! Consumer policy example; run with `cargo run -p oriha-core --example annotate_mentions`.
use std::collections::BTreeMap;

use oriha_core::{
    parse, render_parsed, AnnotationAttributes, AnnotationMetadata, EngineLimits, OrihaEngine,
    ParseStatus, ParsedDocument, RenderInput, TextAnnotation, TextProvenance,
};

// This entire function is consumer policy. oriha-core knows nothing about handles,
// reserved names, case folding, recipient membership, or delivery semantics.
fn candidates(
    document: &ParsedDocument,
    members: &BTreeMap<String, String>,
) -> Result<Vec<TextAnnotation>, ParseStatus> {
    if document.parse_status() != ParseStatus::Complete {
        return Err(document.parse_status());
    }
    let mut annotations = Vec::new();
    for region in document.text_regions() {
        for (start, ch) in region.text.char_indices() {
            if ch != '@' {
                continue;
            }
            let previous = region.text[..start].chars().next_back();
            if previous
                .is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '/' | '.' | ':' | '@'))
            {
                continue;
            }
            let Some(mapping) = region
                .mappings
                .iter()
                .find(|m| m.mapping.display.contains(&start))
            else {
                continue;
            };
            if !matches!(
                mapping.mapping.provenance,
                TextProvenance::Literal | TextProvenance::CharacterReference
            ) {
                continue;
            }
            let length = region.text[start + 1..]
                .bytes()
                .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                .count();
            if length == 0 {
                continue;
            }
            let end = start + 1 + length;
            let handle = region.text[start + 1..end].to_ascii_lowercase();
            if ["all", "here", "channel"].contains(&handle.as_str()) {
                continue;
            }
            let Some(recipient) = members.get(&handle) else {
                continue;
            };
            annotations.push(TextAnnotation {
                region: region.id,
                range: start..end,
                metadata: AnnotationMetadata {
                    kind: "mention".into(),
                    id: format!("occurrence-{}", annotations.len()),
                    attributes: AnnotationAttributes {
                        classes: vec!["mention".into()],
                        data: BTreeMap::from([("recipient".into(), recipient.clone())]),
                    },
                },
            });
        }
    }
    Ok(annotations)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = concat!(
        r"@alice \@alice \\@alice &#64;alice @ali**ce**",
        "\n\n",
        r#"[@ALICE](https://example.test/@hidden) user@alice https://example.test/@alice `@alice` @all @here @channel <span data-user="@alice">HTML</span>"#,
    );
    let parsed = parse(RenderInput {
        markdown: source.into(),
        ..Default::default()
    });
    let first = parsed.text_regions().next().unwrap();
    let provenance: Vec<_> = first
        .text
        .match_indices('@')
        .map(|(start, _)| {
            first
                .mappings
                .iter()
                .find(|m| m.mapping.display.contains(&start))
                .unwrap()
                .mapping
                .provenance
        })
        .collect();
    assert_eq!(
        provenance,
        [
            TextProvenance::Literal,
            TextProvenance::Escape,
            TextProvenance::Literal,
            TextProvenance::CharacterReference,
            TextProvenance::Literal
        ]
    );
    let members = BTreeMap::from([("alice".into(), "member-123".into())]);
    let occurrences = candidates(&parsed, &members).expect("complete parsing");
    assert_eq!(occurrences.len(), 5);
    let decorated = parsed.apply_annotations(&occurrences)?;
    let rendered = render_parsed(&decorated);
    // One formatted occurrence yields two fragments with one logical identity.
    assert_eq!(
        rendered
            .html
            .matches("data-oriha-annotation-id=\"occurrence-3\"")
            .count(),
        2
    );
    assert!(rendered.html.contains("<strong><span"));
    assert!(!render_parsed(&parsed)
        .html
        .contains("data-oriha-annotation-kind"));
    println!(
        "{} eligible occurrences; provenance: {provenance:?}",
        occurrences.len()
    );
    println!("{}", rendered.html);
    let empty = parse(RenderInput {
        markdown: "Hello world".into(),
        ..Default::default()
    });
    assert!(candidates(&empty, &members).unwrap().is_empty());
    let omitted = OrihaEngine::default()
        .with_limits(EngineLimits {
            max_nesting_depth: 0,
            ..EngineLimits::default()
        })
        .parse(RenderInput {
            markdown: "> @alice\n".into(),
            ..Default::default()
        });
    assert!(matches!(
        candidates(&omitted, &members),
        Err(ParseStatus::Incomplete)
    ));
    println!("Complete empty extraction is Ok([]); omitted text is Err(Incomplete).");
    // Delivery still uses the consumer's authenticated membership and recipient
    // resolution. HTML attributes, including source-authored attributes, are not
    // an authorization boundary. Keep the application's existing sanitization.
    Ok(())
}
