use super::*;

#[test]
fn render_stats_json_contract_is_versioned_and_camel_case() {
    let mut stats = RenderStats {
        total_blocks: 7,
        ..RenderStats::default()
    };
    stats.record_block_kind(RenderBlockKind::Heading);
    stats.record_heading_level(2);
    stats.record_code_block_language("ts");
    stats.record_callout_kind("WARNING");
    stats.record_locked_feature(LockedFeatureKind::Math);
    stats.record_unsupported(UnsupportedKind::PlantumlDiagram);
    stats.record_issue(RenderIssueKind::UnknownError);
    stats.finalize();

    let value = serde_json::to_value(&stats).expect("stats should serialize");

    assert_eq!(
        value,
        serde_json::json!({
            "schemaVersion": 1,
            "totalBlocks": 7,
            "blocksByKind": [{ "kind": "heading", "count": 1 }],
            "headingsByLevel": [{ "kind": "h2", "count": 1 }],
            "codeBlocksByLanguageKind": [{ "kind": "typescript", "count": 1 }],
            "calloutsByKind": [{ "kind": "warning", "count": 1 }],
            "lockedFeaturesByKind": [{ "kind": "math", "count": 1 }],
            "unsupportedByKind": [{ "kind": "plantumlDiagram", "count": 1 }],
            "issuesByKind": [{ "kind": "unknownError", "count": 1 }]
        })
    );

    let decoded: RenderStats =
        serde_json::from_value(value).expect("stats JSON should deserialize");
    assert_eq!(decoded.schema_version, 1);
    assert_eq!(
        kind_count(&decoded.code_blocks_by_language_kind, "typescript"),
        1
    );
}

#[test]
fn render_stats_kind_catalog_exposes_the_stats_allowlists() {
    let catalog = render_stats_kind_catalog();

    assert_eq!(catalog.block_kinds, BLOCK_KIND_ORDER);
    assert_eq!(catalog.heading_levels, HEADING_LEVEL_ORDER);
    assert_eq!(catalog.code_language_kinds, CODE_LANGUAGE_ORDER);
    assert_eq!(catalog.callout_kinds, CALLOUT_KIND_ORDER);
    assert_eq!(catalog.locked_feature_kinds, LOCKED_FEATURE_ORDER);
    assert_eq!(catalog.unsupported_kinds, UNSUPPORTED_KIND_ORDER);
    assert_eq!(catalog.issue_kinds, ISSUE_KIND_ORDER);
}

#[test]
fn render_stats_kind_validator_is_bucket_scoped() {
    assert!(is_valid_render_stats_kind(
        RenderStatsBucket::BlocksByKind,
        "paragraph"
    ));
    assert!(is_valid_render_stats_kind(
        RenderStatsBucket::CodeBlocksByLanguageKind,
        "typescript"
    ));
    assert!(!is_valid_render_stats_kind(
        RenderStatsBucket::CodeBlocksByLanguageKind,
        "customer@example.com"
    ));
    assert!(!is_valid_render_stats_kind(
        RenderStatsBucket::CalloutsByKind,
        "paragraph"
    ));
}

#[test]
fn render_stats_kind_count_sorting_uses_bucket_order() {
    let mut counts = vec![
        KindCount {
            kind: "other".to_string(),
            count: 1,
        },
        KindCount {
            kind: "plain".to_string(),
            count: 1,
        },
        KindCount {
            kind: "typescript".to_string(),
            count: 1,
        },
    ];

    sort_render_stats_kind_counts(RenderStatsBucket::CodeBlocksByLanguageKind, &mut counts);

    assert_eq!(
        counts
            .iter()
            .map(|entry| entry.kind.as_str())
            .collect::<Vec<_>>(),
        vec!["plain", "typescript", "other"]
    );
}

#[test]
fn code_language_kind_allowlist_canonicalizes_aliases() {
    for (input, expected) in [
        ("", "plain"),
        (" text ", "plain"),
        (".js", "javascript"),
        ("mjs", "javascript"),
        ("jsx", "jsx"),
        ("ts", "typescript"),
        ("tsx", "tsx"),
        ("rs", "rust"),
        ("py3", "python"),
        ("htm", "html"),
        ("jsonc", "jsonc"),
        ("bash", "shell"),
        ("terminal", "console"),
        ("yml", "yaml"),
        ("postgres", "postgresql"),
        ("md", "markdown"),
        ("patch", "diff"),
        ("puml", "plantuml"),
        ("mermaid", "mermaid"),
        ("tex", "latex"),
        ("customer@example.com", "other"),
        ("internal-secret-language", "other"),
    ] {
        assert_eq!(canonical_code_language_kind(input), expected, "{input}");
    }
}

#[test]
fn code_language_stats_do_not_emit_unknown_literals() {
    let mut stats = RenderStats::default();
    stats.record_code_block_language("customer@example.com");
    stats.record_code_block_language("rust token-value");
    stats.finalize();

    let stats_json = serde_json::to_string(&stats).expect("stats should serialize");

    assert_eq!(kind_count(&stats.code_blocks_by_language_kind, "other"), 2);
    assert!(!stats_json.contains("customer@example.com"));
    assert!(!stats_json.contains("token-value"));
}

#[test]
fn callout_kind_allowlist_redacts_custom_values() {
    for (input, expected) in [
        ("note", "note"),
        ("TIP", "tip"),
        (" important ", "important"),
        ("warning", "warning"),
        ("caution", "caution"),
        ("customer-escalation", "other"),
        ("john@example.com", "other"),
    ] {
        assert_eq!(canonical_callout_kind(input), expected, "{input}");
    }

    let mut stats = RenderStats::default();
    stats.record_callout_kind("customer-escalation");
    stats.finalize();

    let stats_json = serde_json::to_string(&stats).expect("stats should serialize");

    assert_eq!(kind_count(&stats.callouts_by_kind, "other"), 1);
    assert!(!stats_json.contains("customer-escalation"));
}

#[test]
fn unknown_error_diagnostic_codes_converge_to_constant_kind() {
    assert_eq!(
        RenderIssueKind::from_diagnostic_code("customer.secret_code", Severity::Error),
        Some(RenderIssueKind::UnknownError)
    );
    assert_eq!(
        RenderIssueKind::from_diagnostic_code("customer.secret_code", Severity::Warning),
        None
    );
}

fn kind_count(entries: &[KindCount], kind: &str) -> u32 {
    entries
        .iter()
        .find(|entry| entry.kind == kind)
        .map(|entry| entry.count)
        .unwrap_or(0)
}
