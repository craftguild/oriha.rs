//! Stable wire values for bridge option and diagnostic enumerations.
pub(super) const RENDER_MODE: &[(&str, &str)] = &[
    ("StyledHtml", "styledHtml"),
    ("SemanticHtml", "semanticHtml"),
];

pub(super) const APPEARANCE_STYLE: &[(&str, &str)] = &[("Rich", "rich"), ("Simple", "simple")];

pub(super) const TABLE_BORDER_STYLE: &[(&str, &str)] = &[
    ("None", "none"),
    ("Inner", "inner"),
    ("All", "all"),
    ("Vertical", "vertical"),
    ("Horizontal", "horizontal"),
];

pub(super) const RENDER_DIAGNOSTIC_SEVERITY: &[(&str, &str)] =
    &[("Info", "info"), ("Warning", "warning"), ("Error", "error")];

pub(super) const RENDER_DIAGNOSTIC_ENGINE: &[(&str, &str)] =
    &[("Core", "core"), ("Bridge", "bridge")];

pub(super) const RENDER_ASSET_KIND: &[(&str, &str)] =
    &[("Svg", "svg"), ("Css", "css"), ("Other", "other")];

pub(super) const RENDER_PATCH_STRATEGY: &[(&str, &str)] =
    &[("Incremental", "incremental"), ("ReplaceAll", "replaceAll")];

pub(super) const RENDER_PATCH_FALLBACK_REASON: &[(&str, &str)] = &[
    ("MissingSnapshot", "missingSnapshot"),
    ("IncompatibleSnapshot", "incompatibleSnapshot"),
    ("TailChanged", "tailChanged"),
    ("OutputTruncated", "outputTruncated"),
    ("InvalidInput", "invalidInput"),
    ("EnginePanic", "enginePanic"),
];

pub(super) const BRIDGE_FEATURE: &[(&str, &str)] = &[
    ("Gfm", "gfm"),
    ("SyntaxHighlight", "syntaxHighlight"),
    ("DarkMode", "darkMode"),
    ("Tables", "tables"),
    ("TaskLists", "taskLists"),
    ("Footnotes", "footnotes"),
    ("RawHtml", "rawHtml"),
    ("PdfExport", "pdfExport"),
    ("Callouts", "callouts"),
    ("RemovePdfWatermark", "removePdfWatermark"),
    ("PrioritySupport", "prioritySupport"),
    ("Plantuml", "plantuml"),
    ("Mermaid", "mermaid"),
    ("Math", "math"),
    ("SyntaxThemeCustomization", "syntaxThemeCustomization"),
    ("FontCustomization", "fontCustomization"),
];

pub(super) const USAGE_ANALYTICS_OUTBOX_COMMAND_KIND: &[(&str, &str)] = &[
    ("LoadEnvelopes", "loadEnvelopes"),
    ("WriteEnvelope", "writeEnvelope"),
    ("DeleteEnvelope", "deleteEnvelope"),
    ("PurgeEnvelopes", "purgeEnvelopes"),
    ("SendHttp", "sendHttp"),
];

pub(super) const PLAN: &[(&str, &str)] = &[("Free", "free"), ("Premium", "premium")];

pub(super) const COLOR_SCHEME: &[(&str, &str)] = &[("Light", "light"), ("Dark", "dark")];

pub(super) const SYNTAX_THEME: &[(&str, &str)] = &[
    ("GitHubLight", "github-light"),
    ("GitHubDark", "github-dark"),
    ("SolarizedLight", "solarized-light"),
    ("SolarizedDark", "solarized-dark"),
    ("GruvboxLight", "gruvbox-light"),
    ("GruvboxDark", "gruvbox-dark"),
    ("CatppuccinLatte", "catppuccin-latte"),
    ("CatppuccinMocha", "catppuccin-mocha"),
    ("AyuLight", "ayu-light"),
    ("AyuDark", "ayu-dark"),
    ("TokyoNightDay", "tokyo-night-day"),
    ("TokyoNight", "tokyo-night"),
    ("RosePineDawn", "rose-pine-dawn"),
    ("RosePine", "rose-pine"),
    ("HighContrastLight", "high-contrast-light"),
    ("HighContrastDark", "high-contrast-dark"),
    ("Custom", "custom"),
];

pub(super) const HEADING_FONT: &[(&str, &str)] =
    &[("SystemDefault", "system-default"), ("Custom", "custom")];

pub(super) const BODY_FONT: &[(&str, &str)] =
    &[("SystemDefault", "system-default"), ("Custom", "custom")];

pub(super) const CODE_FONT: &[(&str, &str)] =
    &[("SystemDefault", "system-default"), ("Custom", "custom")];
