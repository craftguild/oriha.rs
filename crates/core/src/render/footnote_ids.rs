use super::*;
use std::collections::BTreeSet;

/// One non-SVG document namespace shared by HTML and reference enumeration.
/// Fallbacks contain a separator absent from normal slugs; probing also handles
/// application-authored block IDs that happen to occupy the fallback namespace.
#[derive(Clone, Default)]
pub(super) struct FootnoteIds {
    used: BTreeSet<String>,
    definitions: BTreeMap<String, String>,
    first_references: BTreeMap<String, String>,
    title: Option<String>,
}

impl FootnoteIds {
    pub(super) fn for_document(document: &Document) -> Self {
        let mut ids = Self::default();
        if document.footnotes.is_empty() {
            return ids;
        }
        let mut headings = BTreeMap::new();
        let mut pending: Vec<_> = document.blocks.iter().rev().collect();
        while let Some(block) = pending.pop() {
            if let Some(id) = references::block_anchor(block, &mut headings) {
                ids.used.insert(id);
            }
            pending.extend(block.children.iter().rev());
        }
        ids
    }

    pub(super) fn definition(&mut self, label: &str) -> String {
        if let Some(id) = self.definitions.get(label) {
            return id.clone();
        }
        let base = format!("fn-{}", base_slug(label));
        let id = self.allocate(base.clone(), || format!("{base}--{}", encoded_label(label)));
        self.definitions.insert(label.into(), id.clone());
        id
    }

    pub(super) fn reference(&mut self, label: &str, count: usize) -> (String, String) {
        let definition = self.definition(label);
        let stem = definition.strip_prefix("fn-").expect("definition prefix");
        let preferred = if count == 1 {
            format!("fnref-{stem}")
        } else {
            format!("fnref-{stem}-{count}")
        };
        let reference = self.allocate(preferred, || {
            format!(
                "fnref-{}--{}-{count}",
                base_slug(label),
                encoded_label(label)
            )
        });
        self.first_references
            .entry(label.into())
            .or_insert_with(|| reference.clone());
        (definition, reference)
    }

    pub(super) fn back_reference(&mut self, label: &str) -> String {
        self.first_references
            .get(label)
            .cloned()
            .unwrap_or_else(|| {
                let definition = self.definition(label);
                format!(
                    "fnref-{}",
                    definition.strip_prefix("fn-").expect("definition prefix")
                )
            })
    }

    pub(super) fn title(&mut self) -> String {
        if let Some(id) = &self.title {
            return id.clone();
        }
        let id = self.allocate("oriha-footnotes-label".into(), || {
            "oriha-footnotes-label--footnotes".into()
        });
        self.title = Some(id.clone());
        id
    }

    fn allocate(&mut self, preferred: String, fallback: impl FnOnce() -> String) -> String {
        if self.used.insert(preferred.clone()) {
            return preferred;
        }
        let fallback = fallback();
        let mut id = fallback.clone();
        let mut suffix = 2;
        while !self.used.insert(id.clone()) {
            id = format!("{fallback}-{suffix}");
            suffix += 1;
        }
        id
    }
}

fn encoded_label(label: &str) -> String {
    use std::fmt::Write;
    let mut encoded = String::with_capacity(label.len().saturating_mul(2));
    for byte in label.bytes() {
        write!(&mut encoded, "{byte:02x}").expect("string write");
    }
    encoded
}

fn base_slug(label: &str) -> String {
    let slug = heading_slug(label);
    if slug == "section" {
        "note".into()
    } else {
        slug
    }
}
