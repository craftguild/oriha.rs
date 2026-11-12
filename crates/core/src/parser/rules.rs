//! Built-in rule registry, shared descriptors and parsing coordination.
mod headings;
#[allow(unused_imports)]
use headings::*;
mod code_blocks;
#[allow(unused_imports)]
use code_blocks::*;
mod callouts;
#[allow(unused_imports)]
use callouts::*;
mod quotes;
#[allow(unused_imports)]
use quotes::*;
mod lists;
#[allow(unused_imports)]
use lists::*;
mod list_markers;
#[allow(unused_imports)]
use list_markers::*;
mod paragraphs;
#[allow(unused_imports)]
use paragraphs::*;
mod html_blocks;
#[allow(unused_imports)]
use html_blocks::*;
mod references;
#[allow(unused_imports)]
use references::*;
mod tables;
#[allow(unused_imports)]
use tables::*;

#[cfg(test)]
use super::core::parse_document;
use super::cursor::{SourceCursor, SourceLineRef, SourceSpan};
use super::normalize;
use super::traits::{
    BlockDetector, BlockParser, BlockRuleParser, BlockStartKind, ParseContext, ParseResult,
    ParseStart,
};
use crate::ir::{
    BlockKind, BlockNode, CalloutNode, CodeBlockNode, GfmTableNode, HeadingNode, ListItemNode,
    ListNode, MathBlockNode, ParagraphNode, TableAlignment, TableCellNode, TableRowNode,
    TaskListItemNode,
};
use crate::mapping::TextMapping;
use crate::source::{byte_index_for_column, first_nonspace_index_and_column};
use crate::types::{DiagnosticEngine, RenderDiagnostic, Severity};
use std::collections::BTreeMap;

pub(crate) fn default_block_rules(
    features: &crate::types::FeatureFlags,
) -> Vec<Box<dyn BlockRuleParser>> {
    let mut rules: Vec<Box<dyn BlockRuleParser>> =
        vec![Box::new(HeadingRule), Box::new(FencedCodeRule)];

    if features.math {
        rules.push(Box::new(MathBlockRule));
    }
    if features.callouts {
        rules.push(Box::new(CalloutRule));
    }
    rules.extend([
        Box::new(BlockQuoteRule) as Box<dyn BlockRuleParser>,
        Box::new(ThematicBreakRule),
        Box::new(ListRule),
        Box::new(SetextHeadingRule),
        Box::new(IndentedCodeRule),
        Box::new(HtmlBlockRule),
        Box::new(LinkReferenceDefinitionRule),
        Box::new(ParagraphRule),
    ]);
    if features.gfm {
        let paragraph_index = rules.len().saturating_sub(1);
        rules.insert(paragraph_index, Box::new(GfmTableRule));
    }

    rules
}

fn parse_heuristic_line(cursor: &mut SourceCursor<'_>, next_id: &mut usize) -> BlockNode {
    let span = cursor.advance().map_or(SourceSpan::new(0, 0), |line| {
        SourceSpan::new(line.start, line.end)
    });
    BlockNode {
        id: next_block_id(next_id),
        kind: BlockKind::Paragraph(ParagraphNode {
            children: Vec::new(),
        }),
        span,
        attrs: BTreeMap::new(),
        source_mapping: Default::default(),
        diagnostics: Vec::new(),
        children: Vec::new(),
    }
}

struct HeadingRule;

#[derive(Clone, Copy)]
struct AtxHeading {
    level: u8,
    inline_start: usize,
    inline_end: usize,
}

struct FencedCodeRule;

struct MathBlockRule;

#[derive(Clone)]
struct FenceOpen {
    char: char,
    len: usize,
    info: String,
    indent: usize,
}

struct CalloutRule;

#[derive(Clone, Debug)]
struct CalloutOpen {
    kind: String,
    title: Option<String>,
    syntax: CalloutSyntax,
}

#[derive(Clone, Debug)]
enum CalloutSyntax {
    Fenced,
    BlockQuote,
}

struct ParsedCalloutBody {
    end: usize,
    body_span: SourceSpan,
    normalized_source: String,
    source_mapping: Vec<TextMapping>,
    diagnostics: Vec<RenderDiagnostic>,
    children: Vec<BlockNode>,
}

struct BlockQuoteRule;

struct ListRule;

struct ParagraphRule;

struct SetextHeadingRule;

macro_rules! simple_line_rule {
    ($name:ident, $start:ident, $detect:expr, $kind:expr) => {
        struct $name;
        impl BlockDetector for $name {
            fn detect(
                &self,
                cursor: &SourceCursor<'_>,
                _context: &ParseContext,
            ) -> Option<ParseStart> {
                cursor.current().is_some_and($detect).then_some(ParseStart {
                    kind: BlockStartKind::$start,
                })
            }
        }
        impl BlockParser for $name {
            fn parse(
                &self,
                cursor: &mut SourceCursor<'_>,
                _start: ParseStart,
                _context: &mut ParseContext,
                next_id: &mut usize,
            ) -> ParseResult {
                let line = cursor.advance().expect("detected line exists");
                let span = SourceSpan::new(line.start, line.end);
                ParseResult {
                    span,
                    block: BlockNode {
                        id: next_block_id(next_id),
                        kind: $kind,
                        span,
                        attrs: BTreeMap::new(),
                        source_mapping: Default::default(),
                        diagnostics: Vec::new(),
                        children: Vec::new(),
                    },
                }
            }
        }
    };
}

simple_line_rule!(
    ThematicBreakRule,
    ThematicBreak,
    |line: SourceLineRef<'_>| is_thematic_break(line.text),
    BlockKind::ThematicBreak
);

struct IndentedCodeRule;

struct HtmlBlockRule;

struct LinkReferenceDefinitionRule;

struct GfmTableRule;

fn lines_to_span(lines: &[SourceLineRef<'_>]) -> Option<SourceSpan> {
    let first = lines.first()?;
    let last = lines.last()?;
    Some(SourceSpan::new(first.start, last.end))
}

#[derive(Debug, PartialEq, Eq)]
enum ReferenceLabelPrefix<'a> {
    Complete { label: &'a str, rest: &'a str },
    Incomplete,
    Invalid,
}

struct ParsedListMarker {
    indent: usize,
    ordered: bool,
    raw_marker: String,
    content_column: usize,
    content_byte_start: usize,
    has_content: bool,
}

fn skip_line_break(text: &str, at: usize) -> usize {
    if at >= text.len() {
        return at;
    }
    let tail = &text[at..];
    if tail.starts_with("\r\n") {
        at + 2
    } else if tail.starts_with('\n') || tail.starts_with('\r') {
        at + 1
    } else {
        at
    }
}

fn reassign_block_ids(blocks: &mut [BlockNode], next_id: &mut usize) {
    for block in blocks {
        block.id = next_block_id(next_id);
        if !block.children.is_empty() {
            reassign_block_ids(&mut block.children, next_id);
        }
    }
}

fn is_paragraph_line(line: &str, math_enabled: bool) -> bool {
    !line.trim().is_empty() && !starts_non_paragraph_block(line, math_enabled)
}

fn starts_non_paragraph_block(line: &str, math_enabled: bool) -> bool {
    parse_atx_heading(line).is_some()
        || parse_fence_open(line).is_some()
        || (math_enabled && is_math_block_delimiter(line))
        || is_block_quote_start(line)
        || parse_list_marker(line).is_some()
        || is_thematic_break(line)
        || is_indented_code_start(line)
        || is_html_block_start(line)
        || is_link_reference_definition(line)
}

fn interrupts_paragraph_continuation(line: &str, math_enabled: bool) -> bool {
    parse_atx_heading(line).is_some()
        || parse_fence_open(line).is_some()
        || (math_enabled && is_math_block_delimiter(line))
        || is_block_quote_start(line)
        || parse_list_marker(line).is_some_and(|marker| list_marker_interrupts_paragraph(&marker))
        || is_thematic_break(line)
        || is_html_block_paragraph_interrupt(line)
}

fn list_marker_interrupts_paragraph(marker: &ParsedListMarker) -> bool {
    if !marker.has_content {
        return false;
    }
    if !marker.ordered {
        return true;
    }
    marker
        .raw_marker
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>()
        == "1"
}

fn is_html_block_paragraph_interrupt(line: &str) -> bool {
    if leading_spaces(line) > 3 {
        return false;
    }
    let trimmed = line.trim_start().to_ascii_lowercase();
    if trimmed.starts_with("<script")
        || trimmed.starts_with("<style")
        || trimmed.starts_with("<pre")
        || trimmed.starts_with("<!--")
        || trimmed.starts_with("<?")
        || trimmed.starts_with("<![cdata[")
    {
        return true;
    }
    html_tag_name(&trimmed).is_some_and(is_commonmark_html_block_tag)
}

fn warning(
    code: &str,
    message: &str,
    _source: SourceSpan,
    _block_id: Option<String>,
) -> RenderDiagnostic {
    RenderDiagnostic {
        severity: Severity::Warning,
        code: code.to_string(),
        message: message.to_string(),
        source_range: None,
        block_id: None,
        engine: DiagnosticEngine::Core,
    }
}

fn leading_spaces(line: &str) -> usize {
    let mut columns = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => columns += 1,
            '\t' => columns += 4 - (columns % 4),
            _ => break,
        }
        if columns >= 4 {
            break;
        }
    }
    columns
}

fn next_block_id(next_id: &mut usize) -> String {
    let id = next_id.to_string();
    *next_id += 1;
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EngineLimits, FeatureFlags};

    fn parse_with_math(source: &str, math: bool) -> crate::ir::Document {
        let features = FeatureFlags {
            math,
            ..FeatureFlags::default()
        };
        parse_document(source, &features, EngineLimits::default())
    }

    #[test]
    fn parses_display_math_block_when_enabled() {
        let doc = parse_with_math("intro\n$$\na + b\n$$\n\n~~~latex\nc + d\n~~~", true);
        assert_eq!(doc.blocks.len(), 3);
        assert!(matches!(&doc.blocks[0].kind, BlockKind::Paragraph(_)));
        assert!(matches!(&doc.blocks[1].kind, BlockKind::MathBlock(node)
                if node.source.as_deref() == Some("a + b\n")
                    && node.display
                    && node.closed));
        assert!(matches!(&doc.blocks[2].kind, BlockKind::FencedCode(node)
                if node.language.as_deref() == Some("latex")));
    }

    #[test]
    fn leaves_dollar_block_as_paragraph_when_math_disabled() {
        let doc = parse_with_math("$$\na + b\n$$", false);
        assert_eq!(doc.blocks.len(), 1);
        assert!(matches!(&doc.blocks[0].kind, BlockKind::Paragraph(_)));
    }

    #[test]
    fn records_unclosed_display_math_block() {
        let doc = parse_with_math("$$\na + b", true);
        assert_eq!(doc.blocks.len(), 1);
        assert!(matches!(&doc.blocks[0].kind, BlockKind::MathBlock(node)
                if node.source.as_deref() == Some("a + b")
                    && node.display
                    && !node.closed));
    }
}
