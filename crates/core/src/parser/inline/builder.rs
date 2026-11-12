use super::*;

#[derive(Clone, Copy)]
pub(super) enum InlineDelimiterToken {
    EmphasisOpen,
    EmphasisClose,
    StrongOpen,
    StrongClose,
    StrikethroughOpen,
    StrikethroughClose,
}

pub(super) enum InlineToken {
    Node(InlineNode),
    Delimiter(InlineDelimiterToken, std::ops::Range<usize>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum InlineContainerTag {
    Emphasis,
    Strong,
    Strikethrough,
}

// One pending node preserves token-adjacent text merging without retaining a
// second full vector of InlineNode-sized tokens alongside the final tree.
pub(super) struct InlineBuilder<'a> {
    pub(super) root: Vec<InlineNode>,
    pub(super) stack: Vec<(InlineContainerTag, Vec<InlineNode>, usize)>,
    pub(super) pending: Option<InlineToken>,
    pub(super) mapped: &'a MappedText,
}
impl<'a> InlineBuilder<'a> {
    pub(super) fn depth(&self) -> usize {
        self.stack.len()
    }
    pub(super) fn new(mapped: &'a MappedText) -> Self {
        Self {
            root: Vec::new(),
            stack: Vec::new(),
            pending: None,
            mapped,
        }
    }
    pub(super) fn last_mut(&mut self) -> Option<&mut InlineToken> {
        self.pending.as_mut()
    }
    pub(super) fn push(&mut self, token: InlineToken) {
        self.flush();
        match token {
            InlineToken::Node(_) => self.pending = Some(token),
            _ => self.consume(token),
        }
    }
    pub(super) fn flush(&mut self) {
        if let Some(token) = self.pending.take() {
            self.consume(token);
        }
    }
    pub(super) fn consume(&mut self, token: InlineToken) {
        recursion::check_formatting(self.stack.len());
        match token {
            InlineToken::Node(node) => push_inline_node(&mut self.root, &mut self.stack, node),
            InlineToken::Delimiter(InlineDelimiterToken::EmphasisOpen, span) => {
                self.stack
                    .push((InlineContainerTag::Emphasis, Vec::new(), span.start));
            }
            InlineToken::Delimiter(InlineDelimiterToken::StrongOpen, span) => {
                self.stack
                    .push((InlineContainerTag::Strong, Vec::new(), span.start));
            }
            InlineToken::Delimiter(InlineDelimiterToken::StrikethroughOpen, span) => {
                self.stack
                    .push((InlineContainerTag::Strikethrough, Vec::new(), span.start));
            }
            InlineToken::Delimiter(InlineDelimiterToken::EmphasisClose, span) => {
                close_inline_container(
                    &mut self.root,
                    &mut self.stack,
                    InlineContainerTag::Emphasis,
                    self.mapped,
                    span,
                );
            }
            InlineToken::Delimiter(InlineDelimiterToken::StrongClose, span) => {
                close_inline_container(
                    &mut self.root,
                    &mut self.stack,
                    InlineContainerTag::Strong,
                    self.mapped,
                    span,
                );
            }
            InlineToken::Delimiter(InlineDelimiterToken::StrikethroughClose, span) => {
                close_inline_container(
                    &mut self.root,
                    &mut self.stack,
                    InlineContainerTag::Strikethrough,
                    self.mapped,
                    span,
                );
            }
        }
    }
    pub(super) fn finish(mut self) -> Vec<InlineNode> {
        self.flush();
        while let Some((_, children, _)) = self.stack.pop() {
            for child in children {
                push_inline_node(&mut self.root, &mut self.stack, child);
            }
        }
        self.root
    }
}

pub(super) fn push_inline_node(
    root: &mut Vec<InlineNode>,
    stack: &mut [(InlineContainerTag, Vec<InlineNode>, usize)],
    node: InlineNode,
) {
    if let Some((_, children, _)) = stack.last_mut() {
        children.push(node);
    } else {
        root.push(node);
    }
}

pub(super) fn close_inline_container(
    root: &mut Vec<InlineNode>,
    stack: &mut Vec<(InlineContainerTag, Vec<InlineNode>, usize)>,
    tag: InlineContainerTag,
    mapped: &MappedText,
    span: std::ops::Range<usize>,
) {
    let Some((open_tag, children, start)) = stack.pop() else {
        return;
    };
    if open_tag != tag {
        for child in children {
            push_inline_node(root, stack, child);
        }
        return;
    }

    let node = node_at(
        match tag {
            InlineContainerTag::Emphasis => InlineKind::Emphasis { children },
            InlineContainerTag::Strong => InlineKind::Strong { children },
            InlineContainerTag::Strikethrough => InlineKind::Strikethrough { children },
        },
        mapped,
        start..span.end,
    );
    push_inline_node(root, stack, node);
}

pub(super) fn push_text_token(tokens: &mut InlineBuilder<'_>, value: MappedText) {
    if let Some(InlineToken::Node(InlineNode {
        kind: InlineKind::Text(last),
        source_mapping,
    })) = tokens.last_mut()
    {
        let offset = last.len();
        last.push_str(&value.text);
        for mut m in value.mapping {
            m.display = m.display.start + offset..m.display.end + offset;
            push_mapping(source_mapping, m);
        }
    } else {
        tokens.push(InlineToken::Node(InlineNode {
            kind: InlineKind::Text(value.text),
            source_mapping: value.mapping,
        }));
    }
}

pub(super) fn trim_trailing_spaces(tokens: &mut InlineBuilder<'_>) {
    if let Some(InlineToken::Node(InlineNode {
        kind: InlineKind::Text(last),
        source_mapping,
    })) = tokens.last_mut()
    {
        let len = last.trim_end_matches(' ').len();
        last.truncate(len);
        *source_mapping = source_mapping
            .iter()
            .filter_map(|m| m.clipped(0..len))
            .collect();
    }
}

pub(super) fn mapped_subslice(mapped: &MappedText, part: &str) -> MappedText {
    let start = part.as_ptr() as usize - mapped.text.as_ptr() as usize;
    mapped.slice(start..start + part.len())
}

pub(super) fn node_at(
    kind: InlineKind,
    mapped: &MappedText,
    range: std::ops::Range<usize>,
) -> InlineNode {
    let value = match &kind {
        InlineKind::Emoji { emoji, .. } => {
            mapped.transformed(range, emoji, TextProvenance::Normalized)
        }
        InlineKind::Code(code) => mapped.transformed(range, code, TextProvenance::Normalized),
        InlineKind::Math { source } => {
            mapped.transformed(range, source, TextProvenance::Normalized)
        }
        _ => mapped.slice(range),
    };
    InlineNode {
        kind,
        source_mapping: value.mapping,
    }
}

pub(super) fn autolink_at(
    mut kind: InlineKind,
    mapped: &MappedText,
    start: usize,
    consumed: usize,
    label_offset: usize,
) -> InlineNode {
    if let InlineKind::Link { children, .. } = &mut kind {
        for child in children {
            if let InlineKind::Text(text) = &child.kind {
                child.source_mapping = mapped
                    .slice(start + label_offset..start + label_offset + text.len())
                    .mapping;
            }
        }
    }
    node_at(kind, mapped, start..start + consumed)
}
