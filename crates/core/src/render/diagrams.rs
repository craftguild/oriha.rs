use super::*;

pub(super) enum PlantUmlBlockRender {
    Svg(String),
    Unsupported(String),
}

pub(super) fn render_mermaid_code_block(
    source: &str,
    block_id: &str,
    context: &RenderContext,
) -> Result<MermaidSvg, MermaidError> {
    let options = mermaid_options_for_appearance(&context.appearance);
    Mermaid::new().render_with_measurements(
        source,
        block_id,
        &options,
        &context.mermaid_text_metrics,
    )
}

pub(super) fn mermaid_options_for_appearance(appearance: &AppearanceOptions) -> MermaidOptions {
    let resolved = resolve_theme(appearance);
    let tokens = &resolved.tokens;
    MermaidOptions {
        dark_mode: matches!(appearance.color_scheme, ColorScheme::Dark),
        font_family: appearance.body_font_family.css_stack(),
        body_font_size_px: Some(resolve_body_font_size_px(appearance.body_font_size_px)),
        background: Some(tokens.diagram.background.clone()),
        primary_color: Some(tokens.surface.elevated.clone()),
        primary_text_color: Some(tokens.text.body.clone()),
        primary_border_color: Some(tokens.border.default.clone()),
        line_color: Some(tokens.diagram.border.clone()),
        text_color: Some(tokens.text.body.clone()),
        main_bkg: Some(tokens.surface.elevated.clone()),
        node_bkg: Some(tokens.surface.elevated.clone()),
        node_border: Some(tokens.border.default.clone()),
        cluster_bkg: Some(tokens.surface.subtle.clone()),
        cluster_border: Some(tokens.border.strong.clone()),
        edge_label_background: Some(tokens.surface.base.clone()),
        style: MermaidDiagramStyle::default(),
    }
}

pub(super) fn render_mermaid_error_figure(
    out: &mut String,
    context: &RenderContext,
    block_id: &str,
    language: &str,
    message: &str,
    source: &str,
) {
    let title = render_message(context.locale, RenderMessage::MermaidRenderErrorTitle);
    out.push_html(&format!(
        r#"<figure id="{}" class="oriha-mermaid oriha-mermaid--error" data-oriha-block-kind="codeBlock" data-oriha-code-language="{}"><figcaption class="oriha-mermaid__error-title">{}</figcaption><p class="oriha-mermaid__error-message">{}</p><pre class="oriha-mermaid__source"><code class="language-{}">{}</code></pre></figure>"#,
        escape_attr(block_id),
        escape_attr(language),
        escape_text(title),
        escape_text(message),
        escape_attr(language),
        escape_text(source)
    ));
}

pub(super) fn render_plantuml_code_block(
    source: &str,
    context: &RenderContext,
) -> Result<PlantUmlBlockRender, PlantUmlError> {
    let options = plantuml_options_for_appearance(&context.appearance);
    let measurer = ExternalPlantUmlTextMeasurer {
        measurements: &context.plantuml_text_metrics,
        base_style: options.base_text_style(),
    };
    match PlantUml::new().render_with_text_measurer(source, &options, &measurer) {
        Ok(rendered) => Ok(PlantUmlBlockRender::Svg(rendered.svg)),
        Err(error @ PlantUmlError::UnsupportedDiagram { .. }) => {
            let title = render_message(context.locale, RenderMessage::UnsupportedPlantumlTitle);
            Ok(PlantUmlBlockRender::Unsupported(format!(
                "{title}: {error}"
            )))
        }
        Err(error) => Err(error),
    }
}

pub(super) fn plantuml_options_for_appearance(appearance: &AppearanceOptions) -> PlantUmlOptions {
    PlantUmlOptions {
        font_family: appearance.body_font_family.css_stack(),
        body_font_size_px: Some(resolve_body_font_size_px(appearance.body_font_size_px)),
        style: PlantUmlDiagramStyle::default(),
    }
}

pub(super) fn render_plantuml_error_figure(
    out: &mut String,
    context: &RenderContext,
    block_id: &str,
    language: &str,
    message: &str,
    source: &str,
) {
    let title = render_message(context.locale, RenderMessage::UnsupportedPlantumlTitle);
    out.push_html(&format!(
        r#"<figure id="{}" class="oriha-plantuml oriha-plantuml--error" data-oriha-block-kind="codeBlock" data-oriha-code-language="{}"><figcaption class="oriha-plantuml__error-title">{}</figcaption><p class="oriha-plantuml__error-message">{}</p><pre class="oriha-plantuml__source"><code class="language-{}">{}</code></pre></figure>"#,
        escape_attr(block_id),
        escape_attr(language),
        escape_text(title),
        escape_text(message),
        escape_attr(language),
        escape_text(source)
    ));
}

pub(super) struct ExternalPlantUmlTextMeasurer<'a> {
    pub(super) measurements: &'a BTreeMap<String, InlineTextMeasurementResult>,
    pub(super) base_style: InlineTextMeasurementStyle,
}

impl TextMeasurer for ExternalPlantUmlTextMeasurer<'_> {
    fn base_inline_text_style(&self) -> InlineTextMeasurementStyle {
        self.base_style.clone()
    }

    fn measure_inline_text_request(
        &self,
        request: &InlineTextMeasurementRequest,
    ) -> InlineTextMeasurementResult {
        self.measurements
            .get(&request.id)
            .cloned()
            .unwrap_or_else(|| oriha_plantuml::approximate_inline_text_measurement(request))
    }
}
