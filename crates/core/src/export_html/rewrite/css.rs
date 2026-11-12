//! CSS selector, fragment and renderer-data-attribute rewriting.
use super::*;

pub(super) fn collect_css_classes(css: &str, maps: &mut ObfuscationMaps) {
    let bytes = css.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor] == b'.' {
            let start = cursor + 1;
            let end = css_ident_end(css, start);
            if end > start {
                let class_name = &css[start..end];
                if is_renderer_owned_class(class_name) {
                    maps.class_for(class_name);
                }
                cursor = end;
                continue;
            }
        }
        cursor += 1;
    }
}

pub(super) fn rewrite_css_refs(value: &str, maps: &ObfuscationMaps) -> String {
    let mut out = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'.' => {
                let start = cursor + 1;
                let end = css_ident_end(value, start);
                if end > start {
                    let class_name = &value[start..end];
                    out.push('.');
                    out.push_str(
                        maps.classes
                            .get(class_name)
                            .map(String::as_str)
                            .unwrap_or(class_name),
                    );
                    cursor = end;
                    continue;
                }
                out.push('.');
                cursor += 1;
            }
            b'#' => {
                let start = cursor + 1;
                let end = css_ident_end(value, start);
                if end > start {
                    let id = &value[start..end];
                    out.push('#');
                    out.push_str(maps.ids.get(id).map(String::as_str).unwrap_or(id));
                    cursor = end;
                    continue;
                }
                out.push('#');
                cursor += 1;
            }
            _ => {
                out.push(bytes[cursor] as char);
                cursor += 1;
            }
        }
    }
    out
}

pub(super) fn rewrite_url_fragment_refs(value: &str, maps: &ObfuscationMaps) -> String {
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(pos) = rest.find("url(#") {
        out.push_str(&rest[..pos + 5]);
        let after = &rest[pos + 5..];
        let Some(close) = after.find(')') else {
            out.push_str(after);
            return out;
        };
        let id = &after[..close];
        out.push_str(maps.ids.get(id).map(String::as_str).unwrap_or(id));
        out.push(')');
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

pub(super) fn css_ident_end(value: &str, start: usize) -> usize {
    let bytes = value.as_bytes();
    let mut cursor = start;
    while cursor < bytes.len()
        && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'-' || bytes[cursor] == b'_')
    {
        cursor += 1;
    }
    cursor
}

pub(super) fn strip_css_rules_with_oriha_data_attrs(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut cursor = 0usize;
    while let Some(pos) = css[cursor..].find("data-oriha-") {
        let absolute = cursor + pos;
        let rule_start = css[..absolute]
            .rfind('}')
            .map_or(0, |index| index + 1)
            .max(cursor);
        let Some(open_rel) = css[absolute..].find('{') else {
            out.push_str(&css[cursor..]);
            return out;
        };
        let open = absolute + open_rel;
        let Some(close) = matching_css_rule_end(css, open) else {
            out.push_str(&css[cursor..]);
            return out;
        };
        out.push_str(&css[cursor..rule_start]);
        cursor = close;
    }
    out.push_str(&css[cursor..]);
    out
}

pub(super) fn matching_css_rule_end(css: &str, open: usize) -> Option<usize> {
    let bytes = css.as_bytes();
    let mut cursor = open;
    let mut depth = 0usize;
    let mut quote = None::<u8>;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if let Some(quoted) = quote {
            if byte == quoted && bytes.get(cursor.wrapping_sub(1)) != Some(&b'\\') {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
        } else if byte == b'{' {
            depth += 1;
        } else if byte == b'}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(cursor + 1);
            }
        }
        cursor += 1;
    }
    None
}
