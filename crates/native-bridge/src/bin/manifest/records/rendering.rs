//! Render, HTML export, patch snapshots and statistics records.
use super::*;

pub(super) fn records() -> Vec<RecordSpec> {
    vec![
        record(
            "PrintOptions",
            &[],
            vec![
                field("pageSize", primitive("string"), true),
                field("margin", primitive("string"), true),
            ],
        ),
        record(
            "WatermarkOptions",
            &[],
            vec![
                field("text", primitive("string"), false),
                field("imageSrc", primitive("string"), true),
                field("opacity", primitive("number"), true),
            ],
        ),
        record(
            "RenderOptions",
            &[],
            vec![
                field("watermark", named("WatermarkOptions"), true),
                field("sourceAttributes", primitive("boolean"), true),
                field("tableOfContents", primitive("boolean"), true),
            ],
        ),
        record(
            "ExportOptions",
            &[],
            vec![
                field("watermark", named("WatermarkOptions"), true),
                field("sourceAttributes", primitive("boolean"), true),
                field("tableOfContents", primitive("boolean"), true),
                field("print", named("PrintOptions"), true),
            ],
        ),
        record(
            "HtmlExportOptions",
            &[],
            vec![
                field("minify", primitive("boolean"), true),
                field("obfuscate", primitive("boolean"), true),
                field("preserveHeadingAnchors", primitive("boolean"), true),
                field("stripOrihaDataAttributes", primitive("boolean"), true),
            ],
        ),
        record(
            "ExportHtmlInput",
            &[],
            vec![
                field("input", named("BridgeRenderInput"), false),
                field("options", named("HtmlExportOptions"), true),
            ],
        ),
        record(
            "Viewport",
            &[],
            vec![
                field("width", primitive("number"), false),
                field("scale", primitive("number"), false),
            ],
        ),
        record(
            "RenderOutput",
            &[],
            vec![
                field("html", primitive("string"), false),
                field("css", primitive("string"), true),
                field("stats", named("RenderStats"), false),
                field("diagnostics", array(named("RenderDiagnostic")), false),
                field("assets", array(named("RenderAsset")), false),
                field("sourceMap", array(named("RenderedBlock")), false),
            ],
        ),
        record(
            "ExportHtmlOutput",
            &[],
            vec![
                field("html", primitive("string"), false),
                field("stats", named("RenderStats"), false),
                field("diagnostics", array(named("RenderDiagnostic")), false),
            ],
        ),
        record(
            "RenderPatchOutput",
            &[],
            vec![
                field("output", named("RenderOutput"), false),
                field("snapshot", named("RenderSnapshot"), false),
                field("strategy", named("RenderPatchStrategy"), false),
                field("fallbackReason", named("RenderPatchFallbackReason"), true),
                field("patches", array(named("RenderPatchOperation")), false),
            ],
        ),
        record(
            "RenderPatchOperation",
            &[],
            vec![
                field("op", named("RenderPatchOperationKind"), false),
                field("html", primitive("string"), false),
                field("removeStableIds", array(primitive("string")), true),
                field("insertAfterStableId", primitive("string"), true),
                field("insertBeforeStableId", primitive("string"), true),
            ],
        ),
        record(
            "RenderSnapshot",
            &[],
            vec![
                field("schemaVersion", primitive("number"), false),
                field("settingsFingerprint", primitive("string"), false),
                field("nextStableIndex", primitive("number"), false),
                field("blocks", array(named("RenderSnapshotBlock")), false),
                field("tailHash", primitive("string"), false),
                field("tailHtmlByteLength", primitive("number"), false),
                field("outputResourceFingerprint", primitive("string"), false),
            ],
        ),
        record(
            "RenderSnapshotBlock",
            &[],
            vec![
                field("stableId", primitive("string"), false),
                field("kind", primitive("string"), false),
                field("sourceHash", primitive("string"), false),
                field("htmlHash", primitive("string"), false),
                field("sourceRange", named("SourceRange"), false),
            ],
        ),
        record(
            "RenderStats",
            &[],
            vec![
                field("schemaVersion", primitive("number"), false),
                field("totalBlocks", primitive("number"), false),
                field("blocksByKind", array(named("KindCount")), false),
                field("headingsByLevel", array(named("KindCount")), false),
                field("codeBlocksByLanguageKind", array(named("KindCount")), false),
                field("calloutsByKind", array(named("KindCount")), false),
                field("lockedFeaturesByKind", array(named("KindCount")), false),
                field("unsupportedByKind", array(named("KindCount")), false),
                field("issuesByKind", array(named("KindCount")), false),
            ],
        ),
        record(
            "KindCount",
            &[],
            vec![
                field("kind", primitive("string"), false),
                field("count", primitive("number"), false),
            ],
        ),
    ]
}
