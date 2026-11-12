use super::*;

impl OrihaEngine {
    /// Parse once, retaining effective settings and document-hook state.
    pub fn parse(&self, input: RenderInput) -> ParsedDocument {
        self.parse_with_text_metrics(input, BTreeMap::new(), BTreeMap::new())
    }

    pub(super) fn parse_with_text_metrics(
        &self,
        input: RenderInput,
        plantuml_text_metrics: BTreeMap<String, InlineTextMeasurementResult>,
        mermaid_text_metrics: BTreeMap<String, MermaidMeasurementResult>,
    ) -> ParsedDocument {
        let fallback_input = input.clone();
        catch_engine_panic(
            std::panic::AssertUnwindSafe(|| {
                self.parse_inner(input, plantuml_text_metrics, mermaid_text_metrics)
            }),
            |message| {
                let state = RenderState {
                    diagnostics: invalid_render_output("engine.panic", message).diagnostics,
                    ..Default::default()
                };
                let context = build_context(
                    &fallback_input,
                    &self.hooks,
                    self.inline_renderer.as_ref(),
                    BTreeMap::new(),
                    BTreeMap::new(),
                    BTreeMap::new(),
                    BTreeMap::new(),
                );
                ParsedDocument::new(
                    fallback_input.markdown.clone(),
                    fallback_input,
                    empty_document(),
                    state,
                    context,
                    self.limits,
                    true,
                )
            },
        )
    }

    pub(super) fn parse_inner(
        &self,
        mut input: RenderInput,
        plantuml_text_metrics: BTreeMap<String, InlineTextMeasurementResult>,
        mermaid_text_metrics: BTreeMap<String, MermaidMeasurementResult>,
    ) -> ParsedDocument {
        let original_source = input.markdown.clone();
        if input.features == FeatureFlags::default() {
            input.features = self.features.clone();
        }

        let mut state = RenderState::default();
        let mut context = build_context(
            &input,
            &self.hooks,
            self.inline_renderer.as_ref(),
            BTreeMap::new(),
            BTreeMap::new(),
            plantuml_text_metrics.clone(),
            mermaid_text_metrics.clone(),
        );

        for hook in &self.hooks {
            hook.before_document_parse(&mut input, &mut state, &mut context);
        }

        context = build_context(
            &input,
            &self.hooks,
            self.inline_renderer.as_ref(),
            BTreeMap::new(),
            BTreeMap::new(),
            plantuml_text_metrics,
            mermaid_text_metrics,
        );

        if input.markdown.len() > self.limits.max_source_bytes {
            state.diagnostics.push(RenderDiagnostic {
                severity: Severity::Error,
                code: "engine.max_source_bytes_exceeded".to_string(),
                message: format!(
                    "Input exceeds max_source_bytes limit: {} > {}",
                    input.markdown.len(),
                    self.limits.max_source_bytes
                ),
                source_range: None,
                block_id: None,
                engine: DiagnosticEngine::Core,
            });
            return ParsedDocument::new(
                original_source,
                input,
                empty_document(),
                state,
                context,
                self.limits,
                true,
            );
        }

        let mut document = parse_document(&input.markdown, &input.features, self.limits);
        let parse_status = if document.diagnostics.iter().any(|d| {
            matches!(
                d.code.as_str(),
                "engine.max_blocks_exceeded" | "engine.max_nesting_depth_exceeded"
            )
        }) {
            ParseStatus::Incomplete
        } else {
            ParseStatus::Complete
        };
        if input.markdown != original_source {
            crate::annotations::mark_generated(&mut document);
        }
        context.link_references = document.link_references.clone();
        context.footnotes = footnote_definition_map(&document.footnotes);
        state.diagnostics.extend(document.diagnostics.clone());
        state.assets.extend(document.assets.clone());

        for hook in &self.hooks {
            let before = document.clone();
            hook.after_document_parse(&mut document, &mut state, &mut context);
            crate::hook_mapping::reconcile(&before, &mut document, &original_source);
        }
        for diagnostic in &document.diagnostics {
            if !state
                .diagnostics
                .iter()
                .any(|d| serde_json::to_value(d).ok() == serde_json::to_value(diagnostic).ok())
            {
                state.diagnostics.push(diagnostic.clone());
            }
        }
        context.footnotes = footnote_definition_map(&document.footnotes);

        let mut parsed = ParsedDocument::new(
            original_source,
            input,
            document,
            state,
            context,
            self.limits,
            false,
        );
        parsed.parse_status = parse_status;
        parsed
    }
}

/// Owned Rust parsing result. Rendering never mutates this retained state.
pub struct ParsedDocument {
    pub(super) identity: u64,
    pub(super) original_source: String,
    pub(super) input: RenderInput,
    pub(super) document: Document,
    pub(super) state: RenderState,
    pub(super) context: OwnedRenderContext,
    pub(super) limits: EngineLimits,
    pub(super) fatal: bool,
    pub(super) parse_status: ParseStatus,
}

/// Engine parsing outcome for effective input, captured before after-parse hooks.
/// This does not certify hook-authored content, consumer policy, or render limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseStatus {
    Complete,
    Incomplete,
    Failed,
}

pub(super) fn next_document_identity() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

impl Clone for ParsedDocument {
    fn clone(&self) -> Self {
        self.clone_with_document(self.document.clone())
    }
}

impl ParsedDocument {
    pub(super) fn clone_with_document(&self, document: Document) -> Self {
        let mut state = self.state.clone();
        state.footnote_ids = FootnoteIds::for_document(&document);
        Self {
            identity: next_document_identity(),
            original_source: self.original_source.clone(),
            input: self.input.clone(),
            document,
            state,
            context: self.context.clone(),
            limits: self.limits,
            fatal: self.fatal,
            parse_status: self.parse_status,
        }
    }
}

impl ParsedDocument {
    pub(super) fn new(
        original_source: String,
        input: RenderInput,
        document: Document,
        state: RenderState,
        context: RenderContext<'_>,
        limits: EngineLimits,
        fatal: bool,
    ) -> Self {
        let mut state = state;
        state.footnote_ids = FootnoteIds::for_document(&document);
        let context = OwnedRenderContext {
            appearance: context.appearance,
            code_block: context.code_block,
            locale: context.locale,
            mode: context.mode,
            features: context.features,
            source: context.source,
            link_references: context.link_references,
            footnotes: context.footnotes,
            patch_blocks: context.patch_blocks,
            plantuml_text_metrics: context.plantuml_text_metrics,
            mermaid_text_metrics: context.mermaid_text_metrics,
        };
        Self {
            identity: next_document_identity(),
            original_source,
            input,
            document,
            state,
            context,
            limits,
            fatal,
            parse_status: if fatal {
                ParseStatus::Failed
            } else {
                ParseStatus::Complete
            },
        }
    }

    pub fn text_regions(&self) -> impl Iterator<Item = crate::TextRegion<'_>> {
        crate::annotations::regions(&self.document, self.identity).into_iter()
    }

    pub fn apply_annotations(
        &self,
        annotations: &[crate::TextAnnotation],
    ) -> Result<Self, crate::AnnotationError> {
        let document = crate::annotations::apply(&self.document, self.identity, annotations)?;
        let footnotes = footnote_definition_map(&document.footnotes);
        let mut result = self.clone_with_document(document);
        result.context.footnotes = footnotes;
        Ok(result)
    }

    pub fn source(&self) -> &str {
        &self.original_source
    }
    pub fn document(&self) -> &Document {
        &self.document
    }
    pub fn diagnostics(&self) -> &[RenderDiagnostic] {
        &self.state.diagnostics
    }
    /// Complete only if the engine parsed effective input without limit omissions
    /// or failure. Output limits and arbitrary hook changes are outside this guard.
    pub fn parse_status(&self) -> ParseStatus {
        self.parse_status
    }
}

pub(super) fn empty_document() -> Document {
    Document {
        blocks: Vec::new(),
        diagnostics: Vec::new(),
        assets: Vec::new(),
        link_references: BTreeMap::new(),
        footnotes: Vec::new(),
    }
}

pub fn parse(input: RenderInput) -> ParsedDocument {
    OrihaEngine::default().parse(input)
}
pub fn render_parsed(parsed: &ParsedDocument) -> RenderOutput {
    OrihaEngine::default().render_parsed(parsed)
}
