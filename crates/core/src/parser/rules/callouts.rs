//! Fenced and blockquote callout bodies and child parsing.
use super::*;

impl BlockDetector for CalloutRule {
    fn detect(&self, cursor: &SourceCursor<'_>, _context: &ParseContext) -> Option<ParseStart> {
        cursor.current().and_then(|line| {
            parse_callout_open(line.text).map(|_| ParseStart {
                kind: BlockStartKind::Callout,
            })
        })
    }
}

impl BlockParser for CalloutRule {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        _start: ParseStart,
        context: &mut ParseContext,
        next_id: &mut usize,
    ) -> ParseResult {
        let open = cursor
            .current()
            .expect("callout parser called on callout open line");
        let parsed = parse_callout_open(open.text).expect("callout opener must be prevalidated");
        let parsed_body = match parsed.syntax {
            CalloutSyntax::Fenced => parse_fenced_callout_body(cursor, open, context, next_id),
            CalloutSyntax::BlockQuote => parse_blockquote_callout_body(cursor, context, next_id),
        };
        let body_span = parsed_body.body_span;
        let block_span = SourceSpan::new(open.start, parsed_body.end);
        let mut attrs = BTreeMap::new();
        attrs.insert(
            "normalized_source".to_string(),
            parsed_body.normalized_source,
        );
        ParseResult {
            span: block_span,
            block: BlockNode {
                id: next_block_id(next_id),
                kind: BlockKind::Callout(CalloutNode {
                    kind: parsed.kind,
                    title: parsed.title,
                    body_span,
                }),
                span: block_span,
                attrs,
                source_mapping: crate::ir::BlockSourceMapping {
                    normalized: parsed_body.source_mapping,
                    inline: Vec::new(),
                },
                diagnostics: parsed_body.diagnostics,
                children: parsed_body.children,
            },
        }
    }
}

pub(super) fn parse_callout_open(line: &str) -> Option<CalloutOpen> {
    parse_fenced_callout_open(line).or_else(|| parse_blockquote_callout_open(line))
}

pub(super) fn parse_fenced_callout_open(line: &str) -> Option<CalloutOpen> {
    if leading_spaces(line) > 3 {
        return None;
    }
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix(":::")?;
    let mut parts = rest.splitn(2, char::is_whitespace);
    let kind = parts.next()?.trim().to_ascii_lowercase();
    if kind.is_empty() {
        return None;
    }
    if !kind
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return None;
    }

    let title = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    Some(CalloutOpen {
        kind,
        title,
        syntax: CalloutSyntax::Fenced,
    })
}

pub(super) fn parse_blockquote_callout_open(line: &str) -> Option<CalloutOpen> {
    let stripped = strip_blockquote_prefix_expanded(line)?;
    let trimmed = stripped.trim_start();
    let rest = trimmed.strip_prefix("[!")?;
    let close = rest.find(']')?;
    let raw_kind = &rest[..close];
    if !is_supported_blockquote_callout_kind(raw_kind) {
        return None;
    }
    let after_marker = &rest[close + 1..];
    let title = if let Some(after_colon) = after_marker.strip_prefix(':') {
        after_colon.trim()
    } else {
        after_marker.trim()
    };
    Some(CalloutOpen {
        kind: raw_kind.to_ascii_lowercase(),
        title: (!title.is_empty()).then(|| title.to_string()),
        syntax: CalloutSyntax::BlockQuote,
    })
}

pub(super) fn is_supported_blockquote_callout_kind(kind: &str) -> bool {
    matches!(
        kind.to_ascii_uppercase().as_str(),
        "NOTE" | "TIP" | "IMPORTANT" | "WARNING" | "CAUTION"
    )
}

pub(super) fn parse_fenced_callout_body(
    cursor: &mut SourceCursor<'_>,
    open: SourceLineRef<'_>,
    context: &ParseContext,
    next_id: &mut usize,
) -> ParsedCalloutBody {
    let _ = cursor.advance();
    let mut lines = Vec::new();
    let mut closing_line: Option<SourceLineRef<'_>> = None;
    while let Some(line) = cursor.current() {
        if line.text.trim() == ":::" {
            closing_line = Some(line);
            cursor.advance();
            break;
        }
        lines.push(line);
        cursor.advance();
    }

    let end = closing_line
        .map(|line| line.end)
        .or_else(|| lines.last().map(|line| line.end))
        .unwrap_or(open.end);
    let body_span = lines_to_span(&lines).unwrap_or_else(|| SourceSpan::new(open.end, open.end));
    let mapped = normalize::plain(&lines);
    let normalized_source = mapped.text;
    let mut diagnostics = Vec::new();
    if closing_line.is_none() {
        diagnostics.push(warning(
            "callout.unclosed",
            "Callout block was not closed before end-of-document.",
            SourceSpan::new(open.start, end),
            None,
        ));
    }
    let children = parse_callout_children(&normalized_source, context, next_id, &mut diagnostics);

    ParsedCalloutBody {
        end,
        body_span,
        normalized_source,
        source_mapping: mapped.mapping,
        diagnostics,
        children,
    }
}

pub(super) fn parse_blockquote_callout_body(
    cursor: &mut SourceCursor<'_>,
    context: &ParseContext,
    next_id: &mut usize,
) -> ParsedCalloutBody {
    let mut lines = Vec::new();
    while let Some(line) = cursor.current() {
        if !is_block_quote_start(line.text) {
            break;
        }
        lines.push(line);
        cursor.advance();
    }

    let body_lines = lines.get(1..).unwrap_or_default();
    let end = lines.last().map(|line| line.end).unwrap_or(0);
    let body_span = lines_to_span(body_lines).unwrap_or_else(|| {
        lines.first().map_or(SourceSpan::new(0, 0), |line| {
            SourceSpan::new(line.end, line.end)
        })
    });
    let mapped = normalize::quote(body_lines);
    let normalized_source = mapped.text;
    let mut diagnostics = Vec::new();
    let children = parse_callout_children(&normalized_source, context, next_id, &mut diagnostics);

    ParsedCalloutBody {
        end,
        body_span,
        normalized_source,
        source_mapping: mapped.mapping,
        diagnostics,
        children,
    }
}

pub(super) fn parse_callout_children(
    normalized_source: &str,
    context: &ParseContext,
    next_id: &mut usize,
    diagnostics: &mut Vec<RenderDiagnostic>,
) -> Vec<BlockNode> {
    if normalized_source.trim().is_empty() {
        return Vec::new();
    }
    let Some(child_context) = context.child() else {
        return Vec::new();
    };
    let child_doc =
        crate::parser::core::parse_document_with_context(normalized_source, &child_context);
    diagnostics.extend(child_doc.diagnostics);
    let mut children = child_doc.blocks;
    reassign_block_ids(&mut children, next_id);
    children
}
