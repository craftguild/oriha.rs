use super::*;

#[derive(Default)]
pub struct RenderFragment {
    pub html: String,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Clone, Default)]
pub struct RenderState {
    pub diagnostics: Vec<RenderDiagnostic>,
    pub assets: Vec<RenderAsset>,
    pub attrs: BTreeMap<String, String>,
    pub source_map: Vec<RenderedBlock>,
    pub(crate) rendered_patch_blocks: Vec<RenderedPatchBlock>,
    pub stats: RenderStats,
    pub heading_ids: BTreeMap<String, usize>,
    pub footnotes: FootnoteRenderState,
    pub(super) footnote_ids: FootnoteIds,
}

#[derive(Clone, Default)]
pub struct FootnoteRenderState {
    pub numbers: BTreeMap<String, usize>,
    pub reference_counts: BTreeMap<String, usize>,
    pub order: Vec<String>,
}

pub trait InlineRenderer: Send + Sync {
    fn render(&self, source: &str, features: &FeatureFlags, stats: &mut RenderStats) -> String;

    fn render_with_context(
        &self,
        source: &str,
        _context: &RenderContext,
        stats: &mut RenderStats,
    ) -> String {
        self.render(source, &_context.features, stats)
    }
}

pub trait BlockRenderer: Send + Sync {
    fn kind(&self) -> &'static str;
    /// The block API preserves this explicit decision, including empty HTML.
    /// Existing custom renderers default to HTML. Document APIs ignore it.
    fn block_emission(&self, _block: &BlockNode) -> crate::BlockEmission {
        crate::BlockEmission::Html
    }
    fn render(
        &self,
        block: &mut BlockNode,
        source: &SourceDocument,
        state: &mut RenderState,
        context: &RenderContext,
        renderers: &[Box<dyn BlockRenderer>],
        out: &mut String,
    );
}

pub trait RenderHook: Send + Sync {
    /// Look up immutable data prepared before rendering. The first override in
    /// registration order wins. Structured derivation never calls this method.
    fn image_presentation<'a>(
        &'a self,
        _image: &ImageRenderRequest<'_>,
        _context: &RenderContext<'_>,
    ) -> Option<&'a ImagePresentation> {
        None
    }

    fn before_document_parse(
        &self,
        _input: &mut RenderInput,
        _state: &mut RenderState,
        _context: &mut RenderContext,
    ) {
    }
    fn after_document_parse(
        &self,
        _document: &mut Document,
        _state: &mut RenderState,
        _context: &mut RenderContext,
    ) {
    }
    fn before_block_detect(
        &self,
        _block: &mut BlockNode,
        _state: &mut RenderState,
        _context: &RenderContext,
    ) {
    }
    fn after_block_detect(
        &self,
        _block: &mut BlockNode,
        _state: &mut RenderState,
        _context: &RenderContext,
    ) {
    }
    fn before_block_parse(
        &self,
        _block: &mut BlockNode,
        _state: &mut RenderState,
        _context: &RenderContext,
    ) {
    }
    fn after_block_parse(
        &self,
        _block: &mut BlockNode,
        _state: &mut RenderState,
        _context: &RenderContext,
    ) {
    }
    fn before_block_render(
        &self,
        _block: &mut BlockNode,
        _fragment: &mut RenderFragment,
        _state: &mut RenderState,
        _context: &RenderContext,
    ) {
    }
    fn after_block_render(
        &self,
        _block: &mut BlockNode,
        _fragment: &mut RenderFragment,
        _state: &mut RenderState,
        _context: &RenderContext,
    ) {
    }
    fn before_output_finalize(
        &self,
        _document: &Document,
        _state: &mut RenderState,
        _context: &mut RenderContext,
    ) {
    }
    fn after_output_finalize(
        &self,
        _document: &Document,
        _output: &mut RenderOutput,
        _state: &mut RenderState,
        _context: &mut RenderContext,
    ) {
    }
}
