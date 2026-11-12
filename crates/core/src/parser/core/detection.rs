//! Leaf-block detection, fallback paragraphs, source columns and limit diagnostics.
use super::*;

pub(super) fn fence_open(line: &str) -> Option<(char, usize)> {
    if leading_columns(line) > 3 {
        return None;
    }
    let trimmed = line.trim_start();
    let marker = trimmed.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let len = trimmed.chars().take_while(|ch| *ch == marker).count();
    (len >= 3).then_some((marker, len))
}

pub(super) fn is_fence_close(line: &str, marker: char, min_len: usize) -> bool {
    if leading_columns(line) > 3 {
        return false;
    }
    let trimmed = line.trim_start();
    let len = trimmed.chars().take_while(|ch| *ch == marker).count();
    len >= min_len && trimmed[len..].trim().is_empty()
}

pub(super) fn strip_blockquote_prefix(line: &str) -> Option<&str> {
    if leading_columns(line) > 3 {
        return None;
    }
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix('>')?;
    Some(rest.strip_prefix([' ', '\t']).unwrap_or(rest))
}

pub(super) fn starts_leaf_block(trimmed: &str) -> bool {
    trimmed.starts_with('#')
        || trimmed.starts_with('>')
        || trimmed.starts_with('<')
        || trimmed.starts_with("```")
        || trimmed.starts_with("~~~")
        || thematic_break(trimmed)
        || list_marker(trimmed)
}

pub(super) fn list_marker(trimmed: &str) -> bool {
    let bytes = trimmed.as_bytes();
    if bytes.len() >= 2 && matches!(bytes[0], b'-' | b'+' | b'*') && bytes[1].is_ascii_whitespace()
    {
        return true;
    }
    let digits = bytes
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    digits > 0
        && digits < bytes.len()
        && matches!(bytes[digits], b'.' | b')')
        && bytes.get(digits + 1).is_some_and(u8::is_ascii_whitespace)
}

pub(super) fn thematic_break(line: &str) -> bool {
    let mut marker = None;
    let mut count = 0usize;
    for ch in line.trim().chars() {
        if matches!(ch, '-' | '_' | '*') {
            if marker.is_some_and(|current| current != ch) {
                return false;
            }
            marker = Some(ch);
            count += 1;
        } else if !ch.is_ascii_whitespace() {
            return false;
        }
    }
    count >= 3
}

pub(super) fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|ch| *ch == ' ').count()
}

pub(super) fn leading_columns(line: &str) -> usize {
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

pub(super) fn detect_next_block(
    cursor: &SourceCursor<'_>,
    rules: &[Box<dyn BlockRuleParser>],
    context: &ParseContext,
) -> Option<(usize, ParseStart)> {
    for (index, rule) in rules.iter().enumerate() {
        let Some(start) = rule.detect(cursor, context) else {
            continue;
        };
        return Some((index, start));
    }
    None
}

pub(super) fn parse_detected_block(
    cursor: &mut SourceCursor<'_>,
    rules: &[Box<dyn BlockRuleParser>],
    rule_index: usize,
    start: ParseStart,
    context: &mut ParseContext,
    next_id: &mut usize,
) -> BlockNode {
    let current_position = cursor.position();
    let result = rules[rule_index].parse(cursor, start, context, next_id);
    if cursor.position() == current_position {
        let _ = cursor.advance();
    }
    result.block
}

pub(super) fn parse_fallback_paragraph(
    cursor: &mut SourceCursor<'_>,
    next_id: &mut usize,
) -> BlockNode {
    let span = cursor.advance().map_or(SourceSpan::new(0, 0), |line| {
        SourceSpan::new(line.start, line.end)
    });
    BlockNode {
        id: next_block_id(next_id),
        kind: BlockKind::Paragraph(ParagraphNode {
            children: Vec::new(),
        }),
        span,
        attrs: BTreeMap::new(),
        source_mapping: Default::default(),
        diagnostics: Vec::new(),
        children: Vec::new(),
    }
}

pub(super) fn limit_diagnostic(
    source: &SourceDocument,
    code: &str,
    message: &str,
    span: SourceSpan,
    block_id: Option<String>,
) -> RenderDiagnostic {
    RenderDiagnostic {
        severity: Severity::Warning,
        code: code.to_string(),
        message: message.to_string(),
        source_range: Some(source.to_source_range(span)),
        block_id,
        engine: DiagnosticEngine::Core,
    }
}

pub(super) fn next_block_id(next_id: &mut usize) -> String {
    let id = next_id.to_string();
    *next_id += 1;
    id
}
