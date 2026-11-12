mod locales;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum I18nLocale {
    En,
    Es,
    Fr,
    Ja,
    Ko,
    ZhHans,
    ZhHant,
}

pub const SUPPORTED_LOCALES: &[I18nLocale] = &[
    I18nLocale::En,
    I18nLocale::Es,
    I18nLocale::Fr,
    I18nLocale::Ja,
    I18nLocale::Ko,
    I18nLocale::ZhHans,
    I18nLocale::ZhHant,
];

impl I18nLocale {
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
            Self::Fr => "fr",
            Self::Ja => "ja",
            Self::Ko => "ko",
            Self::ZhHans => "zh-Hans",
            Self::ZhHant => "zh-Hant",
        }
    }
}

pub fn resolve_locale(locale: Option<&str>) -> I18nLocale {
    let Some(locale) = locale else {
        return I18nLocale::En;
    };
    let normalized = locale.replace('_', "-").to_ascii_lowercase();
    let mut parts = normalized.split('-');
    let language = parts.next().unwrap_or_default();
    let region_or_script = parts.collect::<Vec<_>>().join("-");
    match language {
        "es" => I18nLocale::Es,
        "fr" => I18nLocale::Fr,
        "ja" => I18nLocale::Ja,
        "ko" => I18nLocale::Ko,
        "zh" => {
            if region_or_script.contains("hant")
                || region_or_script.contains("tw")
                || region_or_script.contains("hk")
                || region_or_script.contains("mo")
            {
                I18nLocale::ZhHant
            } else {
                I18nLocale::ZhHans
            }
        }
        "en" => I18nLocale::En,
        _ => I18nLocale::En,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LocaleMessages {
    pub core: CoreMessages,
    pub settings: SettingsMessages,
}

#[derive(Clone, Copy, Debug)]
pub struct CoreMessages {
    pub code_copy_label: &'static str,
    pub code_copy_aria_label: &'static str,
    pub unsupported_plantuml_title: &'static str,
    pub mermaid_render_error_title: &'static str,
    pub footnotes_title: &'static str,
    pub footnote_backref_label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct SettingsMessages {
    pub locale: LocaleOptionMessages,
    pub plan: PlanMessages,
    pub color_scheme: ColorSchemeMessages,
    pub syntax_theme: SyntaxThemeMessages,
    pub font: FontMessages,
    pub table_border_style: TableBorderStyleMessages,
    pub locked_reason: LockedReasonMessages,
}

#[derive(Clone, Copy, Debug)]
pub struct LocaleOptionMessages {
    pub en: &'static str,
    pub es: &'static str,
    pub fr: &'static str,
    pub ja: &'static str,
    pub ko: &'static str,
    pub zh_hans: &'static str,
    pub zh_hant: &'static str,
}

impl LocaleOptionMessages {
    pub fn label(self, locale: I18nLocale) -> &'static str {
        match locale {
            I18nLocale::En => self.en,
            I18nLocale::Es => self.es,
            I18nLocale::Fr => self.fr,
            I18nLocale::Ja => self.ja,
            I18nLocale::Ko => self.ko,
            I18nLocale::ZhHans => self.zh_hans,
            I18nLocale::ZhHant => self.zh_hant,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlanMessages {
    pub free_label: &'static str,
    pub free_description: &'static str,
    pub premium_label: &'static str,
    pub premium_description: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct ColorSchemeMessages {
    pub light_label: &'static str,
    pub dark_label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct SyntaxThemeMessages {
    pub github_light: &'static str,
    pub github_dark: &'static str,
    pub solarized_light: &'static str,
    pub solarized_dark: &'static str,
    pub gruvbox_light: &'static str,
    pub gruvbox_dark: &'static str,
    pub catppuccin_latte: &'static str,
    pub catppuccin_mocha: &'static str,
    pub ayu_light: &'static str,
    pub ayu_dark: &'static str,
    pub tokyo_night_day: &'static str,
    pub tokyo_night: &'static str,
    pub rose_pine_dawn: &'static str,
    pub rose_pine: &'static str,
    pub high_contrast_light: &'static str,
    pub high_contrast_dark: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct FontMessages {
    pub system_default_label: &'static str,
    pub custom_label: &'static str,
    pub custom_document_description: &'static str,
    pub custom_code_description: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct TableBorderStyleMessages {
    pub none: &'static str,
    pub inner: &'static str,
    pub all: &'static str,
    pub vertical: &'static str,
    pub horizontal: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct LockedReasonMessages {
    pub syntax_theme: &'static str,
    pub heading_font: &'static str,
    pub body_font: &'static str,
    pub code_font: &'static str,
    pub callouts: &'static str,
    pub diagrams_and_math: &'static str,
}

pub fn messages(locale: I18nLocale) -> &'static LocaleMessages {
    match locale {
        I18nLocale::En => &locales::en::MESSAGES,
        I18nLocale::Es => &locales::es::MESSAGES,
        I18nLocale::Fr => &locales::fr::MESSAGES,
        I18nLocale::Ja => &locales::ja::MESSAGES,
        I18nLocale::Ko => &locales::ko::MESSAGES,
        I18nLocale::ZhHans => &locales::zh_hans::MESSAGES,
        I18nLocale::ZhHant => &locales::zh_hant::MESSAGES,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalizedOption<T> {
    pub id: T,
    pub label: &'static str,
    pub description: Option<&'static str>,
    pub family: Option<&'static str>,
    pub recommended: bool,
}

pub fn supported_locale_options(locale: I18nLocale) -> Vec<LocalizedOption<&'static str>> {
    let labels = messages(locale).settings.locale;
    SUPPORTED_LOCALES
        .iter()
        .map(|supported| LocalizedOption {
            id: supported.code(),
            label: labels.label(*supported),
            description: None,
            family: None,
            recommended: *supported == locale,
        })
        .collect()
}

impl SyntaxThemeMessages {
    pub fn label(self, id: &str) -> Option<&'static str> {
        match id {
            "github-light" => Some(self.github_light),
            "github-dark" => Some(self.github_dark),
            "solarized-light" => Some(self.solarized_light),
            "solarized-dark" => Some(self.solarized_dark),
            "gruvbox-light" => Some(self.gruvbox_light),
            "gruvbox-dark" => Some(self.gruvbox_dark),
            "catppuccin-latte" => Some(self.catppuccin_latte),
            "catppuccin-mocha" => Some(self.catppuccin_mocha),
            "ayu-light" => Some(self.ayu_light),
            "ayu-dark" => Some(self.ayu_dark),
            "tokyo-night-day" => Some(self.tokyo_night_day),
            "tokyo-night" => Some(self.tokyo_night),
            "rose-pine-dawn" => Some(self.rose_pine_dawn),
            "rose-pine" => Some(self.rose_pine),
            "high-contrast-light" => Some(self.high_contrast_light),
            "high-contrast-dark" => Some(self.high_contrast_dark),
            _ => None,
        }
    }
}

impl TableBorderStyleMessages {
    pub fn label(self, id: &str) -> Option<&'static str> {
        match id {
            "none" => Some(self.none),
            "inner" => Some(self.inner),
            "all" => Some(self.all),
            "vertical" => Some(self.vertical),
            "horizontal" => Some(self.horizontal),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_supported_locale_variants() {
        assert_eq!(resolve_locale(Some("ja-JP")), I18nLocale::Ja);
        assert_eq!(resolve_locale(Some("zh_TW")), I18nLocale::ZhHant);
        assert_eq!(resolve_locale(Some("zh-HK")), I18nLocale::ZhHant);
        assert_eq!(resolve_locale(Some("zh-CN")), I18nLocale::ZhHans);
        assert_eq!(resolve_locale(Some("unknown")), I18nLocale::En);
        assert_eq!(resolve_locale(None), I18nLocale::En);
    }

    #[test]
    fn every_supported_locale_has_core_and_settings_messages() {
        for locale in SUPPORTED_LOCALES {
            let messages = messages(*locale);
            assert!(!messages.core.code_copy_label.is_empty());
            assert!(!messages.core.footnotes_title.is_empty());
            assert!(!messages.settings.plan.free_label.is_empty());
            assert!(!messages.settings.table_border_style.inner.is_empty());
            assert!(!messages.settings.locked_reason.syntax_theme.is_empty());
            assert!(!messages.settings.locale.label(*locale).is_empty());
            assert_eq!(
                supported_locale_options(*locale).len(),
                SUPPORTED_LOCALES.len()
            );
        }
    }
}
