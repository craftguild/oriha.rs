//! Block parser state, container progression and document assembly.
mod inline_population;
#[allow(unused_imports)]
use inline_population::*;
mod footnotes;
#[allow(unused_imports)]
use footnotes::*;
mod references;
#[allow(unused_imports)]
use references::*;
mod detection;
#[allow(unused_imports)]
use detection::*;

use super::normalize;
use crate::ir::{
    BlockKind, BlockNode, Document, FootnoteDefinition, InlineKind, InlineNode, LinkReference,
    ParagraphNode,
};
use crate::mapping::MappedText;
use crate::parser::inline::parse_mapped_inlines;
use crate::parser::rules;
use crate::parser::traits::{BlockRuleParser, ParseContext, ParseStart};
use crate::source::{SourceCursor, SourceDocument, SourceSpan};
use crate::types::{DiagnosticEngine, EngineLimits, FeatureFlags, RenderDiagnostic, Severity};
use std::collections::BTreeMap;

pub fn parse_document(markdown: &str, features: &FeatureFlags, limits: EngineLimits) -> Document {
    super::probe_block_parse();
    let context = ParseContext {
        features: features.clone(),
        limits,
        depth: 0,
        omissions: Default::default(),
    };
    let mut document = parse_document_with_context(markdown, &context);
    document.diagnostics.extend(context.omissions.take());
    document
}

pub(super) fn parse_document_with_context(markdown: &str, context: &ParseContext) -> Document {
    let source = SourceDocument::new(markdown.to_string());
    let mut state = BlockParserState::new(&source, context.clone());
    state.parse()
}

struct BlockParserState<'a> {
    source: &'a SourceDocument,
    context: ParseContext,
    features: FeatureFlags,
    limits: EngineLimits,
    line_index: usize,
    container_stack: Vec<ContainerFrame>,
    pending_paragraph: Option<PendingParagraph>,
    can_start_definition: bool,
    fenced: Option<(char, usize)>,
    next_id: usize,
    blocks: Vec<BlockNode>,
    diagnostics: Vec<RenderDiagnostic>,
    link_references: BTreeMap<String, LinkReference>,
    footnotes: Vec<FootnoteDefinitionSource>,
}

impl<'a> BlockParserState<'a> {
    fn new(source: &'a SourceDocument, context: ParseContext) -> Self {
        Self {
            source,
            features: context.features.clone(),
            limits: context.limits,
            context,
            line_index: 0,
            container_stack: vec![ContainerFrame::Document],
            pending_paragraph: None,
            can_start_definition: true,
            fenced: None,
            next_id: 1,
            blocks: Vec::new(),
            diagnostics: Vec::new(),
            link_references: BTreeMap::new(),
            footnotes: Vec::new(),
        }
    }

    fn parse(&mut self) -> Document {
        self.collect_link_references();
        self.parse_blocks();
        populate_block_inlines(
            &MappedText::identity(self.source.text(), 0),
            &self.link_references,
            &self.features,
            &mut self.blocks,
        );
        let footnotes =
            populate_footnote_inlines(&self.footnotes, &self.link_references, &self.features);

        Document {
            blocks: std::mem::take(&mut self.blocks),
            diagnostics: std::mem::take(&mut self.diagnostics),
            assets: Vec::new(),
            link_references: self.link_references.clone(),
            footnotes,
        }
    }

    fn collect_link_references(&mut self) {
        while self.line_index < self.source.line_count() {
            let before = self.line_index;
            self.incorporate_current_line();
            debug_assert!(
                self.line_index > before,
                "block parser incorporation must advance the source position"
            );
            if self.line_index <= before {
                self.line_index = before + 1;
            }
        }
    }

    fn parse_blocks(&mut self) {
        let mut cursor = SourceCursor::new(self.source);
        let rules = rules::default_block_rules(&self.features);
        let mut context = self.context.clone();

        while !cursor.is_eof() {
            if cursor
                .current()
                .is_some_and(|line| line.text.trim().is_empty())
            {
                cursor.skip_blank_lines();
                continue;
            }
            if cursor.is_eof() {
                break;
            }

            if let Some(line) = cursor.current() {
                if self.features.gfm {
                    if let Some((_, _, consumed)) =
                        parse_footnote_definition_at(self.source, cursor.position(), line.text)
                    {
                        for _ in 0..consumed {
                            let _ = cursor.advance();
                        }
                        continue;
                    }
                }
                if let Some((_, _, consumed)) =
                    parse_link_reference_definition_at(self.source, cursor.position(), line.text)
                {
                    for _ in 0..consumed {
                        let _ = cursor.advance();
                    }
                    continue;
                }
            }

            if self.blocks.len() >= self.limits.max_blocks {
                context.omissions.borrow_mut().push(limit_diagnostic(
                    self.source,
                    "engine.max_blocks_exceeded",
                    "Maximum block count exceeded. Remaining blocks were skipped.",
                    SourceSpan::new(0, 0),
                    None,
                ));
                break;
            }

            let block =
                if let Some((rule_index, start)) = detect_next_block(&cursor, &rules, &context) {
                    parse_detected_block(
                        &mut cursor,
                        &rules,
                        rule_index,
                        start,
                        &mut context,
                        &mut self.next_id,
                    )
                } else {
                    parse_fallback_paragraph(&mut cursor, &mut self.next_id)
                };
            self.diagnostics.extend(block.diagnostics.iter().cloned());
            self.blocks.push(block);
        }
    }

    fn incorporate_current_line(&mut self) {
        let Some(line) = self.source.line(self.line_index) else {
            return;
        };
        let text = line.text;
        let definition_line = self.apply_container_continuation(text);

        if let Some((marker, len)) = self.fenced {
            if is_fence_close(definition_line, marker, len) {
                self.fenced = None;
            }
            self.pending_paragraph = None;
            self.line_index += 1;
            return;
        }
        if definition_line.trim().is_empty() {
            self.pending_paragraph = None;
            self.can_start_definition = true;
            self.line_index += 1;
            return;
        }
        if let Some((marker, len)) = fence_open(definition_line) {
            self.fenced = Some((marker, len));
            self.pending_paragraph = None;
            self.can_start_definition = false;
            self.line_index += 1;
            return;
        }
        if leading_columns(definition_line) >= 4 {
            self.pending_paragraph = None;
            self.can_start_definition = false;
            self.line_index += 1;
            return;
        }

        if self.can_start_definition {
            if self.features.gfm {
                if let Some((label, content, consumed)) =
                    parse_footnote_definition_at(self.source, self.line_index, definition_line)
                {
                    if !self
                        .footnotes
                        .iter()
                        .any(|footnote| footnote.label == label)
                    {
                        self.footnotes
                            .push(FootnoteDefinitionSource { label, content });
                    }
                    self.pending_paragraph = None;
                    self.can_start_definition = true;
                    self.line_index += consumed;
                    return;
                }
            }
            if let Some((label, reference, consumed)) =
                parse_link_reference_definition_at(self.source, self.line_index, definition_line)
            {
                self.link_references.entry(label).or_insert(reference);
                self.pending_paragraph = None;
                self.can_start_definition = true;
                self.line_index += consumed;
                return;
            }
        }

        let starts_leaf = starts_leaf_block(definition_line.trim_start());
        if starts_leaf {
            self.pending_paragraph = None;
            self.can_start_definition = true;
        } else {
            let start_line = self
                .pending_paragraph
                .as_ref()
                .map_or(self.line_index, |paragraph| paragraph.start_line);
            self.pending_paragraph = Some(PendingParagraph { start_line });
            self.can_start_definition = false;
        }
        self.line_index += 1;
    }

    fn apply_container_continuation<'line>(&mut self, text: &'line str) -> &'line str {
        let mut remaining = text;
        let mut depth = 0usize;

        while let Some(stripped) = strip_blockquote_prefix(remaining) {
            depth += 1;
            remaining = stripped;
        }

        self.container_stack.truncate(1);
        self.container_stack
            .extend((0..depth).map(|_| ContainerFrame::BlockQuote));

        remaining
    }
}

#[derive(Clone, Debug)]
struct FootnoteDefinitionSource {
    label: String,
    content: MappedText,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ContainerFrame {
    Document,
    BlockQuote,
}

#[derive(Clone, Debug)]
struct PendingParagraph {
    start_line: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum ReferenceLabelPrefix<'a> {
    Complete { label: &'a str, rest: &'a str },
    Incomplete,
    Invalid,
}

use crate::character_reference::decode_markdown;
