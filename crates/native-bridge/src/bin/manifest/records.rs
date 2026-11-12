//! Ordered bridge record manifest assembled from contract families.
use super::*;
#[path = "records/analytics.rs"]
mod analytics;
#[path = "records/diagnostics.rs"]
mod diagnostics;
#[path = "records/measurements.rs"]
mod measurements;
#[path = "records/prepared.rs"]
mod prepared;
#[path = "records/rendering.rs"]
mod rendering;
#[path = "records/settings.rs"]
mod settings;

pub(super) fn records() -> Vec<RecordSpec> {
    [
        settings::records(),
        rendering::records(),
        analytics::records(),
        diagnostics::records(),
        measurements::records(),
        prepared::records(),
    ]
    .into_iter()
    .flatten()
    .collect()
}
