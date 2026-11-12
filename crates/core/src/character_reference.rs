//! CommonMark 0.31.2 section 2.5. Delimiters are checked before any decoding;
//! decoded characters are text and are never parsed again as Markdown syntax.
use std::borrow::Cow;

mod names;

#[cfg(test)]
thread_local! {
    static INSPECTED_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

mod access {
    #[inline]
    pub(super) fn byte(input: &[u8], index: usize) -> Option<u8> {
        input.get(index).copied()
    }
}

// Explicit test imports shadow the production default without runtime state.
#[allow(unused_imports)]
use access::*;
#[cfg(test)]
use instrumented::byte;

#[cfg(test)]
mod instrumented {
    pub(super) fn byte(input: &[u8], index: usize) -> Option<u8> {
        super::INSPECTED_BYTES.with(|count| count.set(count.get() + 1));
        super::access::byte(input, index)
    }
}

/// Decode one complete, semicolon-terminated reference at the start of input.
/// Lookahead is bounded by the vocabulary's maximum name or numeric digit count.
pub(crate) fn decode_at(input: &str) -> Option<(usize, Cow<'static, str>)> {
    let input = input.as_bytes();
    if byte(input, 0)? != b'&' {
        return None;
    }
    if byte(input, 1)? == b'#' {
        let (start, radix, max_digits) = match byte(input, 2)? {
            b'x' | b'X' => (3, 16, 6),
            _ => (2, 10, 7),
        };
        let mut end = start;
        let mut value = 0u32;
        while end - start < max_digits {
            let digit = match byte(input, end)? {
                b'0'..=b'9' => u32::from(input[end] - b'0'),
                b'a'..=b'f' if radix == 16 => u32::from(input[end] - b'a') + 10,
                b'A'..=b'F' if radix == 16 => u32::from(input[end] - b'A') + 10,
                _ => break,
            };
            value = value * radix + digit;
            end += 1;
        }
        if end == start || byte(input, end)? != b';' {
            return None;
        }
        let value = if value == 0 {
            '\u{fffd}'
        } else {
            char::from_u32(value).unwrap_or('\u{fffd}')
        };
        return Some((end + 1, Cow::Owned(value.to_string())));
    }
    let mut end = 1;
    while end <= names::MAX_NAME_BYTES {
        match byte(input, end)? {
            b';' => break,
            c if c.is_ascii_alphanumeric() => end += 1,
            _ => return None,
        }
    }
    if byte(input, end)? != b';' || end == 1 {
        return None;
    }
    let name = std::str::from_utf8(&input[1..end]).ok()?;
    let index = names::NAMES
        .binary_search_by(|(key, _)| key.cmp(&name))
        .ok()?;
    Some((end + 1, Cow::Borrowed(names::NAMES[index].1)))
}

pub(crate) fn decode_all(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut i = 0;
    while i < value.len() {
        if let Some((consumed, decoded)) = decode_at(&value[i..]) {
            out.push_str(&decoded);
            i += consumed;
        } else {
            let ch = value[i..].chars().next().expect("valid UTF-8");
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Decode link metadata and fenced info in one pass. An escaped ampersand
/// stays literal; neither an escape nor a reference can introduce new syntax.
pub(crate) fn decode_markdown(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut i = 0;
    while i < value.len() {
        if value.as_bytes()[i] == b'\\'
            && value
                .as_bytes()
                .get(i + 1)
                .is_some_and(u8::is_ascii_punctuation)
        {
            out.push(value.as_bytes()[i + 1] as char);
            i += 2;
        } else if let Some((consumed, decoded)) = decode_at(&value[i..]) {
            out.push_str(&decoded);
            i += consumed;
        } else {
            let ch = value[i..].chars().next().expect("valid UTF-8");
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_semicolon_vocabulary_matches_authoritative_input() {
        let values: serde_json::Value =
            serde_json::from_str(include_str!("../entity-data/whatwg-entities.json")).unwrap();
        let mut count = 0;
        for (name, value) in values.as_object().unwrap() {
            if !name.ends_with(';') {
                assert!(decode_at(name).is_none(), "{name}");
                continue;
            }
            count += 1;
            let (consumed, decoded) = decode_at(&format!("{name}suffix")).unwrap();
            assert_eq!(consumed, name.len(), "{name}");
            assert_eq!(decoded, value["characters"].as_str().unwrap(), "{name}");
        }
        assert_eq!(count, 2125);
    }

    #[test]
    fn numeric_grammar_and_unicode_replacement_follow_commonmark() {
        for (input, expected) in [
            ("&#64;", "@"),
            ("&#0000064;", "@"),
            ("&#x40;", "@"),
            ("&#X000040;", "@"),
            ("&#0;", "\u{fffd}"),
            ("&#x0;", "\u{fffd}"),
            ("&#1114111;", "\u{10ffff}"),
            ("&#x10FFFF;", "\u{10ffff}"),
            ("&#1114112;", "\u{fffd}"),
            ("&#x110000;", "\u{fffd}"),
            ("&#9999999;", "\u{fffd}"),
            ("&#xFFFFFF;", "\u{fffd}"),
            ("&#1;", "\u{1}"),
            ("&#128;", "\u{80}"),
        ] {
            assert_eq!(decode_all(input), expected, "{input}");
        }
        for input in [
            "&#+64;",
            "&#x+40;",
            "&#-64;",
            "&#x-40;",
            "&#00000064;",
            "&#000000064;",
            "&#x0000040;",
            "&#x00000040;",
            "&#;",
            "&#x;",
            "&#X;",
            "&# 64;",
            "&#x 40;",
            "&#6e1;",
            "&#x0x40;",
            "&#٦٤;",
            "&#６４;",
            "&#64",
            "&#x40",
        ] {
            assert_eq!(decode_all(input), input, "{input}");
        }
        for surrogate in 0xd800..=0xdfff {
            assert_eq!(decode_all(&format!("&#{surrogate};")), "\u{fffd}");
            assert_eq!(decode_all(&format!("&#x{surrogate:x};")), "\u{fffd}");
        }
    }

    #[test]
    fn names_are_case_sensitive_and_decoding_is_not_recursive() {
        assert_eq!(decode_all("&AElig;&aelig;&AELIG;"), "Ææ&AELIG;");
        assert_eq!(decode_all("&amp;commat;&amp;#64;"), "&commat;&#64;");
        assert_eq!(
            decode_all("&not-a-name;&notaname;&copy"),
            "&not-a-name;&notaname;&copy"
        );
    }

    #[test]
    fn invalid_ampersand_suffixes_have_bounded_lookahead() {
        for n in [1000, 2000, 4000, 8000, 16000] {
            for text in [
                "&".repeat(n) + ";",
                "&a".repeat(n) + ";",
                "&#".repeat(n) + ";",
                format!("&{};", "a".repeat(n)),
                format!("&#{};", "9".repeat(n)),
                format!("&#x{};", "F".repeat(n)),
            ] {
                INSPECTED_BYTES.with(|count| count.set(0));
                assert_eq!(decode_all(&text), text);
                let work = INSPECTED_BYTES.with(|count| count.get());
                assert!(
                    work <= 4 * text.len() + 40,
                    "{} bytes: {work} inspections",
                    text.len()
                );
            }
        }
    }
}
