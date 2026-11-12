//! Text measurement styles, requests, runs and results.
use super::*;

pub(super) fn records() -> Vec<RecordSpec> {
    vec![
        record_with_index_signature(
            "TextMeasurementStyle",
            &[],
            vec![
                field("fontFamily", primitive("string"), true),
                field("fontSize", primitive("number"), true),
                field(
                    "fontWeight",
                    union(vec![primitive("string"), primitive("number")]),
                    true,
                ),
                field("fontStyle", primitive("string"), true),
                field("lineHeight", primitive("number"), true),
                field("letterSpacing", primitive("number"), true),
                field("textTransform", primitive("string"), true),
                field("textDecorationLine", primitive("string"), true),
                field("color", primitive("string"), true),
                field("width", primitive("number"), true),
                field("maxWidth", primitive("number"), true),
            ],
            index_signature(
                "key",
                "string",
                union(vec![primitive("string"), primitive("number"), undefined()]),
            ),
        ),
        record(
            "TextMeasurementRun",
            &[],
            vec![
                field("id", primitive("string"), true),
                field("text", primitive("string"), false),
                field("style", named("TextMeasurementStyle"), true),
            ],
        ),
        record(
            "TextMeasurementRequest",
            &[],
            vec![
                field("id", primitive("string"), false),
                field("engine", primitive("string"), true),
                field("kind", primitive("string"), true),
                field("text", primitive("string"), false),
                field("style", named("TextMeasurementStyle"), true),
                field("fontFamily", primitive("string"), true),
                field("fontSize", primitive("number"), true),
                field(
                    "fontWeight",
                    union(vec![primitive("string"), primitive("number")]),
                    true,
                ),
                field("fontStyle", primitive("string"), true),
                field("runs", array(named("TextMeasurementRun")), true),
                field("width", primitive("number"), true),
                field("maxWidth", primitive("number"), true),
                field("wrapMode", primitive("string"), true),
            ],
        ),
        record(
            "TextMeasurementBatch",
            &[],
            vec![field(
                "requests",
                array(named("TextMeasurementRequest")),
                false,
            )],
        ),
        record(
            "TextMeasurementRect",
            &[],
            vec![
                field("x", primitive("number"), false),
                field("y", primitive("number"), false),
                field("width", primitive("number"), false),
                field("height", primitive("number"), false),
            ],
        ),
        record(
            "TextMeasurementRunResult",
            &[],
            vec![
                field("id", primitive("string"), true),
                field("index", primitive("number"), false),
                field("width", primitive("number"), false),
                field("height", primitive("number"), false),
                field("rects", array(named("TextMeasurementRect")), false),
            ],
        ),
        record(
            "TextMeasurementResult",
            &[],
            vec![
                field("id", primitive("string"), false),
                field("width", primitive("number"), false),
                field("height", primitive("number"), true),
                field("lineCount", primitive("number"), true),
                field("left", primitive("number"), true),
                field("right", primitive("number"), true),
                field("rawWidth", primitive("number"), true),
                field("runs", array(named("TextMeasurementRunResult")), true),
            ],
        ),
    ]
}
