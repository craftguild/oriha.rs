//! Syntax-theme selection and named token color palettes.
use super::*;

pub(super) fn syntax_tokens(name: &str, variant: ThemeVariant) -> SyntaxTokens {
    let normalized = normalize_syntax_theme_name(name);
    match normalized.as_str() {
        "github-dark" => github_dark_syntax(),
        "github-light" => github_light_syntax(),
        "solarized-light" => solarized_light_syntax(),
        "solarized-dark" => solarized_dark_syntax(),
        "gruvbox-light" => gruvbox_light_syntax(),
        "gruvbox-dark" => gruvbox_dark_syntax(),
        "catppuccin-latte" => catppuccin_latte_syntax(),
        "catppuccin-mocha" => catppuccin_mocha_syntax(),
        "ayu-light" => ayu_light_syntax(),
        "ayu-dark" => ayu_dark_syntax(),
        "tokyo-night-day" => tokyo_night_day_syntax(),
        "tokyo-night" => tokyo_night_syntax(),
        "rose-pine-dawn" | "rosé-pine-dawn" => rose_pine_dawn_syntax(),
        "rose-pine" | "rosé-pine" => rose_pine_syntax(),
        "high-contrast-light" => high_contrast_light_syntax(),
        "high-contrast-dark" => high_contrast_dark_syntax(),
        "monokai" => monokai_syntax(),
        "auto" | "github" => match variant {
            ThemeVariant::Light => github_light_syntax(),
            ThemeVariant::Dark => github_dark_syntax(),
        },
        "solarized" => match variant {
            ThemeVariant::Light => solarized_light_syntax(),
            ThemeVariant::Dark => solarized_dark_syntax(),
        },
        "gruvbox" => match variant {
            ThemeVariant::Light => gruvbox_light_syntax(),
            ThemeVariant::Dark => gruvbox_dark_syntax(),
        },
        "catppuccin" => match variant {
            ThemeVariant::Light => catppuccin_latte_syntax(),
            ThemeVariant::Dark => catppuccin_mocha_syntax(),
        },
        "ayu" => match variant {
            ThemeVariant::Light => ayu_light_syntax(),
            ThemeVariant::Dark => ayu_dark_syntax(),
        },
        "tokyo" => match variant {
            ThemeVariant::Light => tokyo_night_day_syntax(),
            ThemeVariant::Dark => tokyo_night_syntax(),
        },
        "high-contrast" => match variant {
            ThemeVariant::Light => high_contrast_light_syntax(),
            ThemeVariant::Dark => high_contrast_dark_syntax(),
        },
        _ => github_light_syntax(),
    }
}

pub(super) fn normalize_syntax_theme_name(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_whitespace() || ch == '_' {
                '-'
            } else {
                ch
            }
        })
        .collect()
}

pub(super) fn github_light_syntax() -> SyntaxTokens {
    syntax(
        "#f6f8fa", "#24292e", "#6a737d", "#d73a49", "#032f62", "#005cc5", "#6f42c1", "#e36209",
        "#d73a49", "#6f42c1", "#005cc5", "#005cc5", "#22863a", "#6f42c1", "#6f42c1", "#24292e",
    )
}

pub(super) fn github_dark_syntax() -> SyntaxTokens {
    syntax(
        "#161b22", "#c9d1d9", "#8b949e", "#ff7b72", "#a5d6ff", "#79c0ff", "#d2a8ff", "#ffa657",
        "#ff7b72", "#d2a8ff", "#79c0ff", "#79c0ff", "#7ee787", "#d2a8ff", "#d2a8ff", "#c9d1d9",
    )
}

pub(super) fn monokai_syntax() -> SyntaxTokens {
    syntax(
        "#272822", "#f8f8f2", "#75715e", "#f92672", "#e6db74", "#ae81ff", "#a6e22e", "#f8f8f2",
        "#f92672", "#a6e22e", "#ae81ff", "#66d9ef", "#f92672", "#a6e22e", "#f92672", "#f8f8f2",
    )
}

pub(super) fn solarized_light_syntax() -> SyntaxTokens {
    syntax(
        "#fdf6e3", "#657b83", "#93a1a1", "#859900", "#2aa198", "#d33682", "#268bd2", "#b58900",
        "#859900", "#268bd2", "#d33682", "#dc322f", "#268bd2", "#b58900", "#b58900", "#657b83",
    )
}

pub(super) fn solarized_dark_syntax() -> SyntaxTokens {
    syntax(
        "#002b36", "#839496", "#586e75", "#859900", "#2aa198", "#d33682", "#268bd2", "#b58900",
        "#859900", "#268bd2", "#d33682", "#dc322f", "#268bd2", "#b58900", "#b58900", "#839496",
    )
}

pub(super) fn gruvbox_light_syntax() -> SyntaxTokens {
    syntax(
        "#fbf1c7", "#3c3836", "#928374", "#9d0006", "#79740e", "#8f3f71", "#b57614", "#076678",
        "#af3a03", "#8f3f71", "#8f3f71", "#427b58", "#9d0006", "#b57614", "#076678", "#3c3836",
    )
}

pub(super) fn gruvbox_dark_syntax() -> SyntaxTokens {
    syntax(
        "#282828", "#ebdbb2", "#928374", "#fb4934", "#b8bb26", "#d3869b", "#fabd2f", "#83a598",
        "#fe8019", "#d3869b", "#d3869b", "#8ec07c", "#fb4934", "#fabd2f", "#83a598", "#ebdbb2",
    )
}

pub(super) fn catppuccin_latte_syntax() -> SyntaxTokens {
    syntax(
        "#eff1f5", "#4c4f69", "#8c8fa1", "#8839ef", "#40a02b", "#fe640b", "#1e66f5", "#df8e1d",
        "#04a5e5", "#df8e1d", "#fe640b", "#179299", "#d20f39", "#1e66f5", "#8839ef", "#4c4f69",
    )
}

pub(super) fn catppuccin_mocha_syntax() -> SyntaxTokens {
    syntax(
        "#1e1e2e", "#cdd6f4", "#6c7086", "#cba6f7", "#a6e3a1", "#fab387", "#89b4fa", "#f9e2af",
        "#89dceb", "#f9e2af", "#fab387", "#94e2d5", "#f38ba8", "#89b4fa", "#cba6f7", "#cdd6f4",
    )
}

pub(super) fn ayu_light_syntax() -> SyntaxTokens {
    syntax(
        "#fafafa", "#5c6166", "#abb0b6", "#fa8d3e", "#86b300", "#a37acc", "#f2ae49", "#55b4d4",
        "#ed9366", "#399ee6", "#a37acc", "#4cbf99", "#f07171", "#f2ae49", "#399ee6", "#5c6166",
    )
}

pub(super) fn ayu_dark_syntax() -> SyntaxTokens {
    syntax(
        "#0f1419", "#b3b1ad", "#5c6773", "#ff8f40", "#b8cc52", "#d2a6ff", "#ffb454", "#59c2ff",
        "#f29668", "#73d0ff", "#d2a6ff", "#95e6cb", "#ff3333", "#ffb454", "#73d0ff", "#b3b1ad",
    )
}

pub(super) fn tokyo_night_day_syntax() -> SyntaxTokens {
    syntax(
        "#e1e2e7", "#3760bf", "#848cb5", "#9854f1", "#587539", "#965027", "#2e7de9", "#b15c00",
        "#007197", "#2e7de9", "#965027", "#007197", "#8c4351", "#2e7de9", "#9854f1", "#3760bf",
    )
}

pub(super) fn tokyo_night_syntax() -> SyntaxTokens {
    syntax(
        "#1a1b26", "#c0caf5", "#565f89", "#bb9af7", "#9ece6a", "#ff9e64", "#7aa2f7", "#e0af68",
        "#89ddff", "#7aa2f7", "#ff9e64", "#2ac3de", "#f7768e", "#7aa2f7", "#bb9af7", "#c0caf5",
    )
}

pub(super) fn rose_pine_dawn_syntax() -> SyntaxTokens {
    syntax(
        "#faf4ed", "#575279", "#9893a5", "#907aa9", "#56949f", "#d7827e", "#286983", "#ea9d34",
        "#56949f", "#286983", "#d7827e", "#56949f", "#b4637a", "#286983", "#907aa9", "#575279",
    )
}

pub(super) fn rose_pine_syntax() -> SyntaxTokens {
    syntax(
        "#191724", "#e0def4", "#6e6a86", "#c4a7e7", "#9ccfd8", "#ebbcba", "#31748f", "#f6c177",
        "#9ccfd8", "#31748f", "#ebbcba", "#9ccfd8", "#eb6f92", "#31748f", "#c4a7e7", "#e0def4",
    )
}

pub(super) fn high_contrast_light_syntax() -> SyntaxTokens {
    syntax(
        "#ffffff", "#0a0a0a", "#545454", "#0000cc", "#006600", "#7a1fa2", "#0044cc", "#994500",
        "#000000", "#0044cc", "#7a1fa2", "#005f5f", "#b00020", "#0044cc", "#0000cc", "#0a0a0a",
    )
}

pub(super) fn high_contrast_dark_syntax() -> SyntaxTokens {
    syntax(
        "#000000", "#ffffff", "#b3b3b3", "#79c0ff", "#7ee787", "#ffa657", "#d2a8ff", "#ffdf5d",
        "#ffffff", "#d2a8ff", "#ffa657", "#56d4dd", "#ff7b72", "#79c0ff", "#d2a8ff", "#ffffff",
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn syntax(
    background: &str,
    base: &str,
    comment: &str,
    keyword: &str,
    string: &str,
    number: &str,
    function: &str,
    variable: &str,
    operator: &str,
    class_name: &str,
    constant: &str,
    builtin: &str,
    tag: &str,
    attribute: &str,
    namespace: &str,
    punctuation: &str,
) -> SyntaxTokens {
    SyntaxTokens {
        background: background.to_string(),
        base: base.to_string(),
        comment: comment.to_string(),
        keyword: keyword.to_string(),
        string: string.to_string(),
        number: number.to_string(),
        function: function.to_string(),
        variable: variable.to_string(),
        operator: operator.to_string(),
        class_name: class_name.to_string(),
        constant: constant.to_string(),
        builtin: builtin.to_string(),
        tag: tag.to_string(),
        attribute: attribute.to_string(),
        namespace: namespace.to_string(),
        punctuation: punctuation.to_string(),
    }
}
