use super::*;

pub(super) fn parse_link_or_image_with_options<'a>(
    input: &'a str,
    reject_nested_links: bool,
    link_references: &BTreeMap<String, LinkReference>,
) -> Option<(usize, &'a str, String, Option<String>)> {
    let _depth = recursion::enter();
    if !input.starts_with('[') {
        return None;
    }
    let end_text = find_closing_bracket(input)?;
    let text = &input[1..end_text];
    if reject_nested_links && link_text_contains_inline_link(text, link_references) {
        return None;
    }
    let rest = &input[end_text + 1..];
    if !rest.starts_with('(') {
        return None;
    }
    let close = find_link_close(rest)?;
    let inside = rest[1..close].trim();
    let (url, title) = parse_link_destination_and_title(inside)?;
    let consumed = end_text + 1 + close + 1;
    if code_span_crosses_link_text_boundary(input, end_text, consumed) {
        return None;
    }
    Some((consumed, text, url, title))
}

pub(super) fn link_text_contains_inline_link(
    text: &str,
    link_references: &BTreeMap<String, LinkReference>,
) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        let ch = text[index..].chars().next().expect("valid UTF-8");
        if ch == '`' && (index == 0 || bytes[index - 1] != b'`') {
            if let Some((consumed, _)) = parse_code_span(&text[index..]) {
                index += consumed;
                continue;
            }
        }
        if ch == '['
            && (index == 0 || bytes[index - 1] != b'!')
            && (parse_link_or_image_with_options(&text[index..], true, link_references).is_some()
                || parse_reference_link_or_image(&text[index..], link_references, true).is_some())
        {
            return true;
        }
        index += ch.len_utf8();
    }
    false
}

pub(super) fn code_span_crosses_link_text_boundary(
    input: &str,
    end_text: usize,
    consumed: usize,
) -> bool {
    let mut index = 0usize;
    while index < consumed {
        let ch = input[index..].chars().next().expect("valid UTF-8");
        if ch == '`' && (index == 0 || input.as_bytes()[index - 1] != b'`') {
            if let Some((code_consumed, _)) = parse_code_span(&input[index..]) {
                if index < end_text && index + code_consumed > end_text {
                    return true;
                }
                index += code_consumed;
                continue;
            }
        }
        index += ch.len_utf8();
    }
    false
}

pub(super) fn parse_reference_link_or_image<'a>(
    input: &'a str,
    link_references: &BTreeMap<String, LinkReference>,
    reject_nested_links: bool,
) -> Option<(usize, &'a str, LinkReference)> {
    if !input.starts_with('[') {
        return None;
    }
    let end_text = find_closing_bracket(input)?;
    let text = &input[1..end_text];
    if reject_nested_links && link_text_contains_inline_link(text, link_references) {
        return None;
    }
    let rest = &input[end_text + 1..];

    if let Some(rest) = rest.strip_prefix('[') {
        let end_label = find_unescaped_closing_bracket(rest)?;
        let raw_label = &rest[..end_label];
        let label = if raw_label.is_empty() {
            normalize_reference_label(text)
        } else {
            normalize_reference_label(raw_label)
        };
        let reference = link_references.get(&label)?.clone();
        return Some((end_text + 1 + end_label + 2, text, reference));
    }

    let label = normalize_reference_label(text);
    let reference = link_references.get(&label)?.clone();
    Some((end_text + 1, text, reference))
}

pub(super) fn find_closing_bracket(input: &str) -> Option<usize> {
    if !input.starts_with('[') {
        return None;
    }
    find_unescaped_closing_bracket(&input[1..]).map(|idx| idx + 1)
}

pub(super) fn find_unescaped_closing_bracket(input: &str) -> Option<usize> {
    let mut escaped = false;
    let mut depth = 0usize;
    let mut idx = 0usize;
    while idx < input.len() {
        let ch = input[idx..].chars().next().expect("valid UTF-8");
        if escaped {
            escaped = false;
            idx += ch.len_utf8();
            continue;
        }
        if ch == '\\' {
            escaped = true;
            idx += ch.len_utf8();
            continue;
        }
        if ch == '`' {
            if let Some((consumed, _)) = parse_code_span(&input[idx..]) {
                idx += consumed;
                continue;
            }
        }
        if ch == '<' {
            if let Some((consumed, _)) = parse_autolink_or_inline_html(&input[idx..], false) {
                idx += consumed;
                continue;
            }
            if let Some(close) = input[idx..].find('>') {
                idx += close + 1;
                continue;
            }
        }
        if ch == '[' {
            depth += 1;
            idx += ch.len_utf8();
            continue;
        }
        if ch == ']' {
            if depth == 0 {
                return Some(idx);
            }
            depth -= 1;
        }
        idx += ch.len_utf8();
    }
    None
}

pub(super) fn normalize_reference_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .flat_map(|ch| {
            if ch == 'ß' || ch == 'ẞ' {
                "ss".chars().collect::<Vec<_>>()
            } else {
                ch.to_lowercase().collect::<Vec<_>>()
            }
        })
        .collect()
}

pub(super) fn find_link_close(input: &str) -> Option<usize> {
    let mut escaped = false;
    for (idx, ch) in input.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == ')' && parse_link_destination_and_title(input[1..idx].trim()).is_some() {
            return Some(idx);
        }
    }
    None
}

pub(super) fn parse_link_destination_and_title(input: &str) -> Option<(String, Option<String>)> {
    if input.is_empty() {
        return Some((String::new(), None));
    }
    let (destination, rest) = split_link_destination(input)?;
    let title = if rest.is_empty() || rest.trim().is_empty() {
        None
    } else if rest.starts_with(is_ascii_link_space) {
        Some(parse_link_title(rest.trim())?)
    } else {
        return None;
    };
    Some((decode_markdown(&destination), title))
}

pub(super) fn split_link_destination(input: &str) -> Option<(String, &str)> {
    let input = input.trim();
    if let Some(rest) = input.strip_prefix('<') {
        let mut escaped = false;
        for (idx, ch) in rest.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '\n' || ch == '<' {
                return None;
            }
            if ch == '>' {
                return Some((rest[..idx].to_string(), &rest[idx + ch.len_utf8()..]));
            }
        }
        None
    } else {
        let mut escaped = false;
        let mut paren_depth = 0usize;
        for (idx, ch) in input.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '<' {
                return None;
            }
            if is_ascii_link_space(ch) {
                let destination = &input[..idx];
                return raw_link_destination_is_balanced(destination)
                    .then_some((destination.to_string(), &input[idx..]));
            }
            if ch == '(' {
                paren_depth += 1;
            } else if ch == ')' {
                if paren_depth == 0 {
                    let destination = &input[..idx];
                    return raw_link_destination_is_balanced(destination)
                        .then_some((destination.to_string(), &input[idx..]));
                }
                paren_depth -= 1;
            }
        }
        raw_link_destination_is_balanced(input).then_some((input.to_string(), ""))
    }
}

pub(super) fn raw_link_destination_is_balanced(destination: &str) -> bool {
    let mut escaped = false;
    let mut paren_depth = 0usize;
    for ch in destination.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '(' {
            paren_depth += 1;
        } else if ch == ')' {
            if paren_depth == 0 {
                return false;
            }
            paren_depth -= 1;
        }
    }
    paren_depth == 0
}

pub(super) fn is_ascii_link_space(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r')
}

pub(super) fn parse_link_title(input: &str) -> Option<String> {
    let mut chars = input.chars();
    let open = chars.next()?;
    let close = match open {
        '"' => '"',
        '\'' => '\'',
        '(' => ')',
        _ => return None,
    };
    let body = &input[open.len_utf8()..];
    let mut escaped = false;
    for (idx, ch) in body.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == close {
            if body[idx + ch.len_utf8()..].trim().is_empty() {
                return Some(decode_markdown(&body[..idx]));
            }
            return None;
        }
    }
    None
}

pub(super) fn is_ascii_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '!' | '"'
            | '#'
            | '$'
            | '%'
            | '&'
            | '\''
            | '('
            | ')'
            | '*'
            | '+'
            | ','
            | '-'
            | '.'
            | '/'
            | ':'
            | ';'
            | '<'
            | '='
            | '>'
            | '?'
            | '@'
            | '['
            | '\\'
            | ']'
            | '^'
            | '_'
            | '`'
            | '{'
            | '|'
            | '}'
            | '~'
    )
}
