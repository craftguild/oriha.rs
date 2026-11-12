//! Resolved theme model and byte-preserving CSS template assembly.
mod catalogs;
#[allow(unused_imports)]
use catalogs::*;
pub use catalogs::{
    body_font_catalog, code_font_catalog, heading_font_catalog, syntax_theme_catalog, FontOption,
    SyntaxTheme,
};
mod palettes;
#[allow(unused_imports)]
use palettes::*;
mod syntax_palettes;
#[allow(unused_imports)]
use syntax_palettes::*;
mod css_layout;
#[allow(unused_imports)]
use css_layout::*;
mod fonts;
pub(crate) use fonts::resolve_body_font_size_px;
#[allow(unused_imports)]
use fonts::*;
mod variables;
#[allow(unused_imports)]
use variables::*;

use crate::types::{
    AppearanceOptions, ColorScheme, FontFamilySpec, PrintOptions, TableBorderStyle,
};
use std::collections::BTreeMap;

const DEFAULT_BODY_FONT_SIZE_PX: f32 = 16.0;
const MIN_BODY_FONT_SIZE_PX: f32 = 12.0;
const MAX_BODY_FONT_SIZE_PX: f32 = 24.0;
const CODE_LABEL_FONT_SIZE_RATIO: f32 = 13.0 / DEFAULT_BODY_FONT_SIZE_PX;
const CODE_BLOCK_FONT_SIZE_RATIO: f32 = 14.0 / DEFAULT_BODY_FONT_SIZE_PX;
const WATERMARK_FONT_SIZE_RATIO: f32 = 11.0 / DEFAULT_BODY_FONT_SIZE_PX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemeStyle {
    Rich,
    Simple,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemeVariant {
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedTheme {
    pub name: String,
    pub style: ThemeStyle,
    pub variant: ThemeVariant,
    pub table_border_style: TableBorderStyle,
    pub print: Option<PrintOptions>,
    pub full_table_data: bool,
    pub block_borders: bool,
    pub tokens: ThemeTokens,
    css_variables: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ThemeTokens {
    pub surface: SurfaceTokens,
    pub text: TextTokens,
    pub border: BorderTokens,
    pub table: TableTokens,
    pub code: CodeTokens,
    pub callout: CalloutTokens,
    pub diagram: DiagramTokens,
    pub task: TaskTokens,
    pub watermark: WatermarkTokens,
    pub layout: LayoutTokens,
    pub syntax: SyntaxTokens,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SurfaceTokens {
    pub base: String,
    pub subtle: String,
    pub elevated: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextTokens {
    pub body: String,
    pub heading: String,
    pub muted: String,
    pub link: String,
    pub link_hover: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BorderTokens {
    pub default: String,
    pub strong: String,
    pub blockquote: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TableTokens {
    pub background: String,
    pub header: String,
    pub alternate: String,
    pub border: String,
    pub shadow: String,
    pub radius: String,
    pub margin: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CodeTokens {
    pub block_background: String,
    pub header_background: String,
    pub gutter_background: String,
    pub inline_background: String,
    pub text: String,
    pub line_number: String,
    pub border: String,
    pub shadow: String,
    pub radius: String,
    pub inline_radius: String,
    pub padding: String,
    pub copy_text: String,
    pub copy_background_hover: String,
    pub copy_success_text: String,
    pub copy_success_background: String,
    pub copy_failure_text: String,
    pub copy_failure_background: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CalloutTokens {
    pub generic: CalloutKindTokens,
    pub note: CalloutKindTokens,
    pub tip: CalloutKindTokens,
    pub important: CalloutKindTokens,
    pub warning: CalloutKindTokens,
    pub caution: CalloutKindTokens,
    pub content_background: String,
    pub shadow: String,
    pub radius: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CalloutKindTokens {
    pub background: String,
    pub header: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DiagramTokens {
    pub background: String,
    pub border: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TaskTokens {
    pub checkbox_accent: String,
    pub checkbox_check: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WatermarkTokens {
    pub text: String,
    pub opacity: String,
    pub qr_opacity: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LayoutTokens {
    pub content_padding: String,
    pub content_max_width: String,
    pub line_height: String,
    pub paragraph_margin_top: String,
    pub paragraph_margin_bottom: String,
    pub heading_margin_top: String,
    pub heading_margin_bottom: String,
    pub heading_border: String,
    pub heading_padding_bottom: String,
    pub content_radius: String,
    pub blockquote_border_width: String,
    pub blockquote_padding_left: String,
    pub blockquote_margin_left: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SyntaxTokens {
    pub background: String,
    pub base: String,
    pub comment: String,
    pub keyword: String,
    pub string: String,
    pub number: String,
    pub function: String,
    pub variable: String,
    pub operator: String,
    pub class_name: String,
    pub constant: String,
    pub builtin: String,
    pub tag: String,
    pub attribute: String,
    pub namespace: String,
    pub punctuation: String,
}

pub(crate) fn resolve_theme(options: &AppearanceOptions) -> ResolvedTheme {
    let style = parse_theme_style(&options.style);
    let variant = match options.color_scheme {
        ColorScheme::Light => ThemeVariant::Light,
        ColorScheme::Dark => ThemeVariant::Dark,
    };
    let tokens = ThemeTokens {
        surface: surface_tokens(variant),
        text: text_tokens(variant),
        border: border_tokens(variant),
        table: table_tokens(variant, style),
        code: code_tokens(variant, style),
        callout: callout_tokens(variant, style),
        diagram: diagram_tokens(variant),
        task: task_tokens(variant),
        watermark: watermark_tokens(variant),
        layout: layout_tokens(style),
        syntax: syntax_tokens(&options.syntax_theme, variant),
    };

    let mut css_variables = css_variables_from_tokens(&tokens);
    css_variables.insert(
        "--oriha-font-heading".to_string(),
        sanitize_font_stack(
            &options.heading_font_family.css_stack(),
            "-apple-system, BlinkMacSystemFont, \"Segoe UI\", Helvetica, Arial, sans-serif",
        ),
    );
    css_variables.insert(
        "--oriha-font-body".to_string(),
        sanitize_font_stack(
            &options.body_font_family.css_stack(),
            "-apple-system, BlinkMacSystemFont, \"Segoe UI\", Helvetica, Arial, sans-serif",
        ),
    );
    insert_font_size_variables(&mut css_variables, options);
    css_variables.insert(
        "--oriha-font-mono".to_string(),
        sanitize_font_stack(
            &options.code_font_family.css_stack(),
            "\"SFMono-Regular\", Consolas, \"Liberation Mono\", Menlo, monospace",
        ),
    );
    if !options.rounded_corners {
        disable_rounded_corner_variables(&mut css_variables);
    }
    if !options.shadows {
        disable_shadow_variables(&mut css_variables);
    }

    ResolvedTheme {
        name: resolved_theme_name(style, variant),
        style,
        variant,
        table_border_style: options.table_border_style,
        print: options.print.clone(),
        full_table_data: options.full_table_data,
        block_borders: options.block_borders,
        tokens,
        css_variables,
    }
}

pub(crate) fn render_css(theme: &ResolvedTheme) -> String {
    let variables = theme
        .css_variables
        .iter()
        .map(|(key, value)| format!("  {key}: {value};"))
        .collect::<Vec<_>>()
        .join("\n");

    let table_border_css = table_border_css(theme.table_border_style);
    let table_wrapper_border_css = table_wrapper_border_css(theme.table_border_style);
    let full_table_data_css = full_table_data_css(theme.full_table_data);
    let block_borders_css = block_borders_css(theme.block_borders);

    let print_css = theme
        .print
        .as_ref()
        .map(render_print_css)
        .unwrap_or_default();

    format!(
        concat!(
            include_str!("theme/styles/document.css"),
            include_str!("theme/styles/code-callouts.css"),
            include_str!("theme/styles/diagrams-watermark.css"),
            include_str!("theme/styles/syntax.css"),
            "{block_borders_css}\n{print_css}\n"
        ),
        block_borders_css = block_borders_css,
        full_table_data_css = full_table_data_css,
        print_css = print_css,
        table_border_css = table_border_css,
        table_wrapper_border_css = table_wrapper_border_css,
        variables = variables
    )
}

fn disable_rounded_corner_variables(css_variables: &mut BTreeMap<String, String>) {
    for name in [
        "--oriha-content-radius",
        "--oriha-table-radius",
        "--oriha-code-radius",
        "--oriha-code-inline-radius",
        "--oriha-callout-radius",
    ] {
        css_variables.insert(name.to_string(), "0".to_string());
    }
}

fn disable_shadow_variables(css_variables: &mut BTreeMap<String, String>) {
    for name in ["--oriha-table-shadow", "--oriha-callout-shadow"] {
        css_variables.insert(name.to_string(), "none".to_string());
    }
}

fn resolved_theme_name(style: ThemeStyle, variant: ThemeVariant) -> String {
    format!(
        "{}-{}",
        match style {
            ThemeStyle::Rich => "rich",
            ThemeStyle::Simple => "simple",
        },
        match variant {
            ThemeVariant::Light => "light",
            ThemeVariant::Dark => "dark",
        }
    )
}

fn parse_theme_style(style: &str) -> ThemeStyle {
    if style.eq_ignore_ascii_case("simple") {
        ThemeStyle::Simple
    } else {
        ThemeStyle::Rich
    }
}

fn style_shadow(style: ThemeStyle, variant: ThemeVariant) -> String {
    match style {
        ThemeStyle::Simple => "none".to_string(),
        ThemeStyle::Rich => match variant {
            ThemeVariant::Light => "0 1px 4px rgba(0, 0, 0, 0.12)",
            ThemeVariant::Dark => "0 1px 4px rgba(0, 0, 0, 0.4)",
        }
        .to_string(),
    }
}

fn style_radius(style: ThemeStyle) -> String {
    match style {
        ThemeStyle::Rich => "6px",
        ThemeStyle::Simple => "0",
    }
    .to_string()
}

fn shadow_color(shadow: &str) -> &str {
    if shadow.contains("0.4") {
        "rgba(0, 0, 0, 0.4)"
    } else if shadow == "none" {
        "transparent"
    } else {
        "rgba(0, 0, 0, 0.12)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RenderInput;

    #[test]
    fn resolves_builtin_rich_dark_theme() {
        let theme = resolve_theme(&AppearanceOptions {
            color_scheme: ColorScheme::Dark,
            style: "rich".to_string(),
            syntax_theme: "github-dark".to_string(),
            ..AppearanceOptions::default()
        });

        assert_eq!(theme.style, ThemeStyle::Rich);
        assert_eq!(theme.variant, ThemeVariant::Dark);
        assert_eq!(theme.tokens.surface.base, "#0d1117");
        assert_eq!(theme.tokens.layout.content_padding, "32px");
        assert_eq!(theme.tokens.callout.tip.text, "#86efac");
        assert_eq!(
            theme.tokens.table.background,
            theme.tokens.code.block_background
        );
        assert_eq!(
            theme.tokens.table.header,
            theme.tokens.code.header_background
        );
        assert_eq!(theme.tokens.table.border, theme.tokens.code.border);
    }

    #[test]
    fn appearance_options_select_style_and_variant() {
        let theme = resolve_theme(&AppearanceOptions {
            color_scheme: ColorScheme::Dark,
            style: "simple".to_string(),
            ..AppearanceOptions::default()
        });

        assert_eq!(theme.name, "simple-dark");
        assert_eq!(theme.style, ThemeStyle::Simple);
        assert_eq!(theme.variant, ThemeVariant::Dark);
        assert_eq!(theme.tokens.layout.content_padding, "16px");
    }

    #[test]
    fn default_syntax_theme_is_github_light() {
        let theme = resolve_theme(&AppearanceOptions {
            color_scheme: ColorScheme::Dark,
            ..AppearanceOptions::default()
        });

        assert_eq!(theme.tokens.syntax.background, "#f6f8fa");
        assert_eq!(theme.tokens.syntax.base, "#24292e");
    }

    #[test]
    fn syntax_theme_catalog_resolves_requested_pairs() {
        let cases = [
            ("Solarized Light", "#fdf6e3"),
            ("Solarized Dark", "#002b36"),
            ("GitHub Light", "#f6f8fa"),
            ("GitHub Dark", "#161b22"),
            ("Gruvbox Light", "#fbf1c7"),
            ("Gruvbox Dark", "#282828"),
            ("Catppuccin Latte", "#eff1f5"),
            ("Catppuccin Mocha", "#1e1e2e"),
            ("Ayu Light", "#fafafa"),
            ("Ayu Dark", "#0f1419"),
            ("Tokyo Night Day", "#e1e2e7"),
            ("Tokyo Night", "#1a1b26"),
            ("Rose Pine Dawn", "#faf4ed"),
            ("Rose Pine", "#191724"),
            ("High Contrast Light", "#ffffff"),
            ("High Contrast Dark", "#000000"),
        ];

        for (name, expected_background) in cases {
            assert_eq!(
                syntax_tokens(name, ThemeVariant::Light).background,
                expected_background
            );
        }
    }

    #[test]
    fn css_uses_appearance_fonts_and_table_border_style() {
        let theme = resolve_theme(&AppearanceOptions {
            table_border_style: TableBorderStyle::None,
            ..AppearanceOptions::default()
        });
        let css = render_css(&theme);

        assert!(css.contains("--oriha-font-heading: -apple-system"));
        assert!(css.contains("--oriha-font-body: -apple-system"));
        assert!(css.contains("--oriha-font-size-body: 16px;"));
        assert!(css
            .contains("--oriha-font-size-code-label: calc(var(--oriha-font-size-body) * 0.8125);"));
        assert!(css
            .contains("--oriha-font-size-code-block: calc(var(--oriha-font-size-body) * 0.875);"));
        assert!(
            css.contains("--oriha-font-size-small: calc(var(--oriha-font-size-body) * 0.8125);")
        );
        assert!(css.contains("font-family: var(--oriha-font-heading);"));
        assert!(css.contains("font-size: var(--oriha-font-size-body);"));
        assert!(css.contains("border: 0;"));
    }

    #[test]
    fn css_uses_configured_body_font_size_and_relative_heading_sizes() {
        let theme = resolve_theme(&AppearanceOptions {
            body_font_size_px: Some(20.0),
            ..AppearanceOptions::default()
        });
        let css = render_css(&theme);

        assert!(css.contains("--oriha-font-size-body: 20px;"));
        assert!(css.contains(".oriha-document {\n  color: var(--oriha-color-text);\n  box-sizing: border-box;\n  font-family: var(--oriha-font-body);\n  font-size: var(--oriha-font-size-body);"));
        assert!(css.contains(".oriha-heading--h1 {\n  font-size: 2em;\n}"));
        assert!(css.contains(".oriha-heading--h2 {\n  font-size: 1.5em;\n}"));
        assert!(css.contains(".oriha-heading--h6 {\n  font-size: 0.875em;\n}"));
    }

    #[test]
    fn code_related_font_sizes_follow_body_font_size_variables() {
        let css = render_css(&resolve_theme(&AppearanceOptions {
            body_font_size_px: Some(20.0),
            ..AppearanceOptions::default()
        }));

        assert!(css.contains("--oriha-font-size-body: 20px;"));
        assert!(css.contains(
            ".oriha-code-block-language {\n  font-size: var(--oriha-font-size-code-label);"
        ));
        assert!(css.contains(".oriha-code-copy-button {\n  appearance: none;\n  align-items: center;\n  background: transparent;\n  border: 0;\n  border-radius: 6px;\n  color: var(--oriha-code-copy-text);\n  cursor: pointer;\n  display: inline-flex;\n  flex: 0 0 auto;\n  gap: 7px;\n  font: inherit;\n  font-size: var(--oriha-font-size-code-label);"));
        assert!(css.contains(".oriha-code-block {\n  background: transparent;\n  color: var(--oriha-syntax-base);\n  font-size: var(--oriha-font-size-code-block);"));
        assert!(
            css.contains(".oriha-code-line-number {\n  display: block;\n  line-height: inherit;")
        );
        assert!(!css.contains("font-size: 13px;"));
        assert!(!css.contains("font-size: 14px;"));
    }

    #[test]
    fn body_font_size_is_clamped_and_invalid_values_fall_back() {
        let too_small_css = render_css(&resolve_theme(&AppearanceOptions {
            body_font_size_px: Some(8.0),
            ..AppearanceOptions::default()
        }));
        let too_large_css = render_css(&resolve_theme(&AppearanceOptions {
            body_font_size_px: Some(30.0),
            ..AppearanceOptions::default()
        }));
        let invalid_css = render_css(&resolve_theme(&AppearanceOptions {
            body_font_size_px: Some(f32::NAN),
            ..AppearanceOptions::default()
        }));

        assert!(too_small_css.contains("--oriha-font-size-body: 12px;"));
        assert!(too_large_css.contains("--oriha-font-size-body: 24px;"));
        assert!(invalid_css.contains("--oriha-font-size-body: 16px;"));
    }

    #[test]
    fn document_shell_stays_transparent_without_outer_shadow() {
        let css = render_css(&resolve_theme(&AppearanceOptions {
            color_scheme: ColorScheme::Dark,
            ..AppearanceOptions::default()
        }));
        let document_rule = css
            .split(".oriha-document {")
            .nth(1)
            .and_then(|body| body.split("}").next())
            .expect("document rule should be present");

        assert!(!document_rule.contains("background:"));
        assert!(!document_rule.contains("box-shadow:"));
        assert!(document_rule.contains("box-sizing: border-box;"));
    }

    #[test]
    fn css_keeps_pdf_edge_padding_and_list_marker_space() {
        let css = render_css(&resolve_theme(&AppearanceOptions {
            print: Some(PrintOptions {
                page_size: Some("A4 portrait".to_string()),
                margin: Some("12mm".to_string()),
            }),
            ..AppearanceOptions::default()
        }));

        assert!(css.contains("padding: 0 5px;"));
        assert!(css.contains(".oriha-list {\n  margin-top: 0;\n  margin-left: 0;"));
        assert!(css.contains("padding-left: 1.25em;"));
        assert!(css.contains("ol.oriha-list {\n  padding-left: 1.65em;\n}"));
    }

    #[test]
    fn latex_math_block_does_not_use_shadow() {
        let css = render_css(&resolve_theme(&AppearanceOptions::default()));
        let math_block_rule = css
            .split(".oriha-math-block {")
            .nth(1)
            .and_then(|body| body.split("}").next())
            .expect("math block rule should be present");

        assert!(math_block_rule.contains("background: var(--oriha-table-bg);"));
        assert!(!math_block_rule.contains("box-shadow:"));
    }

    #[test]
    fn rendering_options_control_corners_shadows_tables_and_block_borders() {
        let default_css = render_css(&resolve_theme(&AppearanceOptions::default()));
        assert!(default_css.contains("--oriha-content-radius: "));
        assert!(!default_css.contains("--oriha-content-radius: 0;"));
        assert!(!default_css.contains("overflow-wrap: anywhere;\n  white-space: normal;"));
        assert!(!default_css.contains(".oriha-code-block-wrapper,\n.oriha-callout,"));

        let css = render_css(&resolve_theme(&AppearanceOptions {
            rounded_corners: false,
            shadows: false,
            full_table_data: true,
            block_borders: true,
            ..AppearanceOptions::default()
        }));

        assert!(css.contains("--oriha-content-radius: 0;"));
        assert!(css.contains("--oriha-table-radius: 0;"));
        assert!(css.contains("--oriha-code-radius: 0;"));
        assert!(css.contains("--oriha-table-shadow: none;"));
        assert!(css.contains("overflow-wrap: anywhere;"));
        assert!(css.contains("table-layout: fixed;"));
        assert!(css.contains("width: 100%;"));
        assert!(css.contains("min-width: 0;"));
        assert!(css.contains("word-break: break-all;"));
        assert!(
            css.rfind("white-space: normal;")
                .expect("full table data override should set normal whitespace")
                > css
                    .find("white-space: nowrap;")
                    .expect("base table cells should set nowrap")
        );
        assert!(css.contains(".oriha-code-block-wrapper,\n.oriha-callout,"));
        assert!(css.contains("border: 1px solid var(--oriha-color-border);"));
    }

    #[test]
    fn font_css_variables_target_heading_body_and_code() {
        let theme = resolve_theme(&AppearanceOptions {
            heading_font_family: FontFamilySpec {
                family: "\"Merriweather\"".to_string(),
                fallbacks: vec!["serif".to_string()],
            },
            body_font_family: FontFamilySpec {
                family: "\"Noto Sans\"".to_string(),
                fallbacks: vec!["sans-serif".to_string()],
            },
            code_font_family: FontFamilySpec {
                family: "\"JetBrains Mono\"".to_string(),
                fallbacks: vec!["monospace".to_string()],
            },
            ..AppearanceOptions::default()
        });
        let css = render_css(&theme);

        assert!(!css.contains("fonts.googleapis.com"));
        assert!(!css.contains("@import"));
        assert!(css.starts_with(":root {"));
        assert!(css.contains("--oriha-font-heading: \"Merriweather\", serif;"));
        assert!(css.contains("--oriha-font-body: \"Noto Sans\", sans-serif;"));
        assert!(css.contains("--oriha-font-mono: \"JetBrains Mono\", monospace;"));
        assert!(css.contains(".oriha-heading {\n  color: var(--oriha-color-heading);\n  font-family: var(--oriha-font-heading);"));
        assert!(css.contains(".oriha-document {\n  color: var(--oriha-color-text);\n  box-sizing: border-box;\n  font-family: var(--oriha-font-body);"));
        assert!(css.contains("font-family: var(--oriha-font-mono);"));
    }

    #[test]
    fn public_catalogs_include_representative_options() {
        assert_eq!(syntax_theme_catalog().len(), 16);
        assert_eq!(syntax_theme_catalog()[0].id, "github-light");
        assert!(syntax_theme_catalog()
            .iter()
            .any(|theme| theme.id == "tokyo-night"));
        assert!(syntax_theme_catalog()
            .iter()
            .any(|theme| theme.id == "high-contrast-dark"));

        assert_eq!(heading_font_catalog()[0].id, "system-default");
        assert!(heading_font_catalog()
            .iter()
            .any(|font| font.id == "playfair-display"));
        assert_eq!(body_font_catalog()[0].id, "system-default");
        assert!(body_font_catalog()
            .iter()
            .any(|font| font.id == "noto-sans"));
        assert_eq!(
            heading_font_catalog()
                .iter()
                .map(|font| font.id)
                .collect::<Vec<_>>(),
            body_font_catalog()
                .iter()
                .map(|font| font.id)
                .collect::<Vec<_>>()
        );
        assert_eq!(code_font_catalog()[0].id, "system-default");
        assert!(code_font_catalog()
            .iter()
            .any(|font| font.id == "jetbrains-mono"));
    }

    #[test]
    fn css_targets_current_oriha_selectors() {
        let theme = resolve_theme(&AppearanceOptions {
            style: "simple".to_string(),
            ..AppearanceOptions::default()
        });
        let css = render_css(&theme);

        assert!(css.contains(".oriha-document"));
        assert!(css.contains(".oriha-code-block-wrapper"));
        assert!(css.contains(".oriha-code-block-header"));
        assert!(css.contains(".oriha-code-copy-button"));
        assert!(css.contains(".oriha-code-copy-icon"));
        assert!(css.contains(".oriha-code-copy-label"));
        assert!(css.contains("[data-oriha-copy-state=\"copied\"]"));
        assert!(css.contains("background: var(--oriha-syntax-bg);"));
        assert!(css.contains("--oriha-code-header-bg"));
        assert!(css.contains("--oriha-code-gutter-bg"));
        assert!(css.contains("background: var(--oriha-code-gutter-bg);"));
        assert!(css.contains("color: var(--oriha-code-line-number);"));
        assert!(css.contains(".oriha-code-block"));
        assert!(css.contains(".oriha-code-gutter"));
        assert!(css.contains(".oriha-code-line-number"));
        assert!(css.contains(".oriha-code-scroll"));
        assert!(css.contains(".oriha-code-lines"));
        assert!(css.contains("min-height: 1.45em;"));
        assert!(css.contains("min-height: 1lh;"));
        assert!(css.contains(".oriha-task-list {\n  margin-left: 0;\n  padding-left: 0;\n}"));
        assert!(css.contains(
            ".oriha-task-list-item {\n  align-items: flex-start;\n  display: flex;\n  gap: 0.45em;"
        ));
        assert!(css.contains(".oriha-task-checkbox"));
        assert!(css.contains("margin: calc((1lh - 1em) / 2) 0 0;"));
        assert!(!css.contains(".oriha-task-list-item > input[type=\"checkbox\"]"));
        assert!(css.contains(".oriha-task-checkbox--checked"));
        assert!(css.contains("--oriha-task-checkbox-accent: #0a84ff;"));
        assert!(css.contains("background: var(--oriha-task-checkbox-accent);"));
        assert!(css.contains("border-color: var(--oriha-task-checkbox-accent);"));
        assert!(css.contains(".oriha-task-checkbox--checked::after"));
        assert!(css.contains("border-color: var(--oriha-task-checkbox-check);"));
        assert!(css.contains(".oriha-callout--important"));
        assert!(css.contains(".oriha-callout__icon svg"));
        assert!(css.contains(".oriha-callout__title-text"));
        assert!(!css.contains("border-left: 0.28rem"));
        assert!(css.contains("--oriha-diagram-bg"));
        assert!(css.contains("--oriha-watermark-text"));
        assert!(css.contains(".oriha-watermark {\n  align-items: center;\n  bottom: 16px;"));
        assert!(css.contains("left: 50%;"));
        assert!(css.contains("transform: translateX(-50%);"));
        assert!(css.contains(".oriha-watermark__image {\n  display: block;\n  height: 44px;"));
        assert!(css.contains("width: 44px;"));
        assert!(css.contains(".oriha-watermark__text"));
        assert!(css.contains(".oriha-callout + .oriha-callout"));
        assert!(css.contains("white-space: nowrap;"));
        assert!(css.contains(".oriha-table-scroll-wrapper"));
        assert!(css.contains(".oriha-table-scroll-wrapper .oriha-table"));
        assert!(css.contains(".oriha-document table:not(.oriha-table)"));
        assert!(!css.contains(".oriha-document > table:not(.oriha-table)"));
        assert!(css.contains("box-shadow: none;"));
        assert!(css.contains("display: table;"));
        assert!(css.contains("margin: 0;"));
        assert!(css.contains("line-height: 1.45;"));
        assert!(css.contains("color: var(--oriha-color-text);"));
        assert!(css.contains("height: 36px;"));
        assert!(css.contains("color: var(--oriha-color-heading);"));
        assert!(css.contains("min-height: 36px;"));
        assert!(css.contains("padding: 8px 13px;"));
        assert!(css.contains("padding: 0 13px;"));
        assert!(css.contains("margin: var(--oriha-table-margin);"));
        assert!(css.contains("box-shadow: var(--oriha-table-shadow);"));
        assert!(css.contains(
            ".oriha-plantuml,\n.oriha-mermaid {\n  background: var(--oriha-table-bg);\n  border: 0;\n  border-radius: var(--oriha-table-radius);\n  box-shadow: var(--oriha-table-shadow);\n}"
        ));
        assert!(css.contains(
            ".oriha-math-block {\n  background: var(--oriha-table-bg);\n  border: 0;\n  border-radius: var(--oriha-table-radius);\n}"
        ));
        assert!(css.contains(".oriha-plantuml > svg"));
        assert!(css.contains(".oriha-math-inline"));
        assert!(css.contains(".oriha-math-block > svg"));
        assert!(css.contains(".oriha-math--error"));
        assert!(css.contains(".oriha-math__error-title"));
        assert!(css.contains(".oriha-math__source"));
        assert!(css.contains(".oriha-mermaid > svg"));
        assert!(css.contains(".oriha-plantuml--error"));
        assert!(css.contains(".oriha-plantuml__error-title"));
        assert!(css.contains(".oriha-plantuml__source"));
        assert!(css.contains(".oriha-mermaid--error"));
        assert!(css.contains(".oriha-mermaid__error-title"));
        assert!(css.contains(".oriha-mermaid__source"));
        assert!(css.contains(".oriha-footnote-ref"));
        assert!(css.contains(".oriha-footnotes"));
        assert!(css.contains(".oriha-footnotes__backref"));
        assert!(css.contains(".oriha-image {"));
        assert!(css.contains(".oriha-image__img"));
        assert!(css.contains(".oriha-image__caption"));
        assert!(css.contains(".oriha-inline-image"));
        assert!(css.contains("background: var(--oriha-code-header-bg);"));
        assert!(css.contains(".oriha-document img:not(.oriha-watermark__image):not(.oriha-image__img):not(.oriha-inline-image)"));
        assert!(css.contains("overflow-x: auto;"));
        assert!(css.contains("max-width: none;"));
    }

    #[test]
    fn table_border_all_uses_wrapper_outer_border_and_inner_grid() {
        let theme = resolve_theme(&AppearanceOptions {
            table_border_style: TableBorderStyle::All,
            ..AppearanceOptions::default()
        });
        let css = render_css(&theme);

        assert!(css.contains(
            ".oriha-table-scroll-wrapper {\n  background: var(--oriha-table-bg);\n  border: 1px solid var(--oriha-table-border);"
        ));
        assert!(css.contains("border-radius: var(--oriha-table-radius);"));
        assert!(css.contains("border-right: 1px solid var(--oriha-table-border);"));
        assert!(css.contains("border-bottom: 1px solid var(--oriha-table-border);"));
        assert!(css.contains(".oriha-table th:last-child"));
        assert!(css.contains(".oriha-table tr:last-child td"));
        assert!(css.contains("border-right: 0;"));
        assert!(css.contains("border-bottom: 0;"));
    }

    #[test]
    fn table_border_none_removes_wrapper_and_cell_borders() {
        let theme = resolve_theme(&AppearanceOptions {
            table_border_style: TableBorderStyle::None,
            ..AppearanceOptions::default()
        });
        let css = render_css(&theme);

        assert!(css.contains(
            ".oriha-table-scroll-wrapper {\n  background: var(--oriha-table-bg);\n  border: 0;"
        ));
        assert!(css.contains(".oriha-table th,\n.oriha-table td"));
        assert!(!css.contains("border-right: 1px solid var(--oriha-table-border);"));
        assert!(!css.contains("border-bottom: 1px solid var(--oriha-table-border);"));
    }

    #[test]
    fn print_css_is_omitted_without_print_options() {
        let theme = resolve_theme(&AppearanceOptions::default());
        let css = render_css(&theme);

        assert!(!css.contains("@page"));
        assert!(!css.contains("@media print"));
    }

    #[test]
    fn print_css_uses_page_size_and_margin_when_enabled() {
        let theme = resolve_theme(&AppearanceOptions {
            print: Some(PrintOptions {
                page_size: Some("A4 landscape".to_string()),
                margin: Some("10mm 12mm".to_string()),
            }),
            ..AppearanceOptions::default()
        });
        let css = render_css(&theme);

        assert!(css.contains("@page"));
        assert!(css.contains("size: A4 landscape;"));
        assert!(css.contains("margin: 10mm 12mm;"));
        assert!(css.contains("@media print"));
        assert!(css.contains("print-color-adjust: exact;"));
        assert!(css.contains(".oriha-document"));
        assert!(css.contains(".oriha-callout"));
        assert!(css.contains(
            ".oriha-plantuml,\n  .oriha-diagram,\n  .oriha-math-block,\n  .oriha-mermaid"
        ));
        assert!(css.contains("overflow: visible;"));
        assert!(css.contains(
            ".oriha-plantuml > svg,\n  .oriha-diagram > svg,\n  .oriha-math-block > svg,\n  .oriha-mermaid > svg"
        ));
        assert!(css.contains("max-width: 100%;"));
    }

    #[test]
    fn print_options_deserialize_from_camel_case() {
        let input: RenderInput = serde_json::from_str(
            r#"{
              "markdown": "hello",
              "appearance": {
                "print": {
                  "pageSize": "Letter",
                  "margin": "8mm"
                }
              }
            }"#,
        )
        .expect("valid render input");

        let print = input
            .appearance
            .print
            .expect("print options should deserialize");
        assert_eq!(print.page_size.as_deref(), Some("Letter"));
        assert_eq!(print.margin.as_deref(), Some("8mm"));
    }

    #[test]
    fn malicious_print_values_fall_back_in_css() {
        let theme = resolve_theme(&AppearanceOptions {
            print: Some(PrintOptions {
                page_size: Some("A4;}</style><script>".to_string()),
                margin: Some("10mm{color:red}".to_string()),
            }),
            ..AppearanceOptions::default()
        });
        let css = render_css(&theme);

        assert!(css.contains("size: auto;"));
        assert!(css.contains("margin: 12mm;"));
        assert!(!css.contains("</style>"));
        assert!(!css.contains("<script>"));
        assert!(!css.contains("10mm{color:red}"));
    }

    #[test]
    fn malicious_font_values_fall_back_in_css() {
        let theme = resolve_theme(&AppearanceOptions {
            heading_font_family: crate::types::FontFamilySpec {
                family: "BadHeading;}</style><script>".to_string(),
                fallbacks: vec!["serif".to_string()],
            },
            body_font_family: crate::types::FontFamilySpec {
                family: "BadFont;}</style><script>".to_string(),
                fallbacks: vec!["sans-serif".to_string()],
            },
            code_font_family: crate::types::FontFamilySpec {
                family: "\"Bad Mono\"".to_string(),
                fallbacks: vec!["monospace{".to_string()],
            },
            ..AppearanceOptions::default()
        });
        let css = render_css(&theme);

        assert!(css.contains(
            "--oriha-font-heading: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Helvetica, Arial, sans-serif;"
        ));
        assert!(css.contains(
            "--oriha-font-body: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Helvetica, Arial, sans-serif;"
        ));
        assert!(css.contains(
            "--oriha-font-mono: \"SFMono-Regular\", Consolas, \"Liberation Mono\", Menlo, monospace;"
        ));
        assert!(!css.contains("BadHeading;"));
        assert!(!css.contains("BadFont;"));
        assert!(!css.contains("</style>"));
        assert!(!css.contains("monospace{"));
    }

    #[test]
    fn normal_print_and_quoted_font_values_are_preserved() {
        let theme = resolve_theme(&AppearanceOptions {
            print: Some(PrintOptions {
                page_size: Some("A4 landscape".to_string()),
                margin: Some("10mm 12mm".to_string()),
            }),
            body_font_family: crate::types::FontFamilySpec {
                family: "\"Noto Sans\"".to_string(),
                fallbacks: vec!["Arial".to_string(), "sans-serif".to_string()],
            },
            ..AppearanceOptions::default()
        });
        let css = render_css(&theme);

        assert!(css.contains("size: A4 landscape;"));
        assert!(css.contains("margin: 10mm 12mm;"));
        assert!(css.contains("--oriha-font-body: \"Noto Sans\", Arial, sans-serif;"));
    }
}
