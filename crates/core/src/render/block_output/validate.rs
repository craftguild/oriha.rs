use super::*;
use std::collections::BTreeSet;

pub(super) fn document(document: &Document, limits: EngineLimits) -> Result<(), DerivationError> {
    check_count(&document.blocks, limits.max_blocks)?;
    let mut ids = BTreeSet::new();
    let mut pending: Vec<_> = document
        .blocks
        .iter()
        .map(|b| (b, 0usize, 0usize))
        .collect();
    while let Some((block, depth, absolute_depth)) = pending.pop() {
        if depth > limits.max_nesting_depth || absolute_depth > 256 {
            return Err(depth_error());
        }
        if block.id.is_empty() || !ids.insert(block.id.as_str()) {
            return Err(DerivationError {
                code: "blocks.invalid_identity",
                message: "Block identities must be nonempty and unique throughout the document",
            });
        }
        match &block.kind {
            BlockKind::Paragraph(n) => inline_depth(&n.children)?,
            BlockKind::Heading(n) => inline_depth(&n.children)?,
            BlockKind::GfmTable(t) => {
                for c in t.header.iter().chain(t.rows.iter().flat_map(|r| &r.cells)) {
                    inline_depth(&c.children)?;
                }
            }
            BlockKind::Document
            | BlockKind::FencedCode(_)
            | BlockKind::IndentedCode(_)
            | BlockKind::MathBlock(_)
            | BlockKind::ThematicBreak
            | BlockKind::HtmlBlock
            | BlockKind::LinkReferenceDefinition
            | BlockKind::BlockQuote
            | BlockKind::List(_)
            | BlockKind::ListItem(_)
            | BlockKind::BlankLine
            | BlockKind::Callout(_)
            | BlockKind::ForeignHtml(_) => {}
        }
        let crossing = match block.kind {
            BlockKind::BlockQuote | BlockKind::ListItem(_) | BlockKind::Callout(_) => 1,
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
            | BlockKind::ForeignHtml(_) => 0,
        };
        if !matches!(block.kind, BlockKind::List(_)) {
            check_count(&block.children, limits.max_blocks)?;
        }
        pending.extend(
            block
                .children
                .iter()
                .map(|b| (b, depth + crossing, absolute_depth + 1)),
        );
    }
    for f in &document.footnotes {
        inline_depth(&f.children)?;
    }
    Ok(())
}

fn check_count(blocks: &[BlockNode], limit: usize) -> Result<(), DerivationError> {
    let count = blocks
        .iter()
        .filter(|b| {
            !matches!(
                b.kind,
                BlockKind::BlankLine | BlockKind::LinkReferenceDefinition
            )
        })
        .count();
    if count > limit {
        Err(DerivationError {
            code: "blocks.block_limit",
            message: "Hook-authored IR exceeds retained per-container max_blocks",
        })
    } else {
        Ok(())
    }
}

fn depth_error() -> DerivationError {
    DerivationError {
        code: "blocks.nesting_limit",
        message: "IR exceeds retained container depth or the safety nesting limit (256)",
    }
}

fn inline_depth(nodes: &[InlineNode]) -> Result<(), DerivationError> {
    let mut pending: Vec<_> = nodes.iter().map(|n| (n, 0usize)).collect();
    while let Some((n, depth)) = pending.pop() {
        if depth > 256 {
            return Err(depth_error());
        }
        let children = match &n.kind {
            InlineKind::Annotation { children, .. }
            | InlineKind::Emphasis { children }
            | InlineKind::Strong { children }
            | InlineKind::Strikethrough { children }
            | InlineKind::Link { children, .. } => children.as_slice(),
            InlineKind::Image { alt, .. } => alt,
            InlineKind::Text(_)
            | InlineKind::Emoji { .. }
            | InlineKind::Code(_)
            | InlineKind::Math { .. }
            | InlineKind::Html { .. }
            | InlineKind::SoftBreak
            | InlineKind::HardBreak
            | InlineKind::FootnoteReference { .. } => &[],
        };
        pending.extend(children.iter().map(|n| (n, depth + 1)));
    }
    Ok(())
}
