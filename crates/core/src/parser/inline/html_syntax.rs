use super::*;

pub(super) fn parse_autolink_or_inline_html(input: &str, gfm: bool) -> Option<(usize, InlineKind)> {
    if let Some(consumed) = parse_inline_html_special(input) {
        let raw = input[..consumed].to_string();
        return Some((
            consumed,
            InlineKind::Html {
                tagfilter: gfm && html_is_tagfilter_disallowed(&raw),
                raw,
            },
        ));
    }
    if let Some(end) = input.find('>') {
        let inner = &input[1..end];
        if let Some(colon) = inner.find(':') {
            let scheme = &inner[..colon];
            if is_autolink_scheme(scheme) && !inner.chars().any(char::is_whitespace) {
                return Some((
                    end + 1,
                    InlineKind::Link {
                        destination: inner.to_string(),
                        title: None,
                        children: vec![InlineKind::Text(inner.to_string()).into()],
                    },
                ));
            }
        }
        if is_autolink_email(inner) {
            return Some((
                end + 1,
                InlineKind::Link {
                    destination: format!("mailto:{inner}"),
                    title: None,
                    children: vec![InlineKind::Text(inner.to_string()).into()],
                },
            ));
        }
    }
    let end = find_inline_html_tag_close(input)?;
    let inner = &input[1..end];
    if inner.is_empty() {
        return None;
    }
    if is_inline_html_tag(inner) {
        let raw = input[..=end].to_string();
        return Some((
            end + 1,
            InlineKind::Html {
                tagfilter: gfm && html_is_tagfilter_disallowed(&raw),
                raw,
            },
        ));
    }
    None
}

pub(super) fn parse_inline_html_special(input: &str) -> Option<usize> {
    if input.starts_with("<!-->") {
        return Some("<!-->".len());
    }
    if input.starts_with("<!--->") {
        return Some("<!--->".len());
    }
    if input.starts_with("<!--") {
        return input.find("-->").map(|idx| idx + "-->".len());
    }
    if input.starts_with("<?") {
        return input.find("?>").map(|idx| idx + "?>".len());
    }
    if input.starts_with("<![CDATA[") {
        return input.find("]]>").map(|idx| idx + "]]>".len());
    }
    if input.len() >= 3 {
        let marker = input.as_bytes()[2];
        if input.starts_with("<!") && marker.is_ascii_alphabetic() {
            return input.find('>').map(|idx| idx + 1);
        }
    }
    None
}

pub(super) fn find_inline_html_tag_close(input: &str) -> Option<usize> {
    if !input.starts_with('<') {
        return None;
    }
    let mut quote = None;
    for (idx, ch) in input.char_indices().skip(1) {
        if let Some(current) = quote {
            if ch == current {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }
        if ch == '>' {
            return Some(idx);
        }
    }
    None
}

// CommonMark's angle-delimited syntax allows unregistered schemes. Bare GFM
// links have a narrower protocol policy in parse_extended_autolink below.
pub(super) fn is_autolink_scheme(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '+' || ch == '.' || ch == '-')
        && (2..=32).contains(&value.len())
}

pub(super) fn is_autolink_email(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && local
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(ch))
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.starts_with(|ch: char| ch.is_ascii_alphanumeric())
                && label.ends_with(|ch: char| ch.is_ascii_alphanumeric())
                && label
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
        })
}

pub(super) fn html_is_tagfilter_disallowed(raw: &str) -> bool {
    let inner = raw
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
        .unwrap_or(raw)
        .trim_start();
    let inner = inner.strip_prefix('/').unwrap_or(inner).trim_start();
    let name_len = html_tag_name_len(inner);
    if name_len == 0 {
        return false;
    }
    matches!(
        inner[..name_len].to_ascii_lowercase().as_str(),
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

pub(super) fn is_inline_html_tag(value: &str) -> bool {
    if let Some(rest) = value.strip_prefix('/') {
        let name_len = html_tag_name_len(rest);
        return name_len > 0 && rest[name_len..].trim().is_empty();
    }
    let name_len = html_tag_name_len(value);
    if name_len == 0 {
        return false;
    }
    parse_html_tag_attributes(&value[name_len..])
}

pub(super) fn html_tag_name_len(value: &str) -> usize {
    let mut chars = value.char_indices();
    let Some((_, first)) = chars.next() else {
        return 0;
    };
    if !first.is_ascii_alphabetic() {
        return 0;
    }
    let mut end = first.len_utf8();
    for (idx, ch) in chars {
        if ch.is_ascii_alphanumeric() || ch == '-' {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    end
}

pub(super) fn parse_html_tag_attributes(mut rest: &str) -> bool {
    if rest.is_empty() {
        return true;
    }
    loop {
        let before_ws = rest.len();
        rest = rest.trim_start();
        let consumed_ws = before_ws != rest.len();
        if rest.is_empty() {
            return true;
        }
        if let Some(after_slash) = rest.strip_prefix('/') {
            return after_slash.is_empty();
        }
        if !consumed_ws {
            return false;
        }
        let Some((name_len, _)) = html_attribute_name(rest) else {
            return false;
        };
        rest = &rest[name_len..];
        let trimmed = rest.trim_start();
        if let Some(after_equals) = trimmed.strip_prefix('=') {
            rest = after_equals.trim_start();
            let Some((value_len, _)) = html_attribute_value(rest) else {
                return false;
            };
            rest = &rest[value_len..];
        } else {
            continue;
        }
    }
}

pub(super) fn html_attribute_name(value: &str) -> Option<(usize, &str)> {
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

pub(super) fn html_attribute_value(value: &str) -> Option<(usize, &str)> {
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
