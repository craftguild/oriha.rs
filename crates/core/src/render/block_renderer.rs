use super::*;

pub(super) struct DefaultBlockRenderer;

impl BlockRenderer for DefaultBlockRenderer {
    fn kind(&self) -> &'static str {
        "default"
    }

    fn block_emission(&self, block: &BlockNode) -> crate::BlockEmission {
        use crate::{BlockEmission as E, BlockNullReason as N};
        match &block.kind {
            BlockKind::BlankLine => E::Null(N::BlankLine),
            BlockKind::LinkReferenceDefinition => E::Null(N::ReferenceDefinition),
            BlockKind::Document
            | BlockKind::Paragraph(_)
            | BlockKind::Heading(_)
            | BlockKind::FencedCode(_)
            | BlockKind::IndentedCode(_)
            | BlockKind::MathBlock(_)
            | BlockKind::ThematicBreak
            | BlockKind::HtmlBlock
            | BlockKind::BlockQuote
            | BlockKind::List(_)
            | BlockKind::ListItem(_)
            | BlockKind::GfmTable(_)
            | BlockKind::Callout(_)
            | BlockKind::ForeignHtml(_) => E::Html,
        }
    }

    fn render(
        &self,
        block: &mut BlockNode,
        source: &SourceDocument,
        state: &mut RenderState,
        context: &RenderContext,
        renderers: &[Box<dyn BlockRenderer>],
        out: &mut String,
    ) {
        let span_text = source.source_span(block.span);
        match &block.kind {
            BlockKind::Document => {
                render_blocks_from_source(
                    &mut block.children,
                    source,
                    context,
                    renderers,
                    state,
                    out,
                    false,
                );
                state
                    .source_map
                    .push(mapped_block(block, source, "document", BTreeMap::new()));
            }
            BlockKind::Paragraph(node) => {
                if let Some(image) = standalone_image(&node.children) {
                    render_standalone_image(&block.id, image, state, context, out);
                    state
                        .source_map
                        .push(mapped_block(block, source, "image", BTreeMap::new()));
                    return;
                }

                let rendered_inline =
                    render_inline_ast_with_context(&node.children, state, context);
                if block.attrs.contains_key("tight_list_item") {
                    out.push_html(&rendered_inline);
                    state.source_map.push(mapped_block(
                        block,
                        source,
                        "paragraph",
                        BTreeMap::new(),
                    ));
                    return;
                }
                out.push_html(&format!(
                    r#"<p id="{}" class="oriha-paragraph" data-oriha-block-kind="paragraph">{}"#,
                    escape_attr(&block.id),
                    rendered_inline
                ));
                out.push_html("</p>");
                state
                    .source_map
                    .push(mapped_block(block, source, "paragraph", BTreeMap::new()));
            }
            BlockKind::Heading(node) => {
                let level = node.level.clamp(1, 6);
                let heading_id = unique_heading_id(
                    &mut state.heading_ids,
                    &render_inline_heading_slug_text(&node.children),
                );
                let inner = render_inline_ast_with_context(&node.children, state, context);
                out.push_html(&format!(
                    r#"<h{level} id="{}" class="oriha-heading oriha-heading--h{level}" data-oriha-block-kind="heading" data-oriha-block-id="{}" data-oriha-heading-level="{level}">{inner}</h{level}>"#,
                    escape_attr(&heading_id),
                    escape_attr(&block.id)
                ));
                state.stats.record_heading_level(level);
                state
                    .source_map
                    .push(mapped_block(block, source, "heading", BTreeMap::new()));
            }
            BlockKind::FencedCode(node) => {
                render_fenced_code(node, source, block, out, state, context);
                state.source_map.push(mapped_block(
                    block,
                    source,
                    "codeBlock",
                    metadata_code_block(node),
                ));
            }
            BlockKind::IndentedCode(node) => {
                let code = block
                    .attrs
                    .get("normalized_code")
                    .map_or_else(|| source.source_span(node.body_span), String::as_str);
                render_code_block_html(
                    out,
                    context,
                    CodeBlockRender {
                        id: &block.id,
                        block_kind: "indentedCode",
                        language: "plain",
                        code_class: "",
                        code,
                        syntax_highlight: context.features.syntax_highlight,
                    },
                );
                out.push_html_char('\n');
                state.stats.record_code_block_language("plain");
                state.source_map.push(mapped_block(
                    block,
                    source,
                    "indentedCode",
                    metadata_code_block(node),
                ));
            }
            BlockKind::MathBlock(node) => {
                render_math_block_node(node, source, block, out, state, context);
                state
                    .source_map
                    .push(mapped_block(block, source, "mathBlock", BTreeMap::new()));
            }
            BlockKind::ThematicBreak => {
                out.push_html(&format!(
                    r#"<hr id="{}" class="oriha-hr" data-oriha-block-kind="thematicBreak" />"#,
                    escape_attr(&block.id)
                ));
                state.source_map.push(mapped_block(
                    block,
                    source,
                    "thematicBreak",
                    BTreeMap::new(),
                ));
            }
            BlockKind::HtmlBlock => {
                let html_text = if context.features.raw_html {
                    if context.features.gfm {
                        gfm_filter_disallowed_raw_html(span_text)
                    } else {
                        span_text.to_string()
                    }
                } else {
                    escape_text(span_text)
                };
                out.push_html(&format!(
                    r#"<div id="{}" class="oriha-foreign-html-wrapper" data-oriha-block-kind="htmlBlock">{html_text}</div>"#,
                    escape_attr(&block.id)
                ));
                state
                    .source_map
                    .push(mapped_block(block, source, "htmlBlock", BTreeMap::new()));
            }
            BlockKind::LinkReferenceDefinition => {
                state.source_map.push(mapped_block(
                    block,
                    source,
                    "linkReferenceDefinition",
                    BTreeMap::new(),
                ));
            }
            BlockKind::BlockQuote => {
                out.push_html(&format!(
                    r#"<blockquote id="{}" class="oriha-blockquote" data-oriha-block-kind="blockQuote">"#,
                    escape_attr(&block.id)
                ));
                if !block.children.is_empty() {
                    if let Some(normalized) = block.attrs.get("normalized_source") {
                        let nested_source = SourceDocument::new(normalized.clone());
                        render_blocks_from_source(
                            &mut block.children,
                            &nested_source,
                            context,
                            renderers,
                            state,
                            out,
                            false,
                        );
                    } else {
                        render_blocks_from_source(
                            &mut block.children,
                            source,
                            context,
                            renderers,
                            state,
                            out,
                            false,
                        );
                    }
                }
                out.push_html("</blockquote>");
                state
                    .source_map
                    .push(mapped_block(block, source, "blockQuote", BTreeMap::new()));
            }
            BlockKind::List(node) => {
                let tag = if node.ordered { "ol" } else { "ul" };
                let start_attr = ordered_list_start_attr(node);
                let task_list_class = if is_task_list(block) {
                    " oriha-task-list"
                } else {
                    ""
                };
                out.push_html(&format!(
                    r#"<{tag}{start_attr} id="{}" class="oriha-list{task_list_class}" data-oriha-block-kind="list">"#,
                    escape_attr(&block.id)
                ));
                if block.children.is_empty() {
                } else {
                    render_blocks_from_source(
                        &mut block.children,
                        source,
                        context,
                        renderers,
                        state,
                        out,
                        false,
                    );
                }
                out.push_html(&format!("</{tag}>"));
                state.source_map.push(mapped_block(
                    block,
                    source,
                    "list",
                    BTreeMap::from([
                        ("ordered".to_string(), node.ordered.to_string()),
                        ("marker".to_string(), node.marker.clone()),
                    ]),
                ));
            }
            BlockKind::ListItem(node) => {
                let task_class = if node.task.is_some() {
                    " oriha-task-list-item"
                } else {
                    ""
                };
                out.push_html(&format!(
                    r#"<li id="{}" class="oriha-list-item{task_class}" data-oriha-list-marker="{}">"#,
                    escape_attr(&block.id),
                    escape_attr(&node.raw_prefix)
                ));
                if let Some(task) = &node.task {
                    out.push_html(&render_task_checkbox(task.checked, context.mode));
                }
                if !block.children.is_empty() {
                    let local_source;
                    let child_source =
                        if let Some(normalized) = block.attrs.get("normalized_source") {
                            local_source = SourceDocument::new(normalized.clone());
                            &local_source
                        } else {
                            source
                        };
                    render_list_item_children(
                        &mut block.children,
                        child_source,
                        context,
                        renderers,
                        state,
                        out,
                    );
                }
                out.push_html("</li>");
                state
                    .source_map
                    .push(mapped_block(block, source, "listItem", BTreeMap::new()));
            }
            BlockKind::GfmTable(node) => {
                render_gfm_table(node, state, context, out);
                state
                    .source_map
                    .push(mapped_block(block, source, "table", BTreeMap::new()));
            }
            BlockKind::BlankLine => {
                state
                    .source_map
                    .push(mapped_block(block, source, "blankLine", BTreeMap::new()));
            }
            BlockKind::Callout(node) => {
                let node = node.clone();
                render_callout(block, &node, source, state, context, renderers, out);
            }
            BlockKind::ForeignHtml(node) => {
                let body = if context.features.raw_html {
                    node.html.clone()
                } else {
                    escape_text(&node.html)
                };
                out.push_html(&format!(
                    r#"<div id="{}" class="oriha-foreign-html" data-oriha-origin="{}" data-oriha-block-kind="foreignHtml">{body}</div>"#,
                    escape_attr(&block.id),
                    escape_attr(&node.origin)
                ));
                state
                    .source_map
                    .push(mapped_block(block, source, "foreignHtml", BTreeMap::new()));
            }
        }
    }
}
