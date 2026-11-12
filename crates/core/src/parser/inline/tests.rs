use super::*;
use crate::types::FeatureFlags;
use std::collections::BTreeMap;

#[test]
fn extended_autolink_discovery_has_bounded_processed_bytes() {
    for n in [500, 1000, 2000, 4000, 5040] {
        for source in [
            "simple words\n".repeat(n),
            "simple words\n".repeat(n) + "@tail",
            "simple words\n".repeat(n) + "late:",
            "a".repeat(n * 13),
            "a@".repeat(n * 6),
            "{".repeat(n * 12) + "a@bad",
            "(mailto:".repeat(n) + "a@bad",
            "Std::Lang::Type ".repeat(n),
            "(made-up-scheme:".repeat(n) + "resource",
            "www.example.com/".to_string() + &")".repeat(n * 12),
        ] {
            let mut discovery = ExtendedAutolinks::new(&source);
            let mut i = 0;
            while i < source.len() {
                if is_extended_autolink_boundary(source[..i].chars().next_back()) {
                    if let Some((consumed, _)) = discovery.parse_extended_autolink(i) {
                        i += consumed;
                        continue;
                    }
                }
                i += source[i..].chars().next().unwrap().len_utf8();
            }
            assert!(
                discovery.work <= 96 * source.len(),
                "{} bytes: {} work",
                source.len(),
                discovery.work
            );
            println!(
                "bytes={},charged_discovery_bytes={}",
                source.len(),
                discovery.work
            );
        }
    }
}

#[test]
fn extended_autolinks_are_lexical_and_keep_source_correspondence() {
    for (source, labels) in [
        ("words without delimiters", vec![]),
        ("www. www...", vec![]),
        ("simple words\n@tail", vec![]),
        ("simple words\nlate:", vec![]),
        ("bad@@example.com @tail", vec![]),
        (
            "hello@example.com, (www.example.com/a(b)))",
            vec!["hello@example.com", "www.example.com/a(b)"],
        ),
        (
            "mailto:hello@example.com/ xmpp:hello@example.com/path",
            vec!["mailto:hello@example.com", "xmpp:hello@example.com/path"],
        ),
        // A later token's @ or : must not suppress a valid earlier link.
        (
            "https://example.com @tail late:",
            vec!["https://example.com"],
        ),
        ("hello@example.com late:", vec!["hello@example.com"]),
        (
            "https://example.com/@tail",
            vec!["https://example.com/@tail"],
        ),
        (
            "日本語\r\nwww.example.com/日本語 @tail",
            vec!["www.example.com/日本語"],
        ),
        (
            "hello@mail+xyz.example hello+xyz@mail.example",
            vec!["hello+xyz@mail.example"],
        ),
        (
            "(not-an-email (hello@example.com)",
            vec!["hello@example.com"],
        ),
    ] {
        let nodes = parse_default(source);
        let links: Vec<_> = nodes
            .iter()
            .filter_map(|node| match &node.kind {
                InlineKind::Link { children, .. } => Some(children),
                _ => None,
            })
            .collect();
        let actual: Vec<_> = links
            .iter()
            .map(|children| match &children[0].kind {
                InlineKind::Text(text) => text.as_str(),
                _ => panic!("text label"),
            })
            .collect();
        assert_eq!(actual, labels, "{source}");
        for children in links {
            let InlineKind::Text(text) = &children[0].kind else {
                unreachable!()
            };
            let mapping = &children[0].source_mapping;
            assert_eq!(mapping.len(), 1);
            assert!(mapping[0].exact);
            assert_eq!(&source[mapping[0].source[0].clone()], text);
        }
    }
}

fn parse_with_math(source: &str, math: bool) -> Vec<InlineNode> {
    let features = FeatureFlags {
        math,
        ..FeatureFlags::default()
    };
    parse_inlines(source, &BTreeMap::new(), &features)
}

fn parse_default(source: &str) -> Vec<InlineNode> {
    parse_inlines(source, &BTreeMap::new(), &FeatureFlags::default())
}

#[test]
fn parses_emoji_shortcodes_when_known() {
    let nodes = parse_default("Ship :+1: :thumbs_up: :thumbs_up_light_skin_tone: :rocket:");

    assert!(
        matches!(nodes.first().map(|node| &node.kind), Some(InlineKind::Text(text)) if text == "Ship ")
    );
    assert!(
        matches!(nodes.get(1).map(|node| &node.kind), Some(InlineKind::Emoji { shortcode, emoji }) if shortcode == "+1" && emoji == "👍")
    );
    assert!(
        matches!(nodes.get(2).map(|node| &node.kind), Some(InlineKind::Text(text)) if text == " ")
    );
    assert!(
        matches!(nodes.get(3).map(|node| &node.kind), Some(InlineKind::Emoji { shortcode, emoji }) if shortcode == "thumbs_up" && emoji == "👍")
    );
    assert!(
        matches!(nodes.get(4).map(|node| &node.kind), Some(InlineKind::Text(text)) if text == " ")
    );
    assert!(
        matches!(nodes.get(5).map(|node| &node.kind), Some(InlineKind::Emoji { shortcode, emoji }) if shortcode == "thumbs_up_light_skin_tone" && emoji == "👍🏻")
    );
    assert!(
        matches!(nodes.get(6).map(|node| &node.kind), Some(InlineKind::Text(text)) if text == " ")
    );
    assert!(
        matches!(nodes.get(7).map(|node| &node.kind), Some(InlineKind::Emoji { shortcode, emoji }) if shortcode == "rocket" && emoji == "🚀")
    );
}

#[test]
fn leaves_unknown_or_escaped_emoji_shortcodes_as_text() {
    let unknown = parse_default("Unknown :not_a_real_shortcode:");
    assert!(
        matches!(unknown.as_slice(), [InlineNode { kind: InlineKind::Text(text), .. }] if text == "Unknown :not_a_real_shortcode:")
    );

    let escaped = parse_default(r"\:+1:");
    assert!(
        matches!(escaped.as_slice(), [InlineNode { kind: InlineKind::Text(text), .. }] if text == ":+1:")
    );
}

#[test]
fn does_not_parse_emoji_shortcodes_inside_code_spans_or_html() {
    let nodes = parse_default("`:+1:` <span title=\":+1:\"></span> :+1:");

    assert!(
        matches!(nodes.first().map(|node| &node.kind), Some(InlineKind::Code(code)) if code == ":+1:")
    );
    assert!(
        matches!(nodes.get(1).map(|node| &node.kind), Some(InlineKind::Text(text)) if text == " ")
    );
    assert!(
        matches!(nodes.get(2).map(|node| &node.kind), Some(InlineKind::Html { raw, .. }) if raw == "<span title=\":+1:\">")
    );
    assert!(
        matches!(nodes.last().map(|node| &node.kind), Some(InlineKind::Emoji { shortcode, .. }) if shortcode == "+1")
    );
}

#[test]
fn parses_single_dollar_math_when_enabled() {
    let nodes = parse_with_math("before $E = mc^2$ after", true);
    assert!(
        matches!(nodes.first().map(|node| &node.kind), Some(InlineKind::Text(text)) if text == "before ")
    );
    assert!(
        matches!(nodes.get(1).map(|node| &node.kind), Some(InlineKind::Math { source }) if source == "E = mc^2")
    );
    assert!(
        matches!(nodes.get(2).map(|node| &node.kind), Some(InlineKind::Text(text)) if text == " after")
    );
}

#[test]
fn leaves_dollars_as_text_when_math_disabled_or_invalid() {
    let disabled = parse_with_math("$x$", false);
    assert!(
        matches!(disabled.as_slice(), [InlineNode { kind: InlineKind::Text(text), .. }] if text == "$x$")
    );

    let escaped = parse_with_math(r"\$x$", true);
    assert!(
        matches!(escaped.as_slice(), [InlineNode { kind: InlineKind::Text(text), .. }] if text == "$x$")
    );

    let empty = parse_with_math("$ $", true);
    assert!(
        matches!(empty.as_slice(), [InlineNode { kind: InlineKind::Text(text), .. }] if text == "$ $")
    );

    let double = parse_with_math("$$x$$", true);
    assert!(
        matches!(double.as_slice(), [InlineNode { kind: InlineKind::Text(text), .. }] if text == "$$x$$")
    );

    let unclosed = parse_with_math("$x", true);
    assert!(
        matches!(unclosed.as_slice(), [InlineNode { kind: InlineKind::Text(text), .. }] if text == "$x")
    );
}

#[test]
fn does_not_parse_math_inside_code_spans() {
    let nodes = parse_with_math("`$x$` and $y$", true);
    assert!(
        matches!(nodes.first().map(|node| &node.kind), Some(InlineKind::Code(code)) if code == "$x$")
    );
    assert!(
        matches!(nodes.get(1).map(|node| &node.kind), Some(InlineKind::Text(text)) if text == " and ")
    );
    assert!(
        matches!(nodes.get(2).map(|node| &node.kind), Some(InlineKind::Math { source }) if source == "y")
    );
}

#[test]
fn does_not_parse_emoji_shortcodes_inside_inline_math() {
    let nodes = parse_with_math("$:+1:$ and :+1:", true);

    assert!(
        matches!(nodes.first().map(|node| &node.kind), Some(InlineKind::Math { source }) if source == ":+1:")
    );
    assert!(
        matches!(nodes.get(1).map(|node| &node.kind), Some(InlineKind::Text(text)) if text == " and ")
    );
    assert!(
        matches!(nodes.get(2).map(|node| &node.kind), Some(InlineKind::Emoji { shortcode, .. }) if shortcode == "+1")
    );
}
