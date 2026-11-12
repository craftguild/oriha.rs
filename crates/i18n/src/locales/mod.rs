pub(crate) mod en;
pub(crate) mod es;
pub(crate) mod fr;
pub(crate) mod ja;
pub(crate) mod ko;
pub(crate) mod zh_hans;
pub(crate) mod zh_hant;

use crate::SyntaxThemeMessages;

pub(crate) const SYNTAX_THEME_NAMES: SyntaxThemeMessages = SyntaxThemeMessages {
    github_light: "GitHub Light",
    github_dark: "GitHub Dark",
    solarized_light: "Solarized Light",
    solarized_dark: "Solarized Dark",
    gruvbox_light: "Gruvbox Light",
    gruvbox_dark: "Gruvbox Dark",
    catppuccin_latte: "Catppuccin Latte",
    catppuccin_mocha: "Catppuccin Mocha",
    ayu_light: "Ayu Light",
    ayu_dark: "Ayu Dark",
    tokyo_night_day: "Tokyo Night Day",
    tokyo_night: "Tokyo Night",
    rose_pine_dawn: "Rosé Pine Dawn",
    rose_pine: "Rosé Pine",
    high_contrast_light: "High Contrast Light",
    high_contrast_dark: "High Contrast Dark",
};
