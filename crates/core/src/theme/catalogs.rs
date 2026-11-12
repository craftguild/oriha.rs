//! Public syntax-theme and font catalogs with stable option identities.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyntaxTheme {
    pub id: &'static str,
    pub label: &'static str,
    pub family: &'static str,
    pub color_scheme: ColorScheme,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontOption {
    pub id: &'static str,
    pub label: &'static str,
    pub family: &'static str,
    pub fallbacks: &'static [&'static str],
}

impl FontOption {
    pub fn font_family_spec(self) -> FontFamilySpec {
        FontFamilySpec {
            family: self.family.to_string(),
            fallbacks: self
                .fallbacks
                .iter()
                .map(|fallback| (*fallback).to_string())
                .collect(),
        }
    }
}

pub(super) const SYSTEM_SANS_FALLBACKS: &[&str] = &[
    "BlinkMacSystemFont",
    "\"Segoe UI\"",
    "Helvetica",
    "Arial",
    "sans-serif",
];

pub(super) const SYSTEM_MONO_FALLBACKS: &[&str] = &[
    "Consolas",
    "\"Liberation Mono\"",
    "Menlo",
    "Monaco",
    "\"Courier New\"",
    "monospace",
];

pub(super) const SYNTAX_THEME_CATALOG: &[SyntaxTheme] = &[
    syntax_theme("github-light", "GitHub Light", "GitHub", ColorScheme::Light),
    syntax_theme("github-dark", "GitHub Dark", "GitHub", ColorScheme::Dark),
    syntax_theme(
        "solarized-light",
        "Solarized Light",
        "Solarized",
        ColorScheme::Light,
    ),
    syntax_theme(
        "solarized-dark",
        "Solarized Dark",
        "Solarized",
        ColorScheme::Dark,
    ),
    syntax_theme(
        "gruvbox-light",
        "Gruvbox Light",
        "Gruvbox",
        ColorScheme::Light,
    ),
    syntax_theme("gruvbox-dark", "Gruvbox Dark", "Gruvbox", ColorScheme::Dark),
    syntax_theme(
        "catppuccin-latte",
        "Catppuccin Latte",
        "Catppuccin",
        ColorScheme::Light,
    ),
    syntax_theme(
        "catppuccin-mocha",
        "Catppuccin Mocha",
        "Catppuccin",
        ColorScheme::Dark,
    ),
    syntax_theme("ayu-light", "Ayu Light", "Ayu", ColorScheme::Light),
    syntax_theme("ayu-dark", "Ayu Dark", "Ayu", ColorScheme::Dark),
    syntax_theme(
        "tokyo-night-day",
        "Tokyo Night Day",
        "Tokyo Night",
        ColorScheme::Light,
    ),
    syntax_theme(
        "tokyo-night",
        "Tokyo Night",
        "Tokyo Night",
        ColorScheme::Dark,
    ),
    syntax_theme(
        "rose-pine-dawn",
        "Rosé Pine Dawn",
        "Rosé Pine",
        ColorScheme::Light,
    ),
    syntax_theme("rose-pine", "Rosé Pine", "Rosé Pine", ColorScheme::Dark),
    syntax_theme(
        "high-contrast-light",
        "High Contrast Light",
        "High Contrast",
        ColorScheme::Light,
    ),
    syntax_theme(
        "high-contrast-dark",
        "High Contrast Dark",
        "High Contrast",
        ColorScheme::Dark,
    ),
];

pub(super) const DOCUMENT_FONT_CATALOG: &[FontOption] = &[
    font_option(
        "system-default",
        "System Default",
        "-apple-system",
        SYSTEM_SANS_FALLBACKS,
    ),
    font_option("noto-sans", "Noto Sans", "\"Noto Sans\"", &["sans-serif"]),
    font_option(
        "inter",
        "Inter",
        "Inter",
        &["-apple-system", "BlinkMacSystemFont", "sans-serif"],
    ),
    font_option(
        "source-sans-3",
        "Source Sans 3",
        "\"Source Sans 3\"",
        &["sans-serif"],
    ),
    font_option("roboto", "Roboto", "Roboto", &["sans-serif"]),
    font_option("open-sans", "Open Sans", "\"Open Sans\"", &["sans-serif"]),
    font_option("lato", "Lato", "Lato", &["sans-serif"]),
    font_option("montserrat", "Montserrat", "Montserrat", &["sans-serif"]),
    font_option("poppins", "Poppins", "Poppins", &["sans-serif"]),
    font_option("roboto-slab", "Roboto Slab", "\"Roboto Slab\"", &["serif"]),
    font_option(
        "playfair-display",
        "Playfair Display",
        "\"Playfair Display\"",
        &["serif"],
    ),
    font_option("oswald", "Oswald", "Oswald", &["sans-serif"]),
    font_option("merriweather", "Merriweather", "Merriweather", &["serif"]),
    font_option(
        "source-serif-4",
        "Source Serif 4",
        "\"Source Serif 4\"",
        &["serif"],
    ),
    font_option(
        "libre-baskerville",
        "Libre Baskerville",
        "\"Libre Baskerville\"",
        &["serif"],
    ),
];

pub(super) const CODE_FONT_CATALOG: &[FontOption] = &[
    font_option(
        "system-default",
        "System Default",
        "\"SFMono-Regular\"",
        SYSTEM_MONO_FALLBACKS,
    ),
    font_option(
        "jetbrains-mono",
        "JetBrains Mono",
        "\"JetBrains Mono\"",
        &["\"SFMono-Regular\"", "Consolas", "monospace"],
    ),
    font_option(
        "roboto-mono",
        "Roboto Mono",
        "\"Roboto Mono\"",
        &["\"SFMono-Regular\"", "Consolas", "monospace"],
    ),
    font_option(
        "ibm-plex-mono",
        "IBM Plex Mono",
        "\"IBM Plex Mono\"",
        &["\"SFMono-Regular\"", "Consolas", "monospace"],
    ),
    font_option(
        "source-code-pro",
        "Source Code Pro",
        "\"Source Code Pro\"",
        &["\"SFMono-Regular\"", "Consolas", "monospace"],
    ),
    font_option(
        "noto-sans-mono",
        "Noto Sans Mono",
        "\"Noto Sans Mono\"",
        &["\"SFMono-Regular\"", "Consolas", "monospace"],
    ),
    font_option(
        "inconsolata",
        "Inconsolata",
        "Inconsolata",
        &["\"SFMono-Regular\"", "Consolas", "monospace"],
    ),
    font_option(
        "space-mono",
        "Space Mono",
        "\"Space Mono\"",
        &["\"SFMono-Regular\"", "Consolas", "monospace"],
    ),
];

pub(super) const fn syntax_theme(
    id: &'static str,
    label: &'static str,
    family: &'static str,
    color_scheme: ColorScheme,
) -> SyntaxTheme {
    SyntaxTheme {
        id,
        label,
        family,
        color_scheme,
    }
}

pub(super) const fn font_option(
    id: &'static str,
    label: &'static str,
    family: &'static str,
    fallbacks: &'static [&'static str],
) -> FontOption {
    FontOption {
        id,
        label,
        family,
        fallbacks,
    }
}

pub fn syntax_theme_catalog() -> &'static [SyntaxTheme] {
    SYNTAX_THEME_CATALOG
}

pub fn heading_font_catalog() -> &'static [FontOption] {
    DOCUMENT_FONT_CATALOG
}

pub fn body_font_catalog() -> &'static [FontOption] {
    DOCUMENT_FONT_CATALOG
}

pub fn code_font_catalog() -> &'static [FontOption] {
    CODE_FONT_CATALOG
}
