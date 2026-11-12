//! CommonMark 0.31.2 conformance harness for the public renderer.
//!
//! The normal product renderer emits `oriha-*` wrappers and optional extensions.
//! The harness runs it with a CommonMark feature profile, strips renderer-only
//! decoration, and then requires every official example to match.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use oriha_core::{render, FeatureFlags, RenderInput, RenderMode};
use serde::Deserialize;

#[derive(Deserialize)]
struct SpecExample {
    markdown: String,
    html: String,
    example: u32,
    section: String,
}

fn spec_examples() -> Vec<SpecExample> {
    serde_json::from_str(include_str!("fixtures/commonmark_0_31_2_spec.json"))
        .expect("CommonMark fixture should parse")
}

#[test]
fn commonmark_0_31_2_public_renderer_matches_spec_html() {
    let examples = spec_examples();
    let mut mismatches = Vec::<String>::new();
    let mut by_section = BTreeMap::<String, (usize, usize)>::new();

    for example in &examples {
        let output = render(commonmark_render_input(example.markdown.clone()));
        let entry = by_section.entry(example.section.clone()).or_default();
        entry.1 += 1;
        let actual = normalize_engine_html(&output.html);
        let expected = normalize_spec_html(&example.html);
        if actual == expected {
            entry.0 += 1;
        } else if mismatches.len() < 20 {
            mismatches.push(format!(
                "\n- example={}, section={}\n  markdown={:?}\n  expected={:?}\n  actual={:?}",
                example.example, example.section, example.markdown, expected, actual
            ));
        }
    }

    let passed = by_section.values().map(|(ok, _)| ok).sum::<usize>();
    if !mismatches.is_empty() {
        let mut report = format!(
            "CommonMark 0.31.2 public renderer conformance mismatch: {}/{} examples failed",
            examples.len() - passed,
            examples.len()
        );
        for (section, (ok, total)) in &by_section {
            let _ = writeln!(report, "\n  {section}: {ok}/{total}");
        }
        report.push_str("\nFirst mismatches:");
        report.push_str(&mismatches.join(""));
        panic!("{report}");
    }
}

#[test]
fn no_spec_example_panics_the_engine() {
    for example in spec_examples() {
        let output = render(commonmark_render_input(example.markdown));
        assert!(
            !output.diagnostics.iter().any(|d| d.code == "engine.panic"),
            "spec example {} panicked the engine",
            example.example
        );
    }
}

fn commonmark_render_input(markdown: String) -> RenderInput {
    RenderInput {
        markdown,
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            gfm: false,
            callouts: false,
            syntax_highlight: true,
            raw_html: true,
            math: false,
            mermaid: false,
            plantuml: false,
        },
        ..RenderInput::default()
    }
}

fn normalize_engine_html(html: &str) -> String {
    let mut html = html.to_string();
    html = strip_document_wrapper(html);
    html = remove_oriha_code_block_headers(html);
    html = unwrap_oriha_div_wrappers(html, "class=\"oriha-code-block-wrapper\"");
    html = unwrap_oriha_token_spans(html);
    html = remove_oriha_code_gutters(html);
    html = unwrap_oriha_code_line_spans(html);
    html = unwrap_oriha_div_wrappers(html, "class=\"oriha-table-scroll-wrapper\"");
    html = unwrap_oriha_html_block_wrappers(html);
    html = normalize_oriha_image_figures(html);
    html = strip_oriha_attributes(&html);
    normalize_html_for_compare(&html)
}

fn normalize_spec_html(html: &str) -> String {
    normalize_html_for_compare(html)
}

fn strip_document_wrapper(html: String) -> String {
    let trimmed = html.trim();
    if !trimmed.starts_with("<article ") || !trimmed.ends_with("</article>") {
        return html;
    }
    let Some(open_end) = trimmed.find('>') else {
        return html;
    };
    let body_end = trimmed.len().saturating_sub("</article>".len());
    trimmed[open_end + 1..body_end].to_string()
}

fn remove_oriha_code_block_headers(mut value: String) -> String {
    while let Some(marker_pos) = value.find("class=\"oriha-code-block-header\"") {
        let Some(start) = value[..marker_pos].rfind("<div ") else {
            break;
        };
        let Some(close_rel) = value[start..].find("</div>") else {
            break;
        };
        let close_end = start + close_rel + "</div>".len();
        value.replace_range(start..close_end, "");
    }
    value
}

fn remove_oriha_code_gutters(mut value: String) -> String {
    while let Some(marker_pos) = value.find("class=\"oriha-code-gutter\"") {
        let Some(start) = value[..marker_pos].rfind("<span ") else {
            break;
        };
        let Some(scroll_rel) = value[start..].find(r#"<span class="oriha-code-scroll""#) else {
            break;
        };
        value.replace_range(start..start + scroll_rel, "");
    }
    value
}

fn unwrap_oriha_code_line_spans(value: String) -> String {
    value
        .replace(
            r#"<span class="oriha-code-scroll"><span class="oriha-code-lines">"#,
            "",
        )
        .replace(r#"</span></span></code></pre>"#, "</code></pre>")
        .replace(
            r#"<span class="oriha-code-line"><span class="oriha-code-line-content">"#,
            "",
        )
        .replace("</span></span>", "\n")
}

fn unwrap_oriha_token_spans(mut value: String) -> String {
    while let Some(start) = value.find("<span class=\"token ") {
        let Some(open_end_rel) = value[start..].find("\">") else {
            break;
        };
        let content_start = start + open_end_rel + "\">".len();
        value.replace_range(start..content_start, "");

        let Some(close_rel) = value[start..].find("</span>") else {
            break;
        };
        let close_start = start + close_rel;
        value.replace_range(close_start..close_start + "</span>".len(), "");
    }
    value
}

fn unwrap_oriha_div_wrappers(mut value: String, marker: &str) -> String {
    while let Some(marker_pos) = value.find(marker) {
        let Some(start) = value[..marker_pos].rfind("<div ") else {
            break;
        };
        let Some(open_end_rel) = value[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_rel;
        let body_after_open = &value[open_end + 1..];
        let close_rel = if body_after_open.starts_with("</div>") {
            body_after_open.rfind("</div>")
        } else {
            body_after_open.find("</div>")
        };
        let Some(close_rel) = close_rel else {
            break;
        };
        let close_start = open_end + 1 + close_rel;
        let inner = value[open_end + 1..close_start].to_owned();
        let tail_start = close_start + "</div>".len();
        let mut next = String::with_capacity(value.len());
        next.push_str(&value[..start]);
        next.push_str(&inner);
        next.push_str(&value[tail_start..]);
        value = next;
    }
    value
}

fn unwrap_oriha_html_block_wrappers(mut value: String) -> String {
    let marker = "class=\"oriha-foreign-html-wrapper\"";
    while let Some(marker_pos) = value.find(marker) {
        let Some(start) = value[..marker_pos].rfind("<div ") else {
            break;
        };
        let Some(open_end_rel) = value[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_rel;
        let Some(close_start) = html_wrapper_close_start(&value, start, open_end) else {
            break;
        };
        let inner = value[open_end + 1..close_start].to_owned();
        let tail_start = close_start + "</div>".len();
        let tail = &value[tail_start..];

        let mut next = String::with_capacity(value.len());
        next.push_str(&value[..start]);
        next.push_str(&inner);
        if !inner.is_empty() && !inner.ends_with('\n') && !tail.is_empty() {
            next.push('\n');
        }
        next.push_str(tail);
        value = next;
    }
    value
}

fn html_wrapper_close_start(value: &str, div_start: usize, open_end: usize) -> Option<usize> {
    let body_start = open_end + 1;
    if starts_with_div_close_after_whitespace(value, body_start) {
        let first_close = next_div_close_start(value, body_start)?;
        return next_div_close_start(value, first_close + "</div>".len()).or(Some(first_close));
    }
    let next_wrapper = value[body_start..]
        .find("class=\"oriha-foreign-html-wrapper\"")
        .map(|position| body_start + position);
    let matched = matching_div_close_start(value, div_start)
        .filter(|close_start| next_wrapper.is_none_or(|next| *close_start < next));
    matched.or_else(|| next_div_close_start(value, body_start))
}

fn starts_with_div_close_after_whitespace(value: &str, body_start: usize) -> bool {
    let mut offset = body_start;
    while offset < value.len() {
        let ch = value[offset..].chars().next().expect("valid UTF-8");
        if !ch.is_whitespace() {
            return is_div_close_tag(value, offset);
        }
        offset += ch.len_utf8();
    }
    false
}

fn next_div_close_start(value: &str, from: usize) -> Option<usize> {
    let mut offset = from;
    while offset < value.len() {
        let tag_start = offset + value[offset..].find('<')?;
        if is_div_close_tag(value, tag_start) {
            return Some(tag_start);
        }
        offset = tag_start + 1;
    }
    None
}

fn matching_div_close_start(value: &str, div_start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut offset = div_start;
    while offset < value.len() {
        let tag_start = offset + value[offset..].find('<')?;
        if is_div_open_tag(value, tag_start) {
            depth += 1;
        } else if is_div_close_tag(value, tag_start) {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(tag_start);
            }
        }
        let tag_end = value[tag_start..].find('>')?;
        offset = tag_start + tag_end + 1;
    }
    None
}

fn is_div_open_tag(value: &str, tag_start: usize) -> bool {
    let rest = &value[tag_start..];
    rest.len() >= 4
        && rest[..4].eq_ignore_ascii_case("<div")
        && rest
            .as_bytes()
            .get(4)
            .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'>')
}

fn is_div_close_tag(value: &str, tag_start: usize) -> bool {
    let rest = &value[tag_start..];
    rest.len() >= 5
        && rest[..5].eq_ignore_ascii_case("</div")
        && rest
            .as_bytes()
            .get(5)
            .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b'>')
}

fn normalize_oriha_image_figures(mut value: String) -> String {
    while let Some(marker_pos) = value.find("class=\"oriha-image") {
        let Some(start) = value[..marker_pos].rfind("<figure ") else {
            break;
        };
        let Some(open_end_rel) = value[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_rel;
        let body_after_open = &value[open_end + 1..];
        let Some(close_rel) = body_after_open.find("</figure>") else {
            break;
        };
        let close_start = open_end + 1 + close_rel;
        let inner = &value[open_end + 1..close_start];
        let Some(img_start_rel) = inner.find("<img ") else {
            break;
        };
        let Some(img_end_rel) = inner[img_start_rel..].find("/>") else {
            break;
        };
        let img_end = img_start_rel + img_end_rel + "/>".len();
        let mut img = inner[img_start_rel..img_end].to_string();
        if let Some(caption) = oriha_image_caption(inner) {
            img = insert_img_title_attr(img, caption);
        }
        let tail_start = close_start + "</figure>".len();
        let mut next = String::with_capacity(value.len());
        next.push_str(&value[..start]);
        next.push_str("<p>");
        next.push_str(&img);
        next.push_str("</p>");
        next.push_str(&value[tail_start..]);
        value = next;
    }
    value
}

fn oriha_image_caption(inner: &str) -> Option<&str> {
    let marker = r#"<figcaption class="oriha-image__caption">"#;
    let start = inner.find(marker)? + marker.len();
    let end = inner[start..].find("</figcaption>")?;
    Some(&inner[start..start + end])
}

fn insert_img_title_attr(mut img: String, caption: &str) -> String {
    if img.contains(" title=") {
        return img;
    }
    let Some(insert_at) = img.rfind(" />") else {
        return img;
    };
    img.insert_str(insert_at, &format!(r#" title="{caption}""#));
    img
}

fn strip_oriha_attributes(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(value.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            let ch = value[i..].chars().next().expect("valid UTF-8");
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let Some(tag_end_rel) = value[i..].find('>') else {
            out.push_str(&value[i..]);
            break;
        };
        let tag_end = i + tag_end_rel;
        let tag = &value[i..=tag_end];
        out.push_str(&strip_tag_oriha_attributes(tag));
        i = tag_end + 1;
    }
    out
}

fn strip_tag_oriha_attributes(tag: &str) -> String {
    if !tag.starts_with('<')
        || tag.starts_with("</")
        || tag.starts_with("<!")
        || tag.starts_with("<?")
        || !tag.ends_with('>')
    {
        return tag.to_string();
    }

    let inner = &tag[1..tag.len() - 1];
    if !(inner.contains("data-oriha-") || inner.contains("oriha-")) {
        return tag.to_string();
    }
    let mut out = String::with_capacity(tag.len());
    let bytes = inner.as_bytes();
    let mut i = 0usize;
    out.push('<');

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let name_start = i;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if name_start == i {
        out.push('>');
        return out;
    }
    out.push_str(&inner[name_start..i]);

    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let attr_start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'=' {
            i += 1;
        }
        let name_end = i;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            } else {
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
            }
        }
        let attr_end = i;
        let name = &inner[attr_start..name_end];
        let attr = &inner[attr_start..attr_end];
        let tag_is_oriha = inner.contains("data-oriha-") || inner.contains("oriha-");
        if name == "id"
            || name.starts_with("data-oriha-")
            || (name == "class" && attr.contains("oriha-"))
            || (tag_is_oriha && name == "style")
        {
            continue;
        }
        out.push(' ');
        out.push_str(attr);
    }
    out.push('>');
    out
}

fn normalize_html_for_compare(value: &str) -> String {
    let mut normalized = value
        .trim()
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#x27;", "'");

    while let Some(start) = normalized.find(">\n<") {
        normalized.replace_range(start + 1..start + 2, "");
    }

    while let Some(start) = normalized.find(">\r\n<") {
        normalized.replace_range(start + 1..start + 3, "");
    }

    normalized
}
