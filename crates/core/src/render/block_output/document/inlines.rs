use super::*;

impl Pool<'_> {
    pub(super) fn inlines(&mut self, nodes: &[InlineNode]) -> Result<Vec<String>> {
        nodes.iter().map(|node| self.inline(node)).collect()
    }

    fn inline(&mut self, node: &InlineNode) -> Result<String> {
        let id = self.id()?;
        // Keep this exhaustive, with no wildcard or silent source fallback.
        // A new inline syntax must require an explicit projection decision.
        let content = match &node.kind {
            InlineKind::Text(text) => N::Text {
                text: self.b.copy(text)?,
            },
            InlineKind::Emoji { emoji, shortcode } => N::Emoji {
                text: self.b.copy(emoji)?,
                shortcode: self.b.copy(shortcode)?,
            },
            InlineKind::Code(source) => N::Code {
                source: self.b.copy(source)?,
            },
            InlineKind::Math { source } => N::Math {
                source: self.b.copy(source)?,
                svg: self.math_svg(&id, source, false, true)?,
            },
            InlineKind::Html { raw, tagfilter } => N::Html {
                source: self.b.copy(raw)?,
                tagfilter: *tagfilter,
            },
            InlineKind::SoftBreak => N::SoftBreak {},
            InlineKind::HardBreak => N::HardBreak {},
            InlineKind::Emphasis { children } => N::Emphasis {
                child_ids: self.inlines(children)?,
            },
            InlineKind::Strong { children } => N::Strong {
                child_ids: self.inlines(children)?,
            },
            InlineKind::Strikethrough { children } => N::Strikethrough {
                child_ids: self.inlines(children)?,
            },
            InlineKind::Annotation {
                annotation,
                children,
            } => N::Annotation {
                child_ids: self.inlines(children)?,
                annotation_kind: self.b.copy(&annotation.kind)?,
                annotation_id: self.b.copy(&annotation.id)?,
                classes: annotation
                    .attributes
                    .classes
                    .iter()
                    .map(|value| self.b.copy(value))
                    .collect::<Result<_>>()?,
                data: annotation
                    .attributes
                    .data
                    .iter()
                    .map(|(key, value)| Ok((self.b.copy(key)?, self.b.copy(value)?)))
                    .collect::<Result<_>>()?,
            },
            InlineKind::Link {
                destination,
                title,
                children,
            } => N::Link {
                child_ids: self.inlines(children)?,
                destination: self.b.copy(destination)?,
                normalized_destination: self.normalized(destination)?,
                title: self.b.optional(title.as_deref())?,
            },
            InlineKind::Image {
                destination,
                title,
                alt,
            } => {
                let visible = self.rendering_inlines;
                self.rendering_inlines = false;
                let alt_ids = self.inlines(alt)?;
                self.rendering_inlines = visible;
                N::Image {
                    alt_ids,
                    destination: self.b.copy(destination)?,
                    normalized_destination: self.normalized(destination)?,
                    title: self.b.optional(title.as_deref())?,
                }
            }
            InlineKind::FootnoteReference { label } => {
                let definition_id = self.definitions.get(label).cloned();
                let (ordinal, anchor, definition_anchor) = if definition_id.is_some()
                    && self.rendering_inlines
                {
                    let next = self.note_numbers.len() as u32 + 1;
                    let number = *self.note_numbers.entry(label.clone()).or_insert(next);
                    let backlinks = self.backlinks.entry(label.clone()).or_default();
                    let (definition, anchor) = self.anchors.reference(label, backlinks.len() + 1);
                    self.b.reserve(
                        definition
                            .len()
                            .saturating_add(anchor.len().saturating_mul(2)),
                    )?;
                    backlinks.push(anchor.clone());
                    (Some(number), Some(anchor), Some(definition))
                } else {
                    (None, None, None)
                };
                N::FootnoteReference {
                    label: self.b.copy(label)?,
                    definition_id,
                    ordinal,
                    anchor,
                    definition_anchor,
                }
            }
        };
        self.push(id.clone(), content)?;
        Ok(id)
    }
}
