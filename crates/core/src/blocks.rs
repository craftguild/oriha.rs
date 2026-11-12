//! Owned, versioned block output. This schema is independent of document JSON.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{RenderAsset, RenderDiagnostic};

pub(crate) mod budget;
pub(crate) mod html_budget;
pub(crate) mod inline;
pub(crate) mod structured;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockOutputMode {
    Structured,
    Rendered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockOutputStatus {
    Complete,
    Incomplete,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockOutput {
    Structured(StructuredBlockOutput),
    Rendered(RenderedBlockOutput),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredBlockOutput {
    pub schema_version: u32,
    pub mode: BlockOutputMode,
    pub status: BlockOutputStatus,
    pub blocks: Vec<Option<StructuredBlock>>,
    pub references: BTreeMap<String, StructuredReference>,
    pub footnotes: Vec<StructuredFootnote>,
    pub diagnostics: Vec<RenderDiagnostic>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderedBlockOutput {
    pub schema_version: u32,
    pub mode: BlockOutputMode,
    pub status: BlockOutputStatus,
    pub blocks: Vec<RenderedHtmlBlock>,
    pub assets: Vec<RenderAsset>,
    pub diagnostics: Vec<RenderDiagnostic>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredBlock {
    pub id: String,
    pub text: Option<String>,
    #[serde(flatten)]
    pub content: StructuredBlockProperties,
    pub children: Vec<Option<StructuredBlock>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "properties",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum StructuredBlockProperties {
    Document {},
    Paragraph {
        rich_text: Vec<Option<RichText>>,
        tight: bool,
    },
    Heading {
        level: u8,
        rich_text: Vec<Option<RichText>>,
    },
    FencedCode {
        language: Option<String>,
        info: String,
        closed: bool,
        indent: usize,
    },
    IndentedCode {
        language: Option<String>,
        info: String,
        closed: bool,
        indent: usize,
    },
    MathBlock {
        display: bool,
        closed: bool,
    },
    ThematicBreak {},
    HtmlBlock {},
    LinkReferenceDefinition {
        source: String,
    },
    BlockQuote {},
    List {
        ordered: bool,
        loose: bool,
        marker: String,
        start: Option<String>,
    },
    ListItem {
        marker: String,
        checked: Option<bool>,
    },
    GfmTable {
        alignments: Vec<BlockTableAlignment>,
        header: Vec<StructuredTableCell>,
        rows: Vec<Vec<StructuredTableCell>>,
    },
    BlankLine {},
    Callout {
        kind: String,
        title: Option<String>,
    },
    ForeignHtml {
        origin: String,
        trust_level: BlockTrustLevel,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockTableAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockTrustLevel {
    Untrusted,
    Sanitized,
    Trusted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredTableCell {
    pub text: String,
    pub rich_text: Vec<Option<RichText>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructuredReference {
    pub destination: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredFootnote {
    pub label: String,
    pub rich_text: Vec<Option<RichText>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichText {
    pub text: Option<String>,
    #[serde(flatten)]
    pub content: RichTextProperties,
    pub children: Vec<Option<RichText>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "properties",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RichTextProperties {
    Annotation {
        kind: String,
        id: String,
        classes: Vec<String>,
        data: BTreeMap<String, String>,
    },
    Text {},
    Emoji {
        shortcode: String,
    },
    Code {},
    Math {},
    Html {
        tagfilter: bool,
    },
    SoftBreak {},
    HardBreak {},
    Emphasis {},
    Strong {},
    Strikethrough {},
    Link {
        url: String,
        title: Option<String>,
    },
    Image {
        url: String,
        title: Option<String>,
    },
    FootnoteReference {
        label: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HtmlBlockKind {
    Document,
    Paragraph,
    Heading,
    FencedCode,
    IndentedCode,
    MathBlock,
    ThematicBreak,
    HtmlBlock,
    LinkReferenceDefinition,
    BlockQuote,
    List,
    ListItem,
    GfmTable,
    BlankLine,
    Callout,
    ForeignHtml,
    Footnotes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockNullReason {
    BlankLine,
    ReferenceDefinition,
    Unsupported,
}

/// A renderer's explicit decision; empty HTML is still HTML.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockEmission {
    Html,
    Null(BlockNullReason),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderedHtmlBlock {
    pub id: String,
    pub kind: HtmlBlockKind,
    #[serde(flatten)]
    pub content: RenderedBlockContent,
}

/// Rust prevents an HTML fragment from carrying a Null reason.
#[derive(Clone, Debug)]
pub enum RenderedBlockContent {
    Html(String),
    Null(BlockNullReason),
}

impl RenderedHtmlBlock {
    pub fn html(&self) -> Option<&str> {
        match &self.content {
            RenderedBlockContent::Html(s) => Some(s),
            RenderedBlockContent::Null(_) => None,
        }
    }
}

impl Serialize for RenderedBlockContent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut value = serializer.serialize_struct("RenderedBlockContent", 2)?;
        match self {
            Self::Html(html) => {
                value.serialize_field("html", html)?;
                value.serialize_field("nullReason", &Option::<BlockNullReason>::None)?;
            }
            Self::Null(reason) => {
                value.serialize_field("html", &Option::<String>::None)?;
                value.serialize_field("nullReason", reason)?;
            }
        }
        value.end()
    }
}

impl<'de> Deserialize<'de> for RenderedBlockContent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Fields {
            html: Option<String>,
            null_reason: Option<BlockNullReason>,
        }
        let f = Fields::deserialize(deserializer)?;
        match (f.html, f.null_reason) {
            (Some(html), None) => Ok(Self::Html(html)),
            (None, Some(reason)) => Ok(Self::Null(reason)),
            _ => Err(serde::de::Error::custom(
                "expected HTML or an explicit Null reason",
            )),
        }
    }
}
