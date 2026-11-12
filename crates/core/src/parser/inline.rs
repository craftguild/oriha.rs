use crate::character_reference::decode_at as decode_html_entity_at;
pub(crate) use crate::character_reference::{decode_all as decode_html_entities, decode_markdown};
use crate::emoji::resolve_shortcode;
use crate::ir::{InlineKind, InlineNode, LinkReference};
use crate::mapping::{push_mapping, MappedText, TextProvenance};
use crate::types::FeatureFlags;
use std::collections::BTreeMap;

pub(crate) fn parse_inlines(
    source: &str,
    link_references: &BTreeMap<String, LinkReference>,
    features: &FeatureFlags,
) -> Vec<InlineNode> {
    parse_mapped_inlines(&MappedText::identity(source, 0), link_references, features)
}

pub(crate) fn parse_mapped_inlines(
    mapped: &MappedText,
    link_references: &BTreeMap<String, LinkReference>,
    features: &FeatureFlags,
) -> Vec<InlineNode> {
    let _depth = recursion::enter();
    super::probe_inline_parse();
    parse_inline_tokens(mapped, link_references, features)
}

fn parse_inline_tokens(
    mapped: &MappedText,
    link_references: &BTreeMap<String, LinkReference>,
    features: &FeatureFlags,
) -> Vec<InlineNode> {
    let source = mapped.text.as_str();
    let bytes = source.as_bytes();
    let emphasis_runs = collect_emphasis_runs(source, link_references);
    let strikethrough_runs = if features.gfm {
        collect_strikethrough_runs(source)
    } else {
        BTreeMap::new()
    };
    let mut out = InlineBuilder::new(mapped);
    let mut autolinks = ExtendedAutolinks::new(source);
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'\\' {
            if i + 1 >= bytes.len() {
                push_text_token(&mut out, mapped.slice(i..i + 1));
                i += 1;
                continue;
            }
            if bytes[i + 1] == b'\n' {
                out.push(InlineToken::Node(node_at(
                    InlineKind::HardBreak,
                    mapped,
                    i..i + 2,
                )));
                i += 2;
                continue;
            }
            let next = source[i + 1..].chars().next().expect("valid escaped char");
            if is_ascii_punctuation(next) {
                push_text_token(
                    &mut out,
                    mapped.transformed(
                        i..i + 1 + next.len_utf8(),
                        &next.to_string(),
                        TextProvenance::Escape,
                    ),
                );
                i += 1 + next.len_utf8();
                continue;
            }
            push_text_token(&mut out, mapped.slice(i..i + 1));
            i += 1;
            continue;
        }

        if bytes[i] == b'&' {
            if let Some((consumed, decoded)) = decode_html_entity_at(&source[i..]) {
                push_text_token(
                    &mut out,
                    mapped.transformed(
                        i..i + consumed,
                        &decoded,
                        TextProvenance::CharacterReference,
                    ),
                );
                i += consumed;
                continue;
            }
        }

        if bytes[i] == b':' {
            if let Some((consumed, shortcode, emoji)) = parse_emoji_shortcode(&source[i..]) {
                out.push(InlineToken::Node(node_at(
                    InlineKind::Emoji { shortcode, emoji },
                    mapped,
                    i..i + consumed,
                )));
                i += consumed;
                continue;
            }
        }

        if bytes[i] == b'<' {
            if let Some((consumed, node)) =
                parse_autolink_or_inline_html(&source[i..], features.gfm)
            {
                out.push(InlineToken::Node(autolink_at(node, mapped, i, consumed, 1)));
                i += consumed;
                continue;
            }
        }

        if bytes[i] == b'\n' {
            let hard_break_spaces = source[..i]
                .chars()
                .rev()
                .take_while(|ch| *ch == ' ')
                .count();
            trim_trailing_spaces(&mut out);
            if hard_break_spaces >= 2 {
                out.push(InlineToken::Node(node_at(
                    InlineKind::HardBreak,
                    mapped,
                    i..i + 1,
                )));
            } else {
                out.push(InlineToken::Node(node_at(
                    InlineKind::SoftBreak,
                    mapped,
                    i..i + 1,
                )));
            }
            i += 1;
            continue;
        }

        if bytes[i] == b'*'
            && i + 1 < bytes.len()
            && bytes[i + 1] == b'['
            && (parse_link_or_image_with_options(&source[i + 1..], true, link_references).is_some()
                || parse_reference_link_or_image(&source[i + 1..], link_references, true).is_some())
        {
            push_text_token(&mut out, mapped.slice(i..i + 1));
            i += 1;
            continue;
        }

        if features.gfm && bytes[i] == b'~' {
            if let Some((delimiter, len)) = strikethrough_runs.get(&i) {
                out.push(InlineToken::Delimiter(*delimiter, i..i + len));
                i += len;
                continue;
            }
        }

        if matches!(bytes[i], b'*' | b'_') {
            if let Some(run) = emphasis_runs.get(&i) {
                push_emphasis_run(&mut out, run, mapped, i);
                i += run.len;
                continue;
            }
        }

        if bytes[i] == b'`' && (i == 0 || bytes[i - 1] != b'`') {
            if let Some((consumed, code)) = parse_code_span(&source[i..]) {
                out.push(InlineToken::Node(node_at(
                    InlineKind::Code(code),
                    mapped,
                    i..i + consumed,
                )));
                i += consumed;
                continue;
            }
        }

        if features.math && bytes[i] == b'$' {
            if let Some((consumed, math_source)) = parse_inline_math(source, i) {
                out.push(InlineToken::Node(node_at(
                    InlineKind::Math {
                        source: math_source,
                    },
                    mapped,
                    i..i + consumed,
                )));
                i += consumed;
                continue;
            }
        }

        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            if let Some((consumed, alt, destination, title)) =
                parse_link_or_image_with_options(&source[i + 1..], false, link_references)
            {
                out.push(InlineToken::Node(node_at(
                    InlineKind::Image {
                        destination,
                        title,
                        alt: parse_child_inlines(
                            out.depth(),
                            &mapped_subslice(mapped, alt),
                            link_references,
                            features,
                        ),
                    },
                    mapped,
                    i..i + consumed + 1,
                )));
                i += consumed + 1;
                continue;
            }
            if let Some((consumed, alt, reference)) =
                parse_reference_link_or_image(&source[i + 1..], link_references, false)
            {
                out.push(InlineToken::Node(node_at(
                    InlineKind::Image {
                        destination: reference.destination,
                        title: reference.title,
                        alt: parse_child_inlines(
                            out.depth(),
                            &mapped_subslice(mapped, alt),
                            link_references,
                            features,
                        ),
                    },
                    mapped,
                    i..i + consumed + 1,
                )));
                i += consumed + 1;
                continue;
            }
        }

        if bytes[i] == b'[' {
            if features.gfm {
                if let Some((consumed, label)) = parse_footnote_reference(&source[i..]) {
                    out.push(InlineToken::Node(node_at(
                        InlineKind::FootnoteReference { label },
                        mapped,
                        i..i + consumed,
                    )));
                    i += consumed;
                    continue;
                }
            }
            if let Some((consumed, text, destination, title)) =
                parse_link_or_image_with_options(&source[i..], true, link_references)
            {
                out.push(InlineToken::Node(node_at(
                    InlineKind::Link {
                        destination,
                        title,
                        children: parse_child_inlines(
                            out.depth(),
                            &mapped_subslice(mapped, text),
                            link_references,
                            features,
                        ),
                    },
                    mapped,
                    i..i + consumed,
                )));
                i += consumed;
                continue;
            }
            if let Some((consumed, text, reference)) =
                parse_reference_link_or_image(&source[i..], link_references, true)
            {
                out.push(InlineToken::Node(node_at(
                    InlineKind::Link {
                        destination: reference.destination,
                        title: reference.title,
                        children: parse_child_inlines(
                            out.depth(),
                            &mapped_subslice(mapped, text),
                            link_references,
                            features,
                        ),
                    },
                    mapped,
                    i..i + consumed,
                )));
                i += consumed;
                continue;
            }
        }

        if features.gfm
            && (i == 0 || is_extended_autolink_boundary(source[..i].chars().next_back()))
        {
            if let Some((consumed, node)) = autolinks.parse_extended_autolink(i) {
                out.push(InlineToken::Node(autolink_at(node, mapped, i, consumed, 0)));
                i += consumed;
                continue;
            }
        }

        let ch = source[i..].chars().next().expect("valid char");
        // Materialize a literal run once instead of allocating a String, mapping
        // vector and source-range vector for every ordinary character. Stop at
        // every syntax trigger and potential extended-autolink start.
        let mut end = i + ch.len_utf8();
        let mut previous = ch;
        while end < source.len() {
            let next = source[end..].chars().next().expect("valid char");
            if matches!(
                next,
                '\\' | '&' | ':' | '<' | '\n' | '*' | '_' | '~' | '`' | '$' | '!' | '['
            ) || (features.gfm && is_extended_autolink_boundary(Some(previous)))
            {
                break;
            }
            end += next.len_utf8();
            previous = next;
        }
        push_text_token(&mut out, mapped.slice(i..end));
        i = end;
    }

    out.finish()
}

mod builder;
use builder::*;
mod leaf_syntax;
use leaf_syntax::*;
mod html_syntax;
use html_syntax::*;
mod delimiters;
use delimiters::*;
mod autolinks;
use autolinks::*;
mod links;
use links::*;
mod recursion;
#[cfg(test)]
mod tests;

fn parse_child_inlines(
    depth: usize,
    mapped: &MappedText,
    references: &BTreeMap<String, LinkReference>,
    features: &FeatureFlags,
) -> Vec<InlineNode> {
    let _parent = recursion::parent_formatting(depth);
    parse_mapped_inlines(mapped, references, features)
}
