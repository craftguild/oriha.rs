use jiff::{tz::TimeZone, Span, Timestamp};
use oriha_core::RenderStats;
use oriha_stats_ingest::writer::{
    cleanup_old_raw_partitions, StoredUsageAnalyticsRecord, WriterError,
};
use oriha_stats_schema::ValidatedUsageAnalyticsPayload;

#[test]
fn stored_timestamps_keep_utc_rfc3339_and_nanoseconds() {
    let mut record = StoredUsageAnalyticsRecord::new(
        "digest".to_owned(),
        ValidatedUsageAnalyticsPayload {
            event: "markdown_render_complete".to_owned(),
            schema_version: 1,
            platform: "macOS".to_owned(),
            app_version: "1.0".to_owned(),
            build_version: "1".to_owned(),
            locale: "en".to_owned(),
            subscription_plan: "free".to_owned(),
            event_id: None,
            stats: RenderStats::default(),
        },
    );
    record.received_at = "2026-06-15T00:30:00.123456789+09:00"
        .parse()
        .expect("timestamp");
    let json = serde_json::to_value(record).expect("serialize record");
    assert_eq!(json["receivedAt"], "2026-06-14T15:30:00.123456789Z");
}

#[tokio::test]
async fn invalid_retention_preserves_existing_data() {
    let root = tempfile::tempdir().expect("data directory");
    let partition = root.path().join("raw/dt=2000-01-01");
    std::fs::create_dir_all(&partition).expect("partition");
    let record = partition.join("part.jsonl");
    std::fs::write(&record, b"preserve\n").expect("record");
    assert!(matches!(
        cleanup_old_raw_partitions(root.path(), u64::MAX).await,
        Err(WriterError::InvalidRetention(u64::MAX))
    ));
    assert_eq!(
        std::fs::read(record).expect("preserved record"),
        b"preserve\n"
    );
}

#[tokio::test]
async fn retention_removes_only_expired_utc_date_partitions() {
    let root = tempfile::tempdir().expect("data directory");
    let today = Timestamp::now().to_zoned(TimeZone::UTC).date();
    let old = today.checked_sub(Span::new().days(100)).expect("old date");
    let future = today
        .checked_add(Span::new().days(100))
        .expect("future date");
    let old = root.path().join(format!("raw/dt={old}"));
    let future = root.path().join(format!("raw/dt={future}"));
    let malformed = root.path().join("raw/dt=invalid");
    let unrelated = root.path().join("raw/unrelated");
    for path in [&old, &future, &malformed, &unrelated] {
        std::fs::create_dir_all(path).expect("partition");
        std::fs::write(path.join("record.jsonl"), b"preserve\n").expect("record");
    }
    cleanup_old_raw_partitions(root.path(), 30)
        .await
        .expect("retention cleanup");
    assert!(!old.exists());
    for path in [future, malformed, unrelated] {
        assert_eq!(
            std::fs::read(path.join("record.jsonl")).expect("preserved record"),
            b"preserve\n"
        );
    }
}
