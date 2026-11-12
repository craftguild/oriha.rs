mod distribution;
uniffi::setup_scaffolding!();

mod conversions;
mod diagnostics;
mod plan_policy;
mod render;
mod settings;
mod stats;
mod types;
mod usage_analytics;

pub use render::{
    native_bridge_version_info, prepare_html_export, prepare_html_export_json,
    prepare_markdown_layout, prepare_markdown_layout_json, prepare_markdown_patch,
    prepare_markdown_patch_json, render_prepared_html_export, render_prepared_html_export_json,
    render_prepared_markdown, render_prepared_markdown_json, render_prepared_markdown_patch,
    render_prepared_markdown_patch_json,
};
pub use settings::{resolve_render_settings, resolve_render_settings_json};
pub use types::*;
pub use usage_analytics::{
    usage_analytics_outbox_resume, usage_analytics_outbox_resume_json,
    usage_analytics_outbox_start_delivery_state_change,
    usage_analytics_outbox_start_delivery_state_change_json, usage_analytics_outbox_start_drain,
    usage_analytics_outbox_start_drain_json, usage_analytics_outbox_start_enqueue,
    usage_analytics_outbox_start_enqueue_json,
};

pub(crate) const PREPARED_MARKDOWN_LAYOUT_VERSION: u32 = 2;
pub(crate) const PREPARED_MARKDOWN_PATCH_VERSION: u32 = 1;
pub(crate) const PREPARED_HTML_EXPORT_VERSION: u32 = 2;
pub(crate) const LOCKED_FEATURE_CODE: &str = "plan.locked_feature";

#[cfg(test)]
mod tests;
