//! Footnote definition bodies, labels and continuation indentation.
use super::*;

pub(super) fn parse_footnote_definition_at(
    source: &SourceDocument,
    index: usize,
    first_line: &str,
) -> Option<(String, MappedText, usize)> {
    if leading_spaces(first_line) > 3 {
        return None;
    }
    let trimmed = first_line.trim_start();
    let rest = trimmed.strip_prefix("[^")?;
    let close = rest.find("]:")?;
    let raw_label = &rest[..close];
    let label = normalize_footnote_label(raw_label)?;
    let raw_content = rest[close + 2..].trim_start();
    let first = source.line(index)?;
    let content_start =
        first.start + (raw_content.as_ptr() as usize - first.text.as_ptr() as usize);
    let mut content = MappedText::identity(raw_content, content_start);
    let mut consumed = 1usize;

    while let Some(next) = source.line(index + consumed) {
        if next.text.trim().is_empty() {
            let Some(after_blank) = source.line(index + consumed + 1) else {
                break;
            };
            if leading_columns(after_blank.text) < 4 {
                break;
            }
            let previous = source.line(index + consumed - 1)?;
            content.append(MappedText {
                text: "\n".to_string(),
                mapping: vec![crate::mapping::TextMapping {
                    display: 0..1,
                    source: std::iter::once(previous.end..next.start).collect(),
                    provenance: crate::mapping::TextProvenance::Normalized,
                    exact: false,
                }],
            });
            consumed += 1;
            continue;
        }
        if leading_columns(next.text) < 4 {
            break;
        }
        if !content.text.is_empty() {
            let previous = source.line(index + consumed - 1)?;
            content.append(MappedText {
                text: "\n".to_string(),
                mapping: vec![crate::mapping::TextMapping {
                    display: 0..1,
                    source: std::iter::once(previous.end..next.start).collect(),
                    provenance: crate::mapping::TextProvenance::Normalized,
                    exact: false,
                }],
            });
        }
        let body = strip_footnote_continuation_indent(next.text);
        content.append(MappedText::identity(
            body,
            next.start + next.text.len() - body.len(),
        ));
        consumed += 1;
    }

    Some((label, content, consumed))
}

pub(super) fn normalize_footnote_label(label: &str) -> Option<String> {
    let normalized = label.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_ascii_lowercase())
    }
}

pub(super) fn strip_footnote_continuation_indent(line: &str) -> &str {
    let mut spaces = 0usize;
    let mut index = 0usize;
    for ch in line.chars() {
        if ch == ' ' && spaces < 4 {
            spaces += 1;
            index += ch.len_utf8();
            continue;
        }
        break;
    }
    &line[index..]
}
