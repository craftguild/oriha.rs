// Public native render-option and diagnostic enumerations.

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum Plan {
    #[serde(rename = "free")]
    Free,
    #[serde(rename = "premium")]
    Premium,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum ColorScheme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum AppearanceStyle {
    #[serde(rename = "rich")]
    Rich,
    #[serde(rename = "simple")]
    Simple,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum SyntaxTheme {
    #[serde(rename = "github-light")]
    GitHubLight,
    #[serde(rename = "github-dark")]
    GitHubDark,
    #[serde(rename = "solarized-light")]
    SolarizedLight,
    #[serde(rename = "solarized-dark")]
    SolarizedDark,
    #[serde(rename = "gruvbox-light")]
    GruvboxLight,
    #[serde(rename = "gruvbox-dark")]
    GruvboxDark,
    #[serde(rename = "catppuccin-latte")]
    CatppuccinLatte,
    #[serde(rename = "catppuccin-mocha")]
    CatppuccinMocha,
    #[serde(rename = "ayu-light")]
    AyuLight,
    #[serde(rename = "ayu-dark")]
    AyuDark,
    #[serde(rename = "tokyo-night-day")]
    TokyoNightDay,
    #[serde(rename = "tokyo-night")]
    TokyoNight,
    #[serde(rename = "rose-pine-dawn")]
    RosePineDawn,
    #[serde(rename = "rose-pine")]
    RosePine,
    #[serde(rename = "high-contrast-light")]
    HighContrastLight,
    #[serde(rename = "high-contrast-dark")]
    HighContrastDark,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum HeadingFont {
    #[serde(
        rename = "system-default",
        alias = "noto-sans",
        alias = "inter",
        alias = "source-sans-3",
        alias = "roboto",
        alias = "open-sans",
        alias = "lato",
        alias = "montserrat",
        alias = "poppins",
        alias = "roboto-slab",
        alias = "playfair-display",
        alias = "oswald",
        alias = "merriweather",
        alias = "source-serif-4",
        alias = "libre-baskerville"
    )]
    SystemDefault,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum BodyFont {
    #[serde(
        rename = "system-default",
        alias = "noto-sans",
        alias = "inter",
        alias = "source-sans-3",
        alias = "roboto",
        alias = "open-sans",
        alias = "lato",
        alias = "montserrat",
        alias = "poppins",
        alias = "roboto-slab",
        alias = "playfair-display",
        alias = "oswald",
        alias = "merriweather",
        alias = "source-serif-4",
        alias = "libre-baskerville"
    )]
    SystemDefault,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum CodeFont {
    #[serde(
        rename = "system-default",
        alias = "jetbrains-mono",
        alias = "roboto-mono",
        alias = "ibm-plex-mono",
        alias = "source-code-pro",
        alias = "noto-sans-mono",
        alias = "inconsolata",
        alias = "space-mono"
    )]
    SystemDefault,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum TableBorderStyle {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "inner")]
    Inner,
    #[serde(rename = "all")]
    All,
    #[serde(rename = "vertical")]
    Vertical,
    #[serde(rename = "horizontal")]
    Horizontal,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum RenderMode {
    #[serde(rename = "styledHtml")]
    StyledHtml,
    #[serde(rename = "semanticHtml")]
    SemanticHtml,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum Severity {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum DiagnosticEngine {
    #[serde(rename = "core")]
    Core,
    #[serde(rename = "bridge")]
    Bridge,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, uniffi::Enum)]
pub enum AssetKind {
    #[serde(rename = "svg")]
    Svg,
    #[serde(rename = "css")]
    Css,
    #[serde(rename = "other")]
    Other,
}
