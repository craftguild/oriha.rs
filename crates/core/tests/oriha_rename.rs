use oriha_core::{
    prepare_html_export_json, render_markdown_patch, ExportHtmlInput, RenderInput,
    RenderPatchFallbackReason, RenderPatchInput, RenderPatchOperation, RenderPatchStrategy,
};

#[test]
fn pre_oriha_snapshot_is_replaced_without_reusing_legacy_ids() {
    let input = RenderInput {
        markdown: "# Heading\n\nParagraph.\n".into(),
        ..RenderInput::default()
    };
    let first = render_markdown_patch(RenderPatchInput {
        previous_snapshot: None,
        input: input.clone(),
    });
    let mut legacy = first.snapshot;
    legacy.schema_version = 2;
    for block in &mut legacy.blocks {
        block.stable_id = block.stable_id.replace("oriha-", "mde-");
    }

    let output = render_markdown_patch(RenderPatchInput {
        previous_snapshot: Some(legacy),
        input,
    });

    assert_eq!(output.strategy, RenderPatchStrategy::ReplaceAll);
    assert_eq!(
        output.fallback_reason,
        Some(RenderPatchFallbackReason::IncompatibleSnapshot)
    );
    assert_eq!(output.snapshot.schema_version, 3);
    assert!(output
        .snapshot
        .blocks
        .iter()
        .all(|block| block.stable_id.starts_with("oriha-b-")));
    assert!(matches!(
        output.patches.as_slice(),
        [RenderPatchOperation::ReplaceAll { .. }]
    ));
    assert!(output.output.html.contains("data-oriha-stable-id"));
    assert!(!output.output.html.contains("mde-"));
}

#[test]
fn prepared_html_export_advertises_the_renamed_option_contract() {
    let input = ExportHtmlInput::default();
    let prepared: serde_json::Value = serde_json::from_str(&prepare_html_export_json(
        &serde_json::to_string(&input).unwrap(),
    ))
    .unwrap();

    assert_eq!(prepared["version"], 2);
    assert_eq!(
        prepared["input"]["options"]["stripOrihaDataAttributes"],
        true
    );
    assert!(prepared["input"]["options"]
        .get("stripMdeDataAttributes")
        .is_none());
}
