//! List block assembly and direct-item tightness decisions.
use super::*;

impl BlockDetector for ListRule {
    fn detect(&self, cursor: &SourceCursor<'_>, _context: &ParseContext) -> Option<ParseStart> {
        cursor.current().and_then(|line| {
            if parse_list_marker(line.text).is_some() {
                Some(ParseStart {
                    kind: BlockStartKind::List,
                })
            } else {
                None
            }
        })
    }
}

impl BlockParser for ListRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let marker = parse_list_marker(cursor.current().map_or("", |line| line.text));
        if marker.is_none() {
            let block = parse_heuristic_line(cursor, next_id);
            return ParseResult {
                span: block.span,
                block,
            };
        }
        let list_kind = marker.expect("list marker already parsed");
        let (list_block, seen_any) = parse_list_at_indent(
            cursor,
            context,
            next_id,
            list_kind.indent,
            list_kind.ordered,
            list_kind.raw_marker.clone(),
        );
        if !seen_any {
            return ParseResult {
                span: SourceSpan::new(0, 0),
                block: parse_heuristic_line(cursor, next_id),
            };
        }
        ParseResult {
            span: list_block.span,
            block: list_block,
        }
    }
}

pub(super) fn parse_list_at_indent(
    cursor: &mut SourceCursor<'_>,
    context: &mut ParseContext,
    next_id: &mut usize,
    base_indent: usize,
    ordered: bool,
    marker_text: String,
) -> (BlockNode, bool) {
    let mut children: Vec<BlockNode> = Vec::new();
    let mut list_start = 0usize;
    let mut list_end = 0usize;
    let mut seen_any = false;
    let mut last_content_indent = base_indent + 2;
    let mut list_loose = false;

    while let Some(line) = cursor.current() {
        if seen_any
            && marker_indent_columns(line.text) == base_indent
            && is_thematic_break(line.text)
        {
            break;
        }
        let Some(marker) = parse_list_marker(line.text) else {
            break;
        };
        if marker.indent < base_indent {
            break;
        }
        if !same_list_marker_type(&marker, ordered, &marker_text) {
            break;
        }
        if marker.indent > base_indent && marker.indent >= last_content_indent {
            if let Some(last_item) = children.last_mut() {
                let Some(mut child_context) = context.child() else {
                    while cursor.current().is_some_and(|line| {
                        line.text.trim().is_empty()
                            || marker_indent_columns(line.text) >= marker.indent
                    }) {
                        list_end = cursor.advance().unwrap().end;
                    }
                    continue;
                };
                let (nested, nested_seen) = parse_list_at_indent(
                    cursor,
                    &mut child_context,
                    next_id,
                    marker.indent,
                    marker.ordered,
                    marker.raw_marker.clone(),
                );
                if nested_seen {
                    list_end = nested.span.end;
                    last_item.children.push(nested);
                    continue;
                }
            }
            break;
        }
        if marker.ordered != ordered {
            break;
        }

        let item_start = line.start;
        let mut item_end = line.end;
        let mut content_end = line.end;
        let raw_prefix = marker.raw_marker.clone();
        let continuation_indent = marker.content_column;
        last_content_indent = continuation_indent;
        let content_start = line.start + marker.content_byte_start;
        let _ = cursor.advance();
        if !seen_any {
            list_start = item_start;
        }
        seen_any = true;

        let mut item_lines = vec![line];
        let mut item_blank_seen = false;
        let mut item_fence: Option<(char, usize)> = line
            .text
            .get(marker.content_byte_start..)
            .and_then(parse_fence_open)
            .map(|open| (open.char, open.len));
        let mut has_underindented_lazy_list_marker = false;
        while let Some(next) = cursor.current() {
            if let Some((fence_char, fence_len)) = item_fence {
                item_end = next.end;
                content_end = next.end;
                item_lines.push(next);
                let _ = cursor.advance();
                if is_fence_close(next.text, fence_char, fence_len) {
                    item_fence = None;
                }
                continue;
            }
            let maybe_next_marker = parse_list_marker(next.text);
            if let Some(next_marker) = maybe_next_marker {
                if next_marker.indent <= base_indent || next_marker.indent < continuation_indent {
                    break;
                }
                item_end = next.end;
                content_end = next.end;
                item_lines.push(next);
                let _ = cursor.advance();
                continue;
            }
            if next.text.trim().is_empty() {
                if item_fence.is_none() {
                    item_blank_seen = true;
                }
                item_end = next.end;
                item_lines.push(next);
                let _ = cursor.advance();
                continue;
            }
            let indent = marker_indent_columns(next.text);
            let lazy_continuation = !item_blank_seen
                && content_end > content_start
                && (!starts_non_paragraph_block(next.text, context.features.math)
                    || marker_indent_columns(next.text) >= 4);
            if item_blank_seen && content_start >= content_end {
                break;
            }
            if indent >= continuation_indent || lazy_continuation {
                if lazy_continuation && indent < continuation_indent {
                    has_underindented_lazy_list_marker = has_underindented_lazy_list_marker
                        || parse_list_marker(next.text.trim_start()).is_some();
                }
                item_end = next.end;
                content_end = next.end;
                item_lines.push(next);
                let _ = cursor.advance();
                if item_fence.is_none() {
                    if let Some(open) = parse_fence_open(next.text) {
                        item_fence = Some((open.char, open.len));
                    }
                }
                continue;
            }
            break;
        }

        list_end = item_end;
        if let Some(child_context) = if !line.text[marker.content_byte_start..].trim().is_empty()
            || item_lines
                .iter()
                .skip(1)
                .any(|line| !line.text.trim().is_empty())
        {
            context.child()
        } else {
            Some(context.clone())
        } {
            let mut item_children = Vec::new();
            let had_trailing_blank = item_lines
                .last()
                .is_some_and(|line| line.text.trim().is_empty());
            while item_lines
                .last()
                .is_some_and(|line| line.text.trim().is_empty())
            {
                item_lines.pop();
            }
            item_blank_seen = has_direct_list_item_blank(
                &item_lines,
                marker.content_byte_start,
                continuation_indent,
            );
            if item_blank_seen
                || (had_trailing_blank
                    && cursor
                        .current()
                        .and_then(|line| parse_list_marker(line.text))
                        .is_some_and(|next_marker| {
                            next_marker.indent >= base_indent
                                && same_list_marker_type(&next_marker, ordered, &marker_text)
                        }))
            {
                list_loose = true;
            }
            let mut attrs = BTreeMap::new();
            let mapped =
                normalize::list(&item_lines, marker.content_byte_start, continuation_indent);
            let normalized = mapped.text;
            let task = if context.features.gfm {
                normalized
                    .lines()
                    .find(|line| !line.trim().is_empty())
                    .and_then(detect_task_list_marker)
            } else {
                None
            };
            let mut diagnostics = Vec::new();
            if !normalized.trim().is_empty() {
                let child_doc =
                    crate::parser::core::parse_document_with_context(&normalized, &child_context);
                diagnostics.extend(child_doc.diagnostics);
                let mut parsed_children = child_doc.blocks;
                if has_underindented_lazy_list_marker {
                    merge_lazy_continuation_blocks(&mut parsed_children, &normalized);
                }
                reassign_block_ids(&mut parsed_children, next_id);
                for child in &mut parsed_children {
                    if !item_blank_seen && matches!(child.kind, BlockKind::Paragraph(_)) {
                        child
                            .attrs
                            .insert("tight_list_item".to_string(), "true".to_string());
                    }
                }
                item_children = parsed_children;
                attrs.insert("normalized_source".to_string(), normalized);
            }
            children.push(BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::ListItem(ListItemNode {
                    raw_prefix,
                    content_span: SourceSpan::new(content_start, content_end.max(content_start)),
                    task,
                }),
                span: SourceSpan::new(item_start, item_end),
                attrs,
                source_mapping: crate::ir::BlockSourceMapping {
                    normalized: mapped.mapping,
                    inline: Vec::new(),
                },
                diagnostics,
                children: item_children,
            });
        }
    }

    let span = SourceSpan::new(list_start, list_end.max(list_start));
    if list_loose {
        remove_direct_list_item_tight_attrs(&mut children);
    }

    (
        BlockNode {
            id: next_block_id(next_id),
            kind: BlockKind::List(ListNode {
                ordered,
                loose: list_loose,
                marker: marker_text,
            }),
            span,
            attrs: BTreeMap::new(),
            source_mapping: Default::default(),
            diagnostics: Vec::new(),
            children,
        },
        seen_any,
    )
}

pub(super) fn remove_direct_list_item_tight_attrs(items: &mut [BlockNode]) {
    for item in items {
        if !matches!(item.kind, BlockKind::ListItem(_)) {
            continue;
        }
        for child in &mut item.children {
            if matches!(child.kind, BlockKind::Paragraph(_)) {
                child.attrs.remove("tight_list_item");
            }
        }
    }
}

pub(super) fn has_direct_list_item_blank(
    lines: &[SourceLineRef<'_>],
    first_content_byte_start: usize,
    continuation_indent: usize,
) -> bool {
    let mut fence: Option<(char, usize)> = lines
        .first()
        .and_then(|line| line.text.get(first_content_byte_start..))
        .and_then(parse_fence_open)
        .map(|open| (open.char, open.len));
    for (idx, line) in lines.iter().enumerate().skip(1) {
        let logical = strip_columns_expanding_indentation(line.text, continuation_indent);
        if let Some((marker, len)) = fence {
            if is_fence_close(&logical, marker, len) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = parse_fence_open(&logical) {
            fence = Some((open.char, open.len));
        }
        if !line.text.trim().is_empty() {
            continue;
        }
        let previous_indent = lines[..idx]
            .iter()
            .rev()
            .find(|candidate| !candidate.text.trim().is_empty())
            .map(|candidate| marker_indent_columns(candidate.text))
            .unwrap_or(continuation_indent);
        let next_indent = lines
            .iter()
            .skip(idx + 1)
            .find(|candidate| !candidate.text.trim().is_empty())
            .map(|candidate| marker_indent_columns(candidate.text));
        if previous_indent < continuation_indent
            || next_indent.is_none_or(|indent| indent <= continuation_indent)
        {
            return true;
        }
    }
    false
}
