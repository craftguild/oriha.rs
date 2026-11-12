use super::*;

pub(super) struct CoreInlineRenderer;

impl InlineRenderer for CoreInlineRenderer {
    fn render(&self, source: &str, features: &FeatureFlags, _stats: &mut RenderStats) -> String {
        render_inline_markup(source, &BTreeMap::new(), features)
    }

    fn render_with_context(
        &self,
        source: &str,
        context: &RenderContext,
        _stats: &mut RenderStats,
    ) -> String {
        render_inline_markup_with_appearance(
            source,
            &context.link_references,
            &context.features,
            &context.appearance,
        )
    }
}

pub(super) fn render_inline_markup(
    source: &str,
    link_references: &BTreeMap<String, LinkReference>,
    features: &FeatureFlags,
) -> String {
    render_inline_ast(&parse_inlines(source, link_references, features), features)
}

pub(super) fn render_inline_ast(nodes: &[InlineNode], features: &FeatureFlags) -> String {
    render_inline_ast_with_appearance(nodes, &AppearanceOptions::default(), features)
}

pub(super) fn render_inline_markup_with_appearance(
    source: &str,
    link_references: &BTreeMap<String, LinkReference>,
    features: &FeatureFlags,
    appearance: &AppearanceOptions,
) -> String {
    render_inline_ast_with_appearance(
        &parse_inlines(source, link_references, features),
        appearance,
        features,
    )
}

pub(super) fn render_inline_ast_with_appearance(
    nodes: &[InlineNode],
    appearance: &AppearanceOptions,
    features: &FeatureFlags,
) -> String {
    let mut out = String::new();
    render_inline_nodes(nodes, &mut out, appearance, features);
    out
}

pub(super) fn render_inline_ast_with_context(
    nodes: &[InlineNode],
    state: &mut RenderState,
    context: &RenderContext,
) -> String {
    let mut out = String::new();
    render_inline_nodes_with_context(nodes, &mut out, state, context);
    out
}

pub(super) fn render_annotation_open(annotation: &crate::AnnotationMetadata, out: &mut String) {
    out.push_html(&format!(
        "<span data-oriha-annotation-kind=\"{}\" data-oriha-annotation-id=\"{}\"",
        escape_attr(&annotation.kind),
        escape_attr(&annotation.id)
    ));
    if !annotation.attributes.classes.is_empty() {
        out.push_html(&format!(
            " class=\"{}\"",
            escape_attr(&annotation.attributes.classes.join(" "))
        ));
    }
    for (key, value) in &annotation.attributes.data {
        if !crate::annotations::valid_data_name(key) {
            continue;
        }
        out.push_html(&format!(" data-{}=\"{}\"", key, escape_attr(value)));
    }
    out.push_html_char('>');
}

pub(super) fn render_inline_nodes(
    nodes: &[InlineNode],
    out: &mut String,
    appearance: &AppearanceOptions,
    features: &FeatureFlags,
) {
    for node in nodes {
        match &node.kind {
            InlineKind::Annotation {
                annotation,
                children,
            } => {
                render_annotation_open(annotation, out);
                render_inline_nodes(children, out, appearance, features);
                out.push_html("</span>");
            }
            InlineKind::Text(text) => out.push_html(&escape_text(text)),
            InlineKind::Emoji { emoji, .. } => out.push_html(&escape_text(emoji)),
            InlineKind::Code(code) => {
                out.push_html("<code>");
                out.push_html(&escape_text(code));
                out.push_html("</code>");
            }
            InlineKind::Math { source } => {
                out.push_html(&render_inline_math_or_literal(source, appearance));
            }
            InlineKind::Html { raw, tagfilter } => {
                render_authored_inline_html(raw, *tagfilter, features.raw_html, out);
            }
            InlineKind::SoftBreak => out.push_html_char('\n'),
            InlineKind::HardBreak => out.push_html("<br />\n"),
            InlineKind::Emphasis { children } => {
                out.push_html("<em>");
                render_inline_nodes(children, out, appearance, features);
                out.push_html("</em>");
            }
            InlineKind::Strong { children } => {
                out.push_html("<strong>");
                render_inline_nodes(children, out, appearance, features);
                out.push_html("</strong>");
            }
            InlineKind::Strikethrough { children } => {
                out.push_html("<del>");
                render_inline_nodes(children, out, appearance, features);
                out.push_html("</del>");
            }
            InlineKind::Link {
                destination,
                title,
                children,
            } => {
                let title_attr = title
                    .as_ref()
                    .map(|value| format!(" title=\"{}\"", escape_attr(value)))
                    .unwrap_or_default();
                out.push_html(&format!(
                    "<a href=\"{}\"{}>",
                    escape_attr(&sanitize_url(destination)),
                    title_attr
                ));
                render_inline_nodes(children, out, appearance, features);
                out.push_html("</a>");
            }
            InlineKind::Image {
                destination,
                title,
                alt,
            } => {
                out.push_html(&render_image_element(
                    destination,
                    alt,
                    title.as_deref(),
                    "oriha-inline-image",
                ));
            }
            InlineKind::FootnoteReference { label } => {
                out.push_html("[^");
                out.push_html(&escape_text(label));
                out.push_html_char(']');
            }
        }
    }
}

pub(super) fn render_inline_nodes_with_context(
    nodes: &[InlineNode],
    out: &mut String,
    state: &mut RenderState,
    context: &RenderContext,
) {
    for node in nodes {
        match &node.kind {
            InlineKind::Annotation {
                annotation,
                children,
            } => {
                render_annotation_open(annotation, out);
                render_inline_nodes_with_context(children, out, state, context);
                out.push_html("</span>");
            }
            InlineKind::Text(text) => out.push_html(&escape_text(text)),
            InlineKind::Emoji { emoji, .. } => out.push_html(&escape_text(emoji)),
            InlineKind::Code(code) => {
                out.push_html("<code>");
                out.push_html(&escape_text(code));
                out.push_html("</code>");
            }
            InlineKind::Math { source } => {
                out.push_html(&render_inline_math_or_literal_with_state(
                    source, state, context,
                ));
            }
            InlineKind::Html { raw, tagfilter } => {
                render_authored_inline_html(raw, *tagfilter, context.features.raw_html, out);
            }
            InlineKind::SoftBreak => out.push_html_char('\n'),
            InlineKind::HardBreak => out.push_html("<br />\n"),
            InlineKind::Emphasis { children } => {
                out.push_html("<em>");
                render_inline_nodes_with_context(children, out, state, context);
                out.push_html("</em>");
            }
            InlineKind::Strong { children } => {
                out.push_html("<strong>");
                render_inline_nodes_with_context(children, out, state, context);
                out.push_html("</strong>");
            }
            InlineKind::Strikethrough { children } => {
                out.push_html("<del>");
                render_inline_nodes_with_context(children, out, state, context);
                out.push_html("</del>");
            }
            InlineKind::Link {
                destination,
                title,
                children,
            } => {
                let title_attr = title
                    .as_ref()
                    .map(|value| format!(" title=\"{}\"", escape_attr(value)))
                    .unwrap_or_default();
                out.push_html(&format!(
                    "<a href=\"{}\"{}>",
                    escape_attr(&sanitize_url(destination)),
                    title_attr
                ));
                render_inline_nodes_with_context(children, out, state, context);
                out.push_html("</a>");
            }
            InlineKind::Image {
                destination,
                title,
                alt,
            } => {
                let image = ResolvedImage::new(
                    ImageRenderRequest {
                        destination,
                        alt,
                        title: title.as_deref(),
                        placement: ImagePlacement::Inline,
                    },
                    context,
                );
                out.push_html(&image.render("oriha-inline-image", false, context, state));
            }
            InlineKind::FootnoteReference { label } => {
                render_footnote_reference(label, state, context, out);
            }
        }
    }
}

fn render_authored_inline_html(raw: &str, tagfilter: bool, raw_html: bool, out: &mut String) {
    if !raw_html {
        out.push_html(&escape_text(raw));
    } else if tagfilter {
        out.push_html(&gfm_filter_disallowed_raw_html(raw));
    } else {
        out.push_html(raw);
    }
}

pub(super) fn render_inline_math_or_literal(
    source: &str,
    appearance: &AppearanceOptions,
) -> String {
    render_inline_math_or_literal_inner(source, None, appearance)
}

pub(super) fn render_inline_math_or_literal_with_state(
    source: &str,
    state: &mut RenderState,
    context: &RenderContext,
) -> String {
    render_inline_math_or_literal_inner(source, Some(state), &context.appearance)
}

pub(super) fn render_inline_math_or_literal_inner(
    source: &str,
    state: Option<&mut RenderState>,
    appearance: &AppearanceOptions,
) -> String {
    match render_latex_source_svg(source, LatexRenderMode::Inline, appearance) {
        Ok(RenderedLatex { svg }) => {
            if let Some(state) = state {
                register_bundled_math_font_asset(state);
            }
            format!(r#"<span class="oriha-math-inline">{svg}</span>"#)
        }
        Err(_) => {
            let mut out = String::new();
            out.push_html_char('$');
            out.push_html(&escape_text(source));
            out.push_html_char('$');
            out
        }
    }
}
