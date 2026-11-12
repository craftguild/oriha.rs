use super::*;

/// Normalize a parsed destination exactly as the built-in HTML renderer does.
/// This performs no I/O and does not validate an application's URL policy.
pub fn normalize_link_destination(destination: &str) -> String {
    sanitize_url(destination)
}

/// Non-SVG DOM IDs emitted by the built-in semantic HTML renderer, in emission
/// order. Custom renderers/hooks and authored raw HTML are outside this API.
/// No HTML, image preparation, syntax, math, diagrams or fonts are generated.
pub fn builtin_html_anchors(parsed: &ParsedDocument) -> Vec<String> {
    let mut collector = Anchors {
        ids: Vec::new(),
        headings: BTreeMap::new(),
        references: BTreeMap::new(),
        order: Vec::new(),
        definitions: footnote_definition_map(&parsed.document.footnotes),
        footnote_ids: parsed.state.footnote_ids.clone(),
    };
    let mut pending: Vec<_> = parsed.document.blocks.iter().rev().collect();
    while let Some(block) = pending.pop() {
        if let Some(id) = block_anchor(block, &mut collector.headings) {
            collector.ids.push(id);
        }
        match &block.kind {
            BlockKind::Heading(node) => collector.inlines(&node.children),
            BlockKind::Paragraph(node) if standalone_image(&node.children).is_none() => {
                collector.inlines(&node.children);
            }
            BlockKind::GfmTable(table) => {
                for cell in &table.header {
                    collector.inlines(&cell.children);
                }
                for row in &table.rows {
                    for cell in &row.cells {
                        collector.inlines(&cell.children);
                    }
                }
            }
            _ => {}
        }
        pending.extend(block.children.iter().rev());
    }
    // The built-in renderer snapshots first-use order before emitting definitions.
    if !collector.order.is_empty() {
        collector.ids.push(collector.footnote_ids.title());
        for label in collector.order.clone() {
            collector
                .ids
                .push(collector.footnote_ids.definition(&label));
            let definition = collector.definitions[&label].clone();
            collector.inlines(&definition.children);
        }
    }
    collector.ids
}

struct Anchors {
    ids: Vec<String>,
    headings: BTreeMap<String, usize>,
    references: BTreeMap<String, usize>,
    order: Vec<String>,
    definitions: BTreeMap<String, FootnoteDefinition>,
    footnote_ids: FootnoteIds,
}
impl Anchors {
    fn inlines(&mut self, nodes: &[InlineNode]) {
        let mut pending: Vec<_> = nodes.iter().rev().collect();
        while let Some(node) = pending.pop() {
            match &node.kind {
                InlineKind::FootnoteReference { label } if self.definitions.contains_key(label) => {
                    let count = self.references.entry(label.clone()).or_default();
                    *count += 1;
                    if *count == 1 {
                        self.order.push(label.clone());
                    }
                    self.ids.push(self.footnote_ids.reference(label, *count).1);
                }
                InlineKind::Annotation { children, .. }
                | InlineKind::Emphasis { children }
                | InlineKind::Strong { children }
                | InlineKind::Strikethrough { children }
                | InlineKind::Link { children, .. } => pending.extend(children.iter().rev()),
                _ => {}
            }
        }
    }
}

pub(super) fn block_anchor(
    block: &BlockNode,
    headings: &mut BTreeMap<String, usize>,
) -> Option<String> {
    match &block.kind {
        BlockKind::Document
        | BlockKind::BlankLine
        | BlockKind::LinkReferenceDefinition
        | BlockKind::GfmTable(_) => None,
        BlockKind::Heading(node) => Some(unique_heading_id(
            headings,
            &render_inline_heading_slug_text(&node.children),
        )),
        BlockKind::Paragraph(node)
            if block.attrs.contains_key("tight_list_item")
                && standalone_image(&node.children).is_none() =>
        {
            None
        }
        _ => Some(block.id.clone()),
    }
}
