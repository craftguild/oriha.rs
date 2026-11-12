use super::*;

pub(super) fn render_latex_source_svg(
    source: &str,
    mode: LatexRenderMode,
    appearance: &AppearanceOptions,
) -> Result<RenderedLatex, LatexRenderError> {
    oriha_latex::render_latex_svg(
        source.trim(),
        mode,
        Some(resolve_body_font_size_px(appearance.body_font_size_px)),
    )
}

pub(super) fn register_bundled_math_font_asset(state: &mut RenderState) {
    if state
        .assets
        .iter()
        .filter(|asset| asset.id == oriha_latex::MATH_FONT_ASSET_ID)
        .count()
        == 1
        && state.assets.iter().any(is_bundled_math_font_asset)
    {
        return;
    }

    state
        .assets
        .retain(|asset| asset.id != oriha_latex::MATH_FONT_ASSET_ID);
    state.assets.push(RenderAsset {
        id: oriha_latex::MATH_FONT_ASSET_ID.to_string(),
        kind: AssetKind::Other,
        mime_type: oriha_latex::MATH_FONT_MIME_TYPE.to_string(),
        content: {
            crate::blocks::html_budget::check_size(oriha_latex::math_font_base64().len());
            oriha_latex::math_font_base64().to_string()
        },
    });
}

pub(super) fn has_bundled_math_font_asset(state: &RenderState) -> bool {
    state
        .assets
        .iter()
        .any(|asset| asset.id == oriha_latex::MATH_FONT_ASSET_ID)
}

pub(super) fn is_bundled_math_font_asset(asset: &RenderAsset) -> bool {
    asset.id == oriha_latex::MATH_FONT_ASSET_ID
        && matches!(asset.kind, AssetKind::Other)
        && asset.mime_type == oriha_latex::MATH_FONT_MIME_TYPE
        && asset.content == oriha_latex::math_font_base64()
}

pub(super) fn render_math_block_node(
    node: &MathBlockNode,
    source: &SourceDocument,
    block: &BlockNode,
    out: &mut String,
    state: &mut RenderState,
    context: &RenderContext,
) {
    let math_source = node
        .source
        .as_deref()
        .unwrap_or_else(|| source.source_span(node.source_span));

    if !node.closed {
        let message =
            "LaTeX math could not be rendered: display math block has no closing delimiter"
                .to_string();
        render_math_error_figure(out, &block.id, None, &message, math_source);
        out.push_html_char('\n');
        state.diagnostics.push(RenderDiagnostic {
            severity: Severity::Warning,
            code: "math.render_failed".to_string(),
            message,
            source_range: Some(source.to_source_range(node.source_span)),
            block_id: Some(block.id.clone()),
            engine: DiagnosticEngine::Core,
        });
        return;
    }

    match render_latex_source_svg(math_source, LatexRenderMode::Display, &context.appearance) {
        Ok(RenderedLatex { svg }) => {
            register_bundled_math_font_asset(state);
            out.push_html(&format!(
                r#"<figure id="{}" class="oriha-math-block" data-oriha-block-kind="mathBlock">{}</figure>"#,
                escape_attr(&block.id),
                svg
            ));
            out.push_html_char('\n');
        }
        Err(error) => {
            let message = format!("LaTeX math could not be rendered: {error}");
            render_math_error_figure(out, &block.id, None, &message, math_source);
            out.push_html_char('\n');
            state.diagnostics.push(RenderDiagnostic {
                severity: Severity::Warning,
                code: "math.render_failed".to_string(),
                message,
                source_range: Some(source.to_source_range(node.source_span)),
                block_id: Some(block.id.clone()),
                engine: DiagnosticEngine::Core,
            });
        }
    }
}

pub(super) fn render_math_error_figure(
    out: &mut String,
    block_id: &str,
    language: Option<&str>,
    message: &str,
    source: &str,
) {
    let language_attr = language
        .map(|language| format!(r#" data-oriha-code-language="{}""#, escape_attr(language)))
        .unwrap_or_default();
    let code_language = language.unwrap_or("latex");
    out.push_html(&format!(
        r#"<figure id="{}" class="oriha-math-block oriha-math--error" data-oriha-block-kind="mathBlock"{}><figcaption class="oriha-math__error-title">LaTeX math could not be rendered</figcaption><p class="oriha-math__error-message">{}</p><pre class="oriha-math__source"><code class="language-{}">{}</code></pre></figure>"#,
        escape_attr(block_id),
        language_attr,
        escape_text(message),
        escape_attr(code_language),
        escape_text(source)
    ));
}
