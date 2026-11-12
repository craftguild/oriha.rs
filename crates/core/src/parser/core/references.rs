//! Link reference collection, labels, destinations and title grammar.
use super::*;

pub(super) fn parse_link_reference_definition_at(
    source: &SourceDocument,
    index: usize,
    first_line: &str,
) -> Option<(String, LinkReference, usize)> {
    if leading_spaces(first_line) > 3 {
        return None;
    }
    let mut trimmed = first_line.trim_start().to_string();
    let mut consumed = 1usize;
    let (raw_label, rest) = loop {
        match parse_reference_label_prefix(&trimmed) {
            ReferenceLabelPrefix::Complete { label, rest } => break (label.to_string(), rest),
            ReferenceLabelPrefix::Incomplete => {
                let next = source.line(index + consumed)?;
                if next.text.trim().is_empty() {
                    return None;
                }
                trimmed.push('\n');
                trimmed.push_str(
                    strip_blockquote_prefix(next.text)
                        .unwrap_or(next.text)
                        .trim_start(),
                );
                consumed += 1;
            }
            ReferenceLabelPrefix::Invalid => return None,
        }
    };

    let label = normalize_reference_label(&raw_label);
    let mut definition = rest.trim().to_string();
    if definition.is_empty() {
        let next = source.line(index + consumed)?;
        if next.text.trim().is_empty() {
            return None;
        }
        definition.push_str(
            strip_blockquote_prefix(next.text)
                .unwrap_or(next.text)
                .trim(),
        );
        consumed += 1;
    }
    while should_collect_title_line(
        &definition,
        source.line(index + consumed).map(|line| line.text),
    ) {
        let next = source.line(index + consumed)?;
        let next_text = strip_blockquote_prefix(next.text)
            .unwrap_or(next.text)
            .trim();
        let candidate = if definition.is_empty() {
            next_text.to_string()
        } else {
            format!("{definition}\n{next_text}")
        };
        if parse_destination_and_title(&candidate).is_none()
            && parse_destination_and_title(&definition).is_some()
        {
            break;
        }
        definition = candidate;
        consumed += 1;
    }

    let (destination, title) = parse_destination_and_title(&definition)?;
    Some((label, LinkReference { destination, title }, consumed))
}

pub(super) fn parse_reference_label_prefix(input: &str) -> ReferenceLabelPrefix<'_> {
    if !input.starts_with('[') {
        return ReferenceLabelPrefix::Invalid;
    }

    let mut escaped = false;
    let mut label_len = 0usize;
    for (idx, ch) in input[1..].char_indices() {
        let absolute_idx = idx + 1;
        if escaped {
            escaped = false;
            label_len += 1;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            label_len += 1;
            continue;
        }
        if ch == '[' {
            return ReferenceLabelPrefix::Invalid;
        }
        if ch == ']' {
            let after_label = absolute_idx + ch.len_utf8();
            let Some(rest) = input[after_label..].strip_prefix(':') else {
                return ReferenceLabelPrefix::Invalid;
            };
            let label = &input[1..absolute_idx];
            if label_len > 999 || !is_valid_reference_label(label) {
                return ReferenceLabelPrefix::Invalid;
            }
            return ReferenceLabelPrefix::Complete { label, rest };
        }
        label_len += 1;
    }

    ReferenceLabelPrefix::Incomplete
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

pub(super) fn is_valid_reference_label(label: &str) -> bool {
    let label = label.trim();
    !label.is_empty() && !contains_unescaped_reference_bracket(label)
}

pub(super) fn contains_unescaped_reference_bracket(label: &str) -> bool {
    let mut escaped = false;
    for ch in label.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '[' || ch == ']' {
            return true;
        }
    }
    false
}

pub(super) fn should_collect_title_line(definition: &str, next_line: Option<&str>) -> bool {
    let Some(next_line) = next_line else {
        return false;
    };
    let next_line = strip_blockquote_prefix(next_line)
        .unwrap_or(next_line)
        .trim();
    if next_line.is_empty() {
        return false;
    }
    let Some((_, rest)) = split_destination(definition.trim()) else {
        return false;
    };
    let rest = rest.trim();
    if rest.is_empty() {
        return next_line
            .chars()
            .next()
            .is_some_and(|ch| matches!(ch, '"' | '\'' | '('));
    }
    has_unclosed_title(rest)
}

pub(super) fn parse_destination_and_title(input: &str) -> Option<(String, Option<String>)> {
    let (destination, rest) = split_destination(input)?;
    if destination.is_empty() && input.trim_start() != "<>" {
        return None;
    }
    let title = if rest.is_empty() || rest.trim().is_empty() {
        None
    } else if rest.starts_with(char::is_whitespace) {
        Some(parse_title(rest.trim())?)
    } else {
        return None;
    };
    Some((decode_markdown(destination), title))
}

pub(super) fn split_destination(input: &str) -> Option<(&str, &str)> {
    let input = input.trim();
    if let Some(rest) = input.strip_prefix('<') {
        let end = rest.find('>')?;
        Some((&rest[..end], &rest[end + 1..]))
    } else {
        let split = input
            .char_indices()
            .find(|(_, ch)| ch.is_whitespace())
            .map(|(idx, _)| idx)
            .unwrap_or(input.len());
        Some((&input[..split], &input[split..]))
    }
}

pub(super) fn parse_title(input: &str) -> Option<String> {
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
        if ch == close && body[idx + ch.len_utf8()..].trim().is_empty() {
            return Some(decode_markdown(&body[..idx]));
        }
    }
    None
}

pub(super) fn has_unclosed_title(input: &str) -> bool {
    let mut chars = input.chars();
    let Some(open) = chars.next() else {
        return false;
    };
    let close = match open {
        '"' => '"',
        '\'' => '\'',
        '(' => ')',
        _ => return false,
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
            return !body[idx + ch.len_utf8()..].trim().is_empty();
        }
    }
    true
}
