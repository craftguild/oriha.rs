use super::*;

pub(super) fn render_inline_plain_nodes(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match &node.kind {
            InlineKind::Text(text) | InlineKind::Code(text) => out.push_html(text),
            InlineKind::Emoji { emoji, .. } => out.push_html(emoji),
            InlineKind::Math { source } => out.push_html(source),
            InlineKind::Html { .. } => {}
            InlineKind::SoftBreak | InlineKind::HardBreak => out.push_html_char('\n'),
            InlineKind::Annotation { children, .. }
            | InlineKind::Emphasis { children }
            | InlineKind::Strong { children }
            | InlineKind::Strikethrough { children } => {
                out.push_html(&render_inline_plain_nodes(children));
            }
            InlineKind::Link { children, .. } => {
                out.push_html(&render_inline_plain_nodes(children))
            }
            InlineKind::Image { alt, .. } => out.push_html(&render_inline_plain_nodes(alt)),
            InlineKind::FootnoteReference { label } => out.push_html(label),
        }
    }
    out
}

pub(super) fn render_inline_heading_slug_text(nodes: &[InlineNode]) -> String {
    let mut out = String::new();
    for node in nodes {
        match &node.kind {
            InlineKind::Text(text) | InlineKind::Code(text) => out.push_html(text),
            InlineKind::Emoji { shortcode, .. } => {
                out.push_html_char(':');
                out.push_html(shortcode);
                out.push_html_char(':');
            }
            InlineKind::Math { source } => out.push_html(source),
            InlineKind::Html { .. } => {}
            InlineKind::SoftBreak | InlineKind::HardBreak => out.push_html_char('\n'),
            InlineKind::Annotation { children, .. }
            | InlineKind::Emphasis { children }
            | InlineKind::Strong { children }
            | InlineKind::Strikethrough { children } => {
                out.push_html(&render_inline_heading_slug_text(children));
            }
            InlineKind::Link { children, .. } => {
                out.push_html(&render_inline_heading_slug_text(children));
            }
            InlineKind::Image { alt, .. } => out.push_html(&render_inline_heading_slug_text(alt)),
            InlineKind::FootnoteReference { label } => out.push_html(label),
        }
    }
    out
}

pub(super) fn gfm_filter_disallowed_raw_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut index = 0usize;
    while index < value.len() {
        if value.as_bytes()[index] == b'<' && raw_html_starts_with_disallowed_tag(&value[index..]) {
            out.push_html("&lt;");
            index += 1;
            continue;
        }
        let ch = value[index..].chars().next().expect("valid UTF-8");
        out.push_html_char(ch);
        index += ch.len_utf8();
    }
    out
}

pub(super) fn raw_html_starts_with_disallowed_tag(value: &str) -> bool {
    let Some(rest) = value.strip_prefix('<') else {
        return false;
    };
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('/').unwrap_or(rest).trim_start();
    let name = rest
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .collect::<String>();
    matches!(
        name.to_ascii_lowercase().as_str(),
        "title"
            | "textarea"
            | "style"
            | "xmp"
            | "iframe"
            | "noembed"
            | "noframes"
            | "script"
            | "plaintext"
    )
}

pub(super) fn sanitize_url(url: &str) -> String {
    let decoded = decode_html_entities(url);
    let normalized = decoded.trim().to_ascii_lowercase();
    if normalized.starts_with("javascript:")
        || normalized.starts_with("vbscript:")
        || normalized.starts_with("data:text/html")
    {
        "#".to_string()
    } else {
        // Parsed destinations are already decoded; angle/bare autolinks stay raw.
        percent_encode_url(url)
    }
}

pub(super) fn percent_encode_url(url: &str) -> String {
    let ipv6_brackets = ipv6_authority_brackets(url);
    let mut out = String::with_capacity(url.len());
    for (index, byte) in url.as_bytes().iter().enumerate() {
        match *byte {
            b'[' | b']'
                if ipv6_brackets.is_some_and(|(open, close)| index == open || index == close) =>
            {
                out.push_html_char(*byte as char)
            }
            b' ' => out.push_html("%20"),
            b'"' => out.push_html("%22"),
            b'[' => out.push_html("%5B"),
            b']' => out.push_html("%5D"),
            b'\\' => out.push_html("%5C"),
            b'`' => out.push_html("%60"),
            0x00..=0x1f | 0x7f..=0xff => {
                out.push_html_char('%');
                out.push_html(&format!("{byte:02X}"));
            }
            _ => out.push_html_char(*byte as char),
        }
    }
    out
}

/// Preserve only the delimiters of a validated IPv6 host, never brackets in
/// userinfo, a relative path, query or fragment. Encoding is not URL validation.
fn ipv6_authority_brackets(url: &str) -> Option<(usize, usize)> {
    let authority_start = if url.starts_with("//") {
        2
    } else {
        let scheme_end = url.find("://")?;
        let mut scheme = url[..scheme_end].bytes();
        if !scheme.next()?.is_ascii_alphabetic()
            || !scheme.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.'))
        {
            return None;
        }
        scheme_end + 3
    };
    let end = url[authority_start..]
        .find(['/', '?', '#'])
        .map_or(url.len(), |offset| authority_start + offset);
    let authority = &url[authority_start..end];
    let host_start = authority.rfind('@').map_or(0, |at| at + 1);
    let host = authority[host_start..].strip_prefix('[')?;
    let host_end = host.find(']')?;
    host[..host_end].parse::<std::net::Ipv6Addr>().ok()?;
    let suffix = &host[host_end + 1..];
    if !suffix.is_empty()
        && !suffix
            .strip_prefix(':')?
            .bytes()
            .all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let open = authority_start + host_start;
    Some((open, open + host_end + 1))
}
