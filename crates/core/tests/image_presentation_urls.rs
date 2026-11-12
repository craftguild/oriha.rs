use oriha_core::render::RenderContext;
use oriha_core::*;

struct Prepared(ImagePresentation);
impl RenderHook for Prepared {
    fn image_presentation<'a>(
        &'a self,
        _: &ImageRenderRequest<'_>,
        _: &RenderContext<'_>,
    ) -> Option<&'a ImagePresentation> {
        Some(&self.0)
    }
}
fn input() -> RenderInput {
    RenderInput { markdown: "![alt & \" < >](/orig \"caption & \\\" < >\")\n\ntext ![alt & \" < >](/orig \"tooltip & \\\" < >\")\n\n<script>alert(1)</script>".into(), mode: RenderMode::SemanticHtml, ..Default::default() }
}
// Independent attribute decoding and width-candidate parsing: deliberately no
// production serializer/sanitizer helper is used to establish expected tokens.
fn attr(html: &str, name: &str) -> String {
    html.split_once(&format!(" {name}=\""))
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap()
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}
fn candidates(srcset: &str) -> Vec<(&str, u32)> {
    // Follow the HTML URL-token step: collect through ASCII whitespace, so
    // internal commas (including data: syntax) remain part of the URL. Only
    // the following width descriptor is terminated by a candidate comma.
    let mut rest = srcset;
    let mut result = Vec::new();
    loop {
        rest = rest.trim_start_matches(|c: char| c.is_ascii_whitespace() || c == ',');
        if rest.is_empty() {
            return result;
        }
        let end = rest
            .find(|c: char| c.is_ascii_whitespace())
            .expect("width separator");
        let url = &rest[..end];
        assert!(!url.ends_with(','), "unexpected trailing URL comma");
        rest = &rest[end..];
        let end = rest.find(',').unwrap_or(rest.len());
        let fields: Vec<_> = rest[..end].split_ascii_whitespace().collect();
        assert_eq!(fields.len(), 1, "invalid width descriptors");
        let width = fields[0].strip_suffix('w').unwrap().parse::<u32>().unwrap();
        assert!(width > 0);
        result.push((url, width));
        rest = &rest[end..];
    }
}
fn percent_decode(s: &str) -> String {
    let mut bytes = Vec::new();
    let mut it = s.bytes();
    while let Some(b) = it.next() {
        if b == b'%' {
            let h = (it.next().unwrap() as char).to_digit(16).unwrap();
            let l = (it.next().unwrap() as char).to_digit(16).unwrap();
            bytes.push((h * 16 + l) as u8);
        } else {
            bytes.push(b);
        }
    }
    String::from_utf8(bytes).unwrap()
}

#[test]
fn candidate_tokens_are_independently_parseable_and_escape_delimiters() {
    let urls = [
        "http://[::1]:4000/a,b c.png?x=1&y=\"z\"",
        "//[2001:db8::1]/日本語\t\n\r\u{c}.png",
        "/a%20b%2Cc/[x].png?x=[1]&q='a'",
        "/comma,,and\u{a0}unicode.png,",
    ];
    let widths = [960, 320, 640, 1];
    let sizes = "(max-width: 40em) 100vw, calc(100vw - 2em) & \" ' < >";
    let p = ImagePresentation::new(
        urls[0],
        urls.iter()
            .zip(widths)
            .map(|(u, w)| ImageCandidate::new(*u, w).unwrap())
            .collect(),
        Some(sizes.into()),
        Some((960, 640)),
        ImageLayout::Responsive,
    )
    .unwrap();
    let engine = OrihaEngine::default().add_hook(Prepared(p));
    let parsed = engine.parse(input());
    let BlockOutput::Rendered(b) = engine.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)
    else {
        unreachable!()
    };
    assert_eq!(b.status, BlockOutputStatus::Complete);
    let fragments: String = b.blocks.iter().filter_map(|b| b.html()).collect();
    for html in [
        engine.render(input()).html,
        engine.render_parsed(&parsed).html,
        fragments,
    ] {
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
        for image in html.split("<img ").skip(1) {
            let image = format!(" {image}");
            let srcset = attr(&image, "srcset");
            let parsed = candidates(&srcset);
            assert_eq!(parsed.iter().map(|(_, w)| *w).collect::<Vec<_>>(), widths);
            let decoded: Vec<_> = parsed.iter().map(|(u, _)| percent_decode(u)).collect();
            assert_eq!(
                decoded,
                [urls[0], urls[1], "/a b,c/[x].png?x=[1]&q='a'", urls[3]]
            );
            assert_eq!(parsed[0].0, "http://[::1]:4000/a%2Cb%20c.png?x=1&y=%22z%22");
            assert!(parsed[1].0.starts_with("//[2001:db8::1]/"));
            assert_eq!(parsed[2].0, "/a%20b%2Cc/%5Bx%5D.png?x=%5B1%5D&q='a'");
            assert!(parsed[3].0.ends_with(".png%2C"));
            assert_eq!(attr(&image, "sizes"), sizes);
            assert_eq!(attr(&image, "alt"), "alt & \" < >");
            assert_eq!(
                attr(&image, "src"),
                "http://[::1]:4000/a,b%20c.png?x=1&y=%22z%22"
            );
        }
        assert!(
            html.contains("title=\"tooltip &amp; &quot; &lt; &gt;\""),
            "{html}"
        );
        assert!(
            html.contains(">caption &amp; \" &lt; &gt;</figcaption>"),
            "{html}"
        );
    }
}

#[test]
fn unsafe_presentation_urls_use_the_existing_scheme_rejection() {
    for url in [
        "javascript:alert(1)",
        " JaVaScRiPt:alert(1)",
        "javascript&#58;alert(1)",
        "vbscript:run",
        "data:text/html,<script>",
    ] {
        let p = ImagePresentation::new(
            url,
            vec![ImageCandidate::new(url, 1).unwrap()],
            Some("100vw".into()),
            None,
            ImageLayout::Responsive,
        )
        .unwrap();
        let engine = OrihaEngine::default().add_hook(Prepared(p));
        let BlockOutput::Rendered(b) = engine.derive_blocks(input(), BlockOutputMode::Rendered)
        else {
            unreachable!()
        };
        assert_eq!(b.status, BlockOutputStatus::Complete);
        for html in [
            engine.render(input()).html,
            b.blocks.iter().filter_map(|b| b.html()).collect(),
        ] {
            assert_eq!(attr(&html, "src"), "#", "{html}");
            assert_eq!(candidates(&attr(&html, "srcset")), [("#", 1)]);
        }
    }
}

#[test]
fn data_image_candidates_preserve_syntax_in_all_real_api_paths() {
    const PNG: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
    const UPPER: &str = "DaTa:image/png;charset=utf-8;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
    const SVG: &str = "data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20width='1'%20height='1'%3E%3C!--a,b%2Cc--%3E%3C/svg%3E";
    const SVG_CANDIDATE: &str = "data:image/svg+xml,%3Csvg%20xmlns='http://www.w3.org/2000/svg'%20width='1'%20height='1'%3E%3C!--a%2Cb%2Cc--%3E%3C/svg%3E";
    let mut fixtures = Vec::new();
    for (name, source, urls, expected) in [
        ("png_source", PNG, vec![], vec![]),
        ("png_candidate", PNG, vec![(PNG, 1)], vec![(PNG, 1)]),
        (
            "png_mixed_first",
            PNG,
            vec![(PNG, 1), ("/unused,a b.png", 1000)],
            vec![(PNG, 1), ("/unused%2Ca%20b.png", 1000)],
        ),
        (
            "png_mixed_last",
            PNG,
            vec![("/unused,a b.png", 1000), (PNG, 1)],
            vec![("/unused%2Ca%20b.png", 1000), (PNG, 1)],
        ),
        (
            "uppercase_parameters",
            UPPER,
            vec![(UPPER, 1)],
            vec![(UPPER, 1)],
        ),
        (
            "svg_payload_commas",
            SVG,
            vec![(SVG, 1)],
            vec![(SVG_CANDIDATE, 1)],
        ),
    ] {
        let presentation = ImagePresentation::new(
            source,
            urls.iter()
                .map(|(url, w)| ImageCandidate::new(*url, *w).unwrap())
                .collect(),
            Some("1px".into()),
            Some((1, 1)),
            ImageLayout::Responsive,
        )
        .unwrap();
        let engine = OrihaEngine::default().add_hook(Prepared(presentation));
        let input = RenderInput {
            markdown: "![pixel](/original)\n\nText ![inline](/original)".into(),
            mode: RenderMode::SemanticHtml,
            ..Default::default()
        };
        let parsed = engine.parse(input.clone());
        let block_html = |o: BlockOutput| {
            let BlockOutput::Rendered(b) = o else {
                unreachable!()
            };
            assert_eq!(b.status, BlockOutputStatus::Complete);
            b.blocks.iter().filter_map(|b| b.html()).collect::<String>()
        };
        let document_html = |o: RenderOutput| {
            assert!(o.diagnostics.is_empty(), "{:?}", o.diagnostics);
            o.html
        };
        for (api, html) in [
            (
                "document_direct",
                document_html(engine.render(input.clone())),
            ),
            (
                "document_retained",
                document_html(engine.render_parsed(&parsed)),
            ),
            (
                "blocks_direct",
                block_html(engine.derive_blocks(input.clone(), BlockOutputMode::Rendered)),
            ),
            (
                "blocks_retained",
                block_html(engine.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)),
            ),
        ] {
            assert_eq!(html.matches("<img ").count(), 2);
            let expected_srcset = expected
                .iter()
                .map(|(url, w)| format!("{url} {w}w"))
                .collect::<Vec<_>>()
                .join(", ");
            for image in html.split("<img ").skip(1) {
                let image = format!(" {image}");
                assert_eq!(attr(&image, "src"), source);
                if urls.is_empty() {
                    assert!(!image.contains(" srcset=\""));
                } else {
                    let srcset = attr(&image, "srcset");
                    assert_eq!(candidates(&srcset), expected, "{name}/{api}");
                    // Compare the actual URL token as-is. Percent-decoding the
                    // complete URL would conceal a broken data: separator.
                    assert_eq!(srcset, expected_srcset);
                }
            }
            fixtures.push(serde_json::json!({
                "id":format!("{name}_{api}"), "html":html,
                "expectedSource":source,
                "expectedSelected": if name=="svg_payload_commas" { SVG_CANDIDATE } else { source },
                "expectedSrcset": if urls.is_empty() { None } else { Some(&expected_srcset) },
            }));
        }
    }
    // Optional export for the real-browser regression. Only test HTML/metadata
    // is written; the renderer itself performs no file access or image decoding.
    if let Some(path) = std::env::var_os("ORIHA_DATA_URL_BROWSER_FIXTURES") {
        std::fs::write(path, serde_json::to_vec(&fixtures).unwrap()).unwrap();
    }
}
