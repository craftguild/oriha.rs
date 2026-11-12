//! Blockquote nesting, lazy continuation and source rebasing.
use super::*;

impl BlockDetector for BlockQuoteRule {
    fn detect(&self, cursor: &SourceCursor<'_>, _context: &ParseContext) -> Option<ParseStart> {
        cursor.current().and_then(|line| {
            if is_block_quote_start(line.text) {
                Some(ParseStart {
                    kind: BlockStartKind::BlockQuote,
                })
            } else {
                None
            }
        })
    }
}

impl BlockParser for BlockQuoteRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let mut lines = Vec::new();
        let mut can_lazy_continue = false;
        let mut saw_lazy_setext_underline = false;
        while let Some(line) = cursor.current() {
            if is_block_quote_start(line.text) {
                can_lazy_continue = deepest_lazy_paragraph_start(line.text, context.features.math);
                lines.push(line);
                cursor.advance();
                continue;
            }
            if lines.is_empty() {
                break;
            }
            if can_lazy_continue
                && !line.text.trim().is_empty()
                && (!starts_non_paragraph_block(line.text, context.features.math)
                    || is_indented_code_start(line.text))
            {
                // Lazy continuation inside blockquote paragraphs.
                if is_setext_underline(line.text) {
                    saw_lazy_setext_underline = true;
                }
                lines.push(line);
                cursor.advance();
                continue;
            }
            break;
        }

        let start = lines.first().map(|line| line.start).unwrap_or(0);
        let end = lines.last().map(|line| line.end).unwrap_or(start);
        let block_span = SourceSpan::new(start, end);

        let mut children = Vec::new();
        let mut diagnostics = Vec::new();
        let mapped = normalize::quote(&lines);
        let normalized = mapped.text;
        if !normalized.trim().is_empty() {
            if let Some(child_context) = context.child() {
                let child_doc =
                    crate::parser::core::parse_document_with_context(&normalized, &child_context);
                diagnostics.extend(child_doc.diagnostics);
                children = child_doc.blocks;
                if saw_lazy_setext_underline {
                    suppress_lazy_setext_headings(&mut children, &normalized);
                }
                reassign_block_ids(&mut children, next_id);
            }
        }
        let mut attrs = BTreeMap::new();
        attrs.insert("normalized_source".to_string(), normalized);

        ParseResult {
            span: block_span,
            block: BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::BlockQuote,
                span: block_span,
                attrs,
                source_mapping: crate::ir::BlockSourceMapping {
                    normalized: mapped.mapping,
                    inline: Vec::new(),
                },
                diagnostics,
                children,
            },
        }
    }
}

pub(super) fn suppress_lazy_setext_headings(blocks: &mut [BlockNode], normalized: &str) {
    for block in blocks {
        if matches!(block.kind, BlockKind::Heading(_)) {
            block.kind = BlockKind::Paragraph(ParagraphNode {
                children: Vec::new(),
            });
            block.source_mapping.inline =
                crate::mapping::MappedText::identity(normalized, 0).mapping;
            block
                .attrs
                .insert("inline_source".to_string(), normalized.to_string());
        }
        if !block.children.is_empty() {
            suppress_lazy_setext_headings(&mut block.children, normalized);
        }
    }
}

pub(super) fn deepest_lazy_paragraph_start(line: &str, math_enabled: bool) -> bool {
    let mut current = strip_blockquote_prefix_expanded(line).unwrap_or_default();
    loop {
        while let Some(stripped) = strip_blockquote_prefix_expanded(&current) {
            current = stripped;
        }
        let Some(marker) = parse_list_marker(&current) else {
            break;
        };
        let content = current
            .get(marker.content_byte_start..)
            .unwrap_or_default()
            .to_string();
        current = content;
    }
    !current.trim().is_empty() && !starts_non_paragraph_block(&current, math_enabled)
}

pub(super) fn is_block_quote_start(line: &str) -> bool {
    if leading_spaces(line) > 3 {
        return false;
    }
    line.trim_start().starts_with('>')
}

pub(super) fn strip_blockquote_prefix_expanded(line: &str) -> Option<String> {
    if leading_spaces(line) > 3 {
        return None;
    }
    let (marker_byte, marker_column) = first_nonspace_index_and_column(line);
    let tail = &line[marker_byte..];
    if !tail.starts_with('>') {
        return None;
    }
    let after_marker_byte = marker_byte + 1;
    let after_marker_column = marker_column + 1;
    let rest = &line[after_marker_byte..];
    let Some(first) = rest.chars().next() else {
        return Some(String::new());
    };
    if first != ' ' && first != '\t' {
        return Some(rest.to_string());
    }

    let next_column = if first == '\t' {
        after_marker_column + 4 - (after_marker_column % 4)
    } else {
        after_marker_column + 1
    };
    let mut out = " ".repeat(next_column.saturating_sub(after_marker_column + 1));
    append_rebased_leading_whitespace(&mut out, &rest[first.len_utf8()..], next_column);
    Some(out)
}

pub(super) fn append_rebased_leading_whitespace(
    out: &mut String,
    rest: &str,
    mut source_column: usize,
) {
    let mut body_start = 0usize;
    for (idx, ch) in rest.char_indices() {
        let next_column = match ch {
            ' ' => source_column + 1,
            '\t' => source_column + 4 - (source_column % 4),
            _ => {
                body_start = idx;
                break;
            }
        };
        out.push_str(&" ".repeat(next_column - source_column));
        source_column = next_column;
        body_start = idx + ch.len_utf8();
    }
    out.push_str(&rest[body_start..]);
}
