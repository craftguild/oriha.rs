//! Link reference labels, destinations and multiline titles.
use super::*;

impl BlockDetector for LinkReferenceDefinitionRule {
    fn detect(&self, cursor: &SourceCursor<'_>, _context: &ParseContext) -> Option<ParseStart> {
        is_link_reference_definition_start(cursor).then_some(ParseStart {
            kind: BlockStartKind::LinkReferenceDefinition,
        })
    }
}

impl BlockParser for LinkReferenceDefinitionRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        _context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let count = link_reference_definition_line_count(cursor).unwrap_or(1);
        let mut lines = Vec::new();
        for _ in 0..count {
            let Some(line) = cursor.current() else {
                break;
            };
            lines.push(line);
            let _ = cursor.advance();
        }
        let span = lines_to_span(&lines).unwrap_or_else(|| SourceSpan::new(0, 0));
        ParseResult {
            span,
            block: BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::LinkReferenceDefinition,
                span,
                attrs: BTreeMap::new(),
                source_mapping: Default::default(),
                diagnostics: Vec::new(),
                children: Vec::new(),
            },
        }
    }
}

pub(super) fn is_link_reference_definition(line: &str) -> bool {
    if leading_spaces(line) > 3 {
        return false;
    }
    let trimmed = line.trim_start();
    matches!(
        parse_reference_label_prefix(trimmed),
        ReferenceLabelPrefix::Complete { .. }
    )
}

pub(super) fn is_link_reference_definition_start(cursor: &SourceCursor<'_>) -> bool {
    link_reference_definition_parts(cursor).is_some()
}

pub(super) fn link_reference_definition_line_count(cursor: &SourceCursor<'_>) -> Option<usize> {
    link_reference_definition_parts(cursor).map(|(count, _)| count)
}

pub(super) fn link_reference_definition_parts(
    cursor: &SourceCursor<'_>,
) -> Option<(usize, String)> {
    let line = cursor.current()?;
    if leading_spaces(line.text) > 3 {
        return None;
    }
    let mut raw = line.text.trim_start().to_string();
    let mut count = 1usize;
    let (_, rest) = loop {
        match parse_reference_label_prefix(&raw) {
            ReferenceLabelPrefix::Complete { label, rest } => break (label.to_string(), rest),
            ReferenceLabelPrefix::Incomplete => {
                let next = cursor.peek(count)?;
                if next.text.trim().is_empty() || leading_spaces(next.text) > 3 {
                    return None;
                }
                raw.push('\n');
                raw.push_str(next.text.trim_start());
                count += 1;
            }
            ReferenceLabelPrefix::Invalid => return None,
        }
    };

    let mut definition = rest.trim().to_string();
    if definition.is_empty() {
        let next = cursor.peek(count)?;
        if next.text.trim().is_empty() {
            return None;
        }
        definition.push_str(next.text.trim());
        count += 1;
    }
    while should_collect_reference_title_line(&definition, cursor.peek(count).map(|next| next.text))
    {
        if !definition.is_empty() {
            definition.push('\n');
        }
        definition.push_str(cursor.peek(count)?.text.trim());
        count += 1;
    }
    parse_reference_definition_destination_and_title(&definition)?;
    Some((count, definition))
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
            if label_len > 999 || label.trim().is_empty() {
                return ReferenceLabelPrefix::Invalid;
            }
            return ReferenceLabelPrefix::Complete { label, rest };
        }
        label_len += 1;
    }

    ReferenceLabelPrefix::Incomplete
}

pub(super) fn should_collect_reference_title_line(
    definition: &str,
    next_line: Option<&str>,
) -> bool {
    let Some(next_line) = next_line else {
        return false;
    };
    let next_line = next_line.trim();
    if next_line.is_empty() {
        return false;
    }
    let Some((_, rest)) = split_reference_destination(definition.trim()) else {
        return false;
    };
    let rest = rest.trim();
    if rest.is_empty() {
        return next_line
            .chars()
            .next()
            .is_some_and(|ch| matches!(ch, '"' | '\'' | '('));
    }
    has_unclosed_reference_title(rest)
}

pub(super) fn parse_reference_definition_destination_and_title(
    input: &str,
) -> Option<(&str, Option<&str>)> {
    let (destination, rest) = split_reference_destination(input)?;
    if destination.is_empty() && input.trim_start() != "<>" {
        return None;
    }
    if rest.is_empty() || rest.trim().is_empty() {
        return Some((destination, None));
    }
    if rest.starts_with(char::is_whitespace) {
        parse_reference_title(rest.trim()).map(|title| (destination, Some(title)))
    } else {
        None
    }
}

pub(super) fn split_reference_destination(input: &str) -> Option<(&str, &str)> {
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

pub(super) fn parse_reference_title(input: &str) -> Option<&str> {
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
            return Some(&body[..idx]);
        }
    }
    None
}

pub(super) fn has_unclosed_reference_title(input: &str) -> bool {
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
