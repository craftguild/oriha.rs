use super::*;

pub(super) struct RenderPipelineOutput {
    pub(super) output: RenderOutput,
    pub(super) snapshot: RenderSnapshot,
    pub(super) patch_blocks: Vec<RenderedPatchBlock>,
    pub(super) output_truncated: bool,
}

pub(super) fn build_patch_output_from_pipeline(
    previous_snapshot: Option<&RenderSnapshot>,
    rendered: RenderPipelineOutput,
) -> RenderPatchOutput {
    if rendered.output_truncated {
        return patch::replace_all_output(
            rendered.output,
            RenderSnapshot::default(),
            RenderPatchFallbackReason::OutputTruncated,
        );
    }

    patch::build_patch_output(
        previous_snapshot,
        rendered.output,
        rendered.snapshot,
        rendered.patch_blocks,
    )
}

pub(super) fn build_context<'a>(
    input: &RenderInput,
    hooks: &'a [Box<dyn RenderHook>],
    inline_renderer: &'a dyn InlineRenderer,
    link_references: BTreeMap<String, LinkReference>,
    patch_blocks: BTreeMap<String, AssignedPatchBlock>,
    plantuml_text_metrics: BTreeMap<String, InlineTextMeasurementResult>,
    mermaid_text_metrics: BTreeMap<String, MermaidMeasurementResult>,
) -> RenderContext<'a> {
    RenderContext {
        max_output_bytes: EngineLimits::default().max_output_bytes,
        appearance: input.appearance.clone(),
        code_block: input.render_options.code_block,
        locale: resolve_locale(input.locale.as_deref()),
        mode: input.mode,
        features: input.features.clone(),
        source: SourceDocument::new(input.markdown.clone()),
        link_references,
        footnotes: BTreeMap::new(),
        patch_blocks,
        hooks,
        inline_renderer,
        plantuml_text_metrics,
        mermaid_text_metrics,
    }
}

pub(super) fn footnote_definition_map(
    definitions: &[FootnoteDefinition],
) -> BTreeMap<String, FootnoteDefinition> {
    definitions
        .iter()
        .map(|definition| (definition.label.clone(), definition.clone()))
        .collect()
}

pub(super) fn finalize_output(
    input: &RenderInput,
    context: &RenderContext,
    state: &mut RenderState,
    body: &str,
    wrap_output: bool,
) -> RenderOutput {
    let stats = finalized_stats(state);
    state.source_map.sort_by_key(|block| {
        (
            block.source_range.start_line,
            block.source_range.start_column,
            block.source_range.end_line,
            block.source_range.end_column,
        )
    });

    let body = render_body_with_options(body, input);
    let theme = resolve_theme(&context.appearance);

    let html = if wrap_output {
        render_wrapper("oriha-document", &theme.name, &state.attrs, &body)
    } else {
        body
    };

    let css = match input.mode {
        RenderMode::StyledHtml => Some(render_css_with_options(
            &theme,
            input,
            context.code_block,
            has_bundled_math_font_asset(state),
        )),
        RenderMode::SemanticHtml => None,
    };

    RenderOutput {
        html,
        css,
        stats,
        diagnostics: state.diagnostics.clone(),
        assets: state.assets.clone(),
        source_map: state.source_map.clone(),
    }
}

pub(super) fn finalized_stats(state: &RenderState) -> RenderStats {
    let mut stats = state.stats.clone();
    stats.issues_by_kind.clear();
    for diagnostic in &state.diagnostics {
        stats.record_issue_code(&diagnostic.code, diagnostic.severity);
    }
    stats.finalize();
    stats
}

pub(super) fn render_css_with_options(
    theme: &crate::theme::ResolvedTheme,
    input: &RenderInput,
    code_block: crate::CodeBlockOptions,
    include_math_font_face: bool,
) -> String {
    let mut css = String::new();
    if include_math_font_face {
        css.push_str(oriha_latex::math_font_face_css());
        if !css.ends_with('\n') {
            css.push('\n');
        }
        css.push('\n');
    }
    css.push_str(&render_theme_css(theme));
    if !code_block.show_line_numbers {
        css.push_str("\n.oriha-code-block code { grid-template-columns: minmax(0, 1fr); }\n");
    }
    if !code_block.show_language && code_block.show_copy_button {
        css.push_str("\n.oriha-code-block-header { justify-content: flex-end; }\n");
    }
    if let Some(opacity) = input
        .render_options
        .watermark
        .as_ref()
        .and_then(|watermark| watermark.opacity)
    {
        css.push_str("\n:root {\n  --oriha-watermark-opacity: ");
        css.push_str(&format_css_opacity(opacity));
        css.push_str(";\n}\n");
    }
    css
}

pub(super) fn format_css_opacity(value: f32) -> String {
    let value = if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        1.0
    };
    if value == 0.0 || value == 1.0 {
        format!("{value:.0}")
    } else {
        let formatted = format!("{value:.3}");
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

pub(super) fn render_body_with_options(body: &str, input: &RenderInput) -> String {
    let Some(watermark) = input.render_options.watermark.as_ref() else {
        return body.to_string();
    };
    let image_src_len = watermark.image_src.as_ref().map_or(0, String::len);
    let mut rendered =
        String::with_capacity(body.len() + watermark.text.len() + image_src_len + 96);
    rendered.push_str(body);
    rendered.push_str(r#"<div class="oriha-watermark" aria-hidden="true">"#);
    if let Some(image_src) = watermark.image_src.as_deref() {
        rendered.push_str(r#"<img src=""#);
        rendered.push_str(&escape_attr(image_src));
        rendered.push_str(r#"" alt="" class="oriha-watermark__image" />"#);
    }
    rendered.push_str(r#"<span class="oriha-watermark__text">"#);
    rendered.push_str(&escape_text(&watermark.text));
    rendered.push_str("</span>");
    rendered.push_str("</div>");
    rendered
}

pub(super) fn truncate_utf8_boundary(value: &mut String, max_bytes: usize) {
    if value.len() <= max_bytes {
        return;
    }
    let mut cut = max_bytes.min(value.len());
    while cut > 0 && !value.is_char_boundary(cut) {
        cut -= 1;
    }
    value.truncate(cut);
}
