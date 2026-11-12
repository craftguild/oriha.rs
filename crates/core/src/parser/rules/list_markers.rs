//! List marker grammar, indentation columns and continuation merging.
use super::*;

pub(super) fn merge_lazy_continuation_blocks(blocks: &mut Vec<BlockNode>, normalized: &str) {
    if blocks.len() < 2
        || !matches!(
            blocks.first().map(|block| &block.kind),
            Some(BlockKind::Paragraph(_))
        )
    {
        return;
    }
    let Some(first) = blocks.first_mut() else {
        return;
    };
    first.span = SourceSpan::new(0, normalized.len());
    first.source_mapping.inline = crate::mapping::MappedText::identity(normalized, 0).mapping;
    first
        .attrs
        .insert("inline_source".to_string(), normalized.to_string());
    blocks.truncate(1);
}

pub(super) fn same_list_marker_type(
    marker: &ParsedListMarker,
    ordered: bool,
    marker_text: &str,
) -> bool {
    if marker.ordered != ordered {
        return false;
    }
    if ordered {
        marker
            .raw_marker
            .ends_with('.')
            .eq(&marker_text.ends_with('.'))
            && marker
                .raw_marker
                .ends_with(')')
                .eq(&marker_text.ends_with(')'))
    } else {
        marker.raw_marker == marker_text
    }
}

pub(super) fn marker_indent_columns(line: &str) -> usize {
    let mut columns = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => columns += 1,
            '\t' => columns += 4 - (columns % 4),
            _ => break,
        }
    }
    columns
}

pub(super) fn strip_columns_expanding_indentation(line: &str, target_column: usize) -> String {
    let mut columns = 0usize;
    for (idx, ch) in line.char_indices() {
        if ch != ' ' && ch != '\t' {
            return line[idx..].to_string();
        }
        let next_columns = match ch {
            '\t' => columns + 4 - (columns % 4),
            _ => columns + 1,
        };
        if next_columns >= target_column {
            let mut out = String::new();
            if next_columns > target_column {
                out.push_str(&" ".repeat(next_columns - target_column));
            }
            let mut expanded_columns = next_columns;
            let rest = &line[idx + ch.len_utf8()..];
            let mut body_start = rest.len();
            for (rest_idx, rest_ch) in rest.char_indices() {
                match rest_ch {
                    ' ' => {
                        out.push(' ');
                        expanded_columns += 1;
                    }
                    '\t' => {
                        let next_tab = expanded_columns + 4 - (expanded_columns % 4);
                        out.push_str(&" ".repeat(next_tab - expanded_columns));
                        expanded_columns = next_tab;
                    }
                    _ => {
                        body_start = rest_idx;
                        break;
                    }
                }
            }
            out.push_str(&rest[body_start..]);
            return out;
        }
        columns = next_columns;
    }
    String::new()
}

pub(super) fn detect_task_list_marker(source: &str) -> Option<TaskListItemNode> {
    let trimmed = source.trim_start();
    let checked = if trimmed.starts_with("[ ]") {
        false
    } else if trimmed.starts_with("[x]") || trimmed.starts_with("[X]") {
        true
    } else {
        return None;
    };
    Some(TaskListItemNode { checked })
}

pub(super) fn parse_list_marker(line: &str) -> Option<ParsedListMarker> {
    if leading_spaces(line) > 3 {
        return None;
    }
    let (trimmed_start, indent) = first_nonspace_index_and_column(line);
    let trimmed = &line[trimmed_start..];
    if trimmed.is_empty() {
        return None;
    }
    let bytes = trimmed.as_bytes();
    let first = bytes[0];
    if first == b'-' || first == b'+' || first == b'*' {
        let marker_width = 1usize;
        let after_marker_column = indent + marker_width;
        if trimmed.len() == marker_width || trimmed[marker_width..].trim().is_empty() {
            return Some(ParsedListMarker {
                indent,
                ordered: false,
                raw_marker: trimmed[..marker_width].to_string(),
                content_column: after_marker_column + 1,
                content_byte_start: line.len(),
                has_content: false,
            });
        }
        let second = bytes[1];
        if second == b' ' || second == b'\t' {
            let padding_columns =
                marker_padding_columns(line, trimmed_start + 1, after_marker_column);
            let content_column = content_column_after_marker(after_marker_column, padding_columns);
            return Some(ParsedListMarker {
                indent,
                ordered: false,
                raw_marker: trimmed[..1].to_string(),
                content_column,
                content_byte_start: byte_index_for_column(line, content_column),
                has_content: true,
            });
        }
    }

    let digits = trimmed.bytes().take_while(|ch| ch.is_ascii_digit()).count();
    if digits == 0 || digits > 9 {
        return None;
    }
    if trimmed[digits..].is_empty() {
        return None;
    }
    let marker = trimmed[digits..].chars().next().unwrap();
    let marker_width = digits + 1;
    if (marker == '.' || marker == ')')
        && (trimmed.len() == marker_width || trimmed[marker_width..].trim().is_empty())
    {
        let after_marker_column = indent + marker_width;
        return Some(ParsedListMarker {
            indent,
            ordered: true,
            raw_marker: trimmed[..marker_width].to_string(),
            content_column: after_marker_column + 1,
            content_byte_start: line.len(),
            has_content: false,
        });
    }
    if (marker == '.' || marker == ')')
        && trimmed.get(digits + 1..).is_some_and(|rest| {
            let b = rest.as_bytes();
            !b.is_empty() && (b[0] == b' ' || b[0] == b'\t')
        })
    {
        let after_marker_column = indent + marker_width;
        let padding_columns =
            marker_padding_columns(line, trimmed_start + marker_width, after_marker_column);
        let content_column = content_column_after_marker(after_marker_column, padding_columns);
        return Some(ParsedListMarker {
            indent,
            ordered: true,
            raw_marker: trimmed[..digits + 1].to_string(),
            content_column,
            content_byte_start: byte_index_for_column(line, content_column),
            has_content: true,
        });
    }

    None
}

pub(super) fn marker_padding_columns(line: &str, local_start: usize, start_column: usize) -> usize {
    let mut columns = start_column;
    let mut padding = 0usize;
    for ch in line[local_start..].chars() {
        let next_columns = match ch {
            ' ' => columns + 1,
            '\t' => columns + 4 - (columns % 4),
            _ => break,
        };
        padding += next_columns - columns;
        columns = next_columns;
        if padding > 4 {
            break;
        }
    }
    padding
}

pub(super) fn content_column_after_marker(
    after_marker_column: usize,
    padding_columns: usize,
) -> usize {
    if (1..=4).contains(&padding_columns) {
        after_marker_column + padding_columns
    } else {
        after_marker_column + 1
    }
}
