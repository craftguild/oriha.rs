use crate::{
    ColorSchemeMessages, CoreMessages, FontMessages, LocaleMessages, LocaleOptionMessages,
    LockedReasonMessages, PlanMessages, SettingsMessages, TableBorderStyleMessages,
};

use super::SYNTAX_THEME_NAMES;

pub(crate) const MESSAGES: LocaleMessages = LocaleMessages {
    core: CoreMessages {
        code_copy_label: "복사",
        code_copy_aria_label: "코드 복사",
        unsupported_plantuml_title: "지원되지 않는 PlantUML 다이어그램",
        mermaid_render_error_title: "Mermaid 다이어그램을 렌더링할 수 없습니다",
        footnotes_title: "각주",
        footnote_backref_label: "참조로 돌아가기",
    },
    settings: SettingsMessages {
        locale: LocaleOptionMessages {
            en: "영어",
            es: "스페인어",
            fr: "프랑스어",
            ja: "일본어",
            ko: "한국어",
            zh_hans: "중국어 간체",
            zh_hant: "중국어 번체",
        },
        plan: PlanMessages {
            free_label: "무료",
            free_description: "기본 Markdown 및 GFM",
            premium_label: "프리미엄",
            premium_description: "콜아웃, 다이어그램, 수식, 구문 테마, 글꼴",
        },
        color_scheme: ColorSchemeMessages {
            light_label: "라이트",
            dark_label: "다크",
        },
        syntax_theme: SYNTAX_THEME_NAMES,
        font: FontMessages {
            system_default_label: "시스템 기본값",
            custom_label: "사용자 지정",
            custom_document_description:
                "해당 사용자 지정 글꼴 설정으로 제공되는 설치된 로컬 글꼴입니다.",
            custom_code_description: "customCodeFont로 제공되는 설치된 로컬 글꼴입니다.",
        },
        table_border_style: TableBorderStyleMessages {
            none: "없음",
            inner: "내부만",
            all: "모든 테두리",
            vertical: "세로만",
            horizontal: "가로만",
        },
        locked_reason: LockedReasonMessages {
            syntax_theme: "구문 테마 사용자 지정에는 프리미엄 플랜이 필요합니다.",
            heading_font: "제목 글꼴 사용자 지정에는 프리미엄 플랜이 필요합니다.",
            body_font: "본문 글꼴 사용자 지정에는 프리미엄 플랜이 필요합니다.",
            code_font: "코드 글꼴 사용자 지정에는 프리미엄 플랜이 필요합니다.",
            callouts: "콜아웃에는 프리미엄 플랜이 필요합니다.",
            diagrams_and_math: "PlantUML, Mermaid 및 수식 렌더링에는 프리미엄 플랜이 필요합니다.",
        },
    },
};
