use oriha_stats_viewer::{
    load_report, parse_from_bound, parse_last_window, resolve_last_window, CountEntry, GroupBy,
    ReportOptions,
};
use std::fs;
use std::process::Command;

const RAW_ARCHIVE: &[u8] = include_bytes!("fixtures/raw.tar.7z");

#[test]
fn production_raw_archive_loads_and_displays_expected_report() {
    let temp = tempfile::tempdir().expect("tempdir");
    let archive_path = temp.path().join("raw.tar.7z");
    fs::write(&archive_path, RAW_ARCHIVE).expect("write archive fixture");
    extract_archive(&archive_path, temp.path());

    let report = load_report(ReportOptions {
        inputs: vec![temp.path().to_path_buf()],
        include_open: true,
        group_by: GroupBy::Day,
        ..ReportOptions::default()
    })
    .expect("load production raw archive");

    assert_eq!(report.source_file_count, 83);
    assert_eq!(report.skipped_invalid_line_count, 0);
    assert_eq!(report.summary.renders, 621);
    assert_eq!(report.summary.total_blocks, 55_974);
    assert_eq!(report.summary.issue_count, 5);
    assert_eq!(report.summary.locked_feature_count, 1_456);
    assert_eq!(report.summary.unsupported_count, 0);
    assert_eq!(
        report
            .summary
            .first_received_at
            .map(|value| value.to_rfc3339()),
        Some("2026-05-18T09:52:46.581101041+00:00".to_string())
    );
    assert_eq!(
        report
            .summary
            .last_received_at
            .map(|value| value.to_rfc3339()),
        Some("2026-06-15T09:57:32.274503604+00:00".to_string())
    );
    assert_eq!(report.groups.len(), 19);
    assert_eq!(
        report.dimensions.platforms,
        vec![
            CountEntry {
                kind: "iOS".to_string(),
                count: 441,
            },
            CountEntry {
                kind: "macOS".to_string(),
                count: 119,
            },
            CountEntry {
                kind: "Android".to_string(),
                count: 60,
            },
            CountEntry {
                kind: "Web".to_string(),
                count: 1,
            },
        ]
    );
    assert_eq!(
        report.summary.buckets.issues_by_kind,
        vec![
            CountEntry {
                kind: "plantumlRenderFailed".to_string(),
                count: 4,
            },
            CountEntry {
                kind: "mermaidRenderFailed".to_string(),
                count: 1,
            },
        ]
    );

    let output = Command::new(env!("CARGO_BIN_EXE_oriha-stats-viewer"))
        .arg(temp.path())
        .arg("--include-open")
        .arg("--group-by")
        .arg("day")
        .arg("--top")
        .arg("5")
        .output()
        .expect("run oriha-stats-viewer");
    assert!(
        output.status.success(),
        "viewer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    assert!(stdout.contains("Sources: 83 files"));
    assert!(stdout.contains("renders"));
    assert!(stdout.contains("621"));
    assert!(stdout.contains("total blocks"));
    assert!(stdout.contains("55974"));
    assert!(stdout.contains("By day"));
    assert!(stdout.contains("2026-06-15"));
    assert!(stdout.contains("Issues by kind"));
    assert!(stdout.contains("plantumlRenderFailed"));
}

#[test]
fn production_raw_archive_can_be_filtered_by_relative_window() {
    let temp = tempfile::tempdir().expect("tempdir");
    let archive_path = temp.path().join("raw.tar.7z");
    fs::write(&archive_path, RAW_ARCHIVE).expect("write archive fixture");
    extract_archive(&archive_path, temp.path());

    let now = parse_from_bound("2026-06-15T10:00:00Z").expect("now");
    let (from, to) =
        resolve_last_window(parse_last_window("7d").expect("window"), now).expect("resolve");
    let report = load_report(ReportOptions {
        inputs: vec![temp.path().to_path_buf()],
        include_open: true,
        from: Some(from),
        to: Some(to),
        group_by: GroupBy::Day,
        ..ReportOptions::default()
    })
    .expect("load production raw archive");

    assert_eq!(report.source_file_count, 83);
    assert_eq!(report.summary.renders, 104);
    assert_eq!(report.summary.total_blocks, 12_546);
    assert_eq!(report.summary.issue_count, 5);
    assert_eq!(report.summary.locked_feature_count, 131);
    assert_eq!(report.groups.len(), 7);
    assert_eq!(report.groups[0].key, "2026-06-09");
    assert_eq!(
        report.groups.last().map(|row| row.key.as_str()),
        Some("2026-06-15")
    );
}

fn extract_archive(archive_path: &std::path::Path, output_dir: &std::path::Path) {
    let status = Command::new("tar")
        .arg("-xf")
        .arg(archive_path)
        .arg("-C")
        .arg(output_dir)
        .status()
        .expect("tar command must be available to extract raw.tar.7z fixture");
    assert!(status.success(), "tar failed with status {status}");
}
