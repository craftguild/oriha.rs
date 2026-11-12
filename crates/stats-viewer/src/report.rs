use crate::aggregate::ReportAccumulator;
use crate::error::ViewerError;
use crate::model::{GroupBy, StatsReport, StoredRecord};
use crate::reader::{collect_stats_files, read_stats_file};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct ReportOptions {
    pub inputs: Vec<PathBuf>,
    pub include_open: bool,
    pub skip_invalid: bool,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub group_by: GroupBy,
}

impl Default for ReportOptions {
    fn default() -> Self {
        Self {
            inputs: vec![PathBuf::from("./data/stats")],
            include_open: false,
            skip_invalid: false,
            from: None,
            to: None,
            group_by: GroupBy::Hour,
        }
    }
}

pub fn load_report(options: ReportOptions) -> Result<StatsReport, ViewerError> {
    if let (Some(from), Some(to)) = (options.from, options.to) {
        if from >= to {
            return Err(ViewerError::InvalidTimeRange { from, to });
        }
    }

    let inputs = if options.inputs.is_empty() {
        vec![PathBuf::from("./data/stats")]
    } else {
        options.inputs.clone()
    };
    let source_files = collect_stats_files(&inputs, options.include_open)?;

    let mut skipped_invalid_line_count = 0_u64;
    let mut accumulator = ReportAccumulator::new(options.group_by);

    for path in &source_files {
        let skipped = read_stats_file(path, options.skip_invalid, |record| {
            if record_in_range(record, options.from, options.to) {
                accumulator.record(record);
            }
        })?;
        skipped_invalid_line_count += skipped;
    }

    let (summary, groups, dimensions) = accumulator.into_report_parts();
    Ok(StatsReport {
        source_file_count: source_files.len(),
        source_files: source_files
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        skipped_invalid_line_count,
        from: options.from,
        to: options.to,
        group_by: options.group_by,
        summary,
        groups,
        dimensions,
    })
}

fn record_in_range(
    record: &StoredRecord,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
) -> bool {
    from.is_none_or(|from| record.received_at >= from)
        && to.is_none_or(|to| record.received_at < to)
}
