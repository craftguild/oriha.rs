//! Validation of structural ownership, independently of semantic definition links.
use super::{BlockOutputMode, BlockOutputStatus, DocumentNodeKind as N, StructuredBlockOutput};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentValidationError {
    Version,
    Incomplete,
    Budget,
    Identity,
    Ownership,
    Target,
    Dependency,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Block,
    Item,
    Inline,
    Row,
    Cell,
    Reference,
    Footnote,
}

impl N {
    fn role(&self) -> Role {
        match self {
            Self::Document { .. }
            | Self::Paragraph { .. }
            | Self::Heading { .. }
            | Self::FencedCode { .. }
            | Self::IndentedCode { .. }
            | Self::MathBlock { .. }
            | Self::ThematicBreak {}
            | Self::HtmlBlock { .. }
            | Self::LinkReferenceDefinition { .. }
            | Self::BlockQuote { .. }
            | Self::List { .. }
            | Self::GfmTable { .. }
            | Self::BlankLine {}
            | Self::Callout { .. }
            | Self::ForeignHtml { .. } => Role::Block,
            Self::ListItem { .. } => Role::Item,
            Self::TableRow { .. } => Role::Row,
            Self::TableCell { .. } => Role::Cell,
            Self::ReferenceDefinition { .. } => Role::Reference,
            Self::FootnoteDefinition { .. } => Role::Footnote,
            Self::Text { .. }
            | Self::Emoji { .. }
            | Self::Code { .. }
            | Self::Math { .. }
            | Self::Html { .. }
            | Self::SoftBreak {}
            | Self::HardBreak {}
            | Self::Annotation { .. }
            | Self::Emphasis { .. }
            | Self::Strong { .. }
            | Self::Strikethrough { .. }
            | Self::Link { .. }
            | Self::Image { .. }
            | Self::FootnoteReference { .. } => Role::Inline,
        }
    }

    fn ownership(&self) -> (&[String], Role) {
        match self {
            Self::Document { child_ids }
            | Self::BlockQuote { child_ids }
            | Self::ListItem { child_ids, .. }
            | Self::Callout { child_ids, .. } => (child_ids, Role::Block),
            Self::Paragraph { child_ids, .. }
            | Self::Heading { child_ids, .. }
            | Self::TableCell { child_ids }
            | Self::Annotation { child_ids, .. }
            | Self::Emphasis { child_ids }
            | Self::Strong { child_ids }
            | Self::Strikethrough { child_ids }
            | Self::Link { child_ids, .. }
            | Self::FootnoteDefinition { child_ids, .. } => (child_ids, Role::Inline),
            Self::Image { alt_ids, .. } => (alt_ids, Role::Inline),
            Self::List { child_ids, .. } => (child_ids, Role::Item),
            Self::GfmTable { row_ids, .. } => (row_ids, Role::Row),
            Self::TableRow { cell_ids, .. } => (cell_ids, Role::Cell),
            Self::FencedCode { .. }
            | Self::IndentedCode { .. }
            | Self::MathBlock { .. }
            | Self::ThematicBreak {}
            | Self::HtmlBlock { .. }
            | Self::LinkReferenceDefinition { .. }
            | Self::BlankLine {}
            | Self::ForeignHtml { .. }
            | Self::Text { .. }
            | Self::Emoji { .. }
            | Self::Code { .. }
            | Self::Math { .. }
            | Self::Html { .. }
            | Self::SoftBreak {}
            | Self::HardBreak {}
            | Self::FootnoteReference { .. }
            | Self::ReferenceDefinition { .. } => (&[], Role::Inline),
        }
    }
}

impl StructuredBlockOutput {
    /// A flat wire encoding still needs bounded validation before traversal.
    /// Definition references do not participate in the ownership walk.
    pub fn validate(
        &self,
        max_nodes: usize,
        max_depth: usize,
        max_bytes: usize,
    ) -> Result<(), DocumentValidationError> {
        use DocumentValidationError as E;
        if self.schema_version != 2 || self.mode != BlockOutputMode::Structured {
            return Err(E::Version);
        }
        if self.status != BlockOutputStatus::Complete {
            return Err(E::Incomplete);
        }
        if self.nodes.len() > max_nodes || !super::budget::fits(self, max_bytes) {
            return Err(E::Budget);
        }
        let mut nodes = BTreeMap::new();
        for node in &self.nodes {
            if node.id.is_empty() || nodes.insert(node.id.as_str(), &node.content).is_some() {
                return Err(E::Identity);
            }
        }
        let mut pending = Vec::new();
        for (ids, role) in [
            (&self.root_ids, Role::Block),
            (&self.reference_ids, Role::Reference),
            (&self.footnote_ids, Role::Footnote),
        ] {
            pending.extend(ids.iter().map(|id| (id.as_str(), role, 0usize)));
        }
        let mut visited = BTreeSet::new();
        while let Some((id, role, depth)) = pending.pop() {
            if depth > max_depth {
                return Err(E::Budget);
            }
            if !visited.insert(id) {
                return Err(E::Ownership);
            }
            let node = nodes.get(id).ok_or(E::Target)?;
            if node.role() != role {
                return Err(E::Target);
            }
            let (ids, child_role) = node.ownership();
            pending.extend(ids.iter().map(|id| (id.as_str(), child_role, depth + 1)));
            if let N::FootnoteReference {
                label,
                definition_id: Some(target),
                ..
            } = node
            {
                match nodes.get(target.as_str()) {
                    Some(N::FootnoteDefinition {
                        label: target_label,
                        ..
                    }) if label == target_label => {}
                    _ => return Err(E::Target),
                }
            }
        }
        if visited.len() != nodes.len() {
            return Err(E::Ownership);
        }
        let mut fonts = BTreeSet::new();
        for font in &self.fonts {
            if font.id.is_empty() || !fonts.insert(font.id.as_str()) {
                return Err(E::Dependency);
            }
        }
        for node in &self.nodes {
            let svg = match &node.content {
                N::FencedCode { svg, .. } | N::MathBlock { svg, .. } | N::Math { svg, .. } => svg,
                _ => continue,
            };
            if let Some(svg) = svg {
                if svg.font_ids.iter().any(|id| !fonts.contains(id.as_str()))
                    || [svg.width, svg.height]
                        .into_iter()
                        .flatten()
                        .any(|v| !v.is_finite() || v <= 0.0)
                    || svg.baseline.is_some_and(|v| !v.is_finite())
                {
                    return Err(E::Dependency);
                }
            }
        }
        Ok(())
    }
}
