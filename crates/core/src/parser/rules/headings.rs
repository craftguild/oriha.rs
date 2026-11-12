//! ATX and Setext heading recognition and parsing.
use super::*;

impl BlockDetector for HeadingRule {
    fn detect(&self, cursor: &SourceCursor<'_>, _context: &ParseContext) -> Option<ParseStart> {
        let line = cursor.current()?;
        parse_atx_heading(line.text).map(|_| ParseStart {
            kind: BlockStartKind::Heading,
        })
    }
}

impl BlockParser for HeadingRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        _context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let line = cursor
            .advance()
            .expect("heading parser must be called at a heading line");
        let parsed = parse_atx_heading(line.text).expect("heading line was pre-detected");
        let block_span = SourceSpan::new(line.start, line.end);
        let inline_span = SourceSpan::new(
            line.start + parsed.inline_start,
            line.start + parsed.inline_end,
        );

        let node = BlockNode {
            id: next_block_id(next_id),
            kind: BlockKind::Heading(HeadingNode {
                level: parsed.level,
                inline_span,
                children: Vec::new(),
            }),
            span: block_span,
            attrs: BTreeMap::new(),
            source_mapping: Default::default(),
            diagnostics: Vec::new(),
            children: Vec::new(),
        };
        ParseResult {
            block: node,
            span: block_span,
        }
    }
}

pub(super) fn parse_atx_heading(line: &str) -> Option<AtxHeading> {
    let bytes = line.as_bytes();
    let leading = bytes
        .iter()
        .take(4)
        .take_while(|ch| **ch == b' ' || **ch == b'\t')
        .count();
    if leading >= 4 {
        return None;
    }

    let mut idx = leading;
    let mut level = 0u8;
    while idx < bytes.len() && level < 6 {
        if bytes[idx] != b'#' {
            break;
        }
        idx += 1;
        level += 1;
    }

    if level == 0 {
        return None;
    }

    if idx >= bytes.len() {
        return Some(AtxHeading {
            level,
            inline_start: idx,
            inline_end: idx,
        });
    }

    if !bytes[idx].is_ascii_whitespace() {
        return None;
    }

    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }

    let mut inline = &line[idx..];
    let inline_start = idx;
    inline = inline.trim_end();
    if inline.is_empty() {
        return Some(AtxHeading {
            level,
            inline_start,
            inline_end: inline_start,
        });
    }

    let inline_end = inline_start + inline.len();
    let mut inline = inline.to_string();
    if let Some(last_hash) = inline.rfind(" #") {
        let candidate = &inline[last_hash + 2..];
        if !candidate.is_empty() && candidate.chars().all(|ch| ch == '#') {
            inline.truncate(last_hash);
            let trimmed = inline.trim_end().len();
            return Some(AtxHeading {
                level,
                inline_start,
                inline_end: inline_start + trimmed,
            });
        }
    }

    if inline.chars().all(|ch| ch == '#') {
        return Some(AtxHeading {
            level,
            inline_start,
            inline_end: inline_start,
        });
    }

    Some(AtxHeading {
        level,
        inline_start,
        inline_end,
    })
}

impl BlockDetector for SetextHeadingRule {
    fn detect(&self, cursor: &SourceCursor<'_>, context: &ParseContext) -> Option<ParseStart> {
        cursor
            .current()
            .is_some_and(|line| is_setext_heading_start(cursor, &line, context.features.math))
            .then_some(ParseStart {
                kind: BlockStartKind::SetextHeading,
            })
    }
}

impl BlockParser for SetextHeadingRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        _context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let mut lines = Vec::new();
        while let Some(line) = cursor.current() {
            lines.push(line);
            let _ = cursor.advance();
            if is_setext_underline(line.text) {
                break;
            }
        }
        let span = lines_to_span(&lines).unwrap_or_else(|| SourceSpan::new(0, 0));
        let level = lines
            .last()
            .and_then(|line| parse_setext_heading_level(line.text))
            .unwrap_or(1);
        let inline_span = match (lines.first(), lines.get(lines.len().saturating_sub(2))) {
            (Some(first), Some(last_content)) => SourceSpan::new(first.start, last_content.end),
            _ => SourceSpan::new(span.start, span.start),
        };
        let mapped = normalize::setext(&lines);
        let mut attrs = BTreeMap::new();
        attrs.insert("inline_source".to_string(), mapped.text);
        ParseResult {
            span,
            block: BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::Heading(HeadingNode {
                    level,
                    inline_span,
                    children: Vec::new(),
                }),
                span,
                attrs,
                source_mapping: crate::ir::BlockSourceMapping {
                    normalized: Vec::new(),
                    inline: mapped.mapping,
                },
                diagnostics: Vec::new(),
                children: Vec::new(),
            },
        }
    }
}

pub(super) fn is_setext_heading_start(
    cursor: &SourceCursor<'_>,
    current: &SourceLineRef<'_>,
    math_enabled: bool,
) -> bool {
    if current.text.trim().is_empty() {
        return false;
    }
    if starts_non_paragraph_block(current.text, math_enabled) {
        return false;
    }
    let mut offset = 1usize;
    while let Some(line) = cursor.peek(offset) {
        if line.text.trim().is_empty() {
            return false;
        }
        if is_setext_underline(line.text) {
            return true;
        }
        if interrupts_paragraph_continuation(line.text, math_enabled) {
            return false;
        }
        offset += 1;
    }
    false
}

pub(super) fn is_setext_underline(line: &str) -> bool {
    if leading_spaces(line) > 3 {
        return false;
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    trimmed.chars().all(|ch| ch == '=' || ch == '-')
        && trimmed
            .chars()
            .filter(|ch| ch.is_ascii_alphabetic())
            .count()
            == 0
}

pub(super) fn parse_setext_heading_level(line: &str) -> Option<u8> {
    let trimmed = line.trim();
    if trimmed.chars().all(|ch| ch == '=') {
        Some(1)
    } else if trimmed.chars().all(|ch| ch == '-') {
        Some(2)
    } else {
        None
    }
}
