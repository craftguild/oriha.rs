use crate::{
    ColorSchemeMessages, CoreMessages, FontMessages, LocaleMessages, LocaleOptionMessages,
    LockedReasonMessages, PlanMessages, SettingsMessages, TableBorderStyleMessages,
};

use super::SYNTAX_THEME_NAMES;

pub(crate) const MESSAGES: LocaleMessages = LocaleMessages {
    core: CoreMessages {
        code_copy_label: "Copier",
        code_copy_aria_label: "Copier le code",
        unsupported_plantuml_title: "Diagramme PlantUML non pris en charge",
        mermaid_render_error_title: "Impossible de rendre le diagramme Mermaid",
        footnotes_title: "Notes de bas de page",
        footnote_backref_label: "Retour à la référence",
    },
    settings: SettingsMessages {
        locale: LocaleOptionMessages {
            en: "Anglais",
            es: "Espagnol",
            fr: "Français",
            ja: "Japonais",
            ko: "Coréen",
            zh_hans: "Chinois simplifié",
            zh_hant: "Chinois traditionnel",
        },
        plan: PlanMessages {
            free_label: "Gratuit",
            free_description: "Markdown de base et GFM",
            premium_label: "Premium",
            premium_description: "Callouts, diagrammes, maths, thèmes de syntaxe et polices",
        },
        color_scheme: ColorSchemeMessages {
            light_label: "Clair",
            dark_label: "Sombre",
        },
        syntax_theme: SYNTAX_THEME_NAMES,
        font: FontMessages {
            system_default_label: "Système par défaut",
            custom_label: "Personnalisée",
            custom_document_description:
                "Police locale installée fournie avec la spécification de police personnalisée correspondante.",
            custom_code_description: "Police locale installée fournie avec customCodeFont.",
        },
        table_border_style: TableBorderStyleMessages {
            none: "Aucune bordure",
            inner: "Intérieures uniquement",
            all: "Toutes les bordures",
            vertical: "Verticales uniquement",
            horizontal: "Horizontales uniquement",
        },
        locked_reason: LockedReasonMessages {
            syntax_theme: "La personnalisation du thème de syntaxe nécessite le plan Premium.",
            heading_font: "La personnalisation de la police des titres nécessite le plan Premium.",
            body_font: "La personnalisation de la police du corps nécessite le plan Premium.",
            code_font: "La personnalisation de la police de code nécessite le plan Premium.",
            callouts: "Les callouts nécessitent le plan Premium.",
            diagrams_and_math:
                "PlantUML, Mermaid et le rendu mathématique nécessitent le plan Premium.",
        },
    },
};
