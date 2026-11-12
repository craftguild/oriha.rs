//! Mapped inline population for parsed blocks, footnotes and task markers.
use super::*;

pub(super) fn populate_block_inlines(
    mapped_source: &MappedText,
    link_references: &BTreeMap<String, LinkReference>,
    features: &FeatureFlags,
    blocks: &mut [BlockNode],
) {
    for block in blocks {
        match &mut block.kind {
            BlockKind::Paragraph(node) => {
                let mapped = if let Some(text) = block.attrs.get("inline_source") {
                    MappedText::compose(text, &block.source_mapping.inline, mapped_source)
                } else {
                    mapped_source.slice(block.span.start..block.span.end)
                };
                node.children =
                    parse_mapped_inlines(&normalize::paragraph(&mapped), link_references, features);
            }
            BlockKind::Heading(node) => {
                let mapped = if let Some(text) = block.attrs.get("inline_source") {
                    MappedText::compose(text, &block.source_mapping.inline, mapped_source)
                } else {
                    mapped_source.slice(node.inline_span.start..node.inline_span.end)
                };
                node.children = parse_mapped_inlines(&mapped, link_references, features);
            }
            BlockKind::GfmTable(node) => {
                for cell in node
                    .header
                    .iter_mut()
                    .chain(node.rows.iter_mut().flat_map(|row| row.cells.iter_mut()))
                {
                    let mapped =
                        MappedText::compose(&cell.source, &cell.source_mapping, mapped_source)
                            .trim();
                    cell.children = parse_mapped_inlines(&mapped, link_references, features);
                }
            }
            _ => {}
        }

        if block.children.is_empty() {
            continue;
        }
        if let Some(normalized) = block.attrs.get("normalized_source") {
            populate_block_inlines(
                &MappedText::compose(normalized, &block.source_mapping.normalized, mapped_source),
                link_references,
                features,
                &mut block.children,
            );
        } else {
            populate_block_inlines(
                mapped_source,
                link_references,
                features,
                &mut block.children,
            );
        }
        if matches!(&block.kind, BlockKind::ListItem(node) if node.task.is_some()) {
            strip_task_marker_from_first_paragraph(&mut block.children);
        }
    }
}

pub(super) fn populate_footnote_inlines(
    footnotes: &[FootnoteDefinitionSource],
    link_references: &BTreeMap<String, LinkReference>,
    features: &FeatureFlags,
) -> Vec<FootnoteDefinition> {
    footnotes
        .iter()
        .map(|footnote| FootnoteDefinition {
            label: footnote.label.clone(),
            children: parse_mapped_inlines(
                &footnote
                    .content
                    .slice(0..footnote.content.text.trim_end().len()),
                link_references,
                features,
            ),
        })
        .collect()
}

pub(super) fn strip_task_marker_from_first_paragraph(blocks: &mut [BlockNode]) {
    let Some(first) = blocks.first_mut() else {
        return;
    };
    let BlockKind::Paragraph(node) = &mut first.kind else {
        return;
    };
    strip_task_marker_from_inlines(&mut node.children);
}

pub(super) fn strip_task_marker_from_inlines(inlines: &mut Vec<InlineNode>) {
    let Some(InlineNode {
        kind: InlineKind::Text(text),
        source_mapping,
    }) = inlines.first_mut()
    else {
        return;
    };
    let remove = if text.starts_with("[ ]") || text.starts_with("[x]") || text.starts_with("[X]") {
        let marker_len = 3;
        marker_len
            + text[marker_len..]
                .chars()
                .next()
                .filter(|ch| ch.is_ascii_whitespace())
                .map_or(0, char::len_utf8)
    } else {
        0
    };
    if remove == 0 {
        return;
    }
    *source_mapping = source_mapping
        .iter()
        .filter_map(|m| m.clipped(remove..text.len()))
        .collect();
    text.drain(..remove);
    if text.is_empty() {
        inlines.remove(0);
    }
}
