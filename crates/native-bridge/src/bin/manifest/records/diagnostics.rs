//! Render diagnostics, assets and source mapping records.
use super::*;

pub(super) fn records() -> Vec<RecordSpec> {
    vec![
        record(
            "RenderDiagnostic",
            &[],
            vec![
                field("severity", named("RenderDiagnosticSeverity"), false),
                field("code", primitive("string"), false),
                field("message", primitive("string"), false),
                field("sourceRange", named("SourceRange"), true),
                field("blockId", primitive("string"), true),
                field("engine", named("RenderDiagnosticEngine"), false),
            ],
        ),
        record(
            "SourceRange",
            &[],
            vec![
                field("startLine", primitive("number"), false),
                field("startColumn", primitive("number"), false),
                field("endLine", primitive("number"), false),
                field("endColumn", primitive("number"), false),
            ],
        ),
        record(
            "RenderAsset",
            &[],
            vec![
                field("id", primitive("string"), false),
                field("kind", named("RenderAssetKind"), false),
                field("mimeType", primitive("string"), false),
                field("content", primitive("string"), false),
            ],
        ),
        record(
            "RenderedBlock",
            &[],
            vec![
                field("id", primitive("string"), false),
                field("kind", primitive("string"), false),
                field("sourceRange", named("SourceRange"), false),
                field("metadata", map(primitive("string")), true),
            ],
        ),
    ]
}
