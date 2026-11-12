//! Core render, option and diagnostic conversions and shared numeric normalization.
mod measurements;
#[allow(unused_imports)]
use measurements::*;
mod patches;
#[allow(unused_imports)]
use patches::*;
mod analytics;
#[allow(unused_imports)]
use analytics::*;

use std::collections::{BTreeMap, HashMap};

use oriha_stats_client as stats_client;

use crate::types::*;

impl AppearanceStyle {
    pub(crate) fn as_core_style(&self) -> &'static str {
        match self {
            AppearanceStyle::Rich => "rich",
            AppearanceStyle::Simple => "simple",
        }
    }
}

impl From<WatermarkOptions> for oriha_core::WatermarkOptions {
    fn from(value: WatermarkOptions) -> Self {
        Self {
            text: value.text,
            image_src: value.image_src,
            opacity: value.opacity.and_then(normalized_opacity),
        }
    }
}

impl From<oriha_core::WatermarkOptions> for WatermarkOptions {
    fn from(value: oriha_core::WatermarkOptions) -> Self {
        Self {
            text: value.text,
            image_src: value.image_src,
            opacity: value.opacity,
        }
    }
}

impl From<PrintOptions> for oriha_core::PrintOptions {
    fn from(value: PrintOptions) -> Self {
        Self {
            page_size: value.page_size,
            margin: value.margin,
        }
    }
}

impl From<FontFamilySpec> for oriha_core::FontFamilySpec {
    fn from(value: FontFamilySpec) -> Self {
        Self {
            family: value.family,
            fallbacks: value.fallbacks,
        }
    }
}

impl From<Viewport> for oriha_core::Viewport {
    fn from(value: Viewport) -> Self {
        Self {
            width: value.width,
            scale: positive_f32_or_default(value.scale, 1.0),
        }
    }
}

impl From<RenderMode> for oriha_core::RenderMode {
    fn from(value: RenderMode) -> Self {
        match value {
            RenderMode::StyledHtml => Self::StyledHtml,
            RenderMode::SemanticHtml => Self::SemanticHtml,
        }
    }
}

impl From<ColorScheme> for oriha_core::ColorScheme {
    fn from(value: ColorScheme) -> Self {
        match value {
            ColorScheme::Light => Self::Light,
            ColorScheme::Dark => Self::Dark,
        }
    }
}

impl From<TableBorderStyle> for oriha_core::TableBorderStyle {
    fn from(value: TableBorderStyle) -> Self {
        match value {
            TableBorderStyle::None => Self::None,
            TableBorderStyle::Inner => Self::Inner,
            TableBorderStyle::All => Self::All,
            TableBorderStyle::Vertical => Self::Vertical,
            TableBorderStyle::Horizontal => Self::Horizontal,
        }
    }
}

impl From<oriha_core::RenderOutput> for RenderOutput {
    fn from(value: oriha_core::RenderOutput) -> Self {
        Self {
            html: value.html,
            css: value.css,
            stats: value.stats.into(),
            diagnostics: value.diagnostics.into_iter().map(Into::into).collect(),
            assets: value.assets.into_iter().map(Into::into).collect(),
            source_map: value.source_map.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<oriha_core::ExportHtmlOutput> for ExportHtmlOutput {
    fn from(value: oriha_core::ExportHtmlOutput) -> Self {
        Self {
            html: value.html,
            stats: value.stats.into(),
            diagnostics: value.diagnostics.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<HtmlExportOptions> for oriha_core::HtmlExportOptions {
    fn from(value: HtmlExportOptions) -> Self {
        Self {
            minify: value.minify,
            obfuscate: value.obfuscate,
            preserve_heading_anchors: value.preserve_heading_anchors,
            strip_oriha_data_attributes: value.strip_oriha_data_attributes,
        }
    }
}

impl From<oriha_core::HtmlExportOptions> for HtmlExportOptions {
    fn from(value: oriha_core::HtmlExportOptions) -> Self {
        Self {
            minify: value.minify,
            obfuscate: value.obfuscate,
            preserve_heading_anchors: value.preserve_heading_anchors,
            strip_oriha_data_attributes: value.strip_oriha_data_attributes,
        }
    }
}

impl From<oriha_core::RenderStats> for RenderStats {
    fn from(value: oriha_core::RenderStats) -> Self {
        Self {
            schema_version: value.schema_version,
            total_blocks: value.total_blocks,
            blocks_by_kind: value.blocks_by_kind.into_iter().map(Into::into).collect(),
            headings_by_level: value
                .headings_by_level
                .into_iter()
                .map(Into::into)
                .collect(),
            code_blocks_by_language_kind: value
                .code_blocks_by_language_kind
                .into_iter()
                .map(Into::into)
                .collect(),
            callouts_by_kind: value.callouts_by_kind.into_iter().map(Into::into).collect(),
            locked_features_by_kind: value
                .locked_features_by_kind
                .into_iter()
                .map(Into::into)
                .collect(),
            unsupported_by_kind: value
                .unsupported_by_kind
                .into_iter()
                .map(Into::into)
                .collect(),
            issues_by_kind: value.issues_by_kind.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<RenderStats> for oriha_core::RenderStats {
    fn from(value: RenderStats) -> Self {
        Self {
            schema_version: value.schema_version,
            total_blocks: value.total_blocks,
            blocks_by_kind: value.blocks_by_kind.into_iter().map(Into::into).collect(),
            headings_by_level: value
                .headings_by_level
                .into_iter()
                .map(Into::into)
                .collect(),
            code_blocks_by_language_kind: value
                .code_blocks_by_language_kind
                .into_iter()
                .map(Into::into)
                .collect(),
            callouts_by_kind: value.callouts_by_kind.into_iter().map(Into::into).collect(),
            locked_features_by_kind: value
                .locked_features_by_kind
                .into_iter()
                .map(Into::into)
                .collect(),
            unsupported_by_kind: value
                .unsupported_by_kind
                .into_iter()
                .map(Into::into)
                .collect(),
            issues_by_kind: value.issues_by_kind.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<oriha_core::KindCount> for KindCount {
    fn from(value: oriha_core::KindCount) -> Self {
        Self {
            kind: value.kind,
            count: value.count,
        }
    }
}

impl From<KindCount> for oriha_core::KindCount {
    fn from(value: KindCount) -> Self {
        Self {
            kind: value.kind,
            count: value.count,
        }
    }
}

impl From<oriha_core::RenderDiagnostic> for RenderDiagnostic {
    fn from(value: oriha_core::RenderDiagnostic) -> Self {
        Self {
            severity: value.severity.into(),
            code: value.code,
            message: value.message,
            source_range: value.source_range.map(Into::into),
            block_id: value.block_id,
            engine: DiagnosticEngine::Core,
        }
    }
}

impl From<oriha_core::Severity> for Severity {
    fn from(value: oriha_core::Severity) -> Self {
        match value {
            oriha_core::Severity::Info => Self::Info,
            oriha_core::Severity::Warning => Self::Warning,
            oriha_core::Severity::Error => Self::Error,
        }
    }
}

impl From<oriha_core::RenderAsset> for RenderAsset {
    fn from(value: oriha_core::RenderAsset) -> Self {
        Self {
            id: value.id,
            kind: value.kind.into(),
            mime_type: value.mime_type,
            content: value.content,
        }
    }
}

impl From<oriha_core::AssetKind> for AssetKind {
    fn from(value: oriha_core::AssetKind) -> Self {
        match value {
            oriha_core::AssetKind::Svg => Self::Svg,
            oriha_core::AssetKind::Css => Self::Css,
            oriha_core::AssetKind::Other => Self::Other,
        }
    }
}

impl From<oriha_core::RenderedBlock> for RenderedBlock {
    fn from(value: oriha_core::RenderedBlock) -> Self {
        Self {
            id: value.id,
            kind: value.kind,
            source_range: value.source_range.into(),
            metadata: btree_to_hash(value.metadata),
        }
    }
}

impl From<oriha_core::SourceRange> for SourceRange {
    fn from(value: oriha_core::SourceRange) -> Self {
        Self {
            start_line: value.start_line,
            start_column: value.start_column,
            end_line: value.end_line,
            end_column: value.end_column,
        }
    }
}

impl From<SourceRange> for oriha_core::SourceRange {
    fn from(value: SourceRange) -> Self {
        Self {
            start_line: value.start_line,
            start_column: value.start_column,
            end_line: value.end_line,
            end_column: value.end_column,
        }
    }
}

fn btree_to_hash<T>(value: BTreeMap<String, T>) -> HashMap<String, T> {
    value.into_iter().collect()
}

fn normalized_opacity(value: f32) -> Option<f32> {
    value.is_finite().then_some(value.clamp(0.0, 1.0))
}

fn non_negative_f32(value: f32) -> Option<f32> {
    value.is_finite().then_some(value.max(0.0))
}

fn clamp_non_negative_f32(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_f32_or_zero(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}

fn positive_f32_or_default(value: f32, default: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        default
    }
}
