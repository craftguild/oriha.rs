//! Paragraph assembly and thematic-break recognition.
use super::*;

impl BlockDetector for ParagraphRule {
    fn detect(&self, cursor: &SourceCursor<'_>, context: &ParseContext) -> Option<ParseStart> {
        cursor
            .current()
            .is_some_and(|line| is_paragraph_line(line.text, context.features.math))
            .then_some(ParseStart {
                kind: BlockStartKind::Paragraph,
            })
    }
}

impl BlockParser for ParagraphRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let mut lines = Vec::new();
        while let Some(line) = cursor.current() {
            if line.text.trim().is_empty() {
                break;
            }
            if lines.is_empty() {
                if starts_non_paragraph_block(line.text, context.features.math) {
                    break;
                }
            } else if interrupts_paragraph_continuation(line.text, context.features.math) {
                break;
            }
            lines.push(line);
            let _ = cursor.advance();
        }
        let span = lines_to_span(&lines).unwrap_or_else(|| {
            let line = cursor
                .advance()
                .expect("paragraph parser must consume one line");
            SourceSpan::new(line.start, line.end)
        });
        ParseResult {
            span,
            block: BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::Paragraph(ParagraphNode {
                    children: Vec::new(),
                }),
                span,
                attrs: BTreeMap::new(),
                source_mapping: Default::default(),
                diagnostics: Vec::new(),
                children: Vec::new(),
            },
        }
    }
}

pub(super) fn is_thematic_break(line: &str) -> bool {
    if leading_spaces(line) > 3 {
        return false;
    }
    let mut marker = None;
    let mut marker_count = 0usize;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }
    for ch in trimmed.chars() {
        if ch == '-' || ch == '_' || ch == '*' {
            if marker.is_some_and(|current| current != ch) {
                return false;
            }
            marker = Some(ch);
            marker_count += 1;
        } else if !ch.is_ascii_whitespace() {
            return false;
        }
    }
    marker_count >= 3
}
