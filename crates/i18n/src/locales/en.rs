use crate::{
    ColorSchemeMessages, CoreMessages, FontMessages, LocaleMessages, LocaleOptionMessages,
    LockedReasonMessages, PlanMessages, SettingsMessages, TableBorderStyleMessages,
};

use super::SYNTAX_THEME_NAMES;

pub(crate) const MESSAGES: LocaleMessages = LocaleMessages {
    core: CoreMessages {
        code_copy_label: "Copy",
        code_copy_aria_label: "Copy code",
        unsupported_plantuml_title: "Unsupported PlantUML diagram",
        mermaid_render_error_title: "Mermaid diagram could not be rendered",
        footnotes_title: "Footnotes",
        footnote_backref_label: "Back to reference",
    },
    settings: SettingsMessages {
        locale: LocaleOptionMessages {
            en: "English",
            es: "Spanish",
            fr: "French",
            ja: "Japanese",
            ko: "Korean",
            zh_hans: "Simplified Chinese",
            zh_hant: "Traditional Chinese",
        },
        plan: PlanMessages {
            free_label: "Free",
            free_description: "Core Markdown and GFM",
            premium_label: "Premium",
            premium_description: "Callouts, diagrams, math, syntax themes, and fonts",
        },
        color_scheme: ColorSchemeMessages {
            light_label: "Light",
            dark_label: "Dark",
        },
        syntax_theme: SYNTAX_THEME_NAMES,
        font: FontMessages {
            system_default_label: "System Default",
            custom_label: "Custom",
            custom_document_description:
                "Installed local font supplied through the matching custom font spec.",
            custom_code_description: "Installed local font supplied through customCodeFont.",
        },
        table_border_style: TableBorderStyleMessages {
            none: "None",
            inner: "Inner only",
            all: "All borders",
            vertical: "Vertical only",
            horizontal: "Horizontal only",
        },
        locked_reason: LockedReasonMessages {
            syntax_theme: "Syntax theme customization requires the Premium plan.",
            heading_font: "Heading font customization requires the Premium plan.",
            body_font: "Body font customization requires the Premium plan.",
            code_font: "Code font customization requires the Premium plan.",
            callouts: "Callouts require the Premium plan.",
            diagrams_and_math: "PlantUML, Mermaid, and math rendering require the Premium plan.",
        },
    },
};
