use super::*;
use crate::blocks::DocumentNodeKind as N;
use std::collections::BTreeSet;

mod blocks;
mod inlines;
mod preparation;

type Result<T> = std::result::Result<T, DerivationError>;

pub(super) fn derive(parsed: &ParsedDocument, options: StructuredOptions) -> Result<BlockOutput> {
    let mut pool = Pool::new(parsed, options)?;
    let mut reference_ids = Vec::new();
    for (label, reference) in &parsed.document.link_references {
        let id = pool.id()?;
        let content = N::ReferenceDefinition {
            label: pool.b.copy(label)?,
            destination: pool.b.copy(&reference.destination)?,
            normalized_destination: pool.normalized(&reference.destination)?,
            title: pool.b.optional(reference.title.as_deref())?,
        };
        pool.push(id.clone(), content)?;
        reference_ids.push(id);
    }
    // Reserve definition identities before projecting semantic references. Last
    // duplicate wins resolution, while every retained definition keeps ownership.
    let mut footnote_ids = Vec::new();
    for definition in &parsed.document.footnotes {
        let id = pool.id()?;
        pool.definitions
            .insert(definition.label.clone(), id.clone());
        footnote_ids.push(id);
    }
    let root_ids = pool.blocks(&parsed.document.blocks, &parsed.context.source)?;
    if !pool.note_numbers.is_empty() {
        pool.anchors.title();
    }
    let visible = crate::footnotes::visible_indices(&parsed.document);
    let mut order = visible.clone();
    let visible: BTreeSet<_> = visible.into_iter().collect();
    order.extend((0..parsed.document.footnotes.len()).filter(|index| !visible.contains(index)));
    for index in order {
        let definition = &parsed.document.footnotes[index];
        let id = &footnote_ids[index];
        pool.rendering_inlines = visible.contains(&index);
        let child_ids = pool.inlines(&definition.children)?;
        let anchor = if pool.definitions.get(&definition.label) == Some(id) {
            pool.anchors.definition(&definition.label)
        } else {
            pool.anchors.retained_definition(&definition.label)
        };
        let content = N::FootnoteDefinition {
            visible: visible.contains(&index),
            label: pool.b.copy(&definition.label)?,
            child_ids,
            anchor,
            ordinal: None,
            backlink_ids: Vec::new(),
        };
        pool.push(id.clone(), content)?;
    }
    for node in &mut pool.nodes {
        if let N::FootnoteDefinition {
            label,
            ordinal,
            backlink_ids,
            ..
        } = &mut node.content
        {
            if pool.definitions.get(label) == Some(&node.id) {
                *ordinal = pool.note_numbers.get(label).copied();
                *backlink_ids = pool.backlinks.remove(label).unwrap_or_default();
            }
        }
    }
    pool.diagnostics.extend(tree_diagnostics(
        &parsed.document.blocks,
        &parsed.state.diagnostics,
        parsed.limits.max_output_bytes,
    ));
    let output = StructuredBlockOutput {
        schema_version: 2,
        mode: BlockOutputMode::Structured,
        status: BlockOutputStatus::Complete,
        root_ids,
        nodes: pool.nodes,
        reference_ids,
        footnote_ids,
        fonts: pool.fonts,
        diagnostics: pool.diagnostics,
    };
    output
        .validate(
            parsed.limits.max_output_bytes / 48 + 1,
            512,
            parsed.limits.max_output_bytes,
        )
        .map_err(|error| match error {
            DocumentValidationError::Budget => DerivationError::limit(),
            _ => DerivationError {
                code: "blocks.invalid_graph",
                message: "Structured graph has invalid ownership or targets",
            },
        })?;
    Ok(BlockOutput::Structured(output))
}

struct Pool<'a> {
    parsed: &'a ParsedDocument,
    options: StructuredOptions,
    b: Budget,
    used: BTreeSet<String>,
    next: usize,
    nodes: Vec<DocumentNode>,
    definitions: BTreeMap<String, String>,
    headings: BTreeMap<String, usize>,
    anchors: FootnoteIds,
    note_numbers: BTreeMap<String, u32>,
    rendering_inlines: bool,
    backlinks: BTreeMap<String, Vec<String>>,
    fonts: Vec<DocumentFont>,
    diagnostics: Vec<RenderDiagnostic>,
}

impl<'a> Pool<'a> {
    fn new(parsed: &'a ParsedDocument, options: StructuredOptions) -> Result<Self> {
        let mut b = Budget::new(parsed.limits.max_output_bytes);
        let mut used = BTreeSet::new();
        let mut pending: Vec<_> = parsed.document.blocks.iter().collect();
        while let Some(block) = pending.pop() {
            used.insert(b.copy(&block.id)?);
            pending.extend(&block.children);
        }
        Ok(Self {
            parsed,
            options,
            b,
            used,
            next: 0,
            nodes: Vec::new(),
            definitions: BTreeMap::new(),
            headings: BTreeMap::new(),
            anchors: parsed.state.footnote_ids.clone(),
            note_numbers: BTreeMap::new(),
            rendering_inlines: true,
            backlinks: BTreeMap::new(),
            fonts: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    fn id(&mut self) -> Result<String> {
        loop {
            let id = format!("oriha-node-{}", self.next);
            self.next += 1;
            self.b.reserve(id.len())?;
            if self.used.insert(id.clone()) {
                return Ok(id);
            }
        }
    }

    fn push(&mut self, id: String, content: N) -> Result<()> {
        self.b.reserve(48)?;
        self.nodes.push(DocumentNode { id, content });
        Ok(())
    }

    fn normalized(&mut self, destination: &str) -> Result<String> {
        let value = normalize_link_destination(destination);
        self.b.reserve(value.len())?;
        Ok(value)
    }
}
