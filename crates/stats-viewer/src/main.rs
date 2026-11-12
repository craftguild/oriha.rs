use chrono::{DateTime, Utc};
use clap::{Parser, ValueEnum};
use oriha_stats_viewer::{
    load_report, parse_from_bound, parse_last_window, parse_to_bound, resolve_last_window,
    BucketTotals, CountEntry, DimensionTotals, GroupBy, ReportOptions, ReportRow, StatsReport,
};
use std::path::PathBuf;
use std::process;

type TimeRange = (Option<DateTime<Utc>>, Option<DateTime<Utc>>);

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let (from, to) = resolve_time_range(&cli, Utc::now())?;

    let report = load_report(ReportOptions {
        inputs: cli.input,
        include_open: cli.include_open,
        skip_invalid: cli.skip_invalid,
        from,
        to,
        group_by: cli.group_by.into(),
    })
    .map_err(|error| error.to_string())?;

    match cli.format {
        OutputFormat::Table => print_table_report(&report, cli.top),
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&report)
                .map_err(|error| format!("failed to serialize stats report: {error}"))?;
            println!("{json}");
        }
    }

    Ok(())
}

fn resolve_time_range(cli: &Cli, now: DateTime<Utc>) -> Result<TimeRange, String> {
    if let Some(last) = cli.last.as_deref() {
        if cli.from.is_some() || cli.to.is_some() {
            return Err("--last cannot be combined with --from or --to".to_string());
        }
        let window = parse_last_window(last).map_err(|error| error.to_string())?;
        let (from, to) = resolve_last_window(window, now).map_err(|error| error.to_string())?;
        return Ok((Some(from), Some(to)));
    }

    let from = cli
        .from
        .as_deref()
        .map(parse_from_bound)
        .transpose()
        .map_err(|error| error.to_string())?;
    let to = cli
        .to
        .as_deref()
        .map(parse_to_bound)
        .transpose()
        .map_err(|error| error.to_string())?;
    Ok((from, to))
}

#[derive(Debug, Parser)]
#[command(
    name = "oriha-stats-viewer",
    about = "Summarize oriha-stats-ingest JSONL files"
)]
struct Cli {
    /// Stats data directory or JSONL file. Defaults to ./data/stats.
    #[arg(value_name = "PATH")]
    input: Vec<PathBuf>,
    /// Include active .jsonl.open chunks.
    #[arg(long)]
    include_open: bool,
    /// Skip invalid JSONL lines instead of failing.
    #[arg(long)]
    skip_invalid: bool,
    /// Inclusive lower time bound. Accepts RFC3339 or YYYY-MM-DD.
    #[arg(long, conflicts_with = "last")]
    from: Option<String>,
    /// Exclusive upper time bound. A YYYY-MM-DD value means the next midnight.
    #[arg(long, conflicts_with = "last")]
    to: Option<String>,
    /// Relative time window ending at now. Examples: 24h, 1d, 7d, 1w, 1m.
    #[arg(long, value_name = "WINDOW", conflicts_with_all = ["from", "to"])]
    last: Option<String>,
    /// Group rows in the main table.
    #[arg(long, value_enum, default_value_t = CliGroupBy::Hour)]
    group_by: CliGroupBy,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,
    /// Maximum rows shown per dimension or stats bucket in table output.
    #[arg(long, default_value_t = 12)]
    top: usize,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliGroupBy {
    All,
    Hour,
    Day,
    Platform,
    AppVersion,
    BuildVersion,
    Locale,
    SubscriptionPlan,
}

impl From<CliGroupBy> for GroupBy {
    fn from(value: CliGroupBy) -> Self {
        match value {
            CliGroupBy::All => Self::All,
            CliGroupBy::Hour => Self::Hour,
            CliGroupBy::Day => Self::Day,
            CliGroupBy::Platform => Self::Platform,
            CliGroupBy::AppVersion => Self::AppVersion,
            CliGroupBy::BuildVersion => Self::BuildVersion,
            CliGroupBy::Locale => Self::Locale,
            CliGroupBy::SubscriptionPlan => Self::SubscriptionPlan,
        }
    }
}

fn print_table_report(report: &StatsReport, top: usize) {
    println!("Sources: {} files", report.source_file_count);
    println!("Range: {}", format_range(report));
    if report.skipped_invalid_line_count > 0 {
        println!(
            "Skipped invalid lines: {}",
            report.skipped_invalid_line_count
        );
    }
    println!();

    print_summary(&report.summary);

    if !report.groups.is_empty() {
        println!();
        println!("By {}", report.group_by.as_str());
        print_rows(&report.groups);
    }

    if top > 0 {
        print_dimension_sections(&report.dimensions, top);
        print_bucket_sections(&report.summary.buckets, top);
    }
}

fn format_range(report: &StatsReport) -> String {
    match (report.from, report.to) {
        (Some(from), Some(to)) => format!("{from} <= receivedAt < {to}"),
        (Some(from), None) => format!("receivedAt >= {from}"),
        (None, Some(to)) => format!("receivedAt < {to}"),
        (None, None) => "all records".to_string(),
    }
}

fn print_summary(row: &ReportRow) {
    println!("Summary");
    let rows = vec![
        vec!["renders".to_string(), row.renders.to_string()],
        vec!["total blocks".to_string(), row.total_blocks.to_string()],
        vec![
            "avg blocks/render".to_string(),
            format!("{:.2}", row.average_blocks_per_render),
        ],
        vec![
            "first received".to_string(),
            row.first_received_at
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ],
        vec![
            "last received".to_string(),
            row.last_received_at
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ],
        vec!["issues".to_string(), row.issue_count.to_string()],
        vec![
            "locked features".to_string(),
            row.locked_feature_count.to_string(),
        ],
        vec!["unsupported".to_string(), row.unsupported_count.to_string()],
    ];
    print_table(&["metric", "value"], &rows);
}

fn print_rows(rows: &[ReportRow]) {
    let table_rows = rows
        .iter()
        .map(|row| {
            vec![
                row.key.clone(),
                row.renders.to_string(),
                row.total_blocks.to_string(),
                format!("{:.2}", row.average_blocks_per_render),
                row.issue_count.to_string(),
                row.locked_feature_count.to_string(),
                row.unsupported_count.to_string(),
            ]
        })
        .collect::<Vec<_>>();
    print_table(
        &[
            "key",
            "renders",
            "blocks",
            "avg blocks",
            "issues",
            "locked",
            "unsupported",
        ],
        &table_rows,
    );
}

fn print_dimension_sections(dimensions: &DimensionTotals, top: usize) {
    print_count_section("Platforms", &dimensions.platforms, top);
    print_count_section("Subscription plans", &dimensions.subscription_plans, top);
    print_count_section("Locales", &dimensions.locales, top);
    print_count_section("App versions", &dimensions.app_versions, top);
    print_count_section("Build versions", &dimensions.build_versions, top);
}

fn print_bucket_sections(buckets: &BucketTotals, top: usize) {
    print_count_section("Blocks by kind", &buckets.blocks_by_kind, top);
    print_count_section("Headings by level", &buckets.headings_by_level, top);
    print_count_section(
        "Code blocks by language kind",
        &buckets.code_blocks_by_language_kind,
        top,
    );
    print_count_section("Callouts by kind", &buckets.callouts_by_kind, top);
    print_count_section(
        "Locked features by kind",
        &buckets.locked_features_by_kind,
        top,
    );
    print_count_section("Unsupported by kind", &buckets.unsupported_by_kind, top);
    print_count_section("Issues by kind", &buckets.issues_by_kind, top);
}

fn print_count_section(title: &str, entries: &[CountEntry], top: usize) {
    if entries.is_empty() {
        return;
    }
    let total = entries.iter().map(|entry| entry.count).sum::<u64>();
    let rows = entries
        .iter()
        .take(top)
        .map(|entry| {
            let share = if total == 0 {
                0.0
            } else {
                entry.count as f64 * 100.0 / total as f64
            };
            vec![
                entry.kind.clone(),
                entry.count.to_string(),
                format!("{share:.1}%"),
            ]
        })
        .collect::<Vec<_>>();
    println!();
    println!("{title}");
    print_table(&["kind", "count", "share"], &rows);
}

fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths = headers
        .iter()
        .map(|header| header.len())
        .collect::<Vec<usize>>();
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(cell.len());
            }
        }
    }

    print_table_line(
        &headers
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>(),
        &widths,
    );
    let separator = widths
        .iter()
        .map(|width| "-".repeat(*width))
        .collect::<Vec<_>>();
    print_table_line(&separator, &widths);
    for row in rows {
        print_table_line(row, &widths);
    }
}

fn print_table_line(cells: &[String], widths: &[usize]) {
    for (index, width) in widths.iter().enumerate() {
        if index > 0 {
            print!("  ");
        }
        let value = cells.get(index).map(String::as_str).unwrap_or("");
        if index == 0 {
            print!("{value:<width$}");
        } else {
            print!("{value:>width$}");
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use clap::Parser;

    #[test]
    fn resolves_last_window_to_absolute_range() {
        let cli = Cli::try_parse_from(["oriha-stats-viewer", "--last", "7d"]).expect("parse cli");
        let now = Utc
            .with_ymd_and_hms(2026, 6, 15, 10, 0, 0)
            .single()
            .expect("valid time");

        let (from, to) = resolve_time_range(&cli, now).expect("resolve time range");

        assert_eq!(
            from.map(|value| value.to_rfc3339()),
            Some("2026-06-08T10:00:00+00:00".to_string())
        );
        assert_eq!(
            to.map(|value| value.to_rfc3339()),
            Some("2026-06-15T10:00:00+00:00".to_string())
        );
    }

    #[test]
    fn rejects_last_with_absolute_range() {
        let result =
            Cli::try_parse_from(["oriha-stats-viewer", "--from", "2026-06-01", "--last", "7d"]);

        assert!(result.is_err());
    }
}
