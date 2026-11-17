use oriha_core::ir::*;
use oriha_core::render::{RenderContext, RenderState};
use oriha_core::source::SourceSpan;
use oriha_core::*;
use std::collections::BTreeSet;

fn derive(source: &str) -> StructuredBlockOutput {
    let BlockOutput::Structured(output) = derive_blocks(
        RenderInput {
            markdown: source.into(),
            ..Default::default()
        },
        BlockOutputMode::Structured,
    ) else {
        unreachable!()
    };
    assert_eq!(output.status, BlockOutputStatus::Complete);
    output
}
fn validate(document: &StructuredBlockOutput) -> Result<(), DocumentValidationError> {
    document.validate(10_000, 512, 2_000_000)
}

#[test]
fn malformed_ownership_missing_targets_versions_and_budgets_fail_explicitly() {
    use DocumentValidationError as E;
    let original = derive("**text**");
    validate(&original).unwrap();
    let mut value = original.clone();
    value.schema_version = 1;
    assert_eq!(validate(&value), Err(E::Version));
    let mut value = original.clone();
    value.nodes.push(value.nodes[0].clone());
    assert_eq!(validate(&value), Err(E::Identity));
    let mut value = original.clone();
    value.nodes[0].id.clear();
    assert_eq!(validate(&value), Err(E::Identity));
    let mut value = original.clone();
    value.root_ids.push(value.root_ids[0].clone());
    assert_eq!(validate(&value), Err(E::Ownership));
    let mut value = original.clone();
    value.root_ids[0] = "missing".into();
    assert_eq!(validate(&value), Err(E::Target));
    let mut value = original.clone();
    value.root_ids[0] = value
        .nodes
        .iter()
        .find(|n| matches!(n.content, DocumentNodeKind::Text { .. }))
        .unwrap()
        .id
        .clone();
    assert_eq!(validate(&value), Err(E::Target));
    let mut value = original.clone();
    value.nodes.push(DocumentNode {
        id: "orphan".into(),
        content: DocumentNodeKind::Text {
            text: "orphan".into(),
        },
    });
    assert_eq!(validate(&value), Err(E::Ownership));
    let mut value = original.clone();
    let root = value.root_ids[0].clone();
    value
        .nodes
        .iter_mut()
        .find(|n| n.id == root)
        .unwrap()
        .content = DocumentNodeKind::Document {
        child_ids: vec![root.clone()],
    };
    assert_eq!(validate(&value), Err(E::Ownership));
    assert_eq!(original.validate(0, 512, 2_000_000), Err(E::Budget));
    assert_eq!(original.validate(10_000, 0, 2_000_000), Err(E::Budget));
    assert_eq!(original.validate(10_000, 512, 100), Err(E::Budget));
}

struct Deep;
impl RenderHook for Deep {
    fn after_document_parse(
        &self,
        document: &mut Document,
        _: &mut RenderState,
        _: &mut RenderContext,
    ) {
        let mut text = InlineNode::from(InlineKind::Text("bottom".into()));
        for _ in 0..80 {
            text = InlineKind::Strong {
                children: vec![text],
            }
            .into();
        }
        let span = SourceSpan::new(0, 0);
        let mut block = BlockNode::new(
            "oriha-node-0",
            BlockKind::Paragraph(ParagraphNode {
                children: vec![text],
            }),
            span,
        );
        for index in 1..80 {
            let mut parent =
                BlockNode::new(format!("oriha-node-{index}"), BlockKind::Document, span);
            parent.children.push(block);
            block = parent;
        }
        document.blocks = vec![block];
    }
}

#[test]
fn deeply_nested_block_and_inline_graph_has_collision_free_ids_and_fixed_wire_depth() {
    let engine = OrihaEngine::default().add_hook(Deep);
    let BlockOutput::Structured(value) = engine.derive_blocks(
        RenderInput {
            markdown: "input".into(),
            ..Default::default()
        },
        BlockOutputMode::Structured,
    ) else {
        unreachable!()
    };
    assert_eq!(value.status, BlockOutputStatus::Complete);
    validate(&value).unwrap();
    assert_eq!(value.nodes.len(), 161);
    assert_eq!(
        value
            .nodes
            .iter()
            .map(|node| &node.id)
            .collect::<BTreeSet<_>>()
            .len(),
        value.nodes.len()
    );
    let serialized = serde_json::to_string(&value).unwrap();
    assert!(!serialized.contains("\"children\""));
    let roundtrip: StructuredBlockOutput = serde_json::from_str(&serialized).unwrap();
    validate(&roundtrip).unwrap();
}

#[test]
fn definition_links_do_not_own_or_recursively_expand_definition_content() {
    let document = derive("a[^note] again[^note]\n\n[^note]: self[^note]");
    validate(&document).unwrap();
    assert_eq!(document.footnote_ids.len(), 1);
    let references: Vec<_> = document
        .nodes
        .iter()
        .filter_map(|node| match &node.content {
            DocumentNodeKind::FootnoteReference { definition_id, .. } => Some(definition_id),
            _ => None,
        })
        .collect();
    assert_eq!(references.len(), 3);
    assert!(references
        .iter()
        .all(|id| id.as_ref() == Some(&document.footnote_ids[0])));
    let mut broken = document.clone();
    for node in &mut broken.nodes {
        if let DocumentNodeKind::FootnoteReference { definition_id, .. } = &mut node.content {
            *definition_id = Some(document.root_ids[0].clone());
            break;
        }
    }
    assert_eq!(validate(&broken), Err(DocumentValidationError::Target));
}

#[test]
fn producer_heading_and_footnote_anchors_remain_collision_free() {
    let document =
        derive("a[^a b] [^a-b] [^a b]\n\n# fn a-b\n\n# fnref a-b\n\n[^a b]: space\n[^a-b]: dash");
    validate(&document).unwrap();
    let mut anchors = BTreeSet::new();
    let mut destinations = Vec::new();
    for node in &document.nodes {
        match &node.content {
            DocumentNodeKind::Heading { anchor, .. }
            | DocumentNodeKind::FootnoteDefinition { anchor, .. } => {
                assert!(anchors.insert(anchor.clone()), "{anchor}");
            }
            DocumentNodeKind::FootnoteReference {
                anchor: Some(anchor),
                definition_anchor: Some(destination),
                ..
            } => {
                assert!(anchors.insert(anchor.clone()), "{anchor}");
                destinations.push(destination);
            }
            _ => {}
        }
    }
    assert!(destinations.iter().all(|id| anchors.contains(id.as_str())));
}

#[test]
fn hidden_alt_unused_and_nested_only_notes_preserve_existing_visibility() {
    let document = derive("main[^outer] ![alt[^image]](url)\n\n[^unused]: unused[^outer]\n[^image]: image\n[^inner]: nested\n[^outer]: outer[^inner]\n");
    validate(&document).unwrap();
    let visible: Vec<_> = document
        .nodes
        .iter()
        .filter_map(|node| match &node.content {
            DocumentNodeKind::FootnoteDefinition {
                label,
                visible: true,
                ordinal,
                ..
            } => Some((label.as_str(), *ordinal)),
            _ => None,
        })
        .collect();
    assert_eq!(visible, vec![("outer", Some(1))]);
    for node in &document.nodes {
        if let DocumentNodeKind::FootnoteReference {
            label,
            anchor,
            ordinal,
            ..
        } = &node.content
        {
            match label.as_str() {
                "image" => {
                    assert!(anchor.is_none());
                    assert!(ordinal.is_none());
                }
                "inner" => {
                    assert!(anchor.is_some());
                    assert_eq!(*ordinal, Some(2));
                }
                _ => {}
            }
        }
    }
}
