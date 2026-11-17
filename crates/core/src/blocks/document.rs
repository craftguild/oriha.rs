//! Version 2: one typed pool, with structural ownership expressed by ordered IDs.
use super::{BlockOutputMode, BlockOutputStatus, BlockTableAlignment, BlockTrustLevel};
use crate::RenderDiagnostic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StructuredOptions {
    pub syntax: bool,
    pub math: bool,
    pub mermaid: bool,
    pub plantuml: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredBlockOutput {
    pub schema_version: u32,
    pub mode: BlockOutputMode,
    pub status: BlockOutputStatus,
    pub root_ids: Vec<String>,
    pub nodes: Vec<DocumentNode>,
    pub reference_ids: Vec<String>,
    pub footnote_ids: Vec<String>,
    pub fonts: Vec<DocumentFont>,
    pub diagnostics: Vec<RenderDiagnostic>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentNode {
    pub id: String,
    #[serde(flatten)]
    pub content: DocumentNodeKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DocumentNodeKind {
    Document {
        child_ids: Vec<String>,
    },
    Paragraph {
        child_ids: Vec<String>,
        tight: bool,
    },
    Heading {
        child_ids: Vec<String>,
        level: u8,
        anchor: String,
    },
    FencedCode {
        source: String,
        language: Option<String>,
        info: String,
        closed: bool,
        indent: usize,
        highlighting: Option<CodeHighlighting>,
        svg: Option<SvgContent>,
    },
    IndentedCode {
        source: String,
        language: Option<String>,
        info: String,
        closed: bool,
        indent: usize,
        highlighting: Option<CodeHighlighting>,
    },
    MathBlock {
        source: String,
        display: bool,
        closed: bool,
        svg: Option<SvgContent>,
    },
    ThematicBreak {},
    HtmlBlock {
        source: String,
    },
    LinkReferenceDefinition {
        source: String,
    },
    BlockQuote {
        child_ids: Vec<String>,
    },
    List {
        child_ids: Vec<String>,
        ordered: bool,
        loose: bool,
        marker: String,
        start: Option<String>,
    },
    ListItem {
        child_ids: Vec<String>,
        marker: String,
        checked: Option<bool>,
    },
    GfmTable {
        alignments: Vec<BlockTableAlignment>,
        row_ids: Vec<String>,
    },
    TableRow {
        header: bool,
        cell_ids: Vec<String>,
    },
    TableCell {
        child_ids: Vec<String>,
    },
    BlankLine {},
    Callout {
        child_ids: Vec<String>,
        callout_kind: String,
        title: Option<String>,
    },
    ForeignHtml {
        source: String,
        origin: String,
        trust_level: BlockTrustLevel,
    },
    Text {
        text: String,
    },
    Emoji {
        text: String,
        shortcode: String,
    },
    Code {
        source: String,
    },
    Math {
        source: String,
        svg: Option<SvgContent>,
    },
    Html {
        source: String,
        tagfilter: bool,
    },
    SoftBreak {},
    HardBreak {},
    Annotation {
        child_ids: Vec<String>,
        annotation_kind: String,
        annotation_id: String,
        classes: Vec<String>,
        data: BTreeMap<String, String>,
    },
    Emphasis {
        child_ids: Vec<String>,
    },
    Strong {
        child_ids: Vec<String>,
    },
    Strikethrough {
        child_ids: Vec<String>,
    },
    Link {
        child_ids: Vec<String>,
        destination: String,
        normalized_destination: String,
        title: Option<String>,
    },
    Image {
        alt_ids: Vec<String>,
        destination: String,
        normalized_destination: String,
        title: Option<String>,
    },
    FootnoteReference {
        label: String,
        definition_id: Option<String>,
        ordinal: Option<u32>,
        anchor: Option<String>,
        definition_anchor: Option<String>,
    },
    ReferenceDefinition {
        label: String,
        destination: String,
        normalized_destination: String,
        title: Option<String>,
    },
    FootnoteDefinition {
        visible: bool,
        label: String,
        child_ids: Vec<String>,
        anchor: String,
        ordinal: Option<u32>,
        backlink_ids: Vec<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeHighlighting {
    /// Splitting on LF preserves empty lines and the final empty terminal line.
    pub lines: Vec<CodeLine>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeLine {
    pub tokens: Vec<SyntaxToken>,
}

pub use crate::syntax_highlight::{SyntaxToken, TokenKind};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgContent {
    pub markup: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub baseline: Option<f32>,
    pub font_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFont {
    pub id: String,
    pub family: String,
    pub mime_type: String,
    pub encoding: FontEncoding,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FontEncoding {
    Base64,
}
