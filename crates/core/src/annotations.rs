//! Policy-free traversal and immutable decoration of parsed visible text.
use std::collections::BTreeMap;
use std::ops::Range;

use crate::ir::{BlockKind, BlockNode, Document, InlineKind, InlineNode};
use crate::mapping::{MappedText, TextMapping};

/// Opaque identity scoped to one parsed document instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionId {
    document: u64,
    index: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AnnotationAttributes {
    pub classes: Vec<String>,
    /// Attribute suffixes: `recipient` renders as `data-recipient`.
    pub data: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnnotationMetadata {
    pub kind: String,
    pub id: String,
    pub attributes: AnnotationAttributes,
}

#[derive(Clone, Debug)]
pub struct TextAnnotation {
    pub region: RegionId,
    pub range: Range<usize>,
    pub metadata: AnnotationMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnnotationError {
    ForeignDocument,
    InvalidRange,
    Overlap,
    InvalidAttributes,
}

impl std::fmt::Display for AnnotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ForeignDocument => "region belongs to another parsed document",
            Self::InvalidRange => {
                "range must be nonempty, within one region, and on UTF-8 boundaries"
            }
            Self::Overlap => "annotation overlaps another annotation",
            Self::InvalidAttributes => {
                "annotation kind, identity, class, or data attribute is invalid"
            }
        })
    }
}
impl std::error::Error for AnnotationError {}

/// A segment connecting region bytes to an inline text node and original input.
#[derive(Debug)]
pub struct RegionMapping<'a> {
    pub node: &'a InlineNode,
    pub node_range: Range<usize>,
    /// `display` is relative to the entire region.
    pub mapping: TextMapping,
}

#[derive(Debug)]
pub struct TextRegion<'a> {
    pub id: RegionId,
    pub text: String,
    pub mappings: Vec<RegionMapping<'a>>,
    spans: Vec<TextSpan>,
}

#[derive(Debug)]
struct TextSpan {
    node: usize,
    display: Range<usize>,
    annotated: bool,
}

pub(crate) fn sequences<'a>(document: &'a Document, mut visit: impl FnMut(&'a [InlineNode])) {
    fn blocks<'a>(block_nodes: &'a [BlockNode], visit: &mut impl FnMut(&'a [InlineNode])) {
        for block in block_nodes {
            match &block.kind {
                BlockKind::Paragraph(n) => visit(&n.children),
                BlockKind::Heading(n) => visit(&n.children),
                BlockKind::GfmTable(n) => {
                    for cell in n.header.iter().chain(n.rows.iter().flat_map(|r| &r.cells)) {
                        visit(&cell.children);
                    }
                }
                _ => {}
            }
            blocks(&block.children, visit);
        }
    }
    blocks(&document.blocks, &mut visit);
    for footnote in &document.footnotes {
        visit(&footnote.children);
    }
}

pub(crate) fn sequences_mut(document: &mut Document, visit: impl FnMut(&mut Vec<InlineNode>)) {
    sequences_mut_selected(document, None, visit);
}

fn sequences_mut_selected(
    document: &mut Document,
    footnotes: Option<&[usize]>,
    mut visit: impl FnMut(&mut Vec<InlineNode>),
) {
    fn blocks(block_nodes: &mut [BlockNode], visit: &mut impl FnMut(&mut Vec<InlineNode>)) {
        for block in block_nodes {
            match &mut block.kind {
                BlockKind::Paragraph(n) => visit(&mut n.children),
                BlockKind::Heading(n) => visit(&mut n.children),
                BlockKind::GfmTable(n) => {
                    for cell in n
                        .header
                        .iter_mut()
                        .chain(n.rows.iter_mut().flat_map(|r| &mut r.cells))
                    {
                        visit(&mut cell.children);
                    }
                }
                _ => {}
            }
            blocks(&mut block.children, visit);
        }
    }
    blocks(&mut document.blocks, &mut visit);
    if let Some(indices) = footnotes {
        for &index in indices {
            visit(&mut document.footnotes[index].children);
        }
    } else {
        for footnote in &mut document.footnotes {
            visit(&mut footnote.children);
        }
    }
}

pub(crate) fn regions(document: &Document, identity: u64) -> Vec<TextRegion<'_>> {
    struct Builder<'a> {
        identity: u64,
        regions: Vec<TextRegion<'a>>,
        text: String,
        mappings: Vec<RegionMapping<'a>>,
        spans: Vec<TextSpan>,
        next_node: usize,
    }
    impl<'a> Builder<'a> {
        fn flush(&mut self) {
            if self.text.is_empty() {
                return;
            }
            self.regions.push(TextRegion {
                id: RegionId {
                    document: self.identity,
                    index: self.regions.len(),
                },
                text: std::mem::take(&mut self.text),
                mappings: std::mem::take(&mut self.mappings),
                spans: std::mem::take(&mut self.spans),
            });
        }
        fn walk(&mut self, nodes: &'a [InlineNode], annotated: bool) {
            for node in nodes {
                match &node.kind {
                    InlineKind::Text(text) => {
                        let start = self.text.len();
                        self.text.push_str(text);
                        self.spans.push(TextSpan {
                            node: self.next_node,
                            display: start..self.text.len(),
                            annotated,
                        });
                        self.next_node += 1;
                        for mapping in &node.source_mapping {
                            let mut region_mapping = mapping.clone();
                            region_mapping.display =
                                start + mapping.display.start..start + mapping.display.end;
                            self.mappings.push(RegionMapping {
                                node,
                                node_range: mapping.display.clone(),
                                mapping: region_mapping,
                            });
                        }
                    }
                    InlineKind::Emphasis { children }
                    | InlineKind::Strong { children }
                    | InlineKind::Strikethrough { children }
                    | InlineKind::Link { children, .. } => self.walk(children, annotated),
                    InlineKind::Annotation { children, .. } => self.walk(children, true),
                    _ => self.flush(),
                }
            }
        }
    }
    let mut builder = Builder {
        identity,
        regions: Vec::new(),
        text: String::new(),
        mappings: Vec::new(),
        spans: Vec::new(),
        next_node: 0,
    };
    // Explicit lifetime on the traversal is needed because regions borrow nodes.
    fn walk_blocks<'a>(blocks: &'a [BlockNode], builder: &mut Builder<'a>) {
        for block in blocks {
            match &block.kind {
                BlockKind::Paragraph(n) => {
                    builder.walk(&n.children, false);
                    builder.flush();
                }
                BlockKind::Heading(n) => {
                    builder.walk(&n.children, false);
                    builder.flush();
                }
                BlockKind::GfmTable(n) => {
                    for cell in n.header.iter().chain(n.rows.iter().flat_map(|r| &r.cells)) {
                        builder.walk(&cell.children, false);
                        builder.flush();
                    }
                }
                _ => {}
            }
            walk_blocks(&block.children, builder);
        }
    }
    walk_blocks(&document.blocks, &mut builder);
    for index in crate::footnotes::visible_indices(document) {
        builder.walk(&document.footnotes[index].children, false);
        builder.flush();
    }
    builder.regions
}

pub(crate) fn valid_data_name(key: &str) -> bool {
    key.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && key
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        && key != "oriha-annotation-id"
        && key != "oriha-annotation-kind"
}

fn validate_metadata(metadata: &AnnotationMetadata) -> bool {
    !metadata.kind.is_empty()
        && !metadata.id.is_empty()
        && metadata
            .attributes
            .classes
            .iter()
            .all(|c| !c.is_empty() && !c.chars().any(|ch| ch.is_whitespace() || ch.is_control()))
        && metadata
            .attributes
            .data
            .keys()
            .all(|key| valid_data_name(key))
}

pub(crate) fn apply(
    document: &Document,
    identity: u64,
    annotations: &[TextAnnotation],
) -> Result<Document, AnnotationError> {
    let regions = regions(document, identity);
    let mut sorted: Vec<_> = annotations.iter().collect();
    sorted.sort_by_key(|a| (a.region, a.range.start, a.range.end));
    let mut previous: Option<&TextAnnotation> = None;
    let mut plan = BTreeMap::<usize, Vec<(Range<usize>, AnnotationMetadata)>>::new();
    for annotation in sorted {
        if annotation.region.document != identity {
            return Err(AnnotationError::ForeignDocument);
        }
        let region = regions
            .get(annotation.region.index)
            .ok_or(AnnotationError::InvalidRange)?;
        let range = &annotation.range;
        if range.start >= range.end
            || range.end > region.text.len()
            || !region.text.is_char_boundary(range.start)
            || !region.text.is_char_boundary(range.end)
        {
            return Err(AnnotationError::InvalidRange);
        }
        if !validate_metadata(&annotation.metadata) {
            return Err(AnnotationError::InvalidAttributes);
        }
        if previous.is_some_and(|p| p.region == annotation.region && p.range.end > range.start) {
            return Err(AnnotationError::Overlap);
        }
        previous = Some(annotation);
        for span in &region.spans {
            let start = span.display.start.max(range.start);
            let end = span.display.end.min(range.end);
            if start >= end {
                continue;
            }
            if span.annotated {
                return Err(AnnotationError::Overlap);
            }
            plan.entry(span.node).or_default().push((
                start - span.display.start..end - span.display.start,
                annotation.metadata.clone(),
            ));
        }
    }
    // No IR is copied or modified until every request has passed validation.
    let mut result = document.clone();
    let mut next_node = 0;
    fn decorate(
        nodes: &mut Vec<InlineNode>,
        next_node: &mut usize,
        plan: &BTreeMap<usize, Vec<(Range<usize>, AnnotationMetadata)>>,
    ) {
        let mut out = Vec::with_capacity(nodes.len());
        for mut node in std::mem::take(nodes) {
            match &mut node.kind {
                InlineKind::Text(text) => {
                    let index = *next_node;
                    *next_node += 1;
                    if let Some(annotations) = plan.get(&index) {
                        let mapped = MappedText {
                            text: text.clone(),
                            mapping: node.source_mapping.clone(),
                        };
                        let mut cursor = 0;
                        let fragment = |range| {
                            let part = mapped.slice(range);
                            InlineNode {
                                kind: InlineKind::Text(part.text),
                                source_mapping: part.mapping,
                            }
                        };
                        for (range, metadata) in annotations {
                            if cursor < range.start {
                                out.push(fragment(cursor..range.start));
                            }
                            let child = fragment(range.clone());
                            out.push(InlineNode {
                                source_mapping: child.source_mapping.clone(),
                                kind: InlineKind::Annotation {
                                    annotation: metadata.clone(),
                                    children: vec![child],
                                },
                            });
                            cursor = range.end;
                        }
                        if cursor < text.len() {
                            out.push(fragment(cursor..text.len()));
                        }
                        continue;
                    }
                }
                InlineKind::Emphasis { children }
                | InlineKind::Strong { children }
                | InlineKind::Strikethrough { children }
                | InlineKind::Link { children, .. }
                | InlineKind::Annotation { children, .. } => decorate(children, next_node, plan),
                _ => {}
            }
            out.push(node);
        }
        *nodes = out;
    }
    let footnotes = crate::footnotes::visible_indices(document);
    sequences_mut_selected(&mut result, Some(&footnotes), |nodes| {
        decorate(nodes, &mut next_node, &plan)
    });
    Ok(result)
}

pub(crate) fn mark_generated(document: &mut Document) {
    fn clear(nodes: &mut [InlineNode]) {
        for node in nodes {
            let generated = InlineNode::generated(node.kind.clone());
            node.source_mapping = generated.source_mapping;
            match &mut node.kind {
                InlineKind::Emphasis { children }
                | InlineKind::Strong { children }
                | InlineKind::Strikethrough { children }
                | InlineKind::Link { children, .. }
                | InlineKind::Annotation { children, .. } => clear(children),
                InlineKind::Image { alt, .. } => clear(alt),
                _ => {}
            }
        }
    }
    sequences_mut(document, |nodes| clear(nodes));
}
