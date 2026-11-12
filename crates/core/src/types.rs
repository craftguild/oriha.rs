use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod document;
pub use document::*;
mod patch;
pub use patch::*;
mod prepared;
pub use prepared::*;
mod measurements;
pub use measurements::*;
mod options;
pub use options::*;
mod stats;
pub use stats::*;
mod stats_kinds;
pub use stats_kinds::*;

const fn default_true() -> bool {
    true
}
