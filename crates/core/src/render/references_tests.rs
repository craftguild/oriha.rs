use super::*;
use crate::{BlockOutput, BlockOutputMode};

#[test]
fn anchors_match_builtin_html_without_rendering_for_structured_consumers() {
    let markdown = "# 日本語 :smile: `code` $x$\n\n# 日本語 :smile: `code` $x$\n\n# 123\n\n# !!!\n\n> ## Quoted\n> text[^a]\n\n- tight[^a]\n\n| a | b |\n| - | - |\n| [^b] | x |\n\n![alt](/image.png)\n\n```rust\nlet x = 1;\n```\n\n[^a]: first\n[^a]: last [^b]\n[^b]: second\n";
    let parsed = OrihaEngine::default().parse(RenderInput {
        markdown: markdown.into(),
        ..Default::default()
    });
    let anchors = builtin_html_anchors(&parsed);
    let BlockOutput::Rendered(rendered) =
        OrihaEngine::default().derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)
    else {
        panic!("rendered")
    };
    let mut html = rendered
        .blocks
        .iter()
        .filter_map(|b| b.html())
        .collect::<String>();
    while let Some(start) = html.find("<svg") {
        let end = start + html[start..].find("</svg>").expect("closed svg") + 6;
        html.replace_range(start..end, "");
    }
    let actual: Vec<_> = html
        .split(" id=\"")
        .skip(1)
        .map(|part| part.split('"').next().unwrap().to_string())
        .collect();
    assert_eq!(anchors, actual);
    assert!(anchors.iter().any(|id| id == "日本語-smile-code-x-2"));
    assert!(anchors.iter().any(|id| id == "section-123"));
}

#[test]
fn normalized_destinations_use_the_existing_html_policy() {
    for (url, expected) in [
        ("/日本語 a", "/%E6%97%A5%E6%9C%AC%E8%AA%9E%20a"),
        ("https://[::1]/x[y]", "https://[::1]/x%5By%5D"),
        ("#日本語", "#%E6%97%A5%E6%9C%AC%E8%AA%9E"),
        ("javascript:alert(1)", "#"),
        ("javas&#99;ript:bad", "#"),
        ("../relative", "../relative"),
        (
            "https://example.com/?a=1&b=2",
            "https://example.com/?a=1&b=2",
        ),
    ] {
        assert_eq!(normalize_link_destination(url), expected);
    }
}
