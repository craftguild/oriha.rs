use crate::{
    ColorSchemeMessages, CoreMessages, FontMessages, LocaleMessages, LocaleOptionMessages,
    LockedReasonMessages, PlanMessages, SettingsMessages, TableBorderStyleMessages,
};

use super::SYNTAX_THEME_NAMES;

pub(crate) const MESSAGES: LocaleMessages = LocaleMessages {
    core: CoreMessages {
        code_copy_label: "Copiar",
        code_copy_aria_label: "Copiar código",
        unsupported_plantuml_title: "Diagrama PlantUML no compatible",
        mermaid_render_error_title: "No se pudo renderizar el diagrama Mermaid",
        footnotes_title: "Notas al pie",
        footnote_backref_label: "Volver a la referencia",
    },
    settings: SettingsMessages {
        locale: LocaleOptionMessages {
            en: "Inglés",
            es: "Español",
            fr: "Francés",
            ja: "Japonés",
            ko: "Coreano",
            zh_hans: "Chino simplificado",
            zh_hant: "Chino tradicional",
        },
        plan: PlanMessages {
            free_label: "Gratis",
            free_description: "Markdown básico y GFM",
            premium_label: "Premium",
            premium_description: "Callouts, diagramas, matemáticas, temas de sintaxis y fuentes",
        },
        color_scheme: ColorSchemeMessages {
            light_label: "Claro",
            dark_label: "Oscuro",
        },
        syntax_theme: SYNTAX_THEME_NAMES,
        font: FontMessages {
            system_default_label: "Predeterminada del sistema",
            custom_label: "Personalizada",
            custom_document_description:
                "Fuente local instalada indicada mediante la especificación de fuente personalizada correspondiente.",
            custom_code_description: "Fuente local instalada indicada mediante customCodeFont.",
        },
        table_border_style: TableBorderStyleMessages {
            none: "Sin bordes",
            inner: "Solo interiores",
            all: "Todos los bordes",
            vertical: "Solo verticales",
            horizontal: "Solo horizontales",
        },
        locked_reason: LockedReasonMessages {
            syntax_theme: "La personalización del tema de sintaxis requiere el plan Premium.",
            heading_font: "La personalización de la fuente de encabezados requiere el plan Premium.",
            body_font: "La personalización de la fuente del cuerpo requiere el plan Premium.",
            code_font: "La personalización de la fuente de código requiere el plan Premium.",
            callouts: "Los callouts requieren el plan Premium.",
            diagrams_and_math:
                "PlantUML, Mermaid y el renderizado matemático requieren el plan Premium.",
        },
    },
};
