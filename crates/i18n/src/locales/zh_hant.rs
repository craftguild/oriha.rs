use crate::{
    ColorSchemeMessages, CoreMessages, FontMessages, LocaleMessages, LocaleOptionMessages,
    LockedReasonMessages, PlanMessages, SettingsMessages, TableBorderStyleMessages,
};

use super::SYNTAX_THEME_NAMES;

pub(crate) const MESSAGES: LocaleMessages = LocaleMessages {
    core: CoreMessages {
        code_copy_label: "複製",
        code_copy_aria_label: "複製程式碼",
        unsupported_plantuml_title: "不支援的 PlantUML 圖",
        mermaid_render_error_title: "無法算繪 Mermaid 圖",
        footnotes_title: "註腳",
        footnote_backref_label: "返回引用位置",
    },
    settings: SettingsMessages {
        locale: LocaleOptionMessages {
            en: "英文",
            es: "西班牙文",
            fr: "法文",
            ja: "日文",
            ko: "韓文",
            zh_hans: "簡體中文",
            zh_hant: "繁體中文",
        },
        plan: PlanMessages {
            free_label: "免費",
            free_description: "核心 Markdown 與 GFM",
            premium_label: "進階版",
            premium_description: "提示區塊、圖表、數學公式、語法主題與字型",
        },
        color_scheme: ColorSchemeMessages {
            light_label: "淺色",
            dark_label: "深色",
        },
        syntax_theme: SYNTAX_THEME_NAMES,
        font: FontMessages {
            system_default_label: "系統預設",
            custom_label: "自訂",
            custom_document_description: "透過對應自訂字型設定提供的已安裝本機字型。",
            custom_code_description: "透過 customCodeFont 提供的已安裝本機字型。",
        },
        table_border_style: TableBorderStyleMessages {
            none: "無邊框",
            inner: "僅內部",
            all: "全部邊框",
            vertical: "僅垂直",
            horizontal: "僅水平",
        },
        locked_reason: LockedReasonMessages {
            syntax_theme: "自訂語法主題需要進階版。",
            heading_font: "自訂標題字型需要進階版。",
            body_font: "自訂內文字型需要進階版。",
            code_font: "自訂程式碼字型需要進階版。",
            callouts: "提示區塊需要進階版。",
            diagrams_and_math: "PlantUML、Mermaid 與數學公式算繪需要進階版。",
        },
    },
};
