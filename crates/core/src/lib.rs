pub mod annotations;
pub mod blocks;
mod character_reference;
mod emoji;
mod export_html;
mod footnotes;
mod hook_mapping;
mod html;
pub mod ir;
pub mod mapping;
mod parser;
mod patch;
pub mod render;
pub mod source;
mod syntax_highlight;
mod theme;
mod types;

pub use annotations::{
    AnnotationAttributes, AnnotationError, AnnotationMetadata, RegionId, RegionMapping,
    TextAnnotation, TextRegion,
};
pub use blocks::*;
pub use mapping::{TextMapping, TextProvenance};
pub use render::{
    catch_engine_panic, derive_blocks, derive_parsed_blocks, export_html, export_html_json, parse,
    prepare_html_export, prepare_html_export_json, prepare_markdown_layout,
    prepare_markdown_layout_json, prepare_markdown_patch, prepare_markdown_patch_json, render,
    render_json, render_markdown_patch, render_markdown_patch_json, render_parsed,
    render_prepared_html_export, render_prepared_html_export_json, render_prepared_markdown,
    render_prepared_markdown_json, render_prepared_markdown_patch,
    render_prepared_markdown_patch_json, BlockRenderer, ImageCandidate, ImageLayout,
    ImagePlacement, ImagePresentation, ImagePresentationError, ImageRenderRequest, InlineRenderer,
    OrihaEngine, ParseStatus, ParsedDocument, RenderHook,
};
pub use theme::{
    body_font_catalog, code_font_catalog, heading_font_catalog, syntax_theme_catalog, FontOption,
    SyntaxTheme,
};
pub use types::{
    default_watermark_options, is_valid_render_stats_kind, render_snapshot_schema_version,
    render_stats_kind_catalog, render_stats_schema_version, sort_render_stats_kind_counts,
    AppearanceOptions, AssetKind, CodeBlockOptions, ColorScheme, EngineLimits, ExportHtmlInput,
    ExportHtmlOutput, FeatureFlags, FontFamilySpec, HtmlExportOptions, KindCount,
    LockedFeatureKind, PreparedHtmlExport, PreparedMarkdownLayout, PreparedMarkdownPatch,
    PrintOptions, RenderAsset, RenderBlockKind, RenderDiagnostic, RenderInput, RenderIssueKind,
    RenderMode, RenderOptions, RenderOutput, RenderPatchFallbackReason, RenderPatchInput,
    RenderPatchOperation, RenderPatchOutput, RenderPatchStrategy, RenderSnapshot,
    RenderSnapshotBlock, RenderStats, RenderStatsBucket, RenderStatsKindCatalog, RenderedBlock,
    Severity, SourceRange, TableBorderStyle, TextMeasurementBatch, TextMeasurementRect,
    TextMeasurementRequest, TextMeasurementResult, TextMeasurementResults, TextMeasurementRun,
    TextMeasurementRunResult, TextMeasurementStyle, UnsupportedKind, Viewport, WatermarkOptions,
    DEFAULT_WATERMARK_TEXT, DEFAULT_WATERMARK_URL, WATERMARK_URL_ENV_VAR,
};
