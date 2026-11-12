use super::*;

pub(super) fn parse_footnote_reference(input: &str) -> Option<(usize, String)> {
    let rest = input.strip_prefix("[^")?;
    let close = rest.find(']')?;
    let label = normalize_footnote_reference_label(&rest[..close])?;
    Some((close + 3, label))
}

pub(super) fn parse_emoji_shortcode(input: &str) -> Option<(usize, String, String)> {
    let bytes = input.as_bytes();
    if bytes.first() != Some(&b':') {
        return None;
    }
    let mut end = 1usize;
    while end < bytes.len() && is_emoji_shortcode_byte(bytes[end]) {
        end += 1;
    }
    if end == 1 || bytes.get(end) != Some(&b':') {
        return None;
    }
    let shortcode = &input[1..end];
    let emoji = resolve_shortcode(shortcode)?;
    Some((end + 1, shortcode.to_string(), emoji.to_string()))
}

pub(super) fn is_emoji_shortcode_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'+')
}

pub(super) fn normalize_footnote_reference_label(label: &str) -> Option<String> {
    let normalized = label.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_ascii_lowercase())
    }
}

pub(super) fn parse_code_span(input: &str) -> Option<(usize, String)> {
    let marker_len = input.bytes().take_while(|b| *b == b'`').count();
    if marker_len == 0 {
        return None;
    }
    let rest = &input[marker_len..];
    let close = find_code_span_close(rest, marker_len)?;
    let raw = &rest[..close];
    let mut normalized = raw
        .chars()
        .map(|ch| if ch == '\n' || ch == '\r' { ' ' } else { ch })
        .collect::<String>();
    if normalized.len() >= 2
        && normalized.starts_with(' ')
        && normalized.ends_with(' ')
        && normalized.chars().any(|ch| ch != ' ')
    {
        normalized = normalized[1..normalized.len() - 1].to_string();
    }
    Some((marker_len + close + marker_len, normalized))
}

pub(super) fn find_code_span_close(input: &str, marker_len: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'`' {
            index += input[index..].chars().next()?.len_utf8();
            continue;
        }
        let run = bytes[index..]
            .iter()
            .take_while(|byte| **byte == b'`')
            .count();
        if run == marker_len {
            return Some(index);
        }
        index += run;
    }
    None
}

pub(super) fn parse_inline_math(source: &str, start: usize) -> Option<(usize, String)> {
    let bytes = source.as_bytes();
    if bytes.get(start) != Some(&b'$') {
        return None;
    }
    if start > 0 && bytes[start - 1] == b'$' {
        return None;
    }
    if start + 1 >= bytes.len() || bytes[start + 1] == b'$' {
        return None;
    }

    let mut index = start + 1;
    while index < bytes.len() {
        let ch = source[index..].chars().next().expect("valid UTF-8");
        if ch == '\\' {
            index += ch.len_utf8();
            if let Some(next) = source[index..].chars().next() {
                index += next.len_utf8();
            }
            continue;
        }
        if ch == '$' {
            let previous_is_dollar = index > 0 && bytes[index - 1] == b'$';
            let next_is_dollar = index + 1 < bytes.len() && bytes[index + 1] == b'$';
            if !previous_is_dollar && !next_is_dollar {
                let math_source = &source[start + 1..index];
                if math_source.trim().is_empty() {
                    return None;
                }
                return Some((index - start + 1, math_source.to_string()));
            }
        }
        index += ch.len_utf8();
    }

    None
}
