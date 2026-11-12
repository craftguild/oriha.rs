use super::*;

pub(super) fn derive(
    engine: &OrihaEngine,
    parsed: &ParsedDocument,
) -> Result<BlockOutput, DerivationError> {
    let limit = parsed.limits.max_output_bytes;
    // Reject oversized retained resources before cloning the working state.
    if !fits(&(&parsed.state.assets, &parsed.state.diagnostics), limit) {
        return Err(DerivationError::limit());
    }
    let mut document = parsed.document.clone();
    let mut state = parsed.state.clone();
    let mut context = parsed
        .context
        .borrow(&engine.hooks, engine.inline_renderer.as_ref(), limit);
    context.mode = RenderMode::SemanticHtml;
    let mut blocks = Vec::new();
    let mut b = Budget::new(limit);
    b.reserve(crate::blocks::budget::size(
        &(&state.assets, &state.diagnostics),
        limit,
    )?)?;
    let mut asset_count = state.assets.len();
    let mut diagnostic_count = state.diagnostics.len();
    for block in &mut document.blocks {
        let entry = crate::blocks::html_budget::scoped(b.remaining(), || {
            let mut fragment = RenderFragment::default();
            for hook in context.hooks {
                hook.before_block_render(block, &mut fragment, &mut state, &context);
            }
            let renderer = select_renderer(&engine.block_renderers, block);
            renderer.render(
                block,
                &context.source,
                &mut state,
                &context,
                &engine.block_renderers,
                &mut fragment.html,
            );
            for hook in context.hooks {
                hook.after_block_render(block, &mut fragment, &mut state, &context);
            }
            let content = match renderer.block_emission(block) {
                BlockEmission::Html => RenderedBlockContent::Html(fragment.html),
                BlockEmission::Null(reason) => {
                    if reason == BlockNullReason::Unsupported {
                        state.diagnostics.push(RenderDiagnostic {
                            severity: Severity::Warning,
                            code: "blocks.unsupported_projection".into(),
                            message: format!("Renderer explicitly omitted {:?}", kind(&block.kind)),
                            source_range: None,
                            block_id: Some(block.id.clone()),
                            engine: DiagnosticEngine::Core,
                        });
                    }
                    RenderedBlockContent::Null(reason)
                }
            };
            state.attrs.extend(fragment.attrs);
            state.stats.total_blocks += 1;
            state.stats.record_block_kind(block_stats_kind(&block.kind));
            RenderedHtmlBlock {
                id: block.id.clone(),
                kind: kind(&block.kind),
                content,
            }
        })?;
        b.reserve(crate::blocks::budget::size(&entry, b.remaining())?)?;
        // Charge newly registered resources and diagnostics before accumulating
        // another root. Final aggregate validation accounts for envelope syntax.
        // Count items, not temporary slice/tuple wrappers. Empty deltas cost
        // nothing; final serialization accounts for real arrays and commas.
        for asset in &state.assets[asset_count.min(state.assets.len())..] {
            b.reserve(crate::blocks::budget::size(asset, b.remaining())?)?;
        }
        for diagnostic in &state.diagnostics[diagnostic_count.min(state.diagnostics.len())..] {
            b.reserve(crate::blocks::budget::size(diagnostic, b.remaining())?)?;
        }
        asset_count = state.assets.len();
        diagnostic_count = state.diagnostics.len();
        blocks.push(entry);
    }
    validate::document(&document, parsed.limits)?;
    let mut footnotes = String::new();
    crate::blocks::html_budget::scoped(b.remaining(), || {
        render_footnote_section(&context, &mut state, &mut footnotes)
    })?;
    if !footnotes.is_empty() {
        let mut id = "oriha-generated-footnotes".to_owned();
        let mut suffix = 1;
        let mut ids = std::collections::BTreeSet::new();
        let mut pending: Vec<_> = document.blocks.iter().collect();
        while let Some(block) = pending.pop() {
            ids.insert(block.id.as_str());
            pending.extend(block.children.iter());
        }
        while ids.contains(id.as_str()) {
            id = format!("oriha-generated-footnotes-{suffix}");
            suffix += 1;
        }
        b.reserve(footnotes.len())?;
        blocks.push(RenderedHtmlBlock {
            id,
            kind: HtmlBlockKind::Footnotes,
            content: RenderedBlockContent::Html(footnotes),
        });
    }
    Ok(BlockOutput::Rendered(RenderedBlockOutput {
        schema_version: 1,
        mode: BlockOutputMode::Rendered,
        status: BlockOutputStatus::Complete,
        blocks,
        assets: state.assets,
        diagnostics: tree_diagnostics(&document.blocks, &state.diagnostics, limit),
    }))
}

fn kind(kind: &BlockKind) -> HtmlBlockKind {
    match kind {
        BlockKind::Document => HtmlBlockKind::Document,
        BlockKind::Paragraph(_) => HtmlBlockKind::Paragraph,
        BlockKind::Heading(_) => HtmlBlockKind::Heading,
        BlockKind::FencedCode(_) => HtmlBlockKind::FencedCode,
        BlockKind::IndentedCode(_) => HtmlBlockKind::IndentedCode,
        BlockKind::MathBlock(_) => HtmlBlockKind::MathBlock,
        BlockKind::ThematicBreak => HtmlBlockKind::ThematicBreak,
        BlockKind::HtmlBlock => HtmlBlockKind::HtmlBlock,
        BlockKind::LinkReferenceDefinition => HtmlBlockKind::LinkReferenceDefinition,
        BlockKind::BlockQuote => HtmlBlockKind::BlockQuote,
        BlockKind::List(_) => HtmlBlockKind::List,
        BlockKind::ListItem(_) => HtmlBlockKind::ListItem,
        BlockKind::GfmTable(_) => HtmlBlockKind::GfmTable,
        BlockKind::BlankLine => HtmlBlockKind::BlankLine,
        BlockKind::Callout(_) => HtmlBlockKind::Callout,
        BlockKind::ForeignHtml(_) => HtmlBlockKind::ForeignHtml,
    }
}
