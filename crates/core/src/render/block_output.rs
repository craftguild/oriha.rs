//! Derive owned output without reparsing or document finalization.
use super::*;
use crate::blocks::budget::{fits, Budget, DerivationError};
use crate::blocks::*;

mod rendered;
mod validate;

pub fn derive_blocks(input: RenderInput, mode: BlockOutputMode) -> BlockOutput {
    OrihaEngine::default().derive_blocks(input, mode)
}

pub fn derive_parsed_blocks(parsed: &ParsedDocument, mode: BlockOutputMode) -> BlockOutput {
    OrihaEngine::default().derive_parsed_blocks(parsed, mode)
}

impl OrihaEngine {
    /// Parse once, then derive the selected owned schema-v1 representation.
    pub fn derive_blocks(&self, input: RenderInput, mode: BlockOutputMode) -> BlockOutput {
        catch_engine_panic(
            std::panic::AssertUnwindSafe(|| self.derive_parsed_blocks(&self.parse(input), mode)),
            |_| {
                failure(
                    mode,
                    BlockOutputStatus::Failed,
                    vec![panic_error().diagnostic()],
                    self.limits.max_output_bytes,
                )
            },
        )
    }

    /// Reuse retained effective settings. Render hooks receive an isolated copy;
    /// complete-document finalization hooks are deliberately excluded.
    pub fn derive_parsed_blocks(
        &self,
        parsed: &ParsedDocument,
        mode: BlockOutputMode,
    ) -> BlockOutput {
        let limit = parsed.limits.max_output_bytes;
        catch_engine_panic(
            std::panic::AssertUnwindSafe(|| {
                let status = match parsed.parse_status {
                    ParseStatus::Incomplete => BlockOutputStatus::Incomplete,
                    ParseStatus::Failed => BlockOutputStatus::Failed,
                    ParseStatus::Complete => BlockOutputStatus::Complete,
                };
                if status != BlockOutputStatus::Complete {
                    return failure(
                        mode,
                        status,
                        bounded_diagnostics(&parsed.state.diagnostics, limit),
                        limit,
                    );
                }
                let result =
                    validate::document(&parsed.document, parsed.limits).and_then(|()| match mode {
                        BlockOutputMode::Structured => structured(parsed),
                        BlockOutputMode::Rendered => rendered::derive(self, parsed),
                    });
                match result {
                    Ok(output) => finish(output, limit),
                    Err(error) => failure(
                        mode,
                        BlockOutputStatus::Failed,
                        error_diagnostics(parsed, &error, limit),
                        limit,
                    ),
                }
            }),
            |_| {
                failure(
                    mode,
                    BlockOutputStatus::Failed,
                    error_diagnostics(parsed, &panic_error(), limit),
                    limit,
                )
            },
        )
    }
}

fn panic_error() -> DerivationError {
    DerivationError {
        code: "blocks.panic",
        message: "Block derivation caught an engine or extension panic",
    }
}

fn structured(parsed: &ParsedDocument) -> Result<BlockOutput, DerivationError> {
    let mut b = Budget::new(parsed.limits.max_output_bytes);
    let blocks =
        crate::blocks::structured::blocks(&parsed.document.blocks, &parsed.context.source, &mut b)?;
    let references = parsed
        .document
        .link_references
        .iter()
        .map(|(label, r)| {
            b.reserve(8)?;
            Ok((
                b.copy(label)?,
                StructuredReference {
                    destination: b.copy(&r.destination)?,
                    title: b.optional(r.title.as_deref())?,
                },
            ))
        })
        .collect::<Result<_, DerivationError>>()?;
    let footnotes = parsed
        .document
        .footnotes
        .iter()
        .map(|f| {
            b.reserve(24)?;
            Ok(StructuredFootnote {
                label: b.copy(&f.label)?,
                rich_text: crate::blocks::inline::inlines(&f.children, &mut b)?,
            })
        })
        .collect::<Result<_, DerivationError>>()?;
    Ok(BlockOutput::Structured(StructuredBlockOutput {
        schema_version: 1,
        mode: BlockOutputMode::Structured,
        status: BlockOutputStatus::Complete,
        blocks,
        references,
        footnotes,
        diagnostics: tree_diagnostics(
            &parsed.document.blocks,
            &parsed.state.diagnostics,
            parsed.limits.max_output_bytes,
        ),
    }))
}

fn finish(output: BlockOutput, limit: usize) -> BlockOutput {
    let (mode, diagnostics) = match &output {
        BlockOutput::Structured(o) => (BlockOutputMode::Structured, &o.diagnostics),
        BlockOutput::Rendered(o) => (BlockOutputMode::Rendered, &o.diagnostics),
    };
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return failure(mode, BlockOutputStatus::Failed, diagnostics.clone(), limit);
    }
    if !fits(&output, limit) {
        return failure(
            mode,
            BlockOutputStatus::Failed,
            vec![DerivationError::limit().diagnostic()],
            limit,
        );
    }
    output
}

fn bounded_diagnostics(diagnostics: &[RenderDiagnostic], limit: usize) -> Vec<RenderDiagnostic> {
    if fits(&diagnostics, limit) {
        diagnostics.to_vec()
    } else {
        vec![DerivationError::limit().diagnostic()]
    }
}

fn error_diagnostics(
    parsed: &ParsedDocument,
    error: &DerivationError,
    limit: usize,
) -> Vec<RenderDiagnostic> {
    if error.code == "blocks.output_limit" {
        return vec![error.diagnostic()];
    }
    let mut diagnostics =
        tree_diagnostics(&parsed.document.blocks, &parsed.state.diagnostics, limit);
    diagnostics.push(error.diagnostic());
    bounded_diagnostics(&diagnostics, limit)
}

fn failure(
    mode: BlockOutputMode,
    status: BlockOutputStatus,
    diagnostics: Vec<RenderDiagnostic>,
    limit: usize,
) -> BlockOutput {
    fn empty(
        mode: BlockOutputMode,
        status: BlockOutputStatus,
        diagnostics: Vec<RenderDiagnostic>,
    ) -> BlockOutput {
        match mode {
            BlockOutputMode::Structured => BlockOutput::Structured(StructuredBlockOutput {
                schema_version: 1,
                mode,
                status,
                blocks: Vec::new(),
                references: BTreeMap::new(),
                footnotes: Vec::new(),
                diagnostics,
            }),
            BlockOutputMode::Rendered => BlockOutput::Rendered(RenderedBlockOutput {
                schema_version: 1,
                mode,
                status,
                blocks: Vec::new(),
                assets: Vec::new(),
                diagnostics,
            }),
        }
    }
    let output = empty(mode, status, diagnostics);
    if fits(&output, limit) {
        output
    } else {
        empty(
            mode,
            BlockOutputStatus::Failed,
            vec![DerivationError::limit().diagnostic()],
        )
    }
}

fn tree_diagnostics(
    blocks: &[BlockNode],
    state: &[RenderDiagnostic],
    limit: usize,
) -> Vec<RenderDiagnostic> {
    let mut budget = Budget::new(limit);
    let mut result = Vec::new();
    let mut append = |diagnostics: &[RenderDiagnostic]| -> Result<(), DerivationError> {
        budget.reserve(crate::blocks::budget::size(
            &diagnostics,
            budget.remaining(),
        )?)?;
        result.extend_from_slice(diagnostics);
        Ok(())
    };
    if append(state).is_err() {
        return vec![DerivationError::limit().diagnostic()];
    }
    let mut pending: Vec<_> = blocks.iter().collect();
    while let Some(block) = pending.pop() {
        if !block.diagnostics.is_empty() && append(&block.diagnostics).is_err() {
            return vec![DerivationError::limit().diagnostic()];
        }
        pending.extend(block.children.iter());
    }
    result
}
