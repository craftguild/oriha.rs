use super::*;

#[cfg(test)]
pub(super) fn normalize_spec_output_html(value: &str) -> String {
    let mut out = value.to_string();
    out = unwrap_outer_article(out);
    out = remove_oriha_code_block_headers(out);
    out = unwrap_oriha_div_wrappers(out, "class=\"oriha-code-block-wrapper\"");
    out = unwrap_oriha_token_spans(out);
    out = remove_oriha_code_gutters(out);
    out = unwrap_oriha_code_line_spans(out);
    out = unwrap_oriha_div_wrappers(out, "class=\"oriha-table-scroll-wrapper\"");
    out = unwrap_oriha_div_wrappers(out, "class=\"oriha-foreign-html-wrapper\"");
    out = normalize_oriha_image_figures(out);
    out = strip_oriha_attributes(&out);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push_html_char('\n');
    }
    out
}

#[cfg(test)]
pub(super) fn unwrap_outer_article(value: String) -> String {
    let Some(open_end) = value.find('>') else {
        return value;
    };
    if value.starts_with("<article ") && value.ends_with("</article>") {
        let body_end = value.len().saturating_sub("</article>".len());
        return value[open_end + 1..body_end].to_string();
    }
    value
}

#[cfg(test)]
pub(super) fn remove_oriha_code_block_headers(mut value: String) -> String {
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

#[cfg(test)]
pub(super) fn remove_oriha_code_gutters(mut value: String) -> String {
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

#[cfg(test)]
pub(super) fn unwrap_oriha_code_line_spans(value: String) -> String {
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

#[cfg(test)]
pub(super) fn unwrap_oriha_token_spans(mut value: String) -> String {
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

#[cfg(test)]
pub(super) fn unwrap_oriha_div_wrappers(mut value: String, marker: &str) -> String {
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

#[cfg(test)]
pub(super) fn normalize_oriha_image_figures(mut value: String) -> String {
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

#[cfg(test)]
pub(super) fn oriha_image_caption(inner: &str) -> Option<&str> {
    let marker = r#"<figcaption class="oriha-image__caption">"#;
    let start = inner.find(marker)? + marker.len();
    let end = inner[start..].find("</figcaption>")?;
    Some(&inner[start..start + end])
}

#[cfg(test)]
pub(super) fn insert_img_title_attr(mut img: String, caption: &str) -> String {
    if img.contains(" title=") {
        return img;
    }
    let Some(insert_at) = img.rfind(" />") else {
        return img;
    };
    img.insert_str(insert_at, &format!(r#" title="{caption}""#));
    img
}

#[cfg(test)]
pub(super) fn strip_oriha_attributes(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = String::with_capacity(value.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            let ch = value[i..].chars().next().expect("valid UTF-8");
            out.push_html_char(ch);
            i += ch.len_utf8();
            continue;
        }
        let Some(tag_end_rel) = value[i..].find('>') else {
            out.push_html(&value[i..]);
            break;
        };
        let tag_end = i + tag_end_rel;
        let tag = &value[i..=tag_end];
        out.push_html(&strip_tag_oriha_attributes(tag));
        i = tag_end + 1;
    }
    out
}

#[cfg(test)]
pub(super) fn strip_tag_oriha_attributes(tag: &str) -> String {
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
    out.push_html_char('<');

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let name_start = i;
    while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if name_start == i {
        out.push_html_char('>');
        return out;
    }
    out.push_html(&inner[name_start..i]);

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
        out.push_html_char(' ');
        out.push_html(attr);
    }
    out.push_html_char('>');
    out
}

#[cfg(test)]
pub(crate) struct SpecHtmlRenderer;

#[cfg(test)]
impl SpecHtmlRenderer {
    pub(crate) fn render(markdown: &str) -> String {
        let features = FeatureFlags {
            gfm: false,
            callouts: false,
            syntax_highlight: true,
            raw_html: true,
            math: false,
            mermaid: false,
            plantuml: false,
        };
        let output = OrihaEngine::default()
            .with_features(features.clone())
            .with_unwrapped_output()
            .register_renderer(SpecHtmlBlockRenderer)
            .render(RenderInput {
                markdown: markdown.to_string(),
                mode: RenderMode::SemanticHtml,
                features,
                ..RenderInput::default()
            });
        normalize_spec_output_html(&output.html)
    }

    pub(crate) fn render_gfm(markdown: &str) -> String {
        let features = FeatureFlags {
            gfm: true,
            callouts: false,
            syntax_highlight: true,
            raw_html: true,
            math: false,
            mermaid: false,
            plantuml: false,
        };
        let output = OrihaEngine::default()
            .with_features(features.clone())
            .with_unwrapped_output()
            .register_renderer(SpecHtmlBlockRenderer)
            .render(RenderInput {
                markdown: markdown.to_string(),
                mode: RenderMode::SemanticHtml,
                features,
                ..RenderInput::default()
            });
        normalize_spec_output_html(&output.html)
    }
}

#[cfg(test)]
pub(super) struct SpecHtmlBlockRenderer;

#[cfg(test)]
impl BlockRenderer for SpecHtmlBlockRenderer {
    fn kind(&self) -> &'static str {
        "htmlBlock"
    }

    fn render(
        &self,
        block: &mut BlockNode,
        source: &SourceDocument,
        state: &mut RenderState,
        context: &RenderContext,
        _renderers: &[Box<dyn BlockRenderer>],
        out: &mut String,
    ) {
        let html = source.source_span(block.span);
        if context.features.raw_html {
            if context.features.gfm {
                out.push_html(&gfm_filter_disallowed_raw_html(html));
            } else {
                out.push_html(html);
            }
        } else {
            out.push_html(&escape_text(html));
        }
        if !out.ends_with('\n') {
            out.push_html_char('\n');
        }
        state
            .source_map
            .push(mapped_block(block, source, "htmlBlock", BTreeMap::new()));
    }
}
