use super::budget::{Budget, DerivationError};
use super::{RichText, RichTextProperties as P};
use crate::ir::{InlineKind, InlineNode};

type Result<T> = std::result::Result<T, DerivationError>;

pub(crate) fn inlines(nodes: &[InlineNode], budget: &mut Budget) -> Result<Vec<Option<RichText>>> {
    nodes
        .iter()
        .map(|node| inline(node, budget).map(Some))
        .collect()
}

fn inline(node: &InlineNode, b: &mut Budget) -> Result<RichText> {
    b.reserve(32)?;
    let mut text = None;
    let mut children = Vec::new();
    let content = match &node.kind {
        InlineKind::Text(s) => {
            text = Some(b.copy(s)?);
            P::Text {}
        }
        InlineKind::Emoji { shortcode, emoji } => {
            text = Some(b.copy(emoji)?);
            P::Emoji {
                shortcode: b.copy(shortcode)?,
            }
        }
        InlineKind::Code(s) => {
            text = Some(b.copy(s)?);
            P::Code {}
        }
        InlineKind::Math { source } => {
            text = Some(b.copy(source)?);
            P::Math {}
        }
        InlineKind::Html { raw, tagfilter } => {
            text = Some(b.copy(raw)?);
            P::Html {
                tagfilter: *tagfilter,
            }
        }
        InlineKind::SoftBreak => {
            text = Some(b.copy("\n")?);
            P::SoftBreak {}
        }
        InlineKind::HardBreak => {
            text = Some(b.copy("\n")?);
            P::HardBreak {}
        }
        InlineKind::Annotation {
            annotation,
            children: c,
        } => {
            children = inlines(c, b)?;
            P::Annotation {
                kind: b.copy(&annotation.kind)?,
                id: b.copy(&annotation.id)?,
                classes: annotation
                    .attributes
                    .classes
                    .iter()
                    .map(|s| b.copy(s))
                    .collect::<Result<_>>()?,
                data: annotation
                    .attributes
                    .data
                    .iter()
                    .map(|(k, v)| Ok((b.copy(k)?, b.copy(v)?)))
                    .collect::<Result<_>>()?,
            }
        }
        InlineKind::Emphasis { children: c } => {
            children = inlines(c, b)?;
            P::Emphasis {}
        }
        InlineKind::Strong { children: c } => {
            children = inlines(c, b)?;
            P::Strong {}
        }
        InlineKind::Strikethrough { children: c } => {
            children = inlines(c, b)?;
            P::Strikethrough {}
        }
        InlineKind::Link {
            destination,
            title,
            children: c,
        } => {
            children = inlines(c, b)?;
            P::Link {
                url: b.copy(destination)?,
                title: b.optional(title.as_deref())?,
            }
        }
        InlineKind::Image {
            destination,
            title,
            alt,
        } => {
            children = inlines(alt, b)?;
            P::Image {
                url: b.copy(destination)?,
                title: b.optional(title.as_deref())?,
            }
        }
        InlineKind::FootnoteReference { label } => P::FootnoteReference {
            label: b.copy(label)?,
        },
    };
    Ok(RichText {
        text,
        content,
        children,
    })
}

pub(crate) fn plain(nodes: &[InlineNode], b: &mut Budget) -> Result<String> {
    fn append(nodes: &[InlineNode], out: &mut String, b: &mut Budget) -> Result<()> {
        for node in nodes {
            let text = match &node.kind {
                InlineKind::Text(s) | InlineKind::Code(s) => s.as_str(),
                InlineKind::Emoji { emoji, .. } => emoji,
                InlineKind::Math { source } => source,
                InlineKind::Html { raw, .. } => raw,
                InlineKind::SoftBreak | InlineKind::HardBreak => "\n",
                InlineKind::Annotation { children, .. }
                | InlineKind::Emphasis { children }
                | InlineKind::Strong { children }
                | InlineKind::Strikethrough { children }
                | InlineKind::Link { children, .. } => {
                    append(children, out, b)?;
                    continue;
                }
                InlineKind::Image { alt, .. } => {
                    append(alt, out, b)?;
                    continue;
                }
                InlineKind::FootnoteReference { label } => {
                    b.reserve(label.len().saturating_add(3))?;
                    out.push_str("[^");
                    out.push_str(label);
                    out.push(']');
                    continue;
                }
            };
            b.reserve(text.len())?;
            out.push_str(text);
        }
        Ok(())
    }
    let mut out = String::new();
    append(nodes, &mut out, b)?;
    Ok(out)
}
