use chrono::{DateTime, Utc};
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ViewerError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    InvalidJsonLine {
        path: PathBuf,
        line: usize,
        source: serde_json::Error,
    },
    InvalidTimeBound {
        value: String,
    },
    InvalidLastWindow {
        value: String,
    },
    LastWindowOverflow {
        value: String,
    },
    InvalidTimeRange {
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    },
    NoStatsFiles {
        inputs: Vec<PathBuf>,
        include_open: bool,
    },
}

impl fmt::Display for ViewerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(f, "failed to read {}: {source}", path.display())
            }
            Self::InvalidJsonLine { path, line, source } => write!(
                f,
                "failed to parse JSONL record at {}:{line}: {source}",
                path.display()
            ),
            Self::InvalidTimeBound { value } => write!(
                f,
                "invalid time bound {value:?}; use RFC3339 like 2026-06-15T00:00:00Z or YYYY-MM-DD"
            ),
            Self::InvalidLastWindow { value } => write!(
                f,
                "invalid --last value {value:?}; use a positive window like 24h, 1d, 7d, 1w, or 1m"
            ),
            Self::LastWindowOverflow { value } => {
                write!(f, "--last value {value:?} is too large to resolve")
            }
            Self::InvalidTimeRange { from, to } => {
                write!(f, "--from must be before --to ({from} >= {to})")
            }
            Self::NoStatsFiles {
                inputs,
                include_open,
            } => {
                let paths = inputs
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                if *include_open {
                    write!(
                        f,
                        "no .jsonl or .jsonl.open stats files found under {paths}"
                    )
                } else {
                    write!(
                        f,
                        "no sealed .jsonl stats files found under {paths}; pass --include-open to read active chunks"
                    )
                }
            }
        }
    }
}

impl Error for ViewerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidJsonLine { source, .. } => Some(source),
            _ => None,
        }
    }
}
