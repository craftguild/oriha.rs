use super::html_scan::parse_tag_at;

pub(crate) fn minify_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    let mut preserve_stack = Vec::<String>::new();

    while let Some(start_rel) = html[cursor..].find('<') {
        let start = cursor + start_rel;
        push_minified_text(&mut out, &html[cursor..start], preserve_stack.last());
        let Some((end, tag)) = parse_tag_at(html, start) else {
            out.push_str(&html[start..]);
            return out;
        };
        if let Some(tag) = tag {
            out.push_str(tag.source.trim());
            if is_preserve_tag(&tag.name) {
                if tag.closing {
                    if preserve_stack.last() == Some(&tag.name) {
                        preserve_stack.pop();
                    }
                } else if !tag.self_closing {
                    preserve_stack.push(tag.name);
                }
            }
        } else {
            out.push_str(html[start..end].trim());
        }
        cursor = end;
    }
    push_minified_text(&mut out, &html[cursor..], preserve_stack.last());
    out
}

fn push_minified_text(out: &mut String, text: &str, preserve_tag: Option<&String>) {
    if text.is_empty() {
        return;
    }
    if preserve_tag
        .is_some_and(|tag| tag == "script" || tag == "pre" || tag == "code" || tag == "svg")
    {
        out.push_str(text);
        return;
    }
    if preserve_tag.is_some_and(|tag| tag == "style") {
        out.push_str(&minify_css(text));
        return;
    }
    if text.chars().all(char::is_whitespace) {
        if !out.is_empty() && !out.ends_with(char::is_whitespace) {
            out.push(' ');
        }
        return;
    }
    let mut previous_was_space = out.ends_with(char::is_whitespace);
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !previous_was_space {
                out.push(' ');
                previous_was_space = true;
            }
        } else {
            out.push(ch);
            previous_was_space = false;
        }
    }
}

fn is_preserve_tag(name: &str) -> bool {
    matches!(name, "pre" | "code" | "script" | "style" | "svg")
}

pub(crate) fn minify_css(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let bytes = css.as_bytes();
    let mut cursor = 0usize;
    let mut quote = None::<u8>;
    let mut previous_space = false;

    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if let Some(quoted) = quote {
            out.push(byte as char);
            if byte == quoted && bytes.get(cursor.wrapping_sub(1)) != Some(&b'\\') {
                quote = None;
            }
            cursor += 1;
            continue;
        }
        if byte == b'"' || byte == b'\'' {
            out.push(byte as char);
            quote = Some(byte);
            previous_space = false;
            cursor += 1;
            continue;
        }
        if byte == b'/' && bytes.get(cursor + 1) == Some(&b'*') {
            cursor += 2;
            while cursor + 1 < bytes.len() && !(bytes[cursor] == b'*' && bytes[cursor + 1] == b'/')
            {
                cursor += 1;
            }
            cursor = (cursor + 2).min(bytes.len());
            continue;
        }
        if byte.is_ascii_whitespace() {
            previous_space = true;
            cursor += 1;
            continue;
        }
        let ch = byte as char;
        if "{}:;,>+~()[]=".contains(ch) {
            trim_ascii_space(&mut out);
            out.push(ch);
            previous_space = false;
        } else {
            if previous_space && needs_css_space(out.as_bytes().last().copied(), byte) {
                out.push(' ');
            }
            out.push(ch);
            previous_space = false;
        }
        cursor += 1;
    }
    out.trim().to_string()
}

fn needs_css_space(previous: Option<u8>, next: u8) -> bool {
    previous.is_some_and(|byte| {
        (byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
            && (next.is_ascii_alphanumeric() || next == b'-' || next == b'_')
    })
}

fn trim_ascii_space(value: &mut String) {
    while value.as_bytes().last().is_some_and(u8::is_ascii_whitespace) {
        value.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minifier_preserves_pre_code_text() {
        let html = "<pre><code>  a\n    b\n</code></pre><p> a   b </p>";
        let minified = minify_html(html);
        assert!(minified.contains("<pre><code>  a\n    b\n</code></pre>"));
        assert!(minified.contains("<p> a b </p>"));
    }

    #[test]
    fn minifier_preserves_inline_separator_whitespace() {
        let html = "<p><strong>a</strong> <em>b</em></p>";
        assert_eq!(minify_html(html), html);
    }
}
