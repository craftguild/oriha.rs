use crate::blocks::html_budget::{check_size, HtmlBuffer};

pub(crate) fn escape_text(value: &str) -> String {
    check_size(value.len());
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_html("&amp;"),
            '<' => escaped.push_html("&lt;"),
            '>' => escaped.push_html("&gt;"),
            _ => escaped.push_html_char(ch),
        }
    }
    escaped
}

pub(crate) fn escape_attr(value: &str) -> String {
    check_size(value.len());
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_html("&amp;"),
            '<' => escaped.push_html("&lt;"),
            '>' => escaped.push_html("&gt;"),
            '"' => escaped.push_html("&quot;"),
            '\'' => escaped.push_html("&#39;"),
            _ => escaped.push_html_char(ch),
        }
    }
    escaped
}
