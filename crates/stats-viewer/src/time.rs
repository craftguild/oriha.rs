use crate::error::ViewerError;
use jiff::{civil::Date, tz::TimeZone, Span, Timestamp};

pub fn parse_from_bound(value: &str) -> Result<Timestamp, ViewerError> {
    parse_time_bound(value, DateBound::Start)
}

pub fn parse_to_bound(value: &str) -> Result<Timestamp, ViewerError> {
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
    now: Timestamp,
) -> Result<(Timestamp, Timestamp), ViewerError> {
    let overflow = |_| ViewerError::LastWindowOverflow {
        value: window.label(),
    };
    let span = match window {
        LastWindow::Hours(count) => Span::new().try_hours(count),
        LastWindow::Days(count) => Span::new().try_days(count),
        LastWindow::Weeks(count) => Span::new().try_weeks(count),
        LastWindow::Months(count) => Span::new().try_months(count),
    }
    .map_err(overflow)?;
    let from = now
        .to_zoned(TimeZone::UTC)
        .checked_sub(span)
        .map_err(overflow)?
        .timestamp();
    Ok((from, now))
}

#[derive(Clone, Copy)]
enum DateBound {
    Start,
    End,
}

fn parse_time_bound(value: &str, bound: DateBound) -> Result<Timestamp, ViewerError> {
    if let Ok(value) = value.parse::<Timestamp>() {
        return Ok(value);
    }

    if let Ok(date) = Date::strptime("%Y-%m-%d", value) {
        let date = match bound {
            DateBound::Start => date,
            DateBound::End => date.tomorrow().map_err(|_| ViewerError::InvalidTimeBound {
                value: value.to_string(),
            })?,
        };
        let start = date.at(0, 0, 0, 0).to_zoned(TimeZone::UTC).map_err(|_| {
            ViewerError::InvalidTimeBound {
                value: value.to_string(),
            }
        })?;
        return Ok(start.timestamp());
    }

    Err(ViewerError::InvalidTimeBound {
        value: value.to_string(),
    })
}
