use super::*;

pub struct OrihaEngine {
    pub limits: EngineLimits,
    pub features: FeatureFlags,
    pub hooks: Vec<Box<dyn RenderHook>>,
    pub block_renderers: Vec<Box<dyn BlockRenderer>>,
    pub inline_renderer: Box<dyn InlineRenderer>,
    pub(super) wrap_output: bool,
}

impl Default for OrihaEngine {
    fn default() -> Self {
        Self {
            limits: EngineLimits::default(),
            features: FeatureFlags::default(),
            hooks: Vec::new(),
            block_renderers: vec![Box::new(DefaultBlockRenderer)],
            inline_renderer: Box::new(CoreInlineRenderer),
            wrap_output: true,
        }
    }
}

impl OrihaEngine {
    pub fn with_features(mut self, features: FeatureFlags) -> Self {
        self.features = features;
        self
    }

    pub fn with_limits(mut self, limits: EngineLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn add_hook(mut self, hook: impl RenderHook + 'static) -> Self {
        self.hooks.push(Box::new(hook));
        self
    }

    pub fn register_renderer(mut self, renderer: impl BlockRenderer + 'static) -> Self {
        self.block_renderers.push(Box::new(renderer));
        self
    }

    #[cfg(test)]
    pub(crate) fn with_unwrapped_output(mut self) -> Self {
        self.wrap_output = false;
        self
    }

    pub fn render(&self, input: RenderInput) -> RenderOutput {
        self.render_with_text_metrics(input, BTreeMap::new(), BTreeMap::new())
    }

    pub fn render_patch(&self, input: RenderPatchInput) -> RenderPatchOutput {
        let previous_snapshot = input.previous_snapshot;
        let rendered = self.render_with_text_metrics_for_patch(
            input.input,
            BTreeMap::new(),
            BTreeMap::new(),
            previous_snapshot.as_ref(),
            true,
        );
        build_patch_output_from_pipeline(previous_snapshot.as_ref(), rendered)
    }

    pub(super) fn render_with_text_metrics(
        &self,
        input: RenderInput,
        plantuml_text_metrics: BTreeMap<String, InlineTextMeasurementResult>,
        mermaid_text_metrics: BTreeMap<String, MermaidMeasurementResult>,
    ) -> RenderOutput {
        self.render_with_text_metrics_for_patch(
            input,
            plantuml_text_metrics,
            mermaid_text_metrics,
            None,
            false,
        )
        .output
    }

    /// Render a working copy. Parse-stage hooks are not executed again.
    pub fn render_parsed(&self, parsed: &ParsedDocument) -> RenderOutput {
        catch_engine_panic(
            std::panic::AssertUnwindSafe(|| {
                self.render_parsed_pipeline(parsed, None, false).output
            }),
            |message| invalid_render_output("engine.panic", message),
        )
    }

    pub(super) fn render_with_text_metrics_for_patch(
        &self,
        input: RenderInput,
        plantuml_text_metrics: BTreeMap<String, InlineTextMeasurementResult>,
        mermaid_text_metrics: BTreeMap<String, MermaidMeasurementResult>,
        previous_snapshot: Option<&RenderSnapshot>,
        capture_patch_blocks: bool,
    ) -> RenderPipelineOutput {
        let parsed =
            self.parse_with_text_metrics(input, plantuml_text_metrics, mermaid_text_metrics);
        self.render_parsed_pipeline(&parsed, previous_snapshot, capture_patch_blocks)
    }

    pub(super) fn render_parsed_pipeline(
        &self,
        parsed: &ParsedDocument,
        previous_snapshot: Option<&RenderSnapshot>,
        capture_patch_blocks: bool,
    ) -> RenderPipelineOutput {
        let input = &parsed.input;
        let mut state = parsed.state.clone();
        let mut document = parsed.document.clone();
        let mut context = parsed.context.borrow(
            &self.hooks,
            self.inline_renderer.as_ref(),
            parsed.limits.max_output_bytes,
        );
        let settings_fingerprint = patch::render_settings_fingerprint(input);
        if parsed.fatal {
            let output = finalize_output(input, &context, &mut state, "", self.wrap_output);
            return RenderPipelineOutput {
                output,
                snapshot: patch::empty_snapshot(settings_fingerprint),
                patch_blocks: Vec::new(),
                output_truncated: false,
            };
        }
        let patch_candidates = if self.wrap_output && capture_patch_blocks {
            patch::collect_patch_candidates(&document.blocks, &context.source)
        } else {
            Vec::new()
        };
        let compatible_previous = previous_snapshot.filter(|snapshot| {
            patch::snapshot_settings_are_compatible(snapshot, &settings_fingerprint)
        });
        let patch_assignment = patch::assign_stable_ids(compatible_previous, &patch_candidates);
        context.patch_blocks = patch_assignment.by_block_id;

        for hook in &self.hooks {
            hook.before_output_finalize(&document, &mut state, &mut context);
        }

        let mut body = String::new();
        render_blocks(
            &mut document.blocks,
            &context,
            &self.block_renderers,
            &mut state,
            &mut body,
        );
        let block_body_len = body.len();
        render_footnote_section(&context, &mut state, &mut body);
        let body_with_options = render_body_with_options(&body, input);
        let tail_html = body_with_options
            .get(block_body_len..)
            .unwrap_or_default()
            .to_string();
        let snapshot_blocks = patch::snapshot_blocks_from_rendered(&state.rendered_patch_blocks);
        let mut snapshot = patch::build_snapshot(
            settings_fingerprint,
            snapshot_blocks,
            patch_assignment.next_stable_index,
            &tail_html,
        );

        let mut output = finalize_output(input, &context, &mut state, &body, self.wrap_output);
        let mut output_truncated = false;

        if output.html.len() > parsed.limits.max_output_bytes {
            let issue_code = "engine.max_output_bytes_exceeded";
            output.diagnostics.push(RenderDiagnostic {
                severity: Severity::Error,
                code: issue_code.to_string(),
                message: format!(
                    "Output exceeds max_output_bytes limit: {} > {}",
                    output.html.len(),
                    parsed.limits.max_output_bytes
                ),
                source_range: None,
                block_id: None,
                engine: DiagnosticEngine::Core,
            });
            output
                .stats
                .record_issue(RenderIssueKind::MaxOutputBytesExceeded);
            output.stats.finalize();
            truncate_utf8_boundary(&mut output.html, parsed.limits.max_output_bytes);
            output_truncated = true;
        }

        for hook in &self.hooks {
            hook.after_output_finalize(&document, &mut output, &mut state, &mut context);
        }
        snapshot.output_resource_fingerprint = patch::output_resource_fingerprint(&output);

        RenderPipelineOutput {
            output,
            snapshot,
            patch_blocks: state.rendered_patch_blocks,
            output_truncated,
        }
    }
}
