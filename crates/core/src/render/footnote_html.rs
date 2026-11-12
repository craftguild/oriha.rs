use super::*;

pub(super) fn render_footnote_reference(
    label: &str,
    state: &mut RenderState,
    context: &RenderContext,
    out: &mut String,
) {
    if !context.footnotes.contains_key(label) {
        out.push_html("[^");
        out.push_html(&escape_text(label));
        out.push_html_char(']');
        return;
    }

    let number = if let Some(number) = state.footnotes.numbers.get(label).copied() {
        number
    } else {
        let number = state.footnotes.order.len() + 1;
        state.footnotes.numbers.insert(label.to_string(), number);
        state.footnotes.order.push(label.to_string());
        number
    };
    let ref_count = *state
        .footnotes
        .reference_counts
        .entry(label.to_string())
        .and_modify(|count| *count += 1)
        .or_insert(1);
    let (definition_id, ref_id) = state.footnote_ids.reference(label, ref_count);
    out.push_html(&format!(
        r##"<sup id="{ref_id}" class="oriha-footnote-ref"><a href="#{definition_id}" class="oriha-footnote-ref__link" data-oriha-footnote-ref="true">{number}</a></sup>"##
    ));
}

pub(super) fn render_footnote_section(
    context: &RenderContext,
    state: &mut RenderState,
    out: &mut String,
) {
    if state.footnotes.order.is_empty() {
        return;
    }
    if !out.ends_with('\n') {
        out.push_html_char('\n');
    }
    let title = render_message(context.locale, RenderMessage::FootnotesTitle);
    let title_id = state.footnote_ids.title();
    out.push_html(&format!(
        r#"<section class="oriha-footnotes" data-oriha-block-kind="footnotes" aria-labelledby="{title_id}"><h2 id="{title_id}" class="oriha-footnotes__title">{}</h2><ol class="oriha-footnotes__list">"#,
        escape_text(title)
    ));
    for label in state.footnotes.order.clone() {
        let Some(definition) = context.footnotes.get(&label) else {
            continue;
        };
        let number = state.footnotes.numbers.get(&label).copied().unwrap_or(0);
        let definition_id = state.footnote_ids.definition(&label);
        out.push_html(&format!(
            r#"<li id="{definition_id}" class="oriha-footnotes__item" value="{number}">"#
        ));
        render_inline_nodes_with_context(&definition.children, out, state, context);
        let backref_label = render_message(context.locale, RenderMessage::FootnoteBackrefLabel);
        let reference_id = state.footnote_ids.back_reference(&label);
        out.push_html(&format!(
            r##" <a href="#{reference_id}" class="oriha-footnotes__backref" aria-label="{}">&#8617;</a>"##,
            escape_attr(backref_label)
        ));
        out.push_html("</li>");
    }
    out.push_html("</ol></section>");
}
