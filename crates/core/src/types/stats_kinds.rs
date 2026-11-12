use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderBlockKind {
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
}

impl RenderBlockKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Paragraph => "paragraph",
            Self::Heading => "heading",
            Self::FencedCode => "fencedCode",
            Self::IndentedCode => "indentedCode",
            Self::MathBlock => "mathBlock",
            Self::ThematicBreak => "thematicBreak",
            Self::HtmlBlock => "htmlBlock",
            Self::LinkReferenceDefinition => "linkReferenceDefinition",
            Self::BlockQuote => "blockQuote",
            Self::List => "list",
            Self::ListItem => "listItem",
            Self::GfmTable => "gfmTable",
            Self::BlankLine => "blankLine",
            Self::Callout => "callout",
            Self::ForeignHtml => "foreignHtml",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockedFeatureKind {
    SyntaxTheme,
    HeadingFont,
    BodyFont,
    CodeFont,
    Callout,
    Mermaid,
    Plantuml,
    Math,
}

impl LockedFeatureKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SyntaxTheme => "syntaxTheme",
            Self::HeadingFont => "headingFont",
            Self::BodyFont => "bodyFont",
            Self::CodeFont => "codeFont",
            Self::Callout => "callout",
            Self::Mermaid => "mermaid",
            Self::Plantuml => "plantuml",
            Self::Math => "math",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsupportedKind {
    PlantumlDiagram,
    UnknownUnsupported,
}

impl UnsupportedKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlantumlDiagram => "plantumlDiagram",
            Self::UnknownUnsupported => "unknownUnsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderIssueKind {
    UnclosedFencedCodeBlock,
    UnclosedCallout,
    UnclosedCodeBlock,
    MathRenderFailed,
    MermaidRenderFailed,
    PlantumlRenderFailed,
    PlantumlUnsupportedDiagram,
    MaxBlocksExceeded,
    MaxSourceBytesExceeded,
    MaxOutputBytesExceeded,
    InvalidInputJson,
    InvalidPreparedJson,
    InvalidTextMetricsJson,
    UnknownError,
}

impl RenderIssueKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnclosedFencedCodeBlock => "unclosedFencedCodeBlock",
            Self::UnclosedCallout => "unclosedCallout",
            Self::UnclosedCodeBlock => "unclosedCodeBlock",
            Self::MathRenderFailed => "mathRenderFailed",
            Self::MermaidRenderFailed => "mermaidRenderFailed",
            Self::PlantumlRenderFailed => "plantumlRenderFailed",
            Self::PlantumlUnsupportedDiagram => "plantumlUnsupportedDiagram",
            Self::MaxBlocksExceeded => "maxBlocksExceeded",
            Self::MaxSourceBytesExceeded => "maxSourceBytesExceeded",
            Self::MaxOutputBytesExceeded => "maxOutputBytesExceeded",
            Self::InvalidInputJson => "invalidInputJson",
            Self::InvalidPreparedJson => "invalidPreparedJson",
            Self::InvalidTextMetricsJson => "invalidTextMetricsJson",
            Self::UnknownError => "unknownError",
        }
    }

    pub fn from_diagnostic_code(code: &str, severity: Severity) -> Option<Self> {
        match code {
            "fenced_code_block.unclosed" => Some(Self::UnclosedFencedCodeBlock),
            "callout.unclosed" => Some(Self::UnclosedCallout),
            "code_block.unclosed" => Some(Self::UnclosedCodeBlock),
            "math.render_failed" => Some(Self::MathRenderFailed),
            "mermaid.render_failed" => Some(Self::MermaidRenderFailed),
            "plantuml.render_failed" => Some(Self::PlantumlRenderFailed),
            "plantuml.unsupported_diagram" => Some(Self::PlantumlUnsupportedDiagram),
            "engine.max_blocks_exceeded" => Some(Self::MaxBlocksExceeded),
            "engine.max_source_bytes_exceeded" => Some(Self::MaxSourceBytesExceeded),
            "engine.max_output_bytes_exceeded" => Some(Self::MaxOutputBytesExceeded),
            "input.invalid_json" => Some(Self::InvalidInputJson),
            "prepared.invalid_json" => Some(Self::InvalidPreparedJson),
            "text_metrics.invalid_json" => Some(Self::InvalidTextMetricsJson),
            _ if matches!(severity, Severity::Error) => Some(Self::UnknownError),
            _ => None,
        }
    }
}
