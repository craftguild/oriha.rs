use oriha_i18n::{
    messages, resolve_locale, supported_locale_options, I18nLocale, LocaleMessages,
    SyntaxThemeMessages,
};
use serde::Serialize;

use crate::diagnostics::bridge_diagnostic;
use crate::plan_policy::{apply_plan_policy, BridgeInput};
use crate::types::*;

#[uniffi::export]
pub fn resolve_render_settings(input: BridgeSettingsInput) -> BridgeResolvedSettings {
    resolve_render_settings_inner(input.into())
}

#[uniffi::export]
pub fn resolve_render_settings_json(input_json: String) -> String {
    let settings = match serde_json::from_str::<BridgeSettingsInput>(&input_json) {
        Ok(input) => resolve_render_settings(input),
        Err(error) => {
            let mut settings = resolve_render_settings(BridgeSettingsInput::default());
            settings.diagnostics.insert(
                0,
                bridge_diagnostic(
                    Severity::Error,
                    "input.invalid_json",
                    format!("BridgeSettingsInput JSON could not be parsed: {error}"),
                ),
            );
            settings
        }
    };

    serde_json::to_string(&settings).expect("BridgeResolvedSettings serialization should not fail")
}

fn resolve_render_settings_inner(input: RenderInput) -> BridgeResolvedSettings {
    let locale = resolve_locale(input.locale.as_deref());
    let BridgeInput {
        input, diagnostics, ..
    } = apply_plan_policy(input);
    let premium = matches!(input.plan, Plan::Premium);
    let catalog = messages(locale);
    BridgeResolvedSettings {
        resolved_locale: locale.code().to_string(),
        effective_input: BridgeEffectiveSettingsInput::from_input(input.clone(), locale),
        controls: controls_for_plan(premium, catalog),
        available_options: available_options_for_input(&input, catalog, locale),
        features: bridge_features_for_plan(&input.plan),
        diagnostics,
    }
}

fn controls_for_plan(premium: bool, catalog: &LocaleMessages) -> BridgeSettingsControls {
    BridgeSettingsControls {
        plan: enabled_control(),
        color_scheme: enabled_control(),
        syntax_theme: premium_control(premium, catalog.settings.locked_reason.syntax_theme),
        heading_font: premium_control(premium, catalog.settings.locked_reason.heading_font),
        body_font: premium_control(premium, catalog.settings.locked_reason.body_font),
        code_font: premium_control(premium, catalog.settings.locked_reason.code_font),
        table_border_style: enabled_control(),
    }
}

fn enabled_control() -> BridgeControlAvailability {
    BridgeControlAvailability {
        enabled: true,
        reason: None,
    }
}

fn premium_control(premium: bool, reason: &str) -> BridgeControlAvailability {
    BridgeControlAvailability {
        enabled: premium,
        reason: (!premium).then(|| reason.to_string()),
    }
}

fn available_options_for_input(
    input: &RenderInput,
    catalog: &LocaleMessages,
    locale: I18nLocale,
) -> BridgeAvailableOptions {
    let premium = matches!(input.plan, Plan::Premium);
    BridgeAvailableOptions {
        plan: catalog_plan_options(catalog),
        color_scheme: catalog_color_scheme_options(catalog),
        syntax_theme: if premium {
            catalog_syntax_theme_options(catalog_syntax_themes(), catalog)
        } else {
            catalog_syntax_theme_options(
                vec![input.rendering_options.syntax_theme.clone()],
                catalog,
            )
        },
        heading_font: if premium {
            catalog_heading_font_options(catalog_heading_fonts(), catalog)
        } else {
            catalog_heading_font_options(vec![HeadingFont::SystemDefault], catalog)
        },
        body_font: if premium {
            catalog_body_font_options(catalog_body_fonts(), catalog)
        } else {
            catalog_body_font_options(vec![BodyFont::SystemDefault], catalog)
        },
        code_font: if premium {
            catalog_code_font_options(catalog_code_fonts(), catalog)
        } else {
            catalog_code_font_options(vec![CodeFont::SystemDefault], catalog)
        },
        table_border_style: catalog_table_border_style_options(catalog),
        locale: catalog_locale_options(locale),
    }
}

fn option<T: Serialize>(id: T, label: &str) -> BridgeOption {
    BridgeOption {
        id: option_id(&id),
        label: label.to_string(),
        description: None,
        family: None,
        recommended: false,
    }
}

fn option_with_description<T: Serialize>(id: T, label: &str, description: &str) -> BridgeOption {
    BridgeOption {
        description: Some(description.to_string()),
        ..option(id, label)
    }
}

fn option_with_family<T: Serialize>(id: T, label: &str, family: &str) -> BridgeOption {
    BridgeOption {
        family: Some(family.to_string()),
        ..option(id, label)
    }
}

fn font_option<T: Serialize>(id: T, label: &str) -> BridgeFontOption {
    BridgeFontOption {
        id: option_id(&id),
        label: label.to_string(),
        description: None,
        family: None,
        recommended: false,
        font_family: None,
    }
}

fn font_option_with_description<T: Serialize>(
    id: T,
    label: &str,
    description: &str,
) -> BridgeFontOption {
    BridgeFontOption {
        description: Some(description.to_string()),
        ..font_option(id, label)
    }
}

fn option_id<T: Serialize>(id: &T) -> String {
    serde_json::to_value(id)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_default()
}

fn catalog_plan_options(catalog: &LocaleMessages) -> Vec<BridgeOption> {
    let plan = catalog.settings.plan;
    vec![
        option_with_description(Plan::Free, plan.free_label, plan.free_description),
        option_with_description(Plan::Premium, plan.premium_label, plan.premium_description),
    ]
}

fn catalog_color_scheme_options(catalog: &LocaleMessages) -> Vec<BridgeOption> {
    let color_scheme = catalog.settings.color_scheme;
    vec![
        option(ColorScheme::Light, color_scheme.light_label),
        option(ColorScheme::Dark, color_scheme.dark_label),
    ]
}

fn catalog_syntax_theme_options(
    themes: Vec<SyntaxTheme>,
    catalog: &LocaleMessages,
) -> Vec<BridgeOption> {
    let syntax_theme = catalog.settings.syntax_theme;
    themes
        .into_iter()
        .map(|theme| {
            let label = syntax_theme_label(syntax_theme, &theme);
            let family = syntax_theme_family(&theme);
            option_with_family(theme, label, family)
        })
        .collect()
}

fn syntax_theme_label(messages: SyntaxThemeMessages, theme: &SyntaxTheme) -> &'static str {
    messages
        .label(syntax_theme_option_id(theme))
        .unwrap_or("Custom")
}

fn syntax_theme_option_id(theme: &SyntaxTheme) -> &'static str {
    match theme {
        SyntaxTheme::GitHubLight => "github-light",
        SyntaxTheme::GitHubDark => "github-dark",
        SyntaxTheme::SolarizedLight => "solarized-light",
        SyntaxTheme::SolarizedDark => "solarized-dark",
        SyntaxTheme::GruvboxLight => "gruvbox-light",
        SyntaxTheme::GruvboxDark => "gruvbox-dark",
        SyntaxTheme::CatppuccinLatte => "catppuccin-latte",
        SyntaxTheme::CatppuccinMocha => "catppuccin-mocha",
        SyntaxTheme::AyuLight => "ayu-light",
        SyntaxTheme::AyuDark => "ayu-dark",
        SyntaxTheme::TokyoNightDay => "tokyo-night-day",
        SyntaxTheme::TokyoNight => "tokyo-night",
        SyntaxTheme::RosePineDawn => "rose-pine-dawn",
        SyntaxTheme::RosePine => "rose-pine",
        SyntaxTheme::HighContrastLight => "high-contrast-light",
        SyntaxTheme::HighContrastDark => "high-contrast-dark",
        SyntaxTheme::Custom => "custom",
    }
}

fn syntax_theme_family(theme: &SyntaxTheme) -> &'static str {
    match theme {
        SyntaxTheme::GitHubLight | SyntaxTheme::GitHubDark => "GitHub",
        SyntaxTheme::SolarizedLight | SyntaxTheme::SolarizedDark => "Solarized",
        SyntaxTheme::GruvboxLight | SyntaxTheme::GruvboxDark => "Gruvbox",
        SyntaxTheme::CatppuccinLatte | SyntaxTheme::CatppuccinMocha => "Catppuccin",
        SyntaxTheme::AyuLight | SyntaxTheme::AyuDark => "Ayu",
        SyntaxTheme::TokyoNightDay | SyntaxTheme::TokyoNight => "Tokyo Night",
        SyntaxTheme::RosePineDawn | SyntaxTheme::RosePine => "Rosé Pine",
        SyntaxTheme::HighContrastLight | SyntaxTheme::HighContrastDark => "High Contrast",
        SyntaxTheme::Custom => "Custom",
    }
}

fn catalog_heading_font_options(
    fonts: Vec<HeadingFont>,
    catalog: &LocaleMessages,
) -> Vec<BridgeFontOption> {
    let messages = catalog.settings.font;
    fonts
        .into_iter()
        .map(|font| match font {
            HeadingFont::SystemDefault => {
                font_option(HeadingFont::SystemDefault, messages.system_default_label)
            }
            HeadingFont::Custom => font_option_with_description(
                HeadingFont::Custom,
                messages.custom_label,
                messages.custom_document_description,
            ),
        })
        .collect()
}

fn catalog_body_font_options(
    fonts: Vec<BodyFont>,
    catalog: &LocaleMessages,
) -> Vec<BridgeFontOption> {
    let messages = catalog.settings.font;
    fonts
        .into_iter()
        .map(|font| match font {
            BodyFont::SystemDefault => {
                font_option(BodyFont::SystemDefault, messages.system_default_label)
            }
            BodyFont::Custom => font_option_with_description(
                BodyFont::Custom,
                messages.custom_label,
                messages.custom_document_description,
            ),
        })
        .collect()
}

fn catalog_code_font_options(
    fonts: Vec<CodeFont>,
    catalog: &LocaleMessages,
) -> Vec<BridgeFontOption> {
    let messages = catalog.settings.font;
    fonts
        .into_iter()
        .map(|font| match font {
            CodeFont::SystemDefault => {
                font_option(CodeFont::SystemDefault, messages.system_default_label)
            }
            CodeFont::Custom => font_option_with_description(
                CodeFont::Custom,
                messages.custom_label,
                messages.custom_code_description,
            ),
        })
        .collect()
}

fn catalog_table_border_style_options(catalog: &LocaleMessages) -> Vec<BridgeOption> {
    let table_border_style = catalog.settings.table_border_style;
    vec![
        option(TableBorderStyle::None, table_border_style.none),
        option(TableBorderStyle::Inner, table_border_style.inner),
        option(TableBorderStyle::All, table_border_style.all),
        option(TableBorderStyle::Vertical, table_border_style.vertical),
        option(TableBorderStyle::Horizontal, table_border_style.horizontal),
    ]
}

fn catalog_locale_options(locale: I18nLocale) -> Vec<BridgeOption> {
    supported_locale_options(locale)
        .into_iter()
        .map(|option| BridgeOption {
            id: option.id.to_string(),
            label: option.label.to_string(),
            description: option.description.map(|value| value.to_string()),
            family: option.family.map(|value| value.to_string()),
            recommended: option.recommended,
        })
        .collect()
}

fn catalog_syntax_themes() -> Vec<SyntaxTheme> {
    vec![
        SyntaxTheme::GitHubLight,
        SyntaxTheme::GitHubDark,
        SyntaxTheme::SolarizedLight,
        SyntaxTheme::SolarizedDark,
        SyntaxTheme::GruvboxLight,
        SyntaxTheme::GruvboxDark,
        SyntaxTheme::CatppuccinLatte,
        SyntaxTheme::CatppuccinMocha,
        SyntaxTheme::AyuLight,
        SyntaxTheme::AyuDark,
        SyntaxTheme::TokyoNightDay,
        SyntaxTheme::TokyoNight,
        SyntaxTheme::RosePineDawn,
        SyntaxTheme::RosePine,
        SyntaxTheme::HighContrastLight,
        SyntaxTheme::HighContrastDark,
    ]
}

fn catalog_heading_fonts() -> Vec<HeadingFont> {
    vec![HeadingFont::SystemDefault, HeadingFont::Custom]
}

fn catalog_body_fonts() -> Vec<BodyFont> {
    vec![BodyFont::SystemDefault, BodyFont::Custom]
}

fn catalog_code_fonts() -> Vec<CodeFont> {
    vec![CodeFont::SystemDefault, CodeFont::Custom]
}

fn bridge_features_for_plan(plan: &Plan) -> Vec<BridgeFeature> {
    let mut features = vec![
        BridgeFeature::Gfm,
        BridgeFeature::SyntaxHighlight,
        BridgeFeature::DarkMode,
        BridgeFeature::Tables,
        BridgeFeature::TaskLists,
        BridgeFeature::Footnotes,
        BridgeFeature::RawHtml,
        BridgeFeature::PdfExport,
    ];
    if matches!(plan, Plan::Premium) {
        features.extend([
            BridgeFeature::Callouts,
            BridgeFeature::RemovePdfWatermark,
            BridgeFeature::PrioritySupport,
        ]);
    }
    if matches!(plan, Plan::Premium) {
        features.extend([
            BridgeFeature::Plantuml,
            BridgeFeature::Mermaid,
            BridgeFeature::Math,
            BridgeFeature::SyntaxThemeCustomization,
            BridgeFeature::FontCustomization,
        ]);
    }
    features
}
