//! Coordinated export HTML rewriting and obfuscation maps.
mod css;
#[allow(unused_imports)]
use css::*;

use std::collections::{BTreeMap, BTreeSet};

use crate::html::escape_attr;
use crate::types::HtmlExportOptions;

use super::html_scan::{parse_tag_at, ParsedTag};

pub(crate) fn rewrite_artifacts(
    mut html: String,
    mut css: String,
    options: &HtmlExportOptions,
) -> (String, String) {
    if options.obfuscate || options.strip_oriha_data_attributes {
        let maps = ObfuscationMaps::build(&html, &css, options);
        html = rewrite_html(&html, &maps, options);
        if options.obfuscate {
            css = rewrite_css_refs(&css, &maps);
        }
    }

    if options.strip_oriha_data_attributes {
        css = strip_css_rules_with_oriha_data_attrs(&css);
    }

    (html, css)
}

#[derive(Default)]
struct ObfuscationMaps {
    ids: BTreeMap<String, String>,
    classes: BTreeMap<String, String>,
    preserve_ids: BTreeSet<String>,
}

impl ObfuscationMaps {
    fn build(html: &str, css: &str, options: &HtmlExportOptions) -> Self {
        let mut maps = Self::default();
        let mut cursor = 0usize;
        let mut svg_depth = 0usize;
        let mut raw_div_depth = 0usize;

        while let Some(start_rel) = html[cursor..].find('<') {
            let start = cursor + start_rel;
            let Some((end, tag)) = parse_tag_at(html, start) else {
                break;
            };
            cursor = end;
            let Some(tag) = tag else {
                continue;
            };

            if raw_div_depth > 0 {
                update_raw_div_depth(&tag, &mut raw_div_depth);
                continue;
            }

            if tag.closing {
                if tag.name == "svg" {
                    svg_depth = svg_depth.saturating_sub(1);
                }
                continue;
            }

            if tag.name == "svg" {
                svg_depth += 1;
            }

            if options.preserve_heading_anchors
                && matches!(tag.name.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6")
                && tag_has_class(&tag, "oriha-heading")
            {
                if let Some(id) = tag.attr_value("id") {
                    maps.preserve_ids.insert(id.to_string());
                }
            }

            if let Some(class_value) = tag.attr_value("class") {
                for class_name in class_value.split_ascii_whitespace() {
                    if is_renderer_owned_class(class_name) {
                        maps.class_for(class_name);
                    }
                }
            }

            if let Some(id) = tag.attr_value("id") {
                if !maps.preserve_ids.contains(id)
                    && should_mangle_id(id, svg_depth > 0 || tag.name == "svg")
                {
                    maps.id_for(id);
                }
            }

            if tag_has_class(&tag, "oriha-foreign-html-wrapper")
                || tag_has_class(&tag, "oriha-foreign-html")
            {
                raw_div_depth = 1;
            }
        }

        collect_css_classes(css, &mut maps);
        maps
    }

    fn id_for(&mut self, value: &str) -> String {
        if let Some(mapped) = self.ids.get(value) {
            return mapped.clone();
        }
        let mapped = format!("e{}", self.ids.len() + 1);
        self.ids.insert(value.to_string(), mapped.clone());
        mapped
    }

    fn class_for(&mut self, value: &str) -> String {
        if let Some(mapped) = self.classes.get(value) {
            return mapped.clone();
        }
        let mapped = format!("c{}", self.classes.len() + 1);
        self.classes.insert(value.to_string(), mapped.clone());
        mapped
    }
}

fn rewrite_html(html: &str, maps: &ObfuscationMaps, options: &HtmlExportOptions) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0usize;
    let mut raw_div_depth = 0usize;

    while let Some(start_rel) = html[cursor..].find('<') {
        let start = cursor + start_rel;
        out.push_str(&html[cursor..start]);
        let Some((end, tag)) = parse_tag_at(html, start) else {
            out.push_str(&html[start..]);
            return out;
        };
        cursor = end;
        let Some(tag) = tag else {
            out.push_str(&html[start..end]);
            continue;
        };

        if raw_div_depth > 0 {
            out.push_str(tag.source);
            update_raw_div_depth(&tag, &mut raw_div_depth);
            continue;
        }

        if tag.closing {
            out.push_str(tag.source);
            continue;
        }

        let starts_raw = tag_has_class(&tag, "oriha-foreign-html-wrapper")
            || tag_has_class(&tag, "oriha-foreign-html");
        out.push_str(&rewrite_tag(&tag, maps, options));
        if starts_raw && !tag.self_closing {
            raw_div_depth = 1;
        }
    }
    out.push_str(&html[cursor..]);
    out
}

fn rewrite_tag(tag: &ParsedTag<'_>, maps: &ObfuscationMaps, options: &HtmlExportOptions) -> String {
    let mut out = String::new();
    out.push('<');
    out.push_str(&tag.raw_name);
    for attr in &tag.attrs {
        let name = attr.name.to_ascii_lowercase();
        if options.strip_oriha_data_attributes && name.starts_with("data-oriha-") {
            continue;
        }
        let value = attr.value.as_deref().map(|value| {
            if options.obfuscate {
                rewrite_attr_value(&name, value, maps)
            } else {
                value.to_string()
            }
        });
        out.push(' ');
        out.push_str(&attr.name);
        if let Some(value) = value {
            out.push_str("=\"");
            if attr.value.as_deref() == Some(value.as_str()) {
                out.push_str(&value);
            } else {
                out.push_str(&escape_attr(&value));
            }
            out.push('"');
        }
    }
    if tag.self_closing {
        out.push_str(" />");
    } else {
        out.push('>');
    }
    out
}

fn rewrite_attr_value(name: &str, value: &str, maps: &ObfuscationMaps) -> String {
    match name {
        "id" => maps
            .ids
            .get(value)
            .cloned()
            .unwrap_or_else(|| value.to_string()),
        "class" => value
            .split_ascii_whitespace()
            .map(|class_name| {
                maps.classes
                    .get(class_name)
                    .map(String::as_str)
                    .unwrap_or(class_name)
            })
            .collect::<Vec<_>>()
            .join(" "),
        "href" => rewrite_hash_href(value, maps),
        "aria-labelledby"
        | "aria-describedby"
        | "aria-controls"
        | "aria-owns"
        | "aria-activedescendant" => value
            .split_ascii_whitespace()
            .map(|id| maps.ids.get(id).map(String::as_str).unwrap_or(id))
            .collect::<Vec<_>>()
            .join(" "),
        "for" | "list" => maps
            .ids
            .get(value)
            .cloned()
            .unwrap_or_else(|| value.to_string()),
        "marker-start" | "marker-mid" | "marker-end" | "fill" | "stroke" | "filter" | "mask"
        | "clip-path" | "style" => rewrite_css_refs(value, maps),
        _ => rewrite_url_fragment_refs(value, maps),
    }
}

fn rewrite_hash_href(value: &str, maps: &ObfuscationMaps) -> String {
    let Some(id) = value.strip_prefix('#') else {
        return value.to_string();
    };
    maps.ids
        .get(id)
        .map(|mapped| format!("#{mapped}"))
        .unwrap_or_else(|| value.to_string())
}

fn update_raw_div_depth(tag: &ParsedTag<'_>, depth: &mut usize) {
    if tag.name != "div" {
        return;
    }
    if tag.closing {
        *depth = depth.saturating_sub(1);
    } else if !tag.self_closing {
        *depth += 1;
    }
}

fn tag_has_class(tag: &ParsedTag<'_>, class_name: &str) -> bool {
    tag.attr_value("class").is_some_and(|value| {
        value
            .split_ascii_whitespace()
            .any(|item| item == class_name)
    })
}

fn should_mangle_id(value: &str, in_svg: bool) -> bool {
    in_svg
        || value.chars().all(|ch| ch.is_ascii_digit())
        || value.starts_with("oriha-")
        || value.starts_with("fn-")
        || value.starts_with("fnref-")
        || value.starts_with("mermaid-")
        || value.starts_with("flowchart-")
        || value.starts_with("plantuml-")
}

fn is_renderer_owned_class(value: &str) -> bool {
    value.starts_with("oriha-")
        || value.starts_with("language-")
        || value == "token"
        || is_syntax_token_class(value)
}

fn is_syntax_token_class(value: &str) -> bool {
    matches!(
        value,
        "comment"
            | "prolog"
            | "doctype"
            | "cdata"
            | "punctuation"
            | "property"
            | "tag"
            | "boolean"
            | "number"
            | "constant"
            | "symbol"
            | "deleted"
            | "selector"
            | "attr-name"
            | "string"
            | "char"
            | "builtin"
            | "inserted"
            | "operator"
            | "entity"
            | "url"
            | "atrule"
            | "attr-value"
            | "section"
            | "keyword"
            | "function"
            | "class-name"
            | "regex"
            | "important"
            | "variable"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refs_are_rewritten_with_mangled_svg_ids() {
        let (html, css) = rewrite_artifacts(
            r##"<article class="oriha-document"><svg><defs><marker id="arrow"></marker></defs><path marker-end="url(#arrow)" aria-labelledby="arrow"></path></svg></article>"##.to_string(),
            ".oriha-document svg #arrow { fill: red; }".to_string(),
            &HtmlExportOptions::default(),
        );

        assert!(!html.contains("id=\"arrow\""));
        assert!(html.contains("marker-end=\"url(#e1)\""));
        assert!(html.contains("aria-labelledby=\"e1\""));
        assert!(css.contains("#e1"));
    }
}
