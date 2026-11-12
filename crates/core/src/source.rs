use crate::types::SourceRange;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

impl SourceSpan {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end: end.max(start),
        }
    }

    pub fn with_end(start: usize, len: usize) -> Self {
        Self {
            start,
            end: start + len,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SourceLineRef<'a> {
    pub number: u32,
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

impl<'a> SourceLineRef<'a> {
    pub fn contains_non_empty_text(&self) -> bool {
        !self.text.trim().is_empty()
    }

    pub fn slice_from_column(&self, column: usize) -> LineSlice<'a> {
        let local_start = byte_index_for_column(self.text, column);
        LineSlice {
            text: &self.text[local_start..],
            start: self.start + local_start,
            end: self.end,
            column_start: column,
            column_end: visual_columns(self.text),
        }
    }

    pub fn first_nonspace(&self) -> LineSlice<'a> {
        let (local_start, column) = first_nonspace_index_and_column(self.text);
        LineSlice {
            text: &self.text[local_start..],
            start: self.start + local_start,
            end: self.end,
            column_start: column,
            column_end: visual_columns(self.text),
        }
    }

    pub fn indent_columns(&self) -> usize {
        first_nonspace_index_and_column(self.text).1
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LineSlice<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
    pub column_start: usize,
    pub column_end: usize,
}

impl<'a> LineSlice<'a> {
    pub fn is_blank(&self) -> bool {
        self.text.trim().is_empty()
    }
}

pub fn visual_columns(text: &str) -> usize {
    let mut columns = 0usize;
    for ch in text.chars() {
        match ch {
            '\t' => columns += 4 - (columns % 4),
            _ => columns += 1,
        }
    }
    columns
}

pub fn byte_index_for_column(text: &str, target_column: usize) -> usize {
    let mut columns = 0usize;
    for (idx, ch) in text.char_indices() {
        if columns >= target_column {
            return idx;
        }
        let next_columns = match ch {
            '\t' => columns + 4 - (columns % 4),
            _ => columns + 1,
        };
        if next_columns > target_column {
            return idx + ch.len_utf8();
        }
        columns = next_columns;
    }
    text.len()
}

pub fn first_nonspace_index_and_column(text: &str) -> (usize, usize) {
    let mut columns = 0usize;
    for (idx, ch) in text.char_indices() {
        match ch {
            ' ' => columns += 1,
            '\t' => columns += 4 - (columns % 4),
            _ => return (idx, columns),
        }
    }
    (text.len(), columns)
}

#[derive(Clone, Debug)]
pub struct SourceDocument {
    // Source text and line indexes are immutable. Working render contexts and
    // annotated documents can share them without copying the complete source.
    inner: Arc<SourceData>,
}

#[derive(Debug)]
struct SourceData {
    text: String,
    line_starts: Vec<usize>,
    lines: Vec<LineIndex>,
}

#[derive(Clone, Debug)]
struct LineIndex {
    number: u32,
    start: usize,
    end: usize,
}

impl SourceDocument {
    pub fn new(source: impl Into<String>) -> Self {
        let text = source.into();
        let mut line_starts = Vec::new();
        let mut lines = Vec::new();

        if text.is_empty() {
            return Self {
                inner: Arc::new(SourceData {
                    text,
                    line_starts: vec![0],
                    lines: vec![LineIndex {
                        number: 1,
                        start: 0,
                        end: 0,
                    }],
                }),
            };
        }

        let mut start = 0usize;
        for (line_index, raw_line) in text.split_inclusive('\n').enumerate() {
            let mut line_text = raw_line;
            if line_text.ends_with('\n') {
                line_text = &line_text[..line_text.len() - 1];
            }
            if line_text.ends_with('\r') {
                line_text = &line_text[..line_text.len() - 1];
            }

            line_starts.push(start);
            let end = start + line_text.len();
            lines.push(LineIndex {
                number: (line_index + 1) as u32,
                start,
                end,
            });
            start += raw_line.len();
        }

        Self {
            inner: Arc::new(SourceData {
                text,
                line_starts,
                lines,
            }),
        }
    }

    pub fn text(&self) -> &str {
        &self.inner.text
    }

    pub fn line_count(&self) -> usize {
        self.inner.lines.len()
    }

    pub fn line(&self, index: usize) -> Option<SourceLineRef<'_>> {
        let line = self.inner.lines.get(index)?;
        Some(SourceLineRef {
            number: line.number,
            text: &self.inner.text[line.start..line.end],
            start: line.start,
            end: line.end,
        })
    }

    pub fn source_span(&self, span: SourceSpan) -> &str {
        let end = span.end.min(self.inner.text.len());
        let start = span.start.min(end);
        let safe_start = floor_char_boundary(&self.inner.text, start);
        let safe_end = ceil_char_boundary(&self.inner.text, end);
        &self.inner.text[safe_start..safe_end]
    }

    pub fn to_source_range(&self, span: SourceSpan) -> SourceRange {
        let raw_end = span.end.min(self.inner.text.len());
        let raw_start = span.start.min(raw_end);
        let start = floor_char_boundary(&self.inner.text, raw_start);
        let end = ceil_char_boundary(&self.inner.text, raw_end);
        let (start_line, start_column) = self.line_col(start);
        let last_offset = if end == 0 { 0 } else { end - 1 };
        let (end_line, end_column) = self.line_col(last_offset);

        SourceRange {
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }

    fn line_col(&self, offset: usize) -> (u32, u32) {
        if self.inner.text.is_empty() {
            return (1, 1);
        }

        let mut low = 0;
        let mut high = self.inner.line_starts.len();
        while low < high {
            let mid = (low + high) / 2;
            if self.inner.line_starts[mid] <= offset {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        let line_no = low.saturating_sub(1);
        let line_start = self.inner.line_starts[line_no];
        let column = offset.saturating_sub(line_start) + 1;
        (line_no as u32 + 1, column as u32)
    }
}

fn floor_char_boundary(text: &str, idx: usize) -> usize {
    if idx >= text.len() {
        return text.len();
    }
    let mut i = idx;
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn ceil_char_boundary(text: &str, idx: usize) -> usize {
    if idx >= text.len() {
        return text.len();
    }
    let mut i = idx;
    while i < text.len() && !text.is_char_boundary(i) {
        i += 1;
    }
    i
}

#[derive(Clone, Debug)]
pub struct SourceCursor<'a> {
    document: &'a SourceDocument,
    position: usize,
}

impl<'a> SourceCursor<'a> {
    pub fn new(document: &'a SourceDocument) -> Self {
        Self {
            document,
            position: 0,
        }
    }

    pub fn current(&self) -> Option<SourceLineRef<'a>> {
        self.document.line(self.position)
    }

    pub fn peek(&self, offset: usize) -> Option<SourceLineRef<'a>> {
        self.document.line(self.position + offset)
    }

    pub fn is_eof(&self) -> bool {
        self.position >= self.document.line_count()
    }

    pub fn advance(&mut self) -> Option<SourceLineRef<'a>> {
        let line = self.current();
        if line.is_some() {
            self.position += 1;
        }
        line
    }

    pub fn skip_blank_lines(&mut self) {
        while !self.is_eof()
            && self
                .current()
                .is_some_and(|line| line.text.trim().is_empty())
        {
            let _ = self.advance();
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position.min(self.document.line_count());
    }

    pub fn text(&self) -> &str {
        self.document.text()
    }
}

#[derive(Clone, Debug)]
pub struct LineCursor<'a> {
    document: &'a SourceDocument,
    line_index: usize,
}

impl<'a> LineCursor<'a> {
    pub fn new(document: &'a SourceDocument) -> Self {
        Self {
            document,
            line_index: 0,
        }
    }

    pub fn current(&self) -> Option<SourceLineRef<'a>> {
        self.document.line(self.line_index)
    }

    pub fn peek(&self, offset: usize) -> Option<SourceLineRef<'a>> {
        self.document.line(self.line_index + offset)
    }

    pub fn advance(&mut self) -> Option<SourceLineRef<'a>> {
        let current = self.current();
        if current.is_some() {
            self.line_index += 1;
        }
        current
    }

    pub fn is_eof(&self) -> bool {
        self.line_index >= self.document.line_count()
    }

    pub fn position(&self) -> usize {
        self.line_index
    }

    pub fn set_position(&mut self, position: usize) {
        self.line_index = position.min(self.document.line_count());
    }

    pub fn current_slice_from_column(&self, column: usize) -> Option<LineSlice<'a>> {
        self.current().map(|line| line.slice_from_column(column))
    }
}

impl SourceSpan {
    pub fn from_lines(start: SourceLineRef<'_>, end: SourceLineRef<'_>) -> Self {
        Self {
            start: start.start,
            end: end.end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_slices_track_columns_through_tabs() {
        let document = SourceDocument::new("  \tfoo\n".to_string());
        let line = document.line(0).expect("line exists");

        assert_eq!(line.indent_columns(), 4);
        assert_eq!(line.first_nonspace().text, "foo");
        assert_eq!(line.slice_from_column(4).text, "foo");
    }

    #[test]
    fn line_cursor_advances_monotonically() {
        let document = SourceDocument::new("a\nb\n".to_string());
        let mut cursor = LineCursor::new(&document);

        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.advance().expect("first").text, "a");
        assert_eq!(cursor.position(), 1);
        assert_eq!(cursor.advance().expect("second").text, "b");
        assert!(cursor.is_eof());
    }
}
