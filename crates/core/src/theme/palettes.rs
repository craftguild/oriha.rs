//! Surface, text, border, code, callout and layout token palettes.
use super::*;

pub(super) fn surface_tokens(variant: ThemeVariant) -> SurfaceTokens {
    match variant {
        ThemeVariant::Light => SurfaceTokens {
            base: "#ffffff".to_string(),
            subtle: "#f6f8fa".to_string(),
            elevated: "#ffffff".to_string(),
        },
        ThemeVariant::Dark => SurfaceTokens {
            base: "#0d1117".to_string(),
            subtle: "#161b22".to_string(),
            elevated: "#161b22".to_string(),
        },
    }
}

pub(super) fn text_tokens(variant: ThemeVariant) -> TextTokens {
    match variant {
        ThemeVariant::Light => TextTokens {
            body: "#333333".to_string(),
            heading: "#333333".to_string(),
            muted: "#656d76".to_string(),
            link: "#0969da".to_string(),
            link_hover: "#0860ca".to_string(),
        },
        ThemeVariant::Dark => TextTokens {
            body: "#c9d1d9".to_string(),
            heading: "#c9d1d9".to_string(),
            muted: "#8b949e".to_string(),
            link: "#58a6ff".to_string(),
            link_hover: "#79c0ff".to_string(),
        },
    }
}

pub(super) fn border_tokens(variant: ThemeVariant) -> BorderTokens {
    match variant {
        ThemeVariant::Light => BorderTokens {
            default: "#d1d9e0".to_string(),
            strong: "#d1d9e0".to_string(),
            blockquote: "#d1d9e0".to_string(),
        },
        ThemeVariant::Dark => BorderTokens {
            default: "#30363d".to_string(),
            strong: "#30363d".to_string(),
            blockquote: "#30363d".to_string(),
        },
    }
}

pub(super) fn table_tokens(variant: ThemeVariant, style: ThemeStyle) -> TableTokens {
    let (background, header, alternate, border) = match variant {
        ThemeVariant::Light => ("#ffffff", "#f5f5f5", "#f8fafc", "#e5e7eb"),
        ThemeVariant::Dark => ("#1e1e1e", "#2d2d2d", "#252526", "#3e3e3e"),
    };
    TableTokens {
        background: background.to_string(),
        header: header.to_string(),
        alternate: alternate.to_string(),
        border: border.to_string(),
        shadow: style_shadow(style, variant),
        radius: style_radius(style),
        margin: match style {
            ThemeStyle::Rich => "16px 0",
            ThemeStyle::Simple => "12px 0",
        }
        .to_string(),
    }
}

pub(super) fn code_tokens(variant: ThemeVariant, style: ThemeStyle) -> CodeTokens {
    let (
        block_background,
        header_background,
        gutter_background,
        inline_background,
        text,
        line_number,
        border,
        copy_text,
        copy_background_hover,
        copy_success_text,
        copy_success_background,
        copy_failure_text,
        copy_failure_background,
    ) = match variant {
        ThemeVariant::Light => (
            "#ffffff", "#f5f5f5", "#f8fafc", "#f6f8fa", "#1f2937", "#9ca3af", "#e5e7eb", "#1f2937",
            "#e5e7eb", "#238636", "#effcf4", "#dc2626", "#fff1f2",
        ),
        ThemeVariant::Dark => (
            "#1e1e1e",
            "#2d2d2d",
            "#252526",
            "#2d2d2d",
            "#f8f8f2",
            "#858585",
            "#3e3e3e",
            "#f8f8f2",
            "#3a3a3a",
            "#7ee787",
            "rgba(35, 134, 54, 0.16)",
            "#ff7b72",
            "rgba(248, 81, 73, 0.14)",
        ),
    };
    CodeTokens {
        block_background: block_background.to_string(),
        header_background: header_background.to_string(),
        gutter_background: gutter_background.to_string(),
        inline_background: inline_background.to_string(),
        text: text.to_string(),
        line_number: line_number.to_string(),
        border: border.to_string(),
        shadow: style_shadow(style, variant),
        radius: match style {
            ThemeStyle::Rich => "8px",
            ThemeStyle::Simple => "0",
        }
        .to_string(),
        inline_radius: match style {
            ThemeStyle::Rich => "3px",
            ThemeStyle::Simple => "0",
        }
        .to_string(),
        padding: match style {
            ThemeStyle::Rich => "16px",
            ThemeStyle::Simple => "12px",
        }
        .to_string(),
        copy_text: copy_text.to_string(),
        copy_background_hover: copy_background_hover.to_string(),
        copy_success_text: copy_success_text.to_string(),
        copy_success_background: copy_success_background.to_string(),
        copy_failure_text: copy_failure_text.to_string(),
        copy_failure_background: copy_failure_background.to_string(),
    }
}

pub(super) fn callout_tokens(variant: ThemeVariant, style: ThemeStyle) -> CalloutTokens {
    let (generic, note, tip, important, warning, caution, content_background) = match variant {
        ThemeVariant::Light => (
            callout_kind("#f0f9ff", "#dbeafe", "#1e40af"),
            callout_kind("#f0f9ff", "#dbeafe", "#1e40af"),
            callout_kind("#f0fdf4", "#dcfce7", "#166534"),
            callout_kind("#faf5ff", "#f3e8ff", "#6b21a8"),
            callout_kind("#fffbeb", "#fef3c7", "#92400e"),
            callout_kind("#fef2f2", "#fee2e2", "#991b1b"),
            "#ffffff",
        ),
        ThemeVariant::Dark => (
            callout_kind("#0d1117", "#161b22", "#58a6ff"),
            callout_kind("#0d1117", "#1e3a8a", "#93c5fd"),
            callout_kind("#0d1117", "#14532d", "#86efac"),
            callout_kind("#0d1117", "#581c87", "#d8b4fe"),
            callout_kind("#0d1117", "#78350f", "#fcd34d"),
            callout_kind("#0d1117", "#7f1d1d", "#fca5a5"),
            "#161b22",
        ),
    };
    CalloutTokens {
        generic,
        note,
        tip,
        important,
        warning,
        caution,
        content_background: content_background.to_string(),
        shadow: style_shadow(style, variant),
        radius: style_radius(style),
    }
}

pub(super) fn callout_kind(background: &str, header: &str, text: &str) -> CalloutKindTokens {
    CalloutKindTokens {
        background: background.to_string(),
        header: header.to_string(),
        text: text.to_string(),
    }
}

pub(super) fn diagram_tokens(variant: ThemeVariant) -> DiagramTokens {
    match variant {
        ThemeVariant::Light => DiagramTokens {
            background: "#ffffff".to_string(),
            border: "#d1d9e0".to_string(),
            text: "#333333".to_string(),
        },
        ThemeVariant::Dark => DiagramTokens {
            background: "#161b22".to_string(),
            border: "#30363d".to_string(),
            text: "#c9d1d9".to_string(),
        },
    }
}

pub(super) fn task_tokens(variant: ThemeVariant) -> TaskTokens {
    match variant {
        ThemeVariant::Light => TaskTokens {
            checkbox_accent: "#0a84ff".to_string(),
            checkbox_check: "#ffffff".to_string(),
        },
        ThemeVariant::Dark => TaskTokens {
            checkbox_accent: "#5e9ce6".to_string(),
            checkbox_check: "#ffffff".to_string(),
        },
    }
}

pub(super) fn watermark_tokens(variant: ThemeVariant) -> WatermarkTokens {
    match variant {
        ThemeVariant::Light => WatermarkTokens {
            text: "#656d76".to_string(),
            opacity: "0.45".to_string(),
            qr_opacity: "0.55".to_string(),
        },
        ThemeVariant::Dark => WatermarkTokens {
            text: "#8b949e".to_string(),
            opacity: "0.5".to_string(),
            qr_opacity: "0.6".to_string(),
        },
    }
}

pub(super) fn layout_tokens(style: ThemeStyle) -> LayoutTokens {
    match style {
        ThemeStyle::Rich => LayoutTokens {
            content_padding: "32px".to_string(),
            content_max_width: "980px".to_string(),
            line_height: "1.6".to_string(),
            paragraph_margin_top: "0.5em".to_string(),
            paragraph_margin_bottom: "0.5em".to_string(),
            heading_margin_top: "24px".to_string(),
            heading_margin_bottom: "16px".to_string(),
            heading_border: "1px solid var(--oriha-color-border-strong)".to_string(),
            heading_padding_bottom: "0.3em".to_string(),
            content_radius: "6px".to_string(),
            blockquote_border_width: "4px".to_string(),
            blockquote_padding_left: "16px".to_string(),
            blockquote_margin_left: "0".to_string(),
        },
        ThemeStyle::Simple => LayoutTokens {
            content_padding: "16px".to_string(),
            content_max_width: "100%".to_string(),
            line_height: "1.5".to_string(),
            paragraph_margin_top: "0.5em".to_string(),
            paragraph_margin_bottom: "0.5em".to_string(),
            heading_margin_top: "16px".to_string(),
            heading_margin_bottom: "8px".to_string(),
            heading_border: "none".to_string(),
            heading_padding_bottom: "0".to_string(),
            content_radius: "0".to_string(),
            blockquote_border_width: "2px".to_string(),
            blockquote_padding_left: "12px".to_string(),
            blockquote_margin_left: "0".to_string(),
        },
    }
}
