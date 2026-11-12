use crate::error::ViewerError;
use chrono::{DateTime, Days, Duration, Months, NaiveDate, TimeZone, Utc};

pub fn parse_from_bound(value: &str) -> Result<DateTime<Utc>, ViewerError> {
    parse_time_bound(value, DateBound::Start)
}

pub fn parse_to_bound(value: &str) -> Result<DateTime<Utc>, ViewerError> {
    parse_time_bound(value, DateBound::End)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LastWindow {
    Hours(u32),
    Days(u32),
    Weeks(u32),
    Months(u32),
}

impl LastWindow {
    fn label(self) -> String {
        match self {
            Self::Hours(count) => format!("{count}h"),
            Self::Days(count) => format!("{count}d"),
            Self::Weeks(count) => format!("{count}w"),
            Self::Months(count) => format!("{count}m"),
        }
    }
}

pub fn parse_last_window(value: &str) -> Result<LastWindow, ViewerError> {
    let value = value.trim();
    let digit_len = value
        .char_indices()
        .find_map(|(index, ch)| (!ch.is_ascii_digit()).then_some(index))
        .unwrap_or(value.len());
    if digit_len == 0 || digit_len == value.len() {
        return Err(ViewerError::InvalidLastWindow {
            value: value.to_string(),
        });
    }

    let count = value[..digit_len]
        .parse::<u32>()
        .ok()
        .filter(|count| *count > 0)
        .ok_or_else(|| ViewerError::InvalidLastWindow {
            value: value.to_string(),
        })?;
    let unit = value[digit_len..].trim().to_ascii_lowercase();

    match unit.as_str() {
        "h" | "hour" | "hours" => Ok(LastWindow::Hours(count)),
        "d" | "day" | "days" => Ok(LastWindow::Days(count)),
        "w" | "week" | "weeks" => Ok(LastWindow::Weeks(count)),
        "m" | "mo" | "month" | "months" => Ok(LastWindow::Months(count)),
        _ => Err(ViewerError::InvalidLastWindow {
            value: value.to_string(),
        }),
    }
}

pub fn resolve_last_window(
    window: LastWindow,
    now: DateTime<Utc>,
) -> Result<(DateTime<Utc>, DateTime<Utc>), ViewerError> {
    let from = match window {
        LastWindow::Hours(count) => now.checked_sub_signed(Duration::hours(i64::from(count))),
        LastWindow::Days(count) => now.checked_sub_signed(Duration::days(i64::from(count))),
        LastWindow::Weeks(count) => now.checked_sub_signed(Duration::weeks(i64::from(count))),
        LastWindow::Months(count) => now.checked_sub_months(Months::new(count)),
    }
    .ok_or_else(|| ViewerError::LastWindowOverflow {
        value: window.label(),
    })?;
    Ok((from, now))
}

#[derive(Clone, Copy)]
enum DateBound {
    Start,
    End,
}

fn parse_time_bound(value: &str, bound: DateBound) -> Result<DateTime<Utc>, ViewerError> {
    if let Ok(value) = DateTime::parse_from_rfc3339(value) {
        return Ok(value.with_timezone(&Utc));
    }

    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        let date = match bound {
            DateBound::Start => date,
            DateBound::End => date.checked_add_days(Days::new(1)).ok_or_else(|| {
                ViewerError::InvalidTimeBound {
                    value: value.to_string(),
                }
            })?,
        };
        let start = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| ViewerError::InvalidTimeBound {
                value: value.to_string(),
            })?;
        return Ok(Utc.from_utc_datetime(&start));
    }

    Err(ViewerError::InvalidTimeBound {
        value: value.to_string(),
    })
}
