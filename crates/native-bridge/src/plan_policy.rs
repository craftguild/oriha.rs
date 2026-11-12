use oriha_i18n::{messages, resolve_locale, LocaleMessages};

use crate::stats::increment_kind_count;
use crate::types::*;
use crate::LOCKED_FEATURE_CODE;

pub(crate) struct BridgeInput {
    pub(crate) input: RenderInput,
    pub(crate) core_input: oriha_core::RenderInput,
    pub(crate) diagnostics: Vec<RenderDiagnostic>,
    pub(crate) stats: RenderStats,
}

pub(crate) fn apply_plan_policy(mut input: RenderInput) -> BridgeInput {
    let mut diagnostics = Vec::new();
    let mut stats = RenderStats::default();
    let premium = matches!(input.plan, Plan::Premium);
    let catalog = messages_for_input(&input);

    let default_syntax_theme =
        default_syntax_theme_for_scheme(&input.rendering_options.color_scheme);
    if !premium && input.rendering_options.syntax_theme != default_syntax_theme {
        diagnostics.push(locked_feature_diagnostic(
            catalog.settings.locked_reason.syntax_theme,
        ));
        increment_kind_count(&mut stats.locked_features_by_kind, "syntaxTheme", 1);
        input.rendering_options.syntax_theme = default_syntax_theme;
        input.rendering_options.custom_syntax_theme = None;
    }

    if !premium && input.rendering_options.heading_font != HeadingFont::SystemDefault {
        diagnostics.push(locked_feature_diagnostic(
            catalog.settings.locked_reason.heading_font,
        ));
        increment_kind_count(&mut stats.locked_features_by_kind, "headingFont", 1);
        input.rendering_options.heading_font = HeadingFont::SystemDefault;
        input.rendering_options.custom_heading_font = None;
    }

    if !premium && input.rendering_options.body_font != BodyFont::SystemDefault {
        diagnostics.push(locked_feature_diagnostic(
            catalog.settings.locked_reason.body_font,
        ));
        increment_kind_count(&mut stats.locked_features_by_kind, "bodyFont", 1);
        input.rendering_options.body_font = BodyFont::SystemDefault;
        input.rendering_options.custom_body_font = None;
    }

    if !premium && input.rendering_options.code_font != CodeFont::SystemDefault {
        diagnostics.push(locked_feature_diagnostic(
            catalog.settings.locked_reason.code_font,
        ));
        increment_kind_count(&mut stats.locked_features_by_kind, "codeFont", 1);
        input.rendering_options.code_font = CodeFont::SystemDefault;
        input.rendering_options.custom_code_font = None;
    }

    if matches!(input.plan, Plan::Free) {
        record_locked_markdown_features(&input.markdown, &mut stats);
    }

    if matches!(input.plan, Plan::Free) && count_callout_markers(&input.markdown) > 0 {
        diagnostics.push(locked_feature_diagnostic(
            catalog.settings.locked_reason.callouts,
        ));
    }

    if !premium && contains_premium_diagram_or_math(&input.markdown) {
        diagnostics.push(locked_feature_diagnostic(
            catalog.settings.locked_reason.diagrams_and_math,
        ));
    }

    apply_free_print_watermark_policy(&mut input);

    let core_input = input_to_core(input.clone());
    BridgeInput {
        input,
        core_input,
        diagnostics,
        stats,
    }
}

fn apply_free_print_watermark_policy(input: &mut RenderInput) {
    if matches!(input.plan, Plan::Free) && input.print.is_some() && input.watermark.is_none() {
        input.watermark = Some(oriha_core::default_watermark_options().into());
    }
}

fn messages_for_input(input: &RenderInput) -> &'static LocaleMessages {
    messages(resolve_locale(input.locale.as_deref()))
}

pub(crate) fn input_to_core(input: RenderInput) -> oriha_core::RenderInput {
    let features = features_for_plan(&input.plan);
    let rendering_options = input.rendering_options;
    let color_scheme = rendering_options.color_scheme.clone().into();
    let style = rendering_options
        .appearance_style
        .as_core_style()
        .to_string();
    let syntax_theme = syntax_theme_id(&rendering_options);
    let heading_font_family = heading_font_spec(&rendering_options);
    let body_font_family = body_font_spec(&rendering_options);
    let code_font_family = code_font_spec(&rendering_options);
    oriha_core::RenderInput {
        markdown: input.markdown,
        locale: input.locale,
        appearance: oriha_core::AppearanceOptions {
            color_scheme,
            style,
            syntax_theme,
            heading_font_family,
            body_font_family,
            body_font_size_px: rendering_options.body_font_size_px,
            code_font_family,
            table_border_style: rendering_options.table_border_style.into(),
            print: input.print.map(Into::into),
            rounded_corners: rendering_options.rounded_corners,
            shadows: rendering_options.shadows,
            full_table_data: rendering_options.full_table_data,
            block_borders: rendering_options.block_borders,
        },
        render_options: oriha_core::RenderOptions {
            watermark: input.watermark.map(Into::into),
            source_attributes: input.source_attributes,
            table_of_contents: input.table_of_contents,
            ..Default::default()
        },
        mode: input.render_mode.into(),
        features,
        viewport: input.viewport.map(Into::into),
    }
}

fn features_for_plan(plan: &Plan) -> oriha_core::FeatureFlags {
    match plan {
        Plan::Free => oriha_core::FeatureFlags {
            gfm: true,
            callouts: false,
            syntax_highlight: true,
            raw_html: true,
            math: false,
            mermaid: false,
            plantuml: false,
        },
        Plan::Premium => oriha_core::FeatureFlags {
            gfm: true,
            callouts: true,
            syntax_highlight: true,
            raw_html: true,
            math: true,
            mermaid: true,
            plantuml: true,
        },
    }
}

fn syntax_theme_id(input: &RenderingOptions) -> String {
    match input.syntax_theme {
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
        SyntaxTheme::Custom => input
            .custom_syntax_theme
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("github-light"),
    }
    .to_string()
}

fn heading_font_spec(input: &RenderingOptions) -> oriha_core::FontFamilySpec {
    match input.heading_font {
        HeadingFont::SystemDefault => FontFamilySpec::system(),
        HeadingFont::Custom => input
            .custom_heading_font
            .clone()
            .filter(non_empty_font)
            .unwrap_or_else(FontFamilySpec::system),
    }
    .into()
}

fn body_font_spec(input: &RenderingOptions) -> oriha_core::FontFamilySpec {
    match input.body_font {
        BodyFont::SystemDefault => FontFamilySpec::system(),
        BodyFont::Custom => input
            .custom_body_font
            .clone()
            .filter(non_empty_font)
            .unwrap_or_else(FontFamilySpec::system),
    }
    .into()
}

fn code_font_spec(input: &RenderingOptions) -> oriha_core::FontFamilySpec {
    match input.code_font {
        CodeFont::SystemDefault => FontFamilySpec::system_mono(),
        CodeFont::Custom => input
            .custom_code_font
            .clone()
            .filter(non_empty_font)
            .unwrap_or_else(FontFamilySpec::system_mono),
    }
    .into()
}

fn non_empty_font(value: &FontFamilySpec) -> bool {
    !value.family.trim().is_empty()
}

fn default_syntax_theme_for_scheme(scheme: &ColorScheme) -> SyntaxTheme {
    match scheme {
        ColorScheme::Light => SyntaxTheme::GitHubLight,
        ColorScheme::Dark => SyntaxTheme::GitHubDark,
    }
}

fn count_callout_markers(markdown: &str) -> u32 {
    markdown
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with(":::note")
                || trimmed.starts_with(":::tip")
                || trimmed.starts_with(":::important")
                || trimmed.starts_with(":::warning")
                || trimmed.starts_with(":::caution")
                || trimmed.starts_with("> [!NOTE]")
                || trimmed.starts_with("> [!TIP]")
                || trimmed.starts_with("> [!IMPORTANT]")
                || trimmed.starts_with("> [!WARNING]")
                || trimmed.starts_with("> [!CAUTION]")
        })
        .count() as u32
}

fn record_locked_markdown_features(markdown: &str, stats: &mut RenderStats) {
    let mut math_count = 0;
    let mut mermaid_count = 0;
    let mut plantuml_count = 0;
    let mut in_display_math = false;

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("```plantuml")
            || lower.starts_with("```puml")
            || lower.starts_with("~~~plantuml")
            || lower.starts_with("~~~puml")
        {
            plantuml_count += 1;
        } else if lower.starts_with("```mermaid") || lower.starts_with("~~~mermaid") {
            mermaid_count += 1;
        } else if lower.starts_with("```latex")
            || lower.starts_with("```tex")
            || lower.starts_with("~~~latex")
            || lower.starts_with("~~~tex")
        {
            math_count += 1;
        }

        if trimmed == "$$" || trimmed.starts_with("$$ ") || trimmed.starts_with("$$\t") {
            if !in_display_math {
                math_count += 1;
                in_display_math = true;
            } else {
                in_display_math = false;
            }
        }
    }

    increment_kind_count(
        &mut stats.locked_features_by_kind,
        "callout",
        count_callout_markers(markdown),
    );
    increment_kind_count(&mut stats.locked_features_by_kind, "mermaid", mermaid_count);
    increment_kind_count(
        &mut stats.locked_features_by_kind,
        "plantuml",
        plantuml_count,
    );
    increment_kind_count(&mut stats.locked_features_by_kind, "math", math_count);
}

fn contains_premium_diagram_or_math(markdown: &str) -> bool {
    let lower = markdown.to_ascii_lowercase();
    lower.contains("```plantuml")
        || lower.contains("```puml")
        || lower.contains("~~~plantuml")
        || lower.contains("~~~puml")
        || lower.contains("```mermaid")
        || lower.contains("~~~mermaid")
        || lower.contains("```latex")
        || lower.contains("```tex")
        || lower.contains("~~~latex")
        || lower.contains("~~~tex")
        || lower.contains("$$")
}

fn locked_feature_diagnostic(message: &str) -> RenderDiagnostic {
    RenderDiagnostic {
        severity: Severity::Warning,
        code: LOCKED_FEATURE_CODE.to_string(),
        message: message.to_string(),
        source_range: None,
        block_id: None,
        engine: DiagnosticEngine::Bridge,
    }
}
