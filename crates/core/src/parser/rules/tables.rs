//! GFM table delimiters, cell boundaries and alignments.
use super::*;

impl BlockDetector for GfmTableRule {
    fn detect(&self, cursor: &SourceCursor<'_>, context: &ParseContext) -> Option<ParseStart> {
        (context.features.gfm && is_gfm_table_start(cursor)).then_some(ParseStart {
            kind: BlockStartKind::GfmTable,
        })
    }
}

impl BlockParser for GfmTableRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let mut lines = Vec::new();
        while let Some(line) = cursor.current() {
            if lines.len() >= 2 && line.text.trim().is_empty() {
                break;
            }
            if lines.len() >= 2
                && (is_block_quote_start(line.text)
                    || parse_atx_heading(line.text).is_some()
                    || parse_fence_open(line.text).is_some()
                    || (context.features.math && is_math_block_delimiter(line.text))
                    || parse_list_marker(line.text).is_some()
                    || is_html_block_start(line.text))
            {
                break;
            }
            lines.push(line);
            let _ = cursor.advance();
        }
        let span = lines_to_span(&lines).unwrap_or_else(|| SourceSpan::new(0, 0));
        ParseResult {
            span,
            block: BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::GfmTable(parse_gfm_table_lines(&lines)),
                span,
                attrs: BTreeMap::new(),
                source_mapping: Default::default(),
                diagnostics: Vec::new(),
                children: Vec::new(),
            },
        }
    }
}

pub(super) fn is_gfm_table_start(cursor: &SourceCursor<'_>) -> bool {
    let Some(header) = cursor.current() else {
        return false;
    };
    let Some(delimiter) = cursor.peek(1) else {
        return false;
    };
    header.text.contains('|')
        && is_gfm_table_delimiter(delimiter.text)
        && split_table_cells(header.text).len() == table_alignments(delimiter.text).len()
}

pub(super) fn is_gfm_table_delimiter(line: &str) -> bool {
    let mut has_dash = false;
    for part in line.split('|') {
        let cell = part.trim().trim_matches(':');
        if cell.is_empty() {
            continue;
        }
        if !cell.chars().all(|ch| ch == '-') {
            return false;
        }
        if !cell.is_empty() {
            has_dash = true;
        }
    }
    has_dash
}

pub(super) fn parse_gfm_table_lines(lines: &[SourceLineRef<'_>]) -> GfmTableNode {
    let header = lines
        .first()
        .map_or_else(Vec::new, |line| table_cells(*line));
    let alignments = lines
        .get(1)
        .map_or_else(Vec::new, |line| table_alignments(line.text));
    let width = alignments.len().max(header.len());
    let header = normalize_table_cells(header, width);
    let rows = lines
        .iter()
        .skip(2)
        .map(|line| TableRowNode {
            cells: normalize_table_cells(table_cells(*line), width),
        })
        .collect();

    GfmTableNode {
        alignments,
        header,
        rows,
    }
}

pub(super) fn table_cells(line: SourceLineRef<'_>) -> Vec<TableCellNode> {
    normalize::table_cells(line)
        .into_iter()
        .map(|mapped| TableCellNode {
            source: mapped.text,
            source_mapping: mapped.mapping,
            children: Vec::new(),
        })
        .collect()
}

pub(super) fn normalize_table_cells(
    mut cells: Vec<TableCellNode>,
    width: usize,
) -> Vec<TableCellNode> {
    cells.truncate(width);
    while cells.len() < width {
        cells.push(TableCellNode {
            source_mapping: Vec::new(),
            source: String::new(),
            children: Vec::new(),
        });
    }
    cells
}

pub(super) fn split_table_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or_else(|| trimmed.strip_prefix('|').unwrap_or(trimmed));
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek().is_some_and(|next| *next == '|') {
            current.push('|');
            let _ = chars.next();
        } else if ch == '|' {
            cells.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }
    cells.push(current.trim().to_string());
    cells
}

pub(super) fn table_alignments(line: &str) -> Vec<TableAlignment> {
    split_table_cells(line)
        .into_iter()
        .filter_map(|cell| {
            let trimmed = cell.trim();
            let core = trimmed.trim_matches(':');
            if core.is_empty() || !core.chars().all(|ch| ch == '-') {
                return None;
            }
            Some(match (trimmed.starts_with(':'), trimmed.ends_with(':')) {
                (true, true) => TableAlignment::Center,
                (true, false) => TableAlignment::Left,
                (false, true) => TableAlignment::Right,
                (false, false) => TableAlignment::None,
            })
        })
        .collect()
}
