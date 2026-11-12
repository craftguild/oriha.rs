//! URL regressions at the actual rendered-block and semantic-document boundaries.
use oriha_core::{
    derive_blocks, render, BlockOutput, BlockOutputMode, BlockOutputStatus, RenderInput, RenderMode,
};

fn both_html(markdown: &str) -> [String; 2] {
    let input = RenderInput {
        markdown: markdown.into(),
        mode: RenderMode::SemanticHtml,
        ..Default::default()
    };
    let BlockOutput::Rendered(output) = derive_blocks(input.clone(), BlockOutputMode::Rendered)
    else {
        panic!("expected rendered output")
    };
    assert_eq!(output.status, BlockOutputStatus::Complete);
    [
        output.blocks.iter().filter_map(|b| b.html()).collect(),
        render(input).html,
    ]
}

#[test]
fn rendered_images_and_links_preserve_ipv6_authorities_and_encode_other_brackets() {
    for (source, expected) in [
        (
            "http://[::1]:4000/assets/pixel.png",
            "http://[::1]:4000/assets/pixel.png",
        ),
        ("https://[2001:db8::1]/a.png", "https://[2001:db8::1]/a.png"),
        (
            "HTTP://[2001:DB8::1]:8443/a%20b/[one].png?q=[x]&v=two%20words#frag[2]",
            "HTTP://[2001:DB8::1]:8443/a%20b/%5Bone%5D.png?q=%5Bx%5D&amp;v=two%20words#frag%5B2%5D",
        ),
        (
            "http://[::ffff:192.0.2.128]:8080/a.png",
            "http://[::ffff:192.0.2.128]:8080/a.png",
        ),
        ("//[::1]:4000/a.png", "//[::1]:4000/a.png"),
        (
            "https://user:[secret]@[::1]:8443/a[1].png",
            "https://user:%5Bsecret%5D@[::1]:8443/a%5B1%5D.png",
        ),
    ] {
        for html in both_html(&format!("![image]({source}) [link]({source})")) {
            assert!(html.contains(&format!("src=\"{expected}\"")), "{html}");
            assert!(html.contains(&format!("href=\"{expected}\"")), "{html}");
        }
    }
}

#[test]
fn url_encoding_and_attribute_escaping_remain_in_effect() {
    for (source, expected) in [
        (
            "/assets/a[b].png?q=[x]#f[1]",
            "/assets/a%5Bb%5D.png?q=%5Bx%5D#f%5B1%5D",
        ),
        (
            "https://example.com/a[b].png?x=1&y=2",
            "https://example.com/a%5Bb%5D.png?x=1&amp;y=2",
        ),
        (
            "relative/http://[::1]/file.png",
            "relative/http://%5B::1%5D/file.png",
        ),
        (
            "http://[not-an-ip]/file.png",
            "http://%5Bnot-an-ip%5D/file.png",
        ),
        (
            "http://[::1]extra/file.png",
            "http://%5B::1%5Dextra/file.png",
        ),
        ("http://[::1]:bad/file.png", "http://%5B::1%5D:bad/file.png"),
    ] {
        for html in both_html(&format!("![image]({source}) [link]({source})")) {
            assert!(html.contains(&format!("src=\"{expected}\"")), "{html}");
            assert!(html.contains(&format!("href=\"{expected}\"")), "{html}");
        }
    }
    let source = "![image](<http://[::1]:4000/a b\"c[1].png?x=1&y=2>)";
    for html in both_html(source) {
        assert!(
            html.contains("src=\"http://[::1]:4000/a%20b%22c%5B1%5D.png?x=1&amp;y=2\""),
            "{html}"
        );
    }
}

#[test]
fn unsafe_schemes_still_cannot_supply_image_or_link_destinations() {
    for source in [
        "javascript://[::1]/payload",
        "JaVaScRiPt:alert%281%29",
        "javascript&#58;alert%281%29",
        "vbscript://[::1]/payload",
        "data:text/html,[::1]",
    ] {
        for html in both_html(&format!("![image]({source}) [link]({source})")) {
            assert!(html.contains("src=\"#\""), "{source}: {html}");
            assert!(html.contains("href=\"#\""), "{source}: {html}");
        }
    }
}
