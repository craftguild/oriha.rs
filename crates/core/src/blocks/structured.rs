use super::budget::{Budget, DerivationError};
use super::inline::{inlines, plain};
use super::*;
use crate::ir::{BlockKind, BlockNode, ForeignHtmlTrustLevel, TableAlignment, TableCellNode};
use crate::source::SourceDocument;

type Result<T> = std::result::Result<T, DerivationError>;

pub(crate) fn blocks(
    nodes: &[BlockNode],
    source: &SourceDocument,
    b: &mut Budget,
) -> Result<Vec<Option<StructuredBlock>>> {
    nodes
        .iter()
        .map(|n| block(n, source, b).map(Some))
        .collect()
}

fn cell(c: &TableCellNode, b: &mut Budget) -> Result<StructuredTableCell> {
    b.reserve(16)?;
    Ok(StructuredTableCell {
        text: plain(&c.children, b)?,
        rich_text: inlines(&c.children, b)?,
    })
}

pub(crate) fn ordered_start(marker: &str) -> &str {
    let digits = marker.trim_start_matches('0');
    let end = digits.bytes().take_while(u8::is_ascii_digit).count();
    if end == 0 {
        "0"
    } else {
        &digits[..end]
    }
}

fn block(n: &BlockNode, source: &SourceDocument, b: &mut Budget) -> Result<StructuredBlock> {
    use StructuredBlockProperties as P;
    b.reserve(48)?;
    let id = b.copy(&n.id)?;
    let mut text = None;
    let content = match &n.kind {
        BlockKind::Document => P::Document {},
        BlockKind::Paragraph(p) => {
            text = Some(plain(&p.children, b)?);
            P::Paragraph {
                rich_text: inlines(&p.children, b)?,
                tight: n.attrs.contains_key("tight_list_item"),
            }
        }
        BlockKind::Heading(h) => {
            text = Some(plain(&h.children, b)?);
            P::Heading {
                level: h.level,
                rich_text: inlines(&h.children, b)?,
            }
        }
        BlockKind::FencedCode(c) => {
            let raw = source.source_span(c.body_span);
            // Count normalized bytes before allocating the owned code value.
            let bytes = if c.indent == 0 {
                raw.len()
            } else {
                let mut lines = raw.lines();
                let mut bytes = 0usize;
                if let Some(first) = lines.next() {
                    bytes = first.len()
                        - first
                            .bytes()
                            .take(c.indent)
                            .take_while(|v| *v == b' ')
                            .count();
                }
                for line in lines {
                    bytes = bytes.saturating_add(
                        1 + line.len()
                            - line
                                .bytes()
                                .take(c.indent)
                                .take_while(|v| *v == b' ')
                                .count(),
                    );
                }
                bytes
            };
            b.reserve(bytes)?;
            text = Some(crate::render::normalize_fenced_code_text(raw, c.indent));
            P::FencedCode {
                language: b.optional(c.language.as_deref())?,
                info: b.copy(&c.info)?,
                closed: c.closed,
                indent: c.indent,
            }
        }
        BlockKind::IndentedCode(c) => {
            text = Some(
                b.copy(
                    n.attrs
                        .get("normalized_code")
                        .map_or_else(|| source.source_span(c.body_span), String::as_str),
                )?,
            );
            P::IndentedCode {
                language: b.optional(c.language.as_deref())?,
                info: b.copy(&c.info)?,
                closed: c.closed,
                indent: c.indent,
            }
        }
        BlockKind::MathBlock(m) => {
            text = Some(
                b.copy(
                    m.source
                        .as_deref()
                        .unwrap_or_else(|| source.source_span(m.source_span)),
                )?,
            );
            P::MathBlock {
                display: m.display,
                closed: m.closed,
            }
        }
        BlockKind::ThematicBreak => P::ThematicBreak {},
        BlockKind::HtmlBlock => {
            text = Some(b.copy(source.source_span(n.span))?);
            P::HtmlBlock {}
        }
        BlockKind::LinkReferenceDefinition => P::LinkReferenceDefinition {
            source: b.copy(source.source_span(n.span))?,
        },
        BlockKind::BlockQuote => P::BlockQuote {},
        BlockKind::List(l) => P::List {
            ordered: l.ordered,
            loose: l.loose,
            marker: b.copy(&l.marker)?,
            start: if l.ordered {
                Some(b.copy(ordered_start(&l.marker))?)
            } else {
                None
            },
        },
        BlockKind::ListItem(l) => P::ListItem {
            marker: b.copy(&l.raw_prefix)?,
            checked: l.task.as_ref().map(|t| t.checked),
        },
        BlockKind::GfmTable(t) => {
            b.reserve(t.alignments.len().saturating_mul(2))?;
            P::GfmTable {
                alignments: t
                    .alignments
                    .iter()
                    .map(|a| match a {
                        TableAlignment::None => BlockTableAlignment::None,
                        TableAlignment::Left => BlockTableAlignment::Left,
                        TableAlignment::Center => BlockTableAlignment::Center,
                        TableAlignment::Right => BlockTableAlignment::Right,
                    })
                    .collect(),
                header: t.header.iter().map(|c| cell(c, b)).collect::<Result<_>>()?,
                rows: t
                    .rows
                    .iter()
                    .map(|r| {
                        b.reserve(2)?;
                        r.cells.iter().map(|c| cell(c, b)).collect()
                    })
                    .collect::<Result<_>>()?,
            }
        }
        BlockKind::BlankLine => P::BlankLine {},
        BlockKind::Callout(c) => P::Callout {
            kind: b.copy(&c.kind)?,
            title: b.optional(c.title.as_deref())?,
        },
        BlockKind::ForeignHtml(h) => {
            text = Some(b.copy(&h.html)?);
            P::ForeignHtml {
                origin: b.copy(&h.origin)?,
                trust_level: match h.trust_level {
                    ForeignHtmlTrustLevel::Untrusted => BlockTrustLevel::Untrusted,
                    ForeignHtmlTrustLevel::Sanitized => BlockTrustLevel::Sanitized,
                    ForeignHtmlTrustLevel::Trusted => BlockTrustLevel::Trusted,
                },
            }
        }
    };
    // Only these container kinds establish new source coordinates in HTML too.
    let nested_source;
    let child_source = match &n.kind {
        BlockKind::BlockQuote | BlockKind::ListItem(_) | BlockKind::Callout(_) => {
            if let Some(s) = n.attrs.get("normalized_source") {
                nested_source = SourceDocument::new(s.clone());
                &nested_source
            } else {
                source
            }
        }
        BlockKind::Document
        | BlockKind::Paragraph(_)
        | BlockKind::Heading(_)
        | BlockKind::FencedCode(_)
        | BlockKind::IndentedCode(_)
        | BlockKind::MathBlock(_)
        | BlockKind::ThematicBreak
        | BlockKind::HtmlBlock
        | BlockKind::LinkReferenceDefinition
        | BlockKind::List(_)
        | BlockKind::GfmTable(_)
        | BlockKind::BlankLine
        | BlockKind::ForeignHtml(_) => source,
    };
    Ok(StructuredBlock {
        id,
        text,
        content,
        children: blocks(&n.children, child_source, b)?,
    })
}
