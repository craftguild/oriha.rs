//! Fenced code, math delimiters and indented-code normalization.
use super::*;

impl BlockDetector for FencedCodeRule {
    fn detect(&self, cursor: &SourceCursor<'_>, _context: &ParseContext) -> Option<ParseStart> {
        cursor.current().and_then(|line| {
            parse_fence_open(line.text).map(|_| ParseStart {
                kind: BlockStartKind::FencedCode,
            })
        })
    }
}

impl BlockParser for FencedCodeRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        _context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let open_line = cursor
            .advance()
            .expect("fenced parser called on fence open line");
        let open = parse_fence_open(open_line.text).expect("fence open must be prevalidated");

        let mut closing: Option<SourceLineRef<'_>> = None;
        let mut body_end = open_line.end;
        while let Some(line) = cursor.current() {
            if is_fence_close(line.text, open.char, open.len) {
                closing = Some(line);
                break;
            }
            body_end = line.end;
            cursor.advance();
        }

        let mut diagnostics = Vec::new();
        let block_end;
        let body_span;
        let body_start = skip_line_break(cursor.text(), open_line.end);
        if let Some(close_line) = closing {
            block_end = close_line.end;
            body_span = SourceSpan::new(body_start, close_line.start);
            cursor.advance();
        } else {
            block_end = body_end;
            body_span = SourceSpan::new(body_start, body_end);
            diagnostics.push(RenderDiagnostic {
                severity: Severity::Warning,
                code: "fenced_code_block.unclosed".to_string(),
                message: "Fenced code block was not closed before end-of-document.".to_string(),
                source_range: None,
                block_id: None,
                engine: DiagnosticEngine::Core,
            });
        }

        let language = open.info.split_whitespace().next().map(ToOwned::to_owned);
        let kind = CodeBlockNode {
            language,
            info: open.info,
            body_span,
            closed: closing.is_some(),
            indent: open.indent,
        };
        let block_span = SourceSpan::new(open_line.start, block_end);

        ParseResult {
            span: block_span,
            block: BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::FencedCode(kind),
                span: block_span,
                attrs: BTreeMap::new(),
                source_mapping: Default::default(),
                diagnostics,
                children: Vec::new(),
            },
        }
    }
}

impl BlockDetector for MathBlockRule {
    fn detect(&self, cursor: &SourceCursor<'_>, context: &ParseContext) -> Option<ParseStart> {
        (context.features.math
            && cursor
                .current()
                .is_some_and(|line| is_math_block_delimiter(line.text)))
        .then_some(ParseStart {
            kind: BlockStartKind::MathBlock,
        })
    }
}

impl BlockParser for MathBlockRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        _context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let open_line = cursor
            .advance()
            .expect("math block parser called on opening delimiter");
        let body_start = skip_line_break(cursor.text(), open_line.end);
        let mut closing: Option<SourceLineRef<'_>> = None;
        let mut body_end = open_line.end;

        while let Some(line) = cursor.current() {
            if is_math_block_delimiter(line.text) {
                closing = Some(line);
                break;
            }
            body_end = line.end;
            cursor.advance();
        }

        let (body_span, block_end, closed) = if let Some(close_line) = closing {
            let span = SourceSpan::new(body_start, close_line.start);
            cursor.advance();
            (span, close_line.end, true)
        } else {
            (SourceSpan::new(body_start, body_end), body_end, false)
        };
        let source = cursor.text()[body_span.start..body_span.end].to_string();
        let block_span = SourceSpan::new(open_line.start, block_end);

        ParseResult {
            span: block_span,
            block: BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::MathBlock(MathBlockNode {
                    source_span: body_span,
                    source: Some(source),
                    display: true,
                    closed,
                }),
                span: block_span,
                attrs: BTreeMap::new(),
                source_mapping: Default::default(),
                diagnostics: Vec::new(),
                children: Vec::new(),
            },
        }
    }
}

pub(super) fn parse_fence_open(line: &str) -> Option<FenceOpen> {
    if leading_spaces(line) > 3 {
        return None;
    }
    let trimmed = line.trim_start();
    let indent = line.len().saturating_sub(trimmed.len());
    if trimmed.is_empty() {
        return None;
    }

    let mut chars = trimmed.chars();
    let marker = chars.next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let mut len = 1usize;
    for ch in chars {
        if ch == marker {
            len += 1;
        } else {
            break;
        }
    }
    if len < 3 {
        return None;
    }

    let info = trimmed[len..].trim().to_string();
    if marker == '`' && info.contains('`') {
        return None;
    }
    Some(FenceOpen {
        char: marker,
        len,
        info,
        indent,
    })
}

pub(super) fn is_fence_close(line: &str, fence: char, min_len: usize) -> bool {
    if leading_spaces(line) > 3 {
        return false;
    }
    let trimmed = line.trim_start();
    let mut count = 0usize;
    for ch in trimmed.chars() {
        if ch != fence {
            break;
        }
        count += 1;
    }
    count >= min_len && trimmed[count..].trim().is_empty()
}

pub(super) fn is_math_block_delimiter(line: &str) -> bool {
    leading_spaces(line) <= 3 && line.trim() == "$$"
}

impl BlockDetector for IndentedCodeRule {
    fn detect(&self, cursor: &SourceCursor<'_>, _context: &ParseContext) -> Option<ParseStart> {
        cursor
            .current()
            .is_some_and(|line| is_indented_code_start(line.text))
            .then_some(ParseStart {
                kind: BlockStartKind::IndentedCode,
            })
    }
}

impl BlockParser for IndentedCodeRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        _context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let mut lines = Vec::new();
        while let Some(line) = cursor.current() {
            if !(line.text.trim().is_empty() || is_indented_code_start(line.text)) {
                break;
            }
            lines.push(line);
            let _ = cursor.advance();
        }
        let span = lines_to_span(&lines).unwrap_or_else(|| SourceSpan::new(0, 0));
        let mut attrs = BTreeMap::new();
        attrs.insert(
            "normalized_code".to_string(),
            normalize_indented_code_lines(&lines),
        );
        ParseResult {
            span,
            block: BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::IndentedCode(CodeBlockNode {
                    language: None,
                    info: String::new(),
                    body_span: span,
                    closed: true,
                    indent: 4,
                }),
                span,
                attrs,
                source_mapping: Default::default(),
                diagnostics: Vec::new(),
                children: Vec::new(),
            },
        }
    }
}

pub(super) fn is_indented_code_start(line: &str) -> bool {
    leading_spaces(line) >= 4
}

pub(super) fn normalize_indented_code_lines(lines: &[SourceLineRef<'_>]) -> String {
    let mut start = 0usize;
    while start < lines.len() && lines[start].text.trim().is_empty() {
        start += 1;
    }
    let mut end = lines.len();
    while end > start && lines[end - 1].text.trim().is_empty() {
        end -= 1;
    }
    let mut code = lines[start..end]
        .iter()
        .map(|line| {
            if line.text.trim().is_empty() {
                if is_indented_code_start(line.text) {
                    strip_indented_code_prefix(line.text).unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                strip_indented_code_prefix(line.text).unwrap_or_else(|| line.text.to_string())
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if !code.ends_with('\n') {
        code.push('\n');
    }
    code
}

pub(super) fn strip_indented_code_prefix(line: &str) -> Option<String> {
    let mut columns = 0usize;
    for (idx, ch) in line.char_indices() {
        let next = match ch {
            ' ' => columns + 1,
            '\t' => columns + 4 - (columns % 4),
            _ => return None,
        };
        if next >= 4 {
            let mut out = " ".repeat(next - 4);
            out.push_str(&line[idx + ch.len_utf8()..]);
            return Some(out);
        }
        columns = next;
    }
    None
}
