//! Revalidate hook-edited value mappings without repeated old-document scans.
use std::collections::HashSet;

use crate::annotations::{sequences, sequences_mut};
use crate::ir::{Document, InlineKind, InlineNode};
use crate::mapping::{MappedText, TextMapping, TextProvenance};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ValueKind {
    Text,
    Code,
    Emoji,
}

fn value(node: &InlineNode) -> Option<(ValueKind, &str)> {
    match &node.kind {
        InlineKind::Text(text) => Some((ValueKind::Text, text)),
        InlineKind::Code(text) => Some((ValueKind::Code, text)),
        InlineKind::Emoji { emoji, .. } => Some((ValueKind::Emoji, emoji)),
        _ => None,
    }
}

fn children(node: &InlineNode) -> Option<&[InlineNode]> {
    match &node.kind {
        InlineKind::Emphasis { children }
        | InlineKind::Strong { children }
        | InlineKind::Strikethrough { children }
        | InlineKind::Link { children, .. }
        | InlineKind::Annotation { children, .. } => Some(children),
        InlineKind::Image { alt, .. } => Some(alt),
        _ => None,
    }
}

#[derive(Default)]
struct Original<'a> {
    unchanged: HashSet<(ValueKind, &'a str, &'a [TextMapping])>,
    mappings: HashSet<&'a [TextMapping]>,
}

impl<'a> Original<'a> {
    fn collect(&mut self, nodes: &'a [InlineNode]) {
        for node in nodes {
            if let Some((kind, text)) = value(node) {
                self.unchanged.insert((kind, text, &node.source_mapping));
                self.mappings.insert(&node.source_mapping);
            }
            if let Some(nested) = children(node) {
                self.collect(nested);
            }
        }
    }
}

fn valid(text: &str, mapping: &[TextMapping], source: &str) -> bool {
    let mut end = 0;
    for m in mapping {
        if m.display.start != end
            || m.display.start >= m.display.end
            || m.display.end > text.len()
            || !text.is_char_boundary(m.display.start)
            || !text.is_char_boundary(m.display.end)
        {
            return false;
        }
        if m.source.iter().any(|r| {
            r.start >= r.end
                || r.end > source.len()
                || !source.is_char_boundary(r.start)
                || !source.is_char_boundary(r.end)
        }) {
            return false;
        }
        if (m.provenance == TextProvenance::Generated) != m.source.is_empty() {
            return false;
        }
        if m.exact && (m.source.len() != 1 || m.source[0].len() != m.display.len()) {
            return false;
        }
        end = m.display.end;
    }
    end == text.len()
}

fn reconcile_nodes(nodes: &mut [InlineNode], old: &Original<'_>, source: &str) {
    for node in nodes {
        if let Some((kind, text)) = value(node) {
            let mapping = node.source_mapping.as_slice();
            let unchanged = old.unchanged.contains(&(kind, text, mapping));
            let reused = old.mappings.contains(mapping);
            if (!unchanged && reused) || !valid(text, mapping, source) {
                node.source_mapping = MappedText::generated(text).mapping;
            }
        }
        match &mut node.kind {
            InlineKind::Emphasis { children }
            | InlineKind::Strong { children }
            | InlineKind::Strikethrough { children }
            | InlineKind::Link { children, .. }
            | InlineKind::Annotation { children, .. } => reconcile_nodes(children, old, source),
            InlineKind::Image { alt, .. } => reconcile_nodes(alt, old, source),
            _ => {}
        }
    }
}

pub(crate) fn reconcile(before: &Document, after: &mut Document, source: &str) {
    // Index borrowed values and correspondence once. Hashing/equality include
    // their content; identical strings with distinct source ranges stay distinct.
    let mut old = Original::default();
    sequences(before, |nodes| old.collect(nodes));
    sequences_mut(after, |nodes| reconcile_nodes(nodes, &old, source));
}
