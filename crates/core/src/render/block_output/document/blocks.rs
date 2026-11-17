use super::*;
use crate::ir::ForeignHtmlTrustLevel;

impl Pool<'_> {
    pub(super) fn blocks(
        &mut self,
        blocks: &[BlockNode],
        source: &SourceDocument,
    ) -> Result<Vec<String>> {
        blocks
            .iter()
            .map(|block| self.block(block, source))
            .collect()
    }

    fn block(&mut self, block: &BlockNode, source: &SourceDocument) -> Result<String> {
        let id = self.b.copy(&block.id)?;
        let nested;
        let child_source = if matches!(
            block.kind,
            BlockKind::BlockQuote | BlockKind::ListItem(_) | BlockKind::Callout(_)
        ) {
            if let Some(value) = block.attrs.get("normalized_source") {
                nested = SourceDocument::new(value.clone());
                &nested
            } else {
                source
            }
        } else {
            source
        };
        // A leaf with owned blocks is malformed IR, not content to discard.
        let container = matches!(
            block.kind,
            BlockKind::Document
                | BlockKind::BlockQuote
                | BlockKind::List(_)
                | BlockKind::ListItem(_)
                | BlockKind::Callout(_)
        );
        if !container && !block.children.is_empty() {
            return Err(DerivationError {
                code: "blocks.invalid_ownership",
                message: "Leaf block owns block children",
            });
        }
        // Deliberately exhaustive: adding an IR syntax variant must fail to
        // compile until its typed structured projection is implemented.
        let content = match &block.kind {
            BlockKind::Document => N::Document {
                child_ids: self.blocks(&block.children, child_source)?,
            },
            BlockKind::Paragraph(p) => N::Paragraph {
                child_ids: self.inlines(&p.children)?,
                tight: block.attrs.contains_key("tight_list_item"),
            },
            BlockKind::Heading(h) => N::Heading {
                anchor: references::block_anchor(block, &mut self.headings)
                    .expect("heading anchor"),
                level: h.level,
                child_ids: self.inlines(&h.children)?,
            },
            BlockKind::FencedCode(code) => {
                let raw = source.source_span(code.body_span);
                self.b.reserve(raw.len())?;
                let mut text = normalize_fenced_code_text(raw, code.indent);
                if code.indent > 0 && raw.ends_with('\n') {
                    text.push('\n');
                }
                let language = code.language.as_deref().map(decode_markdown);
                let svg = self.fence_svg(&id, &text, language.as_deref())?;
                let highlighting = self.highlighting(&text, language.as_deref())?;
                N::FencedCode {
                    source: text,
                    language,
                    info: self.b.copy(&code.info)?,
                    closed: code.closed,
                    indent: code.indent,
                    highlighting,
                    svg,
                }
            }
            BlockKind::IndentedCode(code) => {
                let text = self.b.copy(
                    block
                        .attrs
                        .get("normalized_code")
                        .map_or_else(|| source.source_span(code.body_span), String::as_str),
                )?;
                let language = code.language.as_deref().map(decode_markdown);
                let highlighting = self.highlighting(&text, language.as_deref())?;
                N::IndentedCode {
                    source: text,
                    language,
                    info: self.b.copy(&code.info)?,
                    closed: code.closed,
                    indent: code.indent,
                    highlighting,
                }
            }
            BlockKind::MathBlock(math) => {
                let text = self.b.copy(
                    math.source
                        .as_deref()
                        .unwrap_or_else(|| source.source_span(math.source_span)),
                )?;
                let svg = self.math_svg(&id, &text, math.display, math.closed)?;
                N::MathBlock {
                    source: text,
                    display: math.display,
                    closed: math.closed,
                    svg,
                }
            }
            BlockKind::ThematicBreak => N::ThematicBreak {},
            BlockKind::HtmlBlock => N::HtmlBlock {
                source: self.b.copy(source.source_span(block.span))?,
            },
            BlockKind::LinkReferenceDefinition => N::LinkReferenceDefinition {
                source: self.b.copy(source.source_span(block.span))?,
            },
            BlockKind::BlockQuote => N::BlockQuote {
                child_ids: self.blocks(&block.children, child_source)?,
            },
            BlockKind::List(list) => N::List {
                child_ids: self.blocks(&block.children, child_source)?,
                ordered: list.ordered,
                loose: list.loose,
                marker: self.b.copy(&list.marker)?,
                start: if list.ordered {
                    Some(self.b.copy(ordered_start(&list.marker))?)
                } else {
                    None
                },
            },
            BlockKind::ListItem(item) => N::ListItem {
                child_ids: self.blocks(&block.children, child_source)?,
                marker: self.b.copy(&item.raw_prefix)?,
                checked: item.task.as_ref().map(|task| task.checked),
            },
            BlockKind::GfmTable(table) => {
                let mut row_ids = vec![self.row(&table.header, true)?];
                for row in &table.rows {
                    row_ids.push(self.row(&row.cells, false)?);
                }
                self.b.reserve(table.alignments.len())?;
                N::GfmTable {
                    row_ids,
                    alignments: table
                        .alignments
                        .iter()
                        .map(|alignment| match alignment {
                            TableAlignment::None => BlockTableAlignment::None,
                            TableAlignment::Left => BlockTableAlignment::Left,
                            TableAlignment::Center => BlockTableAlignment::Center,
                            TableAlignment::Right => BlockTableAlignment::Right,
                        })
                        .collect(),
                }
            }
            BlockKind::BlankLine => N::BlankLine {},
            BlockKind::Callout(callout) => N::Callout {
                child_ids: self.blocks(&block.children, child_source)?,
                callout_kind: self.b.copy(&callout.kind)?,
                title: self.b.optional(callout.title.as_deref())?,
            },
            BlockKind::ForeignHtml(html) => N::ForeignHtml {
                source: self.b.copy(&html.html)?,
                origin: self.b.copy(&html.origin)?,
                trust_level: match html.trust_level {
                    ForeignHtmlTrustLevel::Untrusted => BlockTrustLevel::Untrusted,
                    ForeignHtmlTrustLevel::Sanitized => BlockTrustLevel::Sanitized,
                    ForeignHtmlTrustLevel::Trusted => BlockTrustLevel::Trusted,
                },
            },
        };
        self.push(id.clone(), content)?;
        Ok(id)
    }

    fn row(&mut self, cells: &[TableCellNode], header: bool) -> Result<String> {
        let id = self.id()?;
        let mut cell_ids = Vec::new();
        for cell in cells {
            let cell_id = self.id()?;
            let child_ids = self.inlines(&cell.children)?;
            self.push(cell_id.clone(), N::TableCell { child_ids })?;
            cell_ids.push(cell_id);
        }
        self.push(id.clone(), N::TableRow { header, cell_ids })?;
        Ok(id)
    }
}

fn ordered_start(marker: &str) -> &str {
    let digits = marker.trim_start_matches('0');
    let end = digits.bytes().take_while(u8::is_ascii_digit).count();
    if end == 0 {
        "0"
    } else {
        &digits[..end]
    }
}
