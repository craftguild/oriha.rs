use super::*;

pub(super) fn render_blocks(
    blocks: &mut [BlockNode],
    context: &RenderContext,
    renderers: &[Box<dyn BlockRenderer>],
    state: &mut RenderState,
    out: &mut String,
) {
    render_blocks_from_source(
        blocks,
        &context.source,
        context,
        renderers,
        state,
        out,
        true,
    );
}

pub(super) fn render_blocks_from_source(
    blocks: &mut [BlockNode],
    source: &SourceDocument,
    context: &RenderContext,
    renderers: &[Box<dyn BlockRenderer>],
    state: &mut RenderState,
    out: &mut String,
    capture_patch_blocks: bool,
) {
    for block in blocks {
        let _depth = crate::blocks::html_budget::enter();
        let mut fragment = RenderFragment::default();
        for hook in context.hooks {
            hook.before_block_render(block, &mut fragment, state, context);
        }

        let renderer = select_renderer(renderers, block);
        renderer.render(block, source, state, context, renderers, &mut fragment.html);

        for hook in context.hooks {
            hook.after_block_render(block, &mut fragment, state, context);
        }

        if !fragment.attrs.is_empty() {
            state.attrs.extend(fragment.attrs);
        }

        if capture_patch_blocks {
            if let Some(patch_block) = context.patch_blocks.get(&block.id) {
                attach_stable_id_attr(&mut fragment.html, &patch_block.stable_id);
                state.rendered_patch_blocks.push(RenderedPatchBlock {
                    stable_id: patch_block.stable_id.clone(),
                    kind: patch_block.kind.clone(),
                    source_hash: patch_block.source_hash.clone(),
                    html_hash: patch::html_hash(&fragment.html),
                    source_range: patch_block.source_range,
                    html: fragment.html.clone(),
                });
            }
        }

        out.push_html(&fragment.html);
        state.stats.total_blocks += 1;
        state.stats.record_block_kind(block_stats_kind(&block.kind));
    }
}

pub(super) fn attach_stable_id_attr(fragment: &mut String, stable_id: &str) {
    if fragment.is_empty() {
        return;
    }

    let Some(tag_start) = fragment.find('<') else {
        return;
    };
    if !fragment[..tag_start].trim().is_empty() {
        return;
    }
    if fragment[tag_start..].starts_with("</") {
        return;
    }

    let Some(tag_end_offset) = fragment[tag_start..].find('>') else {
        return;
    };
    let tag_end = tag_start + tag_end_offset;
    if opening_tag_has_attr(&fragment[tag_start..=tag_end], "data-oriha-stable-id") {
        return;
    }

    let mut insert_at = tag_end;
    let bytes = fragment.as_bytes();
    let mut cursor = tag_end;
    while cursor > tag_start && bytes[cursor - 1].is_ascii_whitespace() {
        cursor -= 1;
    }
    if cursor > tag_start && bytes[cursor - 1] == b'/' {
        insert_at = cursor - 1;
        while insert_at > tag_start && bytes[insert_at - 1].is_ascii_whitespace() {
            insert_at -= 1;
        }
    }

    fragment.insert_str(
        insert_at,
        &format!(r#" data-oriha-stable-id="{}""#, escape_attr(stable_id)),
    );
}

pub(super) fn opening_tag_has_attr(tag: &str, attr_name: &str) -> bool {
    if !tag.starts_with('<')
        || tag.starts_with("</")
        || tag.starts_with("<!")
        || tag.starts_with("<?")
        || !tag.ends_with('>')
    {
        return false;
    }

    let inner = &tag[1..tag.len() - 1];
    let bytes = inner.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
        i += 1;
    }

    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] == b'/' {
            break;
        }

        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let name = &inner[name_start..i];
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() {
                    let done = bytes[i] == quote;
                    i += 1;
                    if done {
                        break;
                    }
                }
            } else {
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'/' {
                    i += 1;
                }
            }
        }

        if name == attr_name {
            return true;
        }
    }

    false
}

pub(super) fn select_renderer<'a>(
    renderers: &'a [Box<dyn BlockRenderer>],
    block: &BlockNode,
) -> &'a dyn BlockRenderer {
    let kind = block_kind_name(&block.kind);
    for renderer in renderers.iter().rev() {
        if renderer.kind() == kind {
            return renderer.as_ref();
        }
    }
    for renderer in renderers.iter().rev() {
        if renderer.kind() == "default" {
            return renderer.as_ref();
        }
    }
    renderers
        .first()
        .expect("at least one block renderer must be registered")
        .as_ref()
}

pub(super) fn render_list_item_children(
    blocks: &mut [BlockNode],
    source: &SourceDocument,
    context: &RenderContext,
    renderers: &[Box<dyn BlockRenderer>],
    state: &mut RenderState,
    out: &mut String,
) {
    let len = blocks.len();
    for (index, block) in blocks.iter_mut().enumerate() {
        render_blocks_from_source(
            std::slice::from_mut(block),
            source,
            context,
            renderers,
            state,
            out,
            false,
        );
        if index + 1 < len && !out.ends_with('\n') {
            out.push_html_char('\n');
        }
    }
}

pub(super) fn block_kind_name(kind: &BlockKind) -> &'static str {
    match kind {
        BlockKind::Document => "document",
        BlockKind::Paragraph(_) => "paragraph",
        BlockKind::Heading(_) => "heading",
        BlockKind::FencedCode(_) => "codeBlock",
        BlockKind::IndentedCode(_) => "indentedCode",
        BlockKind::MathBlock(_) => "mathBlock",
        BlockKind::ThematicBreak => "thematicBreak",
        BlockKind::HtmlBlock => "htmlBlock",
        BlockKind::LinkReferenceDefinition => "linkReferenceDefinition",
        BlockKind::BlockQuote => "blockQuote",
        BlockKind::List(_) => "list",
        BlockKind::ListItem(_) => "listItem",
        BlockKind::GfmTable(_) => "table",
        BlockKind::BlankLine => "blankLine",
        BlockKind::Callout(_) => "callout",
        BlockKind::ForeignHtml(_) => "foreignHtml",
    }
}

pub(super) fn block_stats_kind(kind: &BlockKind) -> RenderBlockKind {
    match kind {
        BlockKind::Document => RenderBlockKind::Document,
        BlockKind::Paragraph(_) => RenderBlockKind::Paragraph,
        BlockKind::Heading(_) => RenderBlockKind::Heading,
        BlockKind::FencedCode(_) => RenderBlockKind::FencedCode,
        BlockKind::IndentedCode(_) => RenderBlockKind::IndentedCode,
        BlockKind::MathBlock(_) => RenderBlockKind::MathBlock,
        BlockKind::ThematicBreak => RenderBlockKind::ThematicBreak,
        BlockKind::HtmlBlock => RenderBlockKind::HtmlBlock,
        BlockKind::LinkReferenceDefinition => RenderBlockKind::LinkReferenceDefinition,
        BlockKind::BlockQuote => RenderBlockKind::BlockQuote,
        BlockKind::List(_) => RenderBlockKind::List,
        BlockKind::ListItem(_) => RenderBlockKind::ListItem,
        BlockKind::GfmTable(_) => RenderBlockKind::GfmTable,
        BlockKind::BlankLine => RenderBlockKind::BlankLine,
        BlockKind::Callout(_) => RenderBlockKind::Callout,
        BlockKind::ForeignHtml(_) => RenderBlockKind::ForeignHtml,
    }
}

pub(super) fn render_wrapper(
    class_name: &str,
    theme_name: &str,
    attrs: &BTreeMap<String, String>,
    body: &str,
) -> String {
    let attributes = attrs
        .iter()
        .map(|(key, value)| {
            format!(
                " data-oriha-{}=\"{}\"",
                key.replace('_', "-"),
                escape_attr(value)
            )
        })
        .collect::<String>();
    format!(
        "<article class=\"{class_name}\" data-oriha-theme=\"{}\"{attributes}>{body}</article>",
        escape_attr(theme_name),
    )
}
