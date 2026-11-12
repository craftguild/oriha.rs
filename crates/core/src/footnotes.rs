//! Default-renderer footnote body selection, in main-flow reference order.
use std::collections::{BTreeMap, BTreeSet};

use crate::ir::{BlockKind, BlockNode, Document, InlineKind, InlineNode};

pub(crate) fn visible_indices(document: &Document) -> Vec<usize> {
    struct References<'a> {
        definitions: BTreeMap<&'a str, usize>,
        seen: BTreeSet<usize>,
        order: Vec<usize>,
    }
    impl References<'_> {
        fn inlines(&mut self, nodes: &[InlineNode]) {
            for node in nodes {
                match &node.kind {
                    InlineKind::FootnoteReference { label } => {
                        if let Some(&index) = self.definitions.get(label.as_str()) {
                            if self.seen.insert(index) {
                                self.order.push(index);
                            }
                        }
                    }
                    InlineKind::Emphasis { children }
                    | InlineKind::Strong { children }
                    | InlineKind::Strikethrough { children }
                    | InlineKind::Link { children, .. }
                    | InlineKind::Annotation { children, .. } => self.inlines(children),
                    // Image alt/code/HTML are not rendered as inline footnote refs.
                    _ => {}
                }
            }
        }
        fn blocks(&mut self, blocks: &[BlockNode]) {
            for block in blocks {
                match &block.kind {
                    BlockKind::Paragraph(n) => self.inlines(&n.children),
                    BlockKind::Heading(n) => self.inlines(&n.children),
                    BlockKind::GfmTable(n) => {
                        for cell in n.header.iter().chain(n.rows.iter().flat_map(|r| &r.cells)) {
                            self.inlines(&cell.children);
                        }
                    }
                    _ => {}
                }
                self.blocks(&block.children);
            }
        }
    }
    // Match render::footnote_definition_map's last-definition-wins rule.
    let definitions = document
        .footnotes
        .iter()
        .enumerate()
        .map(|(i, note)| (note.label.as_str(), i))
        .collect();
    let mut refs = References {
        definitions,
        seen: BTreeSet::new(),
        order: Vec::new(),
    };
    refs.blocks(&document.blocks);
    // The renderer snapshots this order before rendering footnote bodies. Nested
    // references alone do not append more rendered definitions in that snapshot.
    refs.order
}
