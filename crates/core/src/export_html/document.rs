pub(crate) fn standalone_document(body: &str, css: &str) -> String {
    let mut html = String::with_capacity(body.len() + css.len() + 96);
    html.push_str("<!doctype html><html><head><meta charset=\"utf-8\">");
    if !css.trim().is_empty() {
        html.push_str("<style>");
        html.push_str(&css.replace("</style", "<\\/style"));
        html.push_str("</style>");
    }
    html.push_str("</head><body>");
    html.push_str(body);
    html.push_str("</body></html>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standalone_document_escapes_style_end_tag() {
        assert!(standalone_document("", "a{content:\"</style\"}").contains("<\\/style"));
    }
}
