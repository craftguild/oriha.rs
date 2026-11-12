use oriha_stats_schema::{
    validate_json_bytes, validate_payload, UsageAnalyticsPayload, ValidatedUsageAnalyticsPayload,
    ValidationError,
};
use serde_json::{json, Value};

fn valid_sample() -> Value {
    json!({
        "event": "markdown_render_complete",
        "schemaVersion": 1,
        "platform": "macOS",
        "appVersion": "1.2.3",
        "buildVersion": "123",
        "locale": "en-US",
        "subscriptionPlan": "premium",
        "stats": {
            "schemaVersion": 1,
            "totalBlocks": 4,
            "blocksByKind": [
                { "kind": "document", "count": 1 },
                { "kind": "paragraph", "count": 2 },
                { "kind": "heading", "count": 1 }
            ],
            "headingsByLevel": [
                { "kind": "h2", "count": 1 }
            ],
            "codeBlocksByLanguageKind": [
                { "kind": "rust", "count": 1 }
            ],
            "calloutsByKind": [
                { "kind": "note", "count": 1 }
            ],
            "lockedFeaturesByKind": [
                { "kind": "math", "count": 1 }
            ],
            "unsupportedByKind": [
                { "kind": "plantumlDiagram", "count": 1 }
            ],
            "issuesByKind": [
                { "kind": "mathRenderFailed", "count": 1 }
            ]
        }
    })
}

fn parse(value: Value) -> UsageAnalyticsPayload {
    serde_json::from_value(value).expect("payload should deserialize")
}

#[test]
fn accepts_valid_macos_sample() {
    let validated = validate_payload(parse(valid_sample())).expect("sample should validate");

    assert_eq!(validated.event, "markdown_render_complete");
    assert_eq!(validated.platform, "macOS");
    assert_eq!(validated.event_id, None);
    assert_eq!(validated.stats.total_blocks, 4);
    assert_eq!(validated.stats.blocks_by_kind[0].kind, "document");
}

#[test]
fn accepts_optional_event_id_for_retry_dedupe() {
    let mut sample = valid_sample();
    sample["eventId"] = json!("550e8400-e29b-41d4-a716-446655440000");

    let validated = validate_payload(parse(sample)).expect("sample should validate");

    assert_eq!(
        validated.event_id.map(|id| id.to_string()).as_deref(),
        Some("550e8400-e29b-41d4-a716-446655440000")
    );
}

#[test]
fn rejects_non_uuid_v4_event_id() {
    let mut sample = valid_sample();
    sample["eventId"] = json!("device-or-session-id");

    let err = validate_payload(parse(sample)).unwrap_err();

    assert!(err.to_string().contains("UUID v4"));

    let mut sample = valid_sample();
    sample["eventId"] = json!("550e8400-e29b-11d4-a716-446655440000");

    let err = validate_payload(parse(sample)).unwrap_err();

    assert!(err.to_string().contains("UUID v4"));

    let mut sample = valid_sample();
    sample["eventId"] = json!("550e8400-e29b-41d4-0716-446655440000");

    let err = validate_payload(parse(sample)).unwrap_err();

    assert!(err.to_string().contains("UUID v4"));
}

#[test]
fn rejects_unknown_fields() {
    let mut sample = valid_sample();
    sample["deviceId"] = json!("sensitive");

    let err = serde_json::from_value::<UsageAnalyticsPayload>(sample).unwrap_err();
    assert!(err.to_string().contains("unknown field"));
}

#[test]
fn rejects_unknown_kind_using_core_catalog() {
    let mut sample = valid_sample();
    sample["stats"]["blocksByKind"] = json!([{ "kind": "madeUpBlock", "count": 1 }]);

    let err = validate_payload(parse(sample)).unwrap_err();
    assert_eq!(
        err,
        ValidationError::UnknownKind {
            field: "stats.blocksByKind",
            kind: "madeUpBlock".to_string(),
        }
    );
}

#[test]
fn merges_duplicate_kinds_and_sorts_canonically() {
    let mut sample = valid_sample();
    sample["stats"]["totalBlocks"] = json!(6);
    sample["stats"]["blocksByKind"] = json!([
        { "kind": "paragraph", "count": 2 },
        { "kind": "document", "count": 1 },
        { "kind": "paragraph", "count": 3 }
    ]);

    let validated = validate_payload(parse(sample)).expect("payload should validate");
    assert_eq!(validated.stats.blocks_by_kind.len(), 2);
    assert_eq!(validated.stats.blocks_by_kind[0].kind, "document");
    assert_eq!(validated.stats.blocks_by_kind[0].count, 1);
    assert_eq!(validated.stats.blocks_by_kind[1].kind, "paragraph");
    assert_eq!(validated.stats.blocks_by_kind[1].count, 5);
}

#[test]
fn rejects_zero_count() {
    let mut sample = valid_sample();
    sample["stats"]["headingsByLevel"] = json!([{ "kind": "h1", "count": 0 }]);

    let err = validate_payload(parse(sample)).unwrap_err();
    assert_eq!(
        err,
        ValidationError::ZeroCount {
            field: "stats.headingsByLevel",
            kind: "h1".to_string(),
        }
    );
}

#[test]
fn rejects_total_blocks_mismatch() {
    let mut sample = valid_sample();
    sample["stats"]["totalBlocks"] = json!(3);

    let err = validate_payload(parse(sample)).unwrap_err();

    assert_eq!(
        err,
        ValidationError::InvalidBlockTotal {
            total_blocks: 3,
            blocks_by_kind_total: 4,
        }
    );
}

#[test]
fn serialized_validated_output_has_no_sensitive_fields() {
    let validated = validate_payload(parse(valid_sample())).expect("sample should validate");
    let serialized = serde_json::to_value(&validated).expect("validated payload should serialize");

    assert!(serialized.get("deviceId").is_none());
    assert!(serialized.get("userId").is_none());
    assert!(serialized.get("email").is_none());
    assert!(serialized.get("installationId").is_none());
    assert!(serialized.get("eventId").is_none());
    assert_eq!(serialized["event"], "markdown_render_complete");
}

#[test]
fn canonical_json_bytes_are_stable_and_exclude_event_id() {
    let validated = validate_payload(parse(valid_sample())).expect("sample should validate");
    let first = validated.canonical_json_bytes().unwrap();
    let second = validated.hmac_digest_input().unwrap();

    assert_eq!(first, second);
    assert_eq!(
        first,
        serde_json::to_vec(&validated as &ValidatedUsageAnalyticsPayload).unwrap()
    );

    let mut sample_with_event_id = valid_sample();
    sample_with_event_id["eventId"] = json!("550e8400-e29b-41d4-a716-446655440000");
    let with_event_id =
        validate_payload(parse(sample_with_event_id)).expect("sample should validate");

    assert_eq!(first, with_event_id.canonical_json_bytes().unwrap());
    assert!(!String::from_utf8(first).unwrap().contains("550e8400"));
}

#[test]
fn validate_json_bytes_combines_decode_and_validation() {
    let sample = valid_sample();
    let bytes = serde_json::to_vec(&sample).unwrap();

    let validated = validate_json_bytes(&bytes).expect("sample should validate");
    assert_eq!(validated.subscription_plan, "premium");
}
