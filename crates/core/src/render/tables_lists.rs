use super::*;

pub(super) fn ordered_list_start_attr(node: &crate::ir::ListNode) -> String {
    if !node.ordered {
        return String::new();
    }
    let digits = node
        .marker
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    let normalized = digits.trim_start_matches('0');
    let normalized = if normalized.is_empty() {
        "0"
    } else {
        normalized
    };
    if normalized == "1" {
        String::new()
    } else {
        format!(r#" start="{}""#, escape_attr(normalized))
    }
}

pub(super) fn render_task_checkbox(checked: bool, mode: RenderMode) -> String {
    match mode {
        RenderMode::SemanticHtml => {
            if checked {
                r#"<input checked="" disabled="" type="checkbox"> "#.to_string()
            } else {
                r#"<input disabled="" type="checkbox"> "#.to_string()
            }
        }
        RenderMode::StyledHtml => {
            let checked_class = if checked {
                " oriha-task-checkbox--checked"
            } else {
                ""
            };
            let checked_value = if checked { "true" } else { "false" };
            format!(
                r#"<span class="oriha-task-checkbox{checked_class}" role="checkbox" aria-checked="{checked_value}" aria-disabled="true"></span> "#
            )
        }
    }
}

pub(super) fn is_task_list(block: &BlockNode) -> bool {
    !block.children.is_empty()
        && block
            .children
            .iter()
            .all(|child| matches!(&child.kind, BlockKind::ListItem(item) if item.task.is_some()))
}

pub(super) fn render_gfm_table(
    node: &GfmTableNode,
    state: &mut RenderState,
    context: &RenderContext,
    out: &mut String,
) {
    out.push_html(
        r#"<div class="oriha-table-scroll-wrapper" data-oriha-block-kind="table"><table class="oriha-table"><thead><tr>"#,
    );
    for (index, cell) in node.header.iter().enumerate() {
        out.push_html("<th");
        push_table_alignment_attr(node.alignments.get(index).copied(), out);
        out.push_html_char('>');
        render_table_cell(cell, state, context, out);
        out.push_html("</th>");
    }
    out.push_html("</tr></thead>");
    if !node.rows.is_empty() {
        out.push_html("<tbody>");
        for row in &node.rows {
            out.push_html("<tr>");
            for (index, cell) in row.cells.iter().enumerate() {
                out.push_html("<td");
                push_table_alignment_attr(node.alignments.get(index).copied(), out);
                out.push_html_char('>');
                render_table_cell(cell, state, context, out);
                out.push_html("</td>");
            }
            out.push_html("</tr>");
        }
        out.push_html("</tbody>");
    }
    out.push_html("</table></div>");
}

pub(super) fn push_table_alignment_attr(alignment: Option<TableAlignment>, out: &mut String) {
    let Some(alignment) = alignment else {
        return;
    };
    let value = match alignment {
        TableAlignment::None => return,
        TableAlignment::Left => "left",
        TableAlignment::Center => "center",
        TableAlignment::Right => "right",
    };
    out.push_html(r#" align=""#);
    out.push_html(value);
    out.push_html_char('"');
}

pub(super) fn render_table_cell(
    cell: &TableCellNode,
    state: &mut RenderState,
    context: &RenderContext,
    out: &mut String,
) {
    render_inline_nodes_with_context(&cell.children, out, state, context);
}

pub(super) fn mapped_block(
    block: &BlockNode,
    source: &SourceDocument,
    kind: &str,
    metadata: BTreeMap<String, String>,
) -> RenderedBlock {
    RenderedBlock {
        id: block.id.clone(),
        kind: kind.to_string(),
        source_range: source.to_source_range(block.span),
        metadata,
    }
}

pub(super) fn metadata_code_block(node: &CodeBlockNode) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if let Some(language) = &node.language {
        metadata.insert("language".to_string(), language.clone());
    }
    if !node.info.is_empty() {
        metadata.insert("info".to_string(), node.info.clone());
    }
    metadata
}
