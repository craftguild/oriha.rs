use oriha_stats_viewer::{
    load_report, parse_from_bound, parse_last_window, parse_to_bound, resolve_last_window,
    CountEntry, GroupBy, ReportOptions, ViewerError,
};
use serde_json::json;
use std::fs;

#[test]
fn aggregates_hourly_report() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path().join("raw/dt=2026-06-15/hour=03");
    fs::create_dir_all(&dir).expect("create raw partition");
    fs::write(
        dir.join("part-host-1-000000.jsonl"),
        [
            record_json("2026-06-15T03:04:05Z", "macOS", "free", 3),
            record_json("2026-06-15T03:20:05Z", "iOS", "premium", 5),
        ]
        .join("\n"),
    )
    .expect("write jsonl");

    let report = load_report(ReportOptions {
        inputs: vec![temp.path().to_path_buf()],
        group_by: GroupBy::Hour,
        ..ReportOptions::default()
    })
    .expect("load report");

    assert_eq!(report.source_file_count, 1);
    assert_eq!(report.summary.renders, 2);
    assert_eq!(report.summary.total_blocks, 8);
    assert_eq!(report.groups.len(), 1);
    assert_eq!(report.groups[0].key, "2026-06-15 03:00Z");
    assert_eq!(report.groups[0].issue_count, 2);
    assert_eq!(
        report.summary.buckets.blocks_by_kind,
        vec![
            CountEntry {
                kind: "paragraph".to_string(),
                count: 6,
            },
            CountEntry {
                kind: "document".to_string(),
                count: 2,
            },
        ]
    );
    assert_eq!(
        report.dimensions.subscription_plans,
        vec![
            CountEntry {
                kind: "free".to_string(),
                count: 1,
            },
            CountEntry {
                kind: "premium".to_string(),
                count: 1,
            },
        ]
    );
}

#[test]
fn filters_by_date_bounds() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path().join("raw/dt=2026-06-15/hour=03");
    fs::create_dir_all(&dir).expect("create raw partition");
    fs::write(
        dir.join("part-host-1-000000.jsonl"),
        [
            record_json("2026-06-14T23:59:59Z", "macOS", "free", 3),
            record_json("2026-06-15T00:00:00Z", "macOS", "free", 4),
            record_json("2026-06-16T00:00:00Z", "macOS", "free", 5),
        ]
        .join("\n"),
    )
    .expect("write jsonl");

    let report = load_report(ReportOptions {
        inputs: vec![temp.path().to_path_buf()],
        from: Some(parse_from_bound("2026-06-15").expect("from")),
        to: Some(parse_to_bound("2026-06-15").expect("to")),
        ..ReportOptions::default()
    })
    .expect("load report");

    assert_eq!(report.summary.renders, 1);
    assert_eq!(report.summary.total_blocks, 4);
}

#[test]
fn excludes_open_chunks_unless_requested() {
    let temp = tempfile::tempdir().expect("tempdir");
    let dir = temp.path().join("raw/dt=2026-06-15/hour=03");
    fs::create_dir_all(&dir).expect("create raw partition");
    fs::write(
        dir.join("part-host-1-000000.jsonl.open"),
        record_json("2026-06-15T03:04:05Z", "macOS", "free", 3),
    )
    .expect("write open jsonl");

    let sealed_only = load_report(ReportOptions {
        inputs: vec![temp.path().to_path_buf()],
        ..ReportOptions::default()
    });
    assert!(matches!(sealed_only, Err(ViewerError::NoStatsFiles { .. })));

    let report = load_report(ReportOptions {
        inputs: vec![temp.path().to_path_buf()],
        include_open: true,
        ..ReportOptions::default()
    })
    .expect("load report");
    assert_eq!(report.summary.renders, 1);
}

#[test]
fn resolves_relative_last_windows() {
    let now = parse_from_bound("2026-06-15T10:00:00Z").expect("now");

    let (from, to) =
        resolve_last_window(parse_last_window("24h").expect("window"), now).expect("resolve");
    assert_eq!(
        from.display_with_offset(jiff::tz::Offset::UTC).to_string(),
        "2026-06-14T10:00:00+00:00"
    );
    assert_eq!(
        to.display_with_offset(jiff::tz::Offset::UTC).to_string(),
        "2026-06-15T10:00:00+00:00"
    );

    let (from, to) =
        resolve_last_window(parse_last_window("1w").expect("window"), now).expect("resolve");
    assert_eq!(
        from.display_with_offset(jiff::tz::Offset::UTC).to_string(),
        "2026-06-08T10:00:00+00:00"
    );
    assert_eq!(
        to.display_with_offset(jiff::tz::Offset::UTC).to_string(),
        "2026-06-15T10:00:00+00:00"
    );

    let (from, to) =
        resolve_last_window(parse_last_window("1m").expect("window"), now).expect("resolve");
    assert_eq!(
        from.display_with_offset(jiff::tz::Offset::UTC).to_string(),
        "2026-05-15T10:00:00+00:00"
    );
    assert_eq!(
        to.display_with_offset(jiff::tz::Offset::UTC).to_string(),
        "2026-06-15T10:00:00+00:00"
    );
}

#[test]
fn rejects_invalid_last_window() {
    assert!(parse_last_window("0d").is_err());
    assert!(parse_last_window("7").is_err());
    assert!(parse_last_window("15min").is_err());
}

#[test]
fn relative_months_preserve_time_and_clamp_to_the_last_calendar_day() {
    for (now, expected) in [
        (
            "2024-03-31T12:34:56.123456789Z",
            "2024-02-29T12:34:56.123456789Z",
        ),
        (
            "2026-03-31T12:34:56.123456789Z",
            "2026-02-28T12:34:56.123456789Z",
        ),
    ] {
        let now = parse_from_bound(now).expect("now");
        let (from, to) =
            resolve_last_window(parse_last_window("1m").unwrap(), now).expect("calendar month");
        assert_eq!(from, parse_from_bound(expected).expect("expected date"));
        assert_eq!(to, now);
    }
}

#[test]
fn invalid_dates_and_excessive_windows_are_errors() {
    assert!(parse_from_bound("2026-02-29").is_err());
    assert!(parse_to_bound("2026-02-30").is_err());
    let now = parse_from_bound("2026-06-15T10:00:00Z").expect("now");
    assert!(matches!(
        resolve_last_window(parse_last_window("4294967295h").unwrap(), now),
        Err(ViewerError::LastWindowOverflow { .. })
    ));
}

#[test]
fn offset_timestamps_filter_group_and_serialize_in_utc() {
    let root = tempfile::tempdir().expect("data directory");
    let records = root.path().join("records.jsonl");
    fs::write(
        &records,
        [
            record_json("2026-06-15T00:30:00.123456789+09:00", "macOS", "free", 3),
            record_json("2026-06-15T00:30:00-07:00", "macOS", "free", 5),
        ]
        .join("\n"),
    )
    .expect("records");
    let report = load_report(ReportOptions {
        inputs: vec![records],
        from: Some(parse_from_bound("2026-06-14").unwrap()),
        to: Some(parse_to_bound("2026-06-14").unwrap()),
        group_by: GroupBy::Hour,
        ..ReportOptions::default()
    })
    .expect("report");
    assert_eq!(report.summary.renders, 1);
    assert_eq!(report.summary.total_blocks, 3);
    assert_eq!(report.groups[0].key, "2026-06-14 15:00Z");
    let json = serde_json::to_value(report).expect("serialized report");
    assert_eq!(
        json["summary"]["firstReceivedAt"],
        "2026-06-14T15:30:00.123456789Z"
    );
}

fn record_json(
    received_at: &str,
    platform: &str,
    subscription_plan: &str,
    total_blocks: u32,
) -> String {
    json!({
        "receivedAt": received_at,
        "storageId": "2b7e8b3d-8026-4e8a-bef1-a07989692baf",
        "digest": "digest",
        "payload": {
            "event": "markdown_render_complete",
            "schemaVersion": 1,
            "platform": platform,
            "appVersion": "1.2.3",
            "buildVersion": "123",
            "locale": "ja",
            "subscriptionPlan": subscription_plan,
            "stats": {
                "schemaVersion": 1,
                "totalBlocks": total_blocks,
                "blocksByKind": [
                    { "kind": "document", "count": 1 },
                    { "kind": "paragraph", "count": total_blocks - 1 }
                ],
                "headingsByLevel": [],
                "codeBlocksByLanguageKind": [],
                "calloutsByKind": [],
                "lockedFeaturesByKind": [],
                "unsupportedByKind": [],
                "issuesByKind": [
                    { "kind": "unknownError", "count": 1 }
                ]
            }
        }
    })
    .to_string()
}
