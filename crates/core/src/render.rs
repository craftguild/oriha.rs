use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use oriha_i18n::{messages as i18n_messages, resolve_locale, I18nLocale};
use oriha_latex::{LatexRenderError, LatexRenderMode, RenderedLatex};
use oriha_mermaid::{
    DiagramStyle as MermaidDiagramStyle, Mermaid, MermaidError, MermaidMeasurementRequest,
    MermaidMeasurementResult, MermaidOptions, MermaidSvg,
};
use oriha_plantuml::{
    DiagramStyle as PlantUmlDiagramStyle, InlineTextMeasurementRequest,
    InlineTextMeasurementResult, InlineTextMeasurementStyle, InlineTextRunMeasurement, PlantUml,
    PlantUmlError, PlantUmlOptions, TextMeasurer,
};

use crate::blocks::html_budget::HtmlBuffer;
use crate::html::{escape_attr, escape_text};
use crate::ir::{
    BlockKind, BlockNode, CalloutNode, CodeBlockNode, Document, FootnoteDefinition, GfmTableNode,
    InlineKind, InlineNode, LinkReference, MathBlockNode, TableAlignment, TableCellNode,
};
use crate::parser::{decode_html_entities, decode_markdown, parse_document, parse_inlines};
use crate::patch::{self, AssignedPatchBlock, RenderedPatchBlock};
use crate::source::SourceDocument;
use crate::syntax_highlight::highlight_lines;
use crate::theme::{render_css as render_theme_css, resolve_body_font_size_px, resolve_theme};
use crate::types::{
    AppearanceOptions, AssetKind, ColorScheme, DiagnosticEngine, EngineLimits, ExportHtmlInput,
    ExportHtmlOutput, FeatureFlags, PreparedHtmlExport, PreparedMarkdownLayout,
    PreparedMarkdownPatch, RenderAsset, RenderBlockKind, RenderDiagnostic, RenderInput,
    RenderIssueKind, RenderMode, RenderOutput, RenderPatchFallbackReason, RenderPatchInput,
    RenderPatchOutput, RenderSnapshot, RenderStats, RenderedBlock, Severity, TextMeasurementBatch,
    TextMeasurementRequest, TextMeasurementResult, TextMeasurementResults, TextMeasurementRun,
    TextMeasurementStyle, UnsupportedKind,
};

mod block_output;
pub use block_output::{derive_blocks, derive_parsed_blocks};

mod entry_points;
pub use entry_points::*;
mod prepared;
pub use prepared::*;
mod measurements;
use measurements::*;
mod errors;
use errors::*;
mod extensions;
pub use extensions::*;
mod engine;
pub use engine::*;
mod parsed;
pub use parsed::*;
mod finalize;
use finalize::*;
mod context;
pub use context::*;
mod traversal;
use traversal::*;
mod block_renderer;
use block_renderer::*;
mod tables_lists;
use tables_lists::*;
mod code;
use code::*;
mod math;
use math::*;
mod diagrams;
use diagrams::*;
mod containers;
pub(crate) use containers::normalize_fenced_code_text;
use containers::*;
mod inline;
use inline::*;
mod footnote_html;
use footnote_html::*;
mod footnote_ids;
use footnote_ids::*;
mod images;
use images::*;
mod image_presentation;
pub use image_presentation::*;
mod presented_image;
use presented_image::*;
mod inline_text;
use inline_text::*;
mod references;
pub(crate) use measurements::parse_text_measurement_results;
pub use references::*;
#[cfg(test)]
mod spec_html;
#[cfg(test)]
pub(crate) use spec_html::*;
#[cfg(test)]
mod image_presentation_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod references_tests;

#[cfg(test)]
mod footnote_ids_tests;
