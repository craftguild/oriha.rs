use crate::{
    ColorSchemeMessages, CoreMessages, FontMessages, LocaleMessages, LocaleOptionMessages,
    LockedReasonMessages, PlanMessages, SettingsMessages, TableBorderStyleMessages,
};

use super::SYNTAX_THEME_NAMES;

pub(crate) const MESSAGES: LocaleMessages = LocaleMessages {
    core: CoreMessages {
        code_copy_label: "コピー",
        code_copy_aria_label: "コードをコピー",
        unsupported_plantuml_title: "未対応の PlantUML 図",
        mermaid_render_error_title: "Mermaid 図をレンダリングできませんでした",
        footnotes_title: "脚注",
        footnote_backref_label: "参照元に戻る",
    },
    settings: SettingsMessages {
        locale: LocaleOptionMessages {
            en: "英語",
            es: "スペイン語",
            fr: "フランス語",
            ja: "日本語",
            ko: "韓国語",
            zh_hans: "簡体字中国語",
            zh_hant: "繁体字中国語",
        },
        plan: PlanMessages {
            free_label: "無料",
            free_description: "基本的な Markdown と GFM",
            premium_label: "プレミアム",
            premium_description: "コールアウト、図、数式、シンタックステーマ、フォント",
        },
        color_scheme: ColorSchemeMessages {
            light_label: "ライト",
            dark_label: "ダーク",
        },
        syntax_theme: SYNTAX_THEME_NAMES,
        font: FontMessages {
            system_default_label: "システム標準",
            custom_label: "カスタム",
            custom_document_description:
                "対応するカスタムフォント設定で指定したインストール済みローカルフォントです。",
            custom_code_description:
                "customCodeFont で指定したインストール済みローカルフォントです。",
        },
        table_border_style: TableBorderStyleMessages {
            none: "無効",
            inner: "内部のみ",
            all: "すべて",
            vertical: "縦線のみ",
            horizontal: "横線のみ",
        },
        locked_reason: LockedReasonMessages {
            syntax_theme: "シンタックステーマのカスタマイズにはプレミアムプランが必要です。",
            heading_font: "見出しフォントのカスタマイズにはプレミアムプランが必要です。",
            body_font: "本文フォントのカスタマイズにはプレミアムプランが必要です。",
            code_font: "コードフォントのカスタマイズにはプレミアムプランが必要です。",
            callouts: "コールアウトにはプレミアムプランが必要です。",
            diagrams_and_math:
                "PlantUML、Mermaid、数式レンダリングにはプレミアムプランが必要です。",
        },
    },
};
