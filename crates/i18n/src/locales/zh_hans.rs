use crate::{
    ColorSchemeMessages, CoreMessages, FontMessages, LocaleMessages, LocaleOptionMessages,
    LockedReasonMessages, PlanMessages, SettingsMessages, TableBorderStyleMessages,
};

use super::SYNTAX_THEME_NAMES;

pub(crate) const MESSAGES: LocaleMessages = LocaleMessages {
    core: CoreMessages {
        code_copy_label: "复制",
        code_copy_aria_label: "复制代码",
        unsupported_plantuml_title: "不支持的 PlantUML 图",
        mermaid_render_error_title: "无法渲染 Mermaid 图",
        footnotes_title: "脚注",
        footnote_backref_label: "返回引用位置",
    },
    settings: SettingsMessages {
        locale: LocaleOptionMessages {
            en: "英语",
            es: "西班牙语",
            fr: "法语",
            ja: "日语",
            ko: "韩语",
            zh_hans: "简体中文",
            zh_hant: "繁体中文",
        },
        plan: PlanMessages {
            free_label: "免费",
            free_description: "核心 Markdown 和 GFM",
            premium_label: "高级版",
            premium_description: "提示块、图表、数学公式、语法主题和字体",
        },
        color_scheme: ColorSchemeMessages {
            light_label: "浅色",
            dark_label: "深色",
        },
        syntax_theme: SYNTAX_THEME_NAMES,
        font: FontMessages {
            system_default_label: "系统默认",
            custom_label: "自定义",
            custom_document_description: "通过对应自定义字体设置提供的已安装本地字体。",
            custom_code_description: "通过 customCodeFont 提供的已安装本地字体。",
        },
        table_border_style: TableBorderStyleMessages {
            none: "无边框",
            inner: "仅内部",
            all: "全部边框",
            vertical: "仅纵向",
            horizontal: "仅横向",
        },
        locked_reason: LockedReasonMessages {
            syntax_theme: "自定义语法主题需要高级版。",
            heading_font: "自定义标题字体需要高级版。",
            body_font: "自定义正文字体需要高级版。",
            code_font: "自定义代码字体需要高级版。",
            callouts: "提示块需要高级版。",
            diagrams_and_math: "PlantUML、Mermaid 和数学公式渲染需要高级版。",
        },
    },
};
