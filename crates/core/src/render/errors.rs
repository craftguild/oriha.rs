use super::*;

pub(super) fn invalid_render_output(code: &str, message: String) -> RenderOutput {
    let mut stats = RenderStats::default();
    stats.record_issue_code(code, Severity::Error);
    stats.finalize();
    RenderOutput {
        html: String::new(),
        css: None,
        stats,
        diagnostics: vec![RenderDiagnostic {
            severity: Severity::Error,
            code: code.to_string(),
            message,
            source_range: None,
            block_id: None,
            engine: DiagnosticEngine::Core,
        }],
        assets: Vec::new(),
        source_map: Vec::new(),
    }
}

pub(super) fn invalid_render_patch_output(
    code: &str,
    message: String,
    reason: RenderPatchFallbackReason,
) -> RenderPatchOutput {
    patch::replace_all_output(
        invalid_render_output(code, message),
        RenderSnapshot::default(),
        reason,
    )
}
