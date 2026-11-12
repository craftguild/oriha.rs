use super::*;

pub(super) fn code_block_language_is_plantuml(node: &CodeBlockNode) -> bool {
    let Some(language) = node.language.as_ref() else {
        return false;
    };
    let normalized = decode_markdown(language);
    is_plantuml_language(&normalized)
}

pub(super) fn code_block_language_is_mermaid(node: &CodeBlockNode) -> bool {
    let Some(language) = node.language.as_ref() else {
        return false;
    };
    let normalized = decode_markdown(language);
    is_mermaid_language(&normalized)
}

pub(super) fn is_latex_language(language: &str) -> bool {
    matches!(
        language.trim().to_ascii_lowercase().as_str(),
        "latex" | "tex"
    )
}

pub(super) fn is_plantuml_language(language: &str) -> bool {
    matches!(
        language.trim().to_ascii_lowercase().as_str(),
        "plantuml" | "puml"
    )
}

pub(super) fn is_mermaid_language(language: &str) -> bool {
    matches!(
        language.trim().to_ascii_lowercase().as_str(),
        "mermaid" | "mmd"
    )
}

pub(crate) fn normalize_fenced_code_text(source: &str, indent: usize) -> String {
    if indent == 0 {
        return source.to_string();
    }
    source
        .lines()
        .map(|line| strip_up_to_n_leading_spaces(line, indent))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn strip_up_to_n_leading_spaces(line: &str, max: usize) -> &str {
    let mut count = 0usize;
    let mut idx = 0usize;
    for ch in line.chars() {
        if ch == ' ' && count < max {
            idx += 1;
            count += 1;
        } else {
            break;
        }
    }
    &line[idx..]
}

pub(super) fn render_callout(
    block: &mut BlockNode,
    node: &CalloutNode,
    source: &SourceDocument,
    state: &mut RenderState,
    context: &RenderContext,
    renderers: &[Box<dyn BlockRenderer>],
    out: &mut String,
) {
    let kind = normalize_callout_kind(&node.kind);
    let modifier = known_callout_kind(&kind).map(|known| format!(" oriha-callout--{known}"));
    let title = node
        .title
        .clone()
        .unwrap_or_else(|| default_callout_title(&kind));
    let title_markup = if let Some(known) = known_callout_kind(&kind) {
        format!(
            r#"<span class="oriha-callout__icon" aria-hidden="true">{}</span><span class="oriha-callout__title-text">{}</span>"#,
            callout_icon_svg(known),
            escape_text(&title)
        )
    } else {
        format!(
            r#"<span class="oriha-callout__title-text">{}</span>"#,
            escape_text(&title)
        )
    };
    out.push_html(&format!(
        r#"<section id="{}" class="oriha-callout{}" data-oriha-block-kind="callout" data-oriha-callout-kind="{}"><div class="oriha-callout__title">{}</div><div class="oriha-callout__body">"#,
        escape_attr(&block.id),
        modifier.as_deref().unwrap_or_default(),
        escape_attr(&kind),
        title_markup
    ));

    if !block.children.is_empty() {
        if let Some(normalized) = block.attrs.get("normalized_source") {
            let nested_source = SourceDocument::new(normalized.clone());
            render_blocks_from_source(
                &mut block.children,
                &nested_source,
                context,
                renderers,
                state,
                out,
                false,
            );
        } else {
            render_blocks_from_source(
                &mut block.children,
                source,
                context,
                renderers,
                state,
                out,
                false,
            );
        }
    }

    out.push_html("</div></section>");
    state.stats.record_callout_kind(&kind);
    state
        .source_map
        .push(mapped_block(block, source, "callout", BTreeMap::new()));
}

pub(super) fn normalize_callout_kind(kind: &str) -> String {
    kind.trim().to_ascii_lowercase()
}

pub(super) fn known_callout_kind(kind: &str) -> Option<&'static str> {
    match kind.to_ascii_lowercase().as_str() {
        "note" => Some("note"),
        "tip" => Some("tip"),
        "important" => Some("important"),
        "warning" => Some("warning"),
        "caution" => Some("caution"),
        _ => None,
    }
}

pub(super) fn callout_icon_svg(kind: &str) -> &'static str {
    match kind {
        "note" => {
            r#"<svg viewBox="0 0 16 16" role="img" focusable="false"><path d="M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13Zm.75 10h-1.5v-5h1.5v5Zm0-6.25h-1.5v-1.5h1.5v1.5Z"/></svg>"#
        }
        "tip" => {
            r#"<svg viewBox="0 0 16 16" role="img" focusable="false"><path d="M8 1.25a5 5 0 0 0-3 9v2A1.75 1.75 0 0 0 6.75 14h2.5A1.75 1.75 0 0 0 11 12.25v-2a5 5 0 0 0-3-9Zm1.5 9.75h-3v-.75h3V11Zm.35-2H6.15a3.5 3.5 0 1 1 3.7 0Z"/></svg>"#
        }
        "important" => {
            r#"<svg viewBox="0 0 16 16" role="img" focusable="false"><path d="M8 1.5 14.5 8 8 14.5 1.5 8 8 1.5Zm.75 9.75h-1.5v1.5h1.5v-1.5Zm0-6.5h-1.5v5h1.5v-5Z"/></svg>"#
        }
        "warning" => {
            r#"<svg viewBox="0 0 16 16" role="img" focusable="false"><path d="M8 1.5 15 14H1L8 1.5Zm.75 10.25h-1.5v1.5h1.5v-1.5Zm0-5.5h-1.5v4.5h1.5v-4.5Z"/></svg>"#
        }
        "caution" => {
            r#"<svg viewBox="0 0 16 16" role="img" focusable="false"><path d="M5 1.75h6L14.25 5v6L11 14.25H5L1.75 11V5L5 1.75Zm3.75 9.5h-1.5v1.5h1.5v-1.5Zm0-6.5h-1.5v5h1.5v-5Z"/></svg>"#
        }
        _ => "",
    }
}

pub(super) fn default_callout_title(kind: &str) -> String {
    match known_callout_kind(kind) {
        Some("note") => "Note".to_string(),
        Some("tip") => "Tip".to_string(),
        Some("important") => "Important".to_string(),
        Some("warning") => "Warning".to_string(),
        Some("caution") => "Caution".to_string(),
        _ => {
            let mut chars = kind.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => "Callout".to_string(),
            }
        }
    }
}

pub(super) fn unique_heading_id(seen: &mut BTreeMap<String, usize>, text: &str) -> String {
    let base = heading_slug(text);
    let count = seen.entry(base.clone()).or_insert(0);
    *count += 1;
    if *count == 1 {
        base
    } else {
        format!("{base}-{count}")
    }
}

pub(super) fn heading_slug(text: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            for lower in ch.to_lowercase() {
                slug.push(lower);
            }
            pending_dash = false;
        } else if ch.is_whitespace() || matches!(ch, '-' | '_' | '.') {
            pending_dash = !slug.is_empty();
        }
    }
    if slug.is_empty() {
        "section".to_string()
    } else if slug.chars().all(|ch| ch.is_ascii_digit()) {
        format!("section-{slug}")
    } else {
        slug
    }
}
