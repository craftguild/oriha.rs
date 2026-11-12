use super::*;

pub(super) fn render_fenced_code(
    node: &CodeBlockNode,
    source: &SourceDocument,
    block: &BlockNode,
    out: &mut String,
    state: &mut RenderState,
    context: &RenderContext,
) {
    let mut text = normalize_fenced_code_text(source.source_span(node.body_span), node.indent);
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    let normalized_language = node.language.as_ref().map(|lang| decode_markdown(lang));
    let language = normalized_language.as_deref().unwrap_or("plain");
    let class = normalized_language
        .as_ref()
        .map(|lang| format!(" class=\"language-{}\"", escape_attr(lang)))
        .unwrap_or_default();

    if context.features.math && is_latex_language(language) {
        match render_latex_source_svg(&text, LatexRenderMode::Display, &context.appearance) {
            Ok(RenderedLatex { svg }) => {
                register_bundled_math_font_asset(state);
                out.push_html(&format!(
                    r#"<figure id="{}" class="oriha-math-block" data-oriha-block-kind="codeBlock" data-oriha-code-language="{}">{}</figure>"#,
                    escape_attr(&block.id),
                    escape_attr(language),
                    svg
                ));
                out.push_html_char('\n');
                state.stats.record_code_block_language(language);
                return;
            }
            Err(error) => {
                let message = format!("LaTeX math could not be rendered: {error}");
                render_math_error_figure(out, &block.id, Some(language), &message, &text);
                out.push_html_char('\n');
                state.stats.record_code_block_language(language);
                state.diagnostics.push(RenderDiagnostic {
                    severity: Severity::Warning,
                    code: "math.render_failed".to_string(),
                    message,
                    source_range: Some(source.to_source_range(node.body_span)),
                    block_id: Some(block.id.clone()),
                    engine: DiagnosticEngine::Core,
                });
                return;
            }
        }
    }

    if context.features.mermaid && is_mermaid_language(language) {
        match render_mermaid_code_block(&text, &block.id, context) {
            Ok(MermaidSvg { svg, .. }) => {
                out.push_html(&format!(
                    r#"<figure id="{}" class="oriha-mermaid" data-oriha-block-kind="codeBlock" data-oriha-code-language="{}">{}</figure>"#,
                    escape_attr(&block.id),
                    escape_attr(language),
                    svg
                ));
                out.push_html_char('\n');
                state.stats.record_code_block_language(language);
                return;
            }
            Err(error) => {
                let message = format!("Mermaid diagram could not be rendered: {error}");
                render_mermaid_error_figure(out, context, &block.id, language, &message, &text);
                out.push_html_char('\n');
                state.stats.record_code_block_language(language);
                state.diagnostics.push(RenderDiagnostic {
                    severity: Severity::Warning,
                    code: "mermaid.render_failed".to_string(),
                    message,
                    source_range: Some(source.to_source_range(node.body_span)),
                    block_id: Some(block.id.clone()),
                    engine: DiagnosticEngine::Core,
                });
                return;
            }
        }
    }

    if context.features.plantuml && is_plantuml_language(language) {
        match render_plantuml_code_block(&text, context) {
            Ok(PlantUmlBlockRender::Svg(svg)) => {
                out.push_html(&format!(
                    r#"<figure id="{}" class="oriha-plantuml" data-oriha-block-kind="codeBlock" data-oriha-code-language="{}">{}</figure>"#,
                    escape_attr(&block.id),
                    escape_attr(language),
                    svg
                ));
                out.push_html_char('\n');
                state.stats.record_code_block_language(language);
                return;
            }
            Ok(PlantUmlBlockRender::Unsupported(message)) => {
                render_plantuml_error_figure(out, context, &block.id, language, &message, &text);
                out.push_html_char('\n');
                state.stats.record_code_block_language(language);
                state
                    .stats
                    .record_unsupported(UnsupportedKind::PlantumlDiagram);
                state.diagnostics.push(RenderDiagnostic {
                    severity: Severity::Warning,
                    code: "plantuml.unsupported_diagram".to_string(),
                    message,
                    source_range: Some(source.to_source_range(node.body_span)),
                    block_id: Some(block.id.clone()),
                    engine: DiagnosticEngine::Core,
                });
                return;
            }
            Err(error) => {
                state.diagnostics.push(RenderDiagnostic {
                    severity: Severity::Warning,
                    code: "plantuml.render_failed".to_string(),
                    message: format!("PlantUML diagram could not be rendered: {error}"),
                    source_range: Some(source.to_source_range(node.body_span)),
                    block_id: Some(block.id.clone()),
                    engine: DiagnosticEngine::Core,
                });
            }
        }
    }

    render_code_block_html(
        out,
        context,
        CodeBlockRender {
            id: &block.id,
            block_kind: "codeBlock",
            language,
            code_class: &class,
            code: &text,
            syntax_highlight: context.features.syntax_highlight,
        },
    );
    out.push_html_char('\n');

    state.stats.record_code_block_language(language);

    if !node.closed {
        state.diagnostics.push(RenderDiagnostic {
            severity: Severity::Warning,
            code: "code_block.unclosed".to_string(),
            message: "Code block has no closing fence.".to_string(),
            source_range: Some(source.to_source_range(node.body_span)),
            block_id: Some(block.id.clone()),
            engine: DiagnosticEngine::Core,
        });
    }
}

pub(super) struct CodeBlockRender<'a> {
    pub(super) id: &'a str,
    pub(super) block_kind: &'a str,
    pub(super) language: &'a str,
    pub(super) code_class: &'a str,
    pub(super) code: &'a str,
    pub(super) syntax_highlight: bool,
}

pub(super) fn render_code_block_html(
    out: &mut String,
    context: &RenderContext,
    block: CodeBlockRender<'_>,
) {
    let language_attr = if block.block_kind == "codeBlock" {
        format!(
            r#" data-oriha-code-language="{}""#,
            escape_attr(block.language)
        )
    } else {
        String::new()
    };
    let options = context.code_block;
    out.push_html(&format!(
        r#"<div id="{}" class="oriha-code-block-wrapper" data-oriha-block-kind="{}"{}"#,
        escape_attr(block.id),
        escape_attr(block.block_kind),
        language_attr,
    ));
    if options.show_copy_button {
        out.push_html(&format!(
            r#" data-oriha-code-copy="{}""#,
            escape_attr(block.code)
        ));
    }
    out.push_html(">");
    if options.show_language || options.show_copy_button {
        out.push_html(r#"<div class="oriha-code-block-header">"#);
        if options.show_language {
            out.push_html(&format!(
                r#"<span class="oriha-code-block-language">{}</span>"#,
                escape_text(&code_language_label(block.language)),
            ));
        }
        if options.show_copy_button {
            out.push_html(&format!(
                r#"<button type="button" class="oriha-code-copy-button" data-oriha-copy-button="true" aria-label="{}">{}<span class="oriha-code-copy-label" data-oriha-copy-label="true">{}</span></button>"#,
                escape_attr(render_message(context.locale, RenderMessage::CodeCopyAriaLabel)),
                copy_icon_svg(),
                escape_text(render_message(context.locale, RenderMessage::CodeCopyLabel)),
            ));
        }
        out.push_html("</div>");
    }
    out.push_html(&format!(
        r#"<pre class="oriha-code-block"><code{}>"#,
        block.code_class
    ));
    render_code_lines(
        out,
        block.code,
        block.language,
        block.syntax_highlight,
        options.show_line_numbers,
    );
    out.push_html("</code></pre></div>");
}

pub(super) fn render_code_lines(
    out: &mut String,
    code: &str,
    language: &str,
    syntax_highlight: bool,
    show_line_numbers: bool,
) {
    let display_code = code.strip_suffix('\n').unwrap_or(code);
    let lines = if syntax_highlight {
        highlight_lines(display_code, language)
    } else {
        display_code.split('\n').map(escape_text).collect()
    };
    let lines = if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    };

    if show_line_numbers {
        out.push_html(r#"<span class="oriha-code-gutter" aria-hidden="true">"#);
        for number in 1..=lines.len() {
            out.push_html(&format!(
                r#"<span class="oriha-code-line-number">{number}</span>"#
            ));
        }
        out.push_html("</span>");
    }
    out.push_html(r#"<span class="oriha-code-scroll"><span class="oriha-code-lines">"#);

    for line in &lines {
        out.push_html(r#"<span class="oriha-code-line"><span class="oriha-code-line-content">"#);
        out.push_html(line);
        out.push_html("</span></span>");
    }
    out.push_html("</span></span>");
}

pub(super) fn code_language_label(language: &str) -> String {
    match language.trim().to_ascii_lowercase().as_str() {
        "" | "plain" | "text" | "txt" => "Plain text",
        "js" | "javascript" | "mjs" | "cjs" => "JavaScript",
        "jsx" => "JSX",
        "ts" | "typescript" => "TypeScript",
        "tsx" => "TSX",
        "rs" | "rust" => "Rust",
        "py" | "python" | "py3" => "Python",
        "html" | "htm" => "HTML",
        "xml" => "XML",
        "svg" => "SVG",
        "json" => "JSON",
        "jsonc" => "JSONC",
        "css" => "CSS",
        "scss" => "SCSS",
        "sass" => "Sass",
        "sh" | "bash" | "zsh" | "shell" => "Shell",
        "console" => "Console",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "sql" => "SQL",
        "mysql" => "MySQL",
        "postgres" | "postgresql" => "PostgreSQL",
        "md" | "markdown" => "Markdown",
        "diff" | "patch" => "Diff",
        "plantuml" | "puml" => "PlantUML",
        "mermaid" => "Mermaid",
        "latex" | "tex" => "LaTeX",
        _ => language.trim(),
    }
    .to_string()
}

pub(super) fn copy_icon_svg() -> &'static str {
    r#"<svg class="oriha-code-copy-icon size-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" aria-hidden="true" focusable="false"><path stroke-linecap="round" stroke-linejoin="round" d="M9 12h3.75M9 15h3.75M9 18h3.75m3 .75H18a2.25 2.25 0 0 0 2.25-2.25V6.108c0-1.135-.845-2.098-1.976-2.192a48.424 48.424 0 0 0-1.123-.08m-5.801 0c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 0 0 .75-.75 2.25 2.25 0 0 0-.1-.664m-5.8 0A2.251 2.251 0 0 1 13.5 2.25H15c1.012 0 1.867.668 2.15 1.586m-5.8 0c-.376.023-.75.05-1.124.08C9.095 4.01 8.25 4.973 8.25 6.108V8.25m0 0H4.875c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125V9.375c0-.621-.504-1.125-1.125-1.125H8.25ZM6.75 12h.008v.008H6.75V12Zm0 3h.008v.008H6.75V15Zm0 3h.008v.008H6.75V18Z" /></svg>"#
}
