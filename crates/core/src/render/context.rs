use super::*;

pub struct RenderContext<'a> {
    pub(super) max_output_bytes: usize,
    pub appearance: AppearanceOptions,
    pub code_block: crate::CodeBlockOptions,
    pub(super) locale: I18nLocale,
    pub mode: RenderMode,
    pub features: FeatureFlags,
    pub source: SourceDocument,
    pub link_references: BTreeMap<String, LinkReference>,
    pub footnotes: BTreeMap<String, FootnoteDefinition>,
    pub(crate) patch_blocks: BTreeMap<String, AssignedPatchBlock>,
    pub hooks: &'a [Box<dyn RenderHook>],
    pub inline_renderer: &'a dyn InlineRenderer,
    pub plantuml_text_metrics: BTreeMap<String, InlineTextMeasurementResult>,
    pub mermaid_text_metrics: BTreeMap<String, MermaidMeasurementResult>,
}

#[derive(Clone)]
pub(super) struct OwnedRenderContext {
    pub appearance: AppearanceOptions,
    pub code_block: crate::CodeBlockOptions,
    pub(super) locale: I18nLocale,
    pub mode: RenderMode,
    pub features: FeatureFlags,
    pub source: SourceDocument,
    pub link_references: BTreeMap<String, LinkReference>,
    pub footnotes: BTreeMap<String, FootnoteDefinition>,
    pub(crate) patch_blocks: BTreeMap<String, AssignedPatchBlock>,
    pub plantuml_text_metrics: BTreeMap<String, InlineTextMeasurementResult>,
    pub mermaid_text_metrics: BTreeMap<String, MermaidMeasurementResult>,
}

impl OwnedRenderContext {
    pub(super) fn borrow<'a>(
        &self,
        hooks: &'a [Box<dyn RenderHook>],
        inline_renderer: &'a dyn InlineRenderer,
        max_output_bytes: usize,
    ) -> RenderContext<'a> {
        RenderContext {
            max_output_bytes,
            appearance: self.appearance.clone(),
            code_block: self.code_block,
            locale: self.locale,
            mode: self.mode,
            features: self.features.clone(),
            source: self.source.clone(),
            link_references: self.link_references.clone(),
            footnotes: self.footnotes.clone(),
            patch_blocks: self.patch_blocks.clone(),
            plantuml_text_metrics: self.plantuml_text_metrics.clone(),
            mermaid_text_metrics: self.mermaid_text_metrics.clone(),
            hooks,
            inline_renderer,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RenderMessage {
    CodeCopyLabel,
    CodeCopyAriaLabel,
    UnsupportedPlantumlTitle,
    MermaidRenderErrorTitle,
    FootnotesTitle,
    FootnoteBackrefLabel,
}

pub(super) fn render_message(locale: I18nLocale, message: RenderMessage) -> &'static str {
    let messages = i18n_messages(locale).core;
    match message {
        RenderMessage::CodeCopyLabel => messages.code_copy_label,
        RenderMessage::CodeCopyAriaLabel => messages.code_copy_aria_label,
        RenderMessage::UnsupportedPlantumlTitle => messages.unsupported_plantuml_title,
        RenderMessage::MermaidRenderErrorTitle => messages.mermaid_render_error_title,
        RenderMessage::FootnotesTitle => messages.footnotes_title,
        RenderMessage::FootnoteBackrefLabel => messages.footnote_backref_label,
    }
}
