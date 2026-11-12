use crate::mapping::{MappedText, TextProvenance};
use crate::source::{first_nonspace_index_and_column, SourceLineRef};

pub(super) fn join_lines(
    lines: &[SourceLineRef<'_>],
    mut map: impl FnMut(usize, SourceLineRef<'_>) -> MappedText,
) -> MappedText {
    let mut out = MappedText::default();
    for (i, line) in lines.iter().copied().enumerate() {
        if i > 0 {
            let previous = lines[i - 1];
            let mut newline = MappedText::identity("\n", previous.end);
            if line.start - previous.end != 1 {
                newline.mapping[0].source = std::iter::once(previous.end..line.start).collect();
                newline.mapping[0].exact = false;
                newline.mapping[0].provenance = TextProvenance::Normalized;
            }
            out.append(newline);
        }
        out.append(map(i, line));
    }
    out
}

pub(super) fn plain(lines: &[SourceLineRef<'_>]) -> MappedText {
    join_lines(lines, |_, line| MappedText::identity(line.text, line.start))
}

fn expand_leading(mapped: &MappedText, mut column: usize) -> MappedText {
    let mut out = MappedText::default();
    let mut body = 0;
    for (i, ch) in mapped.text.char_indices() {
        if ch != ' ' && ch != '\t' {
            break;
        }
        let width = if ch == '\t' { 4 - column % 4 } else { 1 };
        if ch == '\t' {
            out.append(mapped.transformed(
                i..i + 1,
                &" ".repeat(width),
                TextProvenance::Normalized,
            ));
        } else {
            out.append(mapped.slice(i..i + 1));
        }
        column += width;
        body = i + ch.len_utf8();
    }
    out.append(mapped.slice(body..mapped.text.len()));
    out
}

pub(super) fn quote_line(line: SourceLineRef<'_>) -> MappedText {
    let mapped = MappedText::identity(line.text, line.start);
    let (marker, column) = first_nonspace_index_and_column(line.text);
    if line.text.chars().take_while(|c| *c == ' ').count() > 3
        || !line.text[marker..].starts_with('>')
    {
        return mapped;
    }
    let start = marker + 1;
    let Some(ch) = line.text[start..].chars().next() else {
        return MappedText::default();
    };
    if ch != ' ' && ch != '\t' {
        return mapped.slice(start..line.text.len());
    }
    let after_marker = column + 1;
    let next = after_marker + if ch == '\t' { 4 - after_marker % 4 } else { 1 };
    let mut out = mapped.transformed(
        start..start + 1,
        &" ".repeat(next - after_marker - 1),
        TextProvenance::Normalized,
    );
    out.append(expand_leading(
        &mapped.slice(start + 1..line.text.len()),
        next,
    ));
    out
}

pub(super) fn quote(lines: &[SourceLineRef<'_>]) -> MappedText {
    join_lines(lines, |_, line| quote_line(line))
}

pub(super) fn strip_columns(line: SourceLineRef<'_>, target: usize) -> MappedText {
    let mapped = MappedText::identity(line.text, line.start);
    let mut column = 0;
    for (i, ch) in line.text.char_indices() {
        if ch != ' ' && ch != '\t' {
            return mapped.slice(i..line.text.len());
        }
        let next = column + if ch == '\t' { 4 - column % 4 } else { 1 };
        if next >= target {
            let mut out = mapped.transformed(
                i..i + 1,
                &" ".repeat(next - target),
                TextProvenance::Normalized,
            );
            out.append(expand_leading(&mapped.slice(i + 1..line.text.len()), next));
            return out;
        }
        column = next;
    }
    MappedText::default()
}

pub(super) fn list(
    lines: &[SourceLineRef<'_>],
    first_start: usize,
    continuation: usize,
) -> MappedText {
    join_lines(lines, |i, line| {
        let mapped = MappedText::identity(line.text, line.start);
        if i == 0 {
            let start = first_start.min(line.text.len());
            let column = crate::source::visual_columns(&line.text[..start]);
            let mut out = if column > continuation && start > 0 {
                mapped.transformed(
                    start - 1..start,
                    &" ".repeat(column - continuation),
                    TextProvenance::Normalized,
                )
            } else {
                MappedText::default()
            };
            out.append(expand_leading(
                &mapped.slice(start..line.text.len()),
                column,
            ));
            out
        } else if line.text.trim().is_empty() {
            MappedText::default()
        } else {
            let stripped = strip_columns(line, continuation);
            if stripped.text.is_empty() {
                mapped.slice(line.text.len() - line.text.trim_start().len()..line.text.len())
            } else {
                stripped
            }
        }
    })
}

pub(super) fn setext(lines: &[SourceLineRef<'_>]) -> MappedText {
    join_lines(&lines[..lines.len().saturating_sub(1)], |i, line| {
        let start = if i == 0 {
            line.text.bytes().take(3).take_while(|b| *b == b' ').count()
        } else {
            line.text.len() - line.text.trim_start().len()
        };
        MappedText::identity(line.text, line.start)
            .slice(start..line.text.trim_end().len().max(start))
    })
}

pub(super) fn paragraph(mapped: &MappedText) -> MappedText {
    let mut out = MappedText::default();
    let mut offset = 0;
    for (i, line) in mapped.text.lines().enumerate() {
        if i > 0 {
            // offset is immediately after the preceding line ending.
            let newline_start =
                if offset >= 2 && mapped.text.as_bytes().get(offset - 2..offset) == Some(b"\r\n") {
                    offset - 2
                } else {
                    offset - 1
                };
            out.append(mapped.transformed(
                newline_start..offset,
                "\n",
                if offset - newline_start == 1 {
                    TextProvenance::Literal
                } else {
                    TextProvenance::Normalized
                },
            ));
        }
        let start = if i == 0 {
            line.bytes().take(3).take_while(|b| *b == b' ').count()
        } else {
            line.len() - line.trim_start().len()
        };
        out.append(mapped.slice(offset + start..offset + line.len()));
        offset += line.len();
        if mapped.text[offset..].starts_with("\r\n") {
            offset += 2;
        } else if mapped.text[offset..].starts_with('\n') {
            offset += 1;
        }
    }
    let end = out.text.trim_end().len();
    out.slice(0..end)
}

pub(super) fn table_cells(line: SourceLineRef<'_>) -> Vec<MappedText> {
    let input = MappedText::identity(line.text, line.start).trim();
    let start = usize::from(input.text.starts_with('|'));
    let end = input.text.len() - usize::from(input.text.ends_with('|') && input.text.len() > start);
    let mut cells = Vec::new();
    let mut cell = MappedText::default();
    let mut i = start;
    while i < end {
        let ch = input.text[i..].chars().next().unwrap();
        if ch == '\\' && input.text[i + 1..end].starts_with('|') {
            cell.append(input.transformed(i..i + 2, "|", TextProvenance::Escape));
            i += 2;
        } else if ch == '|' {
            cells.push(cell.trim());
            cell = MappedText::default();
            i += 1;
        } else {
            cell.append(input.slice(i..i + ch.len_utf8()));
            i += ch.len_utf8();
        }
    }
    cells.push(cell.trim());
    cells
}
