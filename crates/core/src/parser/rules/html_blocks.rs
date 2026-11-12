//! CommonMark HTML block classification and attribute grammar.
use super::*;

impl BlockDetector for HtmlBlockRule {
    fn detect(&self, cursor: &SourceCursor<'_>, _context: &ParseContext) -> Option<ParseStart> {
        cursor
            .current()
            .is_some_and(|line| is_html_block_start(line.text))
            .then_some(ParseStart {
                kind: BlockStartKind::HtmlBlock,
            })
    }
}

impl BlockParser for HtmlBlockRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        _context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let mut lines = Vec::new();
        if let Some(first) = cursor.current() {
            if let Some(close_tag) = html_block_closing_tag(first.text) {
                while let Some(line) = cursor.current() {
                    let lower = line.text.to_ascii_lowercase();
                    lines.push(line);
                    let _ = cursor.advance();
                    if lower.contains(close_tag) {
                        break;
                    }
                }
            } else {
                while let Some(line) = cursor.current() {
                    if line.text.trim().is_empty() {
                        break;
                    }
                    lines.push(line);
                    let _ = cursor.advance();
                }
            }
        }
        let span = lines_to_span(&lines).unwrap_or_else(|| SourceSpan::new(0, 0));
        ParseResult {
            span,
            block: BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::HtmlBlock,
                span,
                attrs: BTreeMap::new(),
                source_mapping: Default::default(),
                diagnostics: Vec::new(),
                children: Vec::new(),
            },
        }
    }
}

pub(super) fn is_html_block_start(line: &str) -> bool {
    if leading_spaces(line) > 3 {
        return false;
    }
    let trimmed_original = line.trim_start();
    let trimmed = trimmed_original.to_ascii_lowercase();
    if trimmed.starts_with("<!--") || trimmed.starts_with("<?") || trimmed.starts_with("<![cdata[")
    {
        return true;
    }
    if trimmed_original.starts_with("<!") {
        return trimmed_original
            .chars()
            .nth(2)
            .is_some_and(|ch| ch.is_ascii_uppercase());
    }
    if trimmed.starts_with("<script")
        || trimmed.starts_with("<style")
        || trimmed.starts_with("<pre")
        || trimmed.starts_with("<textarea")
    {
        return true;
    }
    let Some(name) = html_tag_name(&trimmed) else {
        return false;
    };
    is_commonmark_html_block_tag(name) || is_complete_html_open_or_closing_tag(&trimmed)
}

pub(super) fn html_block_closing_tag(line: &str) -> Option<&'static str> {
    let lower = line.trim_start().to_ascii_lowercase();
    if lower.starts_with("<script") {
        Some("</script>")
    } else if lower.starts_with("<pre") {
        Some("</pre>")
    } else if lower.starts_with("<style") {
        Some("</style>")
    } else if lower.starts_with("<textarea") {
        Some("</textarea>")
    } else if lower.starts_with("<!--") {
        Some("-->")
    } else if lower.starts_with("<?") {
        Some("?>")
    } else if lower.starts_with("<![cdata[") {
        Some("]]>")
    } else {
        None
    }
}

pub(super) fn is_complete_html_open_or_closing_tag(line: &str) -> bool {
    if !line.starts_with('<') || line.starts_with("<!--") {
        return false;
    }
    let Some(end) = line.find('>') else {
        return false;
    };
    let tag = &line[..=end];
    let Some(name) = html_tag_name(tag) else {
        return false;
    };
    let prefix_len = if tag.starts_with("</") { 2 } else { 1 };
    let after_name = &tag[prefix_len + name.len()..tag.len() - 1];
    if tag.starts_with("</") {
        return after_name.trim().is_empty() && line[end + 1..].trim().is_empty();
    }
    if !after_name.is_empty()
        && !after_name.starts_with('>')
        && !after_name.starts_with('/')
        && !after_name.starts_with(char::is_whitespace)
    {
        return false;
    }
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        && parse_html_block_tag_attributes(after_name)
        && line[end + 1..].trim().is_empty()
}

pub(super) fn parse_html_block_tag_attributes(mut rest: &str) -> bool {
    let mut first_attr = true;
    loop {
        let before_ws = rest.len();
        rest = rest.trim_start();
        let consumed_ws = before_ws != rest.len();
        if rest.is_empty() {
            return true;
        }
        if rest == "/" {
            return true;
        }
        if let Some(after_slash) = rest.strip_prefix('/') {
            return after_slash.trim().is_empty();
        }
        if !first_attr && !consumed_ws {
            return false;
        }
        let Some((name_len, _)) = html_block_attribute_name(rest) else {
            return false;
        };
        first_attr = false;
        rest = &rest[name_len..];
        let before_ws = rest.len();
        rest = rest.trim_start();
        if let Some(after_equals) = rest.strip_prefix('=') {
            rest = after_equals.trim_start();
            let Some((value_len, _)) = html_block_attribute_value(rest) else {
                return false;
            };
            rest = &rest[value_len..];
        } else if before_ws == rest.len() && !rest.is_empty() && rest != "/" {
            return false;
        }
    }
}

pub(super) fn html_block_attribute_name(value: &str) -> Option<(usize, &str)> {
    let mut chars = value.char_indices();
    let (_, first) = chars.next()?;
    if !(first.is_ascii_alphabetic() || first == '_' || first == ':') {
        return None;
    }
    let mut end = first.len_utf8();
    for (idx, ch) in chars {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | ':' | '.' | '-') {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    Some((end, &value[..end]))
}

pub(super) fn html_block_attribute_value(value: &str) -> Option<(usize, &str)> {
    let first = value.chars().next()?;
    if first == '"' || first == '\'' {
        let body = &value[first.len_utf8()..];
        let end = body.find(first)?;
        let consumed = first.len_utf8() + end + first.len_utf8();
        return Some((consumed, &value[..consumed]));
    }
    let end = value
        .char_indices()
        .find(|(_, ch)| ch.is_ascii_whitespace() || *ch == '/')
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    if end == 0 || value[..end].chars().any(|ch| "\"'=<>`".contains(ch)) {
        return None;
    }
    Some((end, &value[..end]))
}

pub(super) fn html_tag_name(line: &str) -> Option<&str> {
    let line = line.strip_prefix("</").or_else(|| line.strip_prefix('<'))?;
    let end = line
        .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-'))
        .unwrap_or(line.len());
    (end > 0).then_some(&line[..end])
}

pub(super) fn is_commonmark_html_block_tag(name: &str) -> bool {
    matches!(
        name,
        "address"
            | "article"
            | "aside"
            | "base"
            | "basefont"
            | "blockquote"
            | "body"
            | "caption"
            | "center"
            | "col"
            | "colgroup"
            | "dd"
            | "details"
            | "dialog"
            | "dir"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "frame"
            | "frameset"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "head"
            | "header"
            | "hr"
            | "html"
            | "iframe"
            | "legend"
            | "li"
            | "link"
            | "main"
            | "menu"
            | "menuitem"
            | "nav"
            | "noframes"
            | "ol"
            | "optgroup"
            | "option"
            | "p"
            | "param"
            | "search"
            | "section"
            | "summary"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "title"
            | "tr"
            | "track"
            | "ul"
    )
}
