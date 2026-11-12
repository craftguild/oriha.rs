use crate::source::SourceSpan;
use crate::types::{RenderAsset, RenderDiagnostic, SourceRange};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Document {
    pub blocks: Vec<BlockNode>,
    pub diagnostics: Vec<RenderDiagnostic>,
    pub assets: Vec<RenderAsset>,
    pub link_references: BTreeMap<String, LinkReference>,
    pub footnotes: Vec<FootnoteDefinition>,
}

#[derive(Clone, Debug)]
pub struct BlockNode {
    pub id: String,
    pub kind: BlockKind,
    pub span: SourceSpan,
    pub attrs: BTreeMap<String, String>,
    pub(crate) source_mapping: BlockSourceMapping,
    pub diagnostics: Vec<RenderDiagnostic>,
    pub children: Vec<BlockNode>,
}

#[derive(Clone, Debug)]
pub struct LinkReference {
    pub destination: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug)]
pub struct InlineNode {
    pub kind: InlineKind,
    /// Text/emoji/code mappings index their visible value. Container and syntax
    /// node mappings index the parsed syntax extent; traversal exposes text only.
    pub source_mapping: Vec<crate::mapping::TextMapping>,
}

impl InlineNode {
    /// Construct application-generated content without invented source positions.
    pub fn generated(kind: InlineKind) -> Self {
        let len = match &kind {
            InlineKind::Text(s) | InlineKind::Code(s) => s.len(),
            InlineKind::Emoji { emoji, .. } => emoji.len(),
            _ => 0,
        };
        Self {
            kind,
            source_mapping: if len == 0 {
                Vec::new()
            } else {
                vec![crate::mapping::TextMapping::generated(0..len)]
            },
        }
    }
}

impl From<InlineKind> for InlineNode {
    fn from(kind: InlineKind) -> Self {
        Self::generated(kind)
    }
}

/// Parser correspondence for source strings retained by block normalization.
#[derive(Clone, Debug, Default)]
pub(crate) struct BlockSourceMapping {
    pub normalized: Vec<crate::mapping::TextMapping>,
    pub inline: Vec<crate::mapping::TextMapping>,
}

#[derive(Clone, Debug)]
pub enum InlineKind {
    Annotation {
        annotation: crate::annotations::AnnotationMetadata,
        children: Vec<InlineNode>,
    },
    Text(String),
    Emoji {
        shortcode: String,
        emoji: String,
    },
    Code(String),
    Math {
        source: String,
    },
    Html {
        raw: String,
        tagfilter: bool,
    },
    SoftBreak,
    HardBreak,
    Emphasis {
        children: Vec<InlineNode>,
    },
    Strong {
        children: Vec<InlineNode>,
    },
    Strikethrough {
        children: Vec<InlineNode>,
    },
    Link {
        destination: String,
        title: Option<String>,
        children: Vec<InlineNode>,
    },
    Image {
        destination: String,
        title: Option<String>,
        alt: Vec<InlineNode>,
    },
    FootnoteReference {
        label: String,
    },
}

#[derive(Clone, Debug)]
pub struct FootnoteDefinition {
    pub label: String,
    pub children: Vec<InlineNode>,
}

#[derive(Clone, Debug)]
pub enum BlockKind {
    Document,
    Paragraph(ParagraphNode),
    Heading(HeadingNode),
    FencedCode(CodeBlockNode),
    IndentedCode(CodeBlockNode),
    MathBlock(MathBlockNode),
    ThematicBreak,
    HtmlBlock,
    LinkReferenceDefinition,
    BlockQuote,
    List(ListNode),
    ListItem(ListItemNode),
    GfmTable(GfmTableNode),
    BlankLine,
    Callout(CalloutNode),
    ForeignHtml(ForeignHtmlNode),
}

#[derive(Clone, Debug)]
pub struct ParagraphNode {
    pub children: Vec<InlineNode>,
}

#[derive(Clone, Debug)]
pub struct HeadingNode {
    pub level: u8,
    pub inline_span: SourceSpan,
    pub children: Vec<InlineNode>,
}

#[derive(Clone, Debug)]
pub struct CodeBlockNode {
    pub language: Option<String>,
    pub info: String,
    pub body_span: SourceSpan,
    pub closed: bool,
    pub indent: usize,
}

#[derive(Clone, Debug)]
pub struct MathBlockNode {
    pub source_span: SourceSpan,
    pub source: Option<String>,
    pub display: bool,
    pub closed: bool,
}

#[derive(Clone, Debug)]
pub struct ListNode {
    pub ordered: bool,
    pub loose: bool,
    pub marker: String,
}

#[derive(Clone, Debug)]
pub struct ListItemNode {
    pub raw_prefix: String,
    pub content_span: SourceSpan,
    pub task: Option<TaskListItemNode>,
}

#[derive(Clone, Debug)]
pub struct TaskListItemNode {
    pub checked: bool,
}

#[derive(Clone, Debug)]
pub struct GfmTableNode {
    pub alignments: Vec<TableAlignment>,
    pub header: Vec<TableCellNode>,
    pub rows: Vec<TableRowNode>,
}

#[derive(Clone, Debug)]
pub struct TableRowNode {
    pub cells: Vec<TableCellNode>,
}

#[derive(Clone, Debug)]
pub struct TableCellNode {
    pub source: String,
    pub(crate) source_mapping: Vec<crate::mapping::TextMapping>,
    pub children: Vec<InlineNode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug)]
pub struct CalloutNode {
    pub kind: String,
    pub title: Option<String>,
    pub body_span: SourceSpan,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ForeignHtmlNode {
    pub html: String,
    pub origin: String,
    pub trust_level: ForeignHtmlTrustLevel,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ForeignHtmlTrustLevel {
    Untrusted,
    Sanitized,
    Trusted,
}

pub fn block_source_range(block: &BlockNode, source: &str) -> SourceRange {
    let source_doc = crate::source::SourceDocument::new(source.to_string());
    source_doc.to_source_range(block.span)
}

impl BlockNode {
    /// Construct an application-provided block with no parser normalization state.
    pub fn new(id: impl Into<String>, kind: BlockKind, span: SourceSpan) -> Self {
        Self {
            id: id.into(),
            kind,
            span,
            attrs: BTreeMap::new(),
            source_mapping: BlockSourceMapping::default(),
            diagnostics: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl TableCellNode {
    pub fn new(source: impl Into<String>, children: Vec<InlineNode>) -> Self {
        Self {
            source: source.into(),
            source_mapping: Vec::new(),
            children,
        }
    }
}
