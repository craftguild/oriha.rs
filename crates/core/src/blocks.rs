//! Owned, versioned block output. This schema is independent of document JSON.

use serde::{Deserialize, Serialize};

use crate::{RenderAsset, RenderDiagnostic};

pub(crate) mod budget;
mod document;
mod graph;
pub(crate) mod html_budget;
pub use document::*;
pub use graph::DocumentValidationError;

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
pub struct RenderedBlockOutput {
    pub schema_version: u32,
    pub mode: BlockOutputMode,
    pub status: BlockOutputStatus,
    pub blocks: Vec<RenderedHtmlBlock>,
    pub assets: Vec<RenderAsset>,
    pub diagnostics: Vec<RenderDiagnostic>,
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
