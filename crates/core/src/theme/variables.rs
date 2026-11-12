//! Deterministic projection from resolved theme tokens to CSS variables.
use super::*;

pub(super) fn css_variables_from_tokens(tokens: &ThemeTokens) -> BTreeMap<String, String> {
    let mut vars = BTreeMap::new();
    insert_var(&mut vars, "--oriha-color-text", &tokens.text.body);
    insert_var(&mut vars, "--oriha-color-heading", &tokens.text.heading);
    insert_var(&mut vars, "--oriha-color-muted", &tokens.text.muted);
    insert_var(&mut vars, "--oriha-color-link", &tokens.text.link);
    insert_var(
        &mut vars,
        "--oriha-color-link-hover",
        &tokens.text.link_hover,
    );
    insert_var(&mut vars, "--oriha-color-surface", &tokens.surface.base);
    insert_var(
        &mut vars,
        "--oriha-color-surface-subtle",
        &tokens.surface.subtle,
    );
    insert_var(
        &mut vars,
        "--oriha-color-surface-elevated",
        &tokens.surface.elevated,
    );
    insert_var(&mut vars, "--oriha-color-border", &tokens.border.default);
    insert_var(
        &mut vars,
        "--oriha-color-border-strong",
        &tokens.border.strong,
    );
    insert_var(
        &mut vars,
        "--oriha-color-blockquote-border",
        &tokens.border.blockquote,
    );
    insert_var(&mut vars, "--oriha-table-bg", &tokens.table.background);
    insert_var(&mut vars, "--oriha-table-header-bg", &tokens.table.header);
    insert_var(&mut vars, "--oriha-table-alt-bg", &tokens.table.alternate);
    insert_var(&mut vars, "--oriha-table-border", &tokens.table.border);
    insert_var(&mut vars, "--oriha-table-shadow", &tokens.table.shadow);
    insert_var(&mut vars, "--oriha-table-radius", &tokens.table.radius);
    insert_var(&mut vars, "--oriha-table-margin", &tokens.table.margin);
    insert_var(&mut vars, "--oriha-code-bg", &tokens.code.block_background);
    insert_var(
        &mut vars,
        "--oriha-code-header-bg",
        &tokens.code.header_background,
    );
    insert_var(
        &mut vars,
        "--oriha-code-gutter-bg",
        &tokens.code.gutter_background,
    );
    insert_var(
        &mut vars,
        "--oriha-code-inline-bg",
        &tokens.code.inline_background,
    );
    insert_var(&mut vars, "--oriha-code-text", &tokens.code.text);
    insert_var(
        &mut vars,
        "--oriha-code-line-number",
        &tokens.code.line_number,
    );
    insert_var(&mut vars, "--oriha-code-border", &tokens.code.border);
    insert_var(&mut vars, "--oriha-code-shadow", &tokens.code.shadow);
    insert_var(&mut vars, "--oriha-code-radius", &tokens.code.radius);
    insert_var(
        &mut vars,
        "--oriha-code-inline-radius",
        &tokens.code.inline_radius,
    );
    insert_var(&mut vars, "--oriha-code-padding", &tokens.code.padding);
    insert_var(&mut vars, "--oriha-code-copy-text", &tokens.code.copy_text);
    insert_var(
        &mut vars,
        "--oriha-code-copy-hover-bg",
        &tokens.code.copy_background_hover,
    );
    insert_var(
        &mut vars,
        "--oriha-code-copy-success-text",
        &tokens.code.copy_success_text,
    );
    insert_var(
        &mut vars,
        "--oriha-code-copy-success-bg",
        &tokens.code.copy_success_background,
    );
    insert_var(
        &mut vars,
        "--oriha-code-copy-failure-text",
        &tokens.code.copy_failure_text,
    );
    insert_var(
        &mut vars,
        "--oriha-code-copy-failure-bg",
        &tokens.code.copy_failure_background,
    );
    insert_callout_vars(&mut vars, "generic", &tokens.callout.generic);
    insert_callout_vars(&mut vars, "note", &tokens.callout.note);
    insert_callout_vars(&mut vars, "tip", &tokens.callout.tip);
    insert_callout_vars(&mut vars, "important", &tokens.callout.important);
    insert_callout_vars(&mut vars, "warning", &tokens.callout.warning);
    insert_callout_vars(&mut vars, "caution", &tokens.callout.caution);
    insert_var(
        &mut vars,
        "--oriha-callout-content-bg",
        &tokens.callout.content_background,
    );
    insert_var(&mut vars, "--oriha-callout-shadow", &tokens.callout.shadow);
    insert_var(&mut vars, "--oriha-callout-radius", &tokens.callout.radius);
    insert_var(&mut vars, "--oriha-diagram-bg", &tokens.diagram.background);
    insert_var(&mut vars, "--oriha-diagram-border", &tokens.diagram.border);
    insert_var(&mut vars, "--oriha-diagram-text", &tokens.diagram.text);
    insert_var(
        &mut vars,
        "--oriha-task-checkbox-accent",
        &tokens.task.checkbox_accent,
    );
    insert_var(
        &mut vars,
        "--oriha-task-checkbox-check",
        &tokens.task.checkbox_check,
    );
    insert_var(&mut vars, "--oriha-watermark-text", &tokens.watermark.text);
    insert_var(
        &mut vars,
        "--oriha-watermark-opacity",
        &tokens.watermark.opacity,
    );
    insert_var(
        &mut vars,
        "--oriha-watermark-qr-opacity",
        &tokens.watermark.qr_opacity,
    );
    insert_var(
        &mut vars,
        "--oriha-content-padding",
        &tokens.layout.content_padding,
    );
    insert_var(
        &mut vars,
        "--oriha-content-max-width",
        &tokens.layout.content_max_width,
    );
    insert_var(&mut vars, "--oriha-line-height", &tokens.layout.line_height);
    insert_var(
        &mut vars,
        "--oriha-paragraph-margin-top",
        &tokens.layout.paragraph_margin_top,
    );
    insert_var(
        &mut vars,
        "--oriha-paragraph-margin-bottom",
        &tokens.layout.paragraph_margin_bottom,
    );
    insert_var(
        &mut vars,
        "--oriha-heading-margin-top",
        &tokens.layout.heading_margin_top,
    );
    insert_var(
        &mut vars,
        "--oriha-heading-margin-bottom",
        &tokens.layout.heading_margin_bottom,
    );
    insert_var(
        &mut vars,
        "--oriha-heading-border",
        &tokens.layout.heading_border,
    );
    insert_var(
        &mut vars,
        "--oriha-heading-padding-bottom",
        &tokens.layout.heading_padding_bottom,
    );
    insert_var(
        &mut vars,
        "--oriha-content-radius",
        &tokens.layout.content_radius,
    );
    insert_var(
        &mut vars,
        "--oriha-blockquote-border-width",
        &tokens.layout.blockquote_border_width,
    );
    insert_var(
        &mut vars,
        "--oriha-blockquote-padding-left",
        &tokens.layout.blockquote_padding_left,
    );
    insert_var(
        &mut vars,
        "--oriha-blockquote-margin-left",
        &tokens.layout.blockquote_margin_left,
    );
    insert_var(&mut vars, "--oriha-syntax-bg", &tokens.syntax.background);
    insert_var(&mut vars, "--oriha-syntax-base", &tokens.syntax.base);
    insert_var(&mut vars, "--oriha-syntax-comment", &tokens.syntax.comment);
    insert_var(&mut vars, "--oriha-syntax-keyword", &tokens.syntax.keyword);
    insert_var(&mut vars, "--oriha-syntax-string", &tokens.syntax.string);
    insert_var(&mut vars, "--oriha-syntax-number", &tokens.syntax.number);
    insert_var(
        &mut vars,
        "--oriha-syntax-function",
        &tokens.syntax.function,
    );
    insert_var(
        &mut vars,
        "--oriha-syntax-variable",
        &tokens.syntax.variable,
    );
    insert_var(
        &mut vars,
        "--oriha-syntax-operator",
        &tokens.syntax.operator,
    );
    insert_var(&mut vars, "--oriha-syntax-class", &tokens.syntax.class_name);
    insert_var(
        &mut vars,
        "--oriha-syntax-constant",
        &tokens.syntax.constant,
    );
    insert_var(&mut vars, "--oriha-syntax-builtin", &tokens.syntax.builtin);
    insert_var(&mut vars, "--oriha-syntax-tag", &tokens.syntax.tag);
    insert_var(
        &mut vars,
        "--oriha-syntax-attribute",
        &tokens.syntax.attribute,
    );
    insert_var(
        &mut vars,
        "--oriha-syntax-namespace",
        &tokens.syntax.namespace,
    );
    insert_var(
        &mut vars,
        "--oriha-syntax-punctuation",
        &tokens.syntax.punctuation,
    );

    insert_var(&mut vars, "--oriha-color-accent", &tokens.callout.note.text);
    insert_var(&mut vars, "--oriha-color-success", &tokens.callout.tip.text);
    insert_var(
        &mut vars,
        "--oriha-color-important",
        &tokens.callout.important.text,
    );
    insert_var(
        &mut vars,
        "--oriha-color-warning",
        &tokens.callout.warning.text,
    );
    insert_var(
        &mut vars,
        "--oriha-color-danger",
        &tokens.callout.caution.text,
    );
    insert_var(
        &mut vars,
        "--oriha-shadow-color",
        shadow_color(&tokens.table.shadow),
    );
    insert_var(
        &mut vars,
        "--oriha-font-heading",
        "-apple-system, BlinkMacSystemFont, \"Segoe UI\", Helvetica, Arial, sans-serif",
    );
    insert_var(
        &mut vars,
        "--oriha-font-body",
        "-apple-system, BlinkMacSystemFont, \"Segoe UI\", Helvetica, Arial, sans-serif",
    );
    insert_var(
        &mut vars,
        "--oriha-font-mono",
        "\"SFMono-Regular\", Consolas, \"Liberation Mono\", Menlo, monospace",
    );
    vars
}

pub(super) fn insert_callout_vars(
    vars: &mut BTreeMap<String, String>,
    name: &str,
    tokens: &CalloutKindTokens,
) {
    insert_var(
        vars,
        &format!("--oriha-callout-{name}-bg"),
        &tokens.background,
    );
    insert_var(
        vars,
        &format!("--oriha-callout-{name}-header-bg"),
        &tokens.header,
    );
    insert_var(vars, &format!("--oriha-callout-{name}-text"), &tokens.text);
}

pub(super) fn insert_var(vars: &mut BTreeMap<String, String>, key: &str, value: &str) {
    vars.insert(key.to_string(), value.to_string());
}
