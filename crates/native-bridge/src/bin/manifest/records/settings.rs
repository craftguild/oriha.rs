//! Settings controls, render options and font descriptors.
use super::*;

pub(super) fn records() -> Vec<RecordSpec> {
    vec![
        record(
            "FontFamilySpec",
            &[],
            vec![
                field("family", primitive("string"), false),
                field("fallbacks", array(primitive("string")), true),
            ],
        ),
        record(
            "BridgeOption",
            &[],
            vec![
                field("id", primitive("string"), false),
                field("label", primitive("string"), false),
                field("description", primitive("string"), true),
                field("family", primitive("string"), true),
                field("recommended", primitive("boolean"), true),
            ],
        ),
        record(
            "BridgeFontOption",
            &[],
            vec![
                field("id", primitive("string"), false),
                field("label", primitive("string"), false),
                field("description", primitive("string"), true),
                field("family", primitive("string"), true),
                field("recommended", primitive("boolean"), true),
                field("fontFamily", nullable(named("FontFamilySpec")), false),
            ],
        ),
        record(
            "BridgeRenderInput",
            &[],
            vec![
                field("markdown", primitive("string"), false),
                field("locale", primitive("string"), true),
                field("plan", named("Plan"), false),
                field("renderingOptions", named("BridgeRenderingOptions"), false),
                field("print", named("PrintOptions"), true),
                field("renderMode", named("RenderMode"), false),
                field("watermark", named("WatermarkOptions"), true),
                field("sourceAttributes", primitive("boolean"), false),
                field("tableOfContents", primitive("boolean"), false),
                field("viewport", named("Viewport"), true),
            ],
        ),
        record(
            "BridgeSettingsInput",
            &[],
            vec![
                field("locale", primitive("string"), true),
                field("plan", named("Plan"), false),
                field("renderingOptions", named("BridgeRenderingOptions"), false),
                field("mode", named("RenderMode"), false),
                field("viewport", named("Viewport"), true),
            ],
        ),
        record(
            "BridgeRenderingOptions",
            &[],
            vec![
                field("colorScheme", named("ColorScheme"), false),
                field("appearanceStyle", named("AppearanceStyle"), false),
                field("syntaxTheme", named("SyntaxTheme"), false),
                field("customSyntaxTheme", primitive("string"), true),
                field("headingFont", named("HeadingFont"), false),
                field("customHeadingFont", named("FontFamilySpec"), true),
                field("bodyFont", named("BodyFont"), false),
                field("customBodyFont", named("FontFamilySpec"), true),
                field("bodyFontSizePx", primitive("number"), true),
                field("codeFont", named("CodeFont"), false),
                field("customCodeFont", named("FontFamilySpec"), true),
                field("tableBorderStyle", named("TableBorderStyle"), false),
                field("roundedCorners", primitive("boolean"), false),
                field("shadows", primitive("boolean"), false),
                field("fullTableData", primitive("boolean"), false),
                field("blockBorders", primitive("boolean"), false),
            ],
        ),
        record(
            "BridgeEffectiveSettingsInput",
            &[],
            vec![
                field("locale", named("OrihaBuiltinLocale"), false),
                field("plan", named("Plan"), false),
                field("renderingOptions", named("BridgeRenderingOptions"), false),
                field("mode", named("RenderMode"), false),
                field("viewport", named("Viewport"), true),
            ],
        ),
        record(
            "BridgeControlAvailability",
            &[],
            vec![
                field("enabled", primitive("boolean"), false),
                field("reason", primitive("string"), true),
            ],
        ),
        record(
            "BridgeSettingsControls",
            &[],
            vec![
                field("plan", named("BridgeControlAvailability"), false),
                field("colorScheme", named("BridgeControlAvailability"), false),
                field("syntaxTheme", named("BridgeControlAvailability"), false),
                field("headingFont", named("BridgeControlAvailability"), false),
                field("bodyFont", named("BridgeControlAvailability"), false),
                field("codeFont", named("BridgeControlAvailability"), false),
                field(
                    "tableBorderStyle",
                    named("BridgeControlAvailability"),
                    false,
                ),
            ],
        ),
        record(
            "BridgeAvailableOptions",
            &[],
            vec![
                field("plan", array(named("BridgeOption")), false),
                field("colorScheme", array(named("BridgeOption")), false),
                field("syntaxTheme", array(named("BridgeOption")), false),
                field("headingFont", array(named("BridgeFontOption")), false),
                field("bodyFont", array(named("BridgeFontOption")), false),
                field("codeFont", array(named("BridgeFontOption")), false),
                field("tableBorderStyle", array(named("BridgeOption")), false),
                field("locale", array(named("BridgeOption")), false),
            ],
        ),
        record(
            "BridgeResolvedSettings",
            &[],
            vec![
                field("resolvedLocale", named("OrihaBuiltinLocale"), false),
                field(
                    "effectiveInput",
                    named("BridgeEffectiveSettingsInput"),
                    false,
                ),
                field("controls", named("BridgeSettingsControls"), false),
                field("availableOptions", named("BridgeAvailableOptions"), false),
                field("features", array(named("BridgeFeature")), false),
                field("diagnostics", array(named("RenderDiagnostic")), false),
            ],
        ),
    ]
}
