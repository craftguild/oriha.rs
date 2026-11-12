//! Quoted text boundaries, aliases and shared lexical values.

pub(super) fn split_first_word(input: &str) -> (&str, &str) {
    let input = input.trim();
    input
        .find(char::is_whitespace)
        .map(|index| (&input[..index], &input[index..]))
        .unwrap_or((input, ""))
}

pub(super) fn strip_keyword<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let (found, rest) = split_first_word(line);
    found.eq_ignore_ascii_case(keyword).then_some(rest)
}

pub(super) fn take_value(input: &str) -> Option<(String, &str)> {
    let input = input.trim_start();
    if input.is_empty() {
        return None;
    }

    if let Some(rest) = input.strip_prefix('"') {
        let mut escaped = false;
        for (index, character) in rest.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == '"' {
                let value = decode_plantuml_text(&rest[..index].replace("\\\"", "\""));
                return Some((value, &rest[index + 1..]));
            }
        }
        return None;
    }

    let end = input.find(char::is_whitespace).unwrap_or(input.len());
    Some((decode_plantuml_text(&input[..end]), &input[end..]))
}

pub(super) fn split_as_alias(input: &str) -> Option<(&str, &str)> {
    find_word_outside_quotes(input, "as").map(|index| {
        let after = index + "as".len();
        (&input[..index], &input[after..])
    })
}

pub(super) fn find_word_outside_quotes(input: &str, word: &str) -> Option<usize> {
    let mut in_quotes = false;
    let mut escaped = false;

    for (index, character) in input.char_indices() {
        if in_quotes && escaped {
            escaped = false;
            continue;
        }
        if in_quotes && character == '\\' {
            escaped = true;
            continue;
        }
        if character == '"' {
            in_quotes = !in_quotes;
            continue;
        }
        let word_end = index + word.len();
        if in_quotes || word_end > input.len() || !input[index..word_end].eq_ignore_ascii_case(word)
        {
            continue;
        }

        let before = input[..index].chars().next_back();
        let after = input[index + word.len()..].chars().next();
        if before.is_some_and(char::is_whitespace) && after.is_some_and(char::is_whitespace) {
            return Some(index);
        }
    }

    None
}

pub(super) fn find_outside_quotes(input: &str, needle: &str) -> Option<usize> {
    let mut in_quotes = false;
    let mut escaped = false;

    for (index, character) in input.char_indices() {
        if in_quotes && escaped {
            escaped = false;
            continue;
        }
        if in_quotes && character == '\\' {
            escaped = true;
            continue;
        }
        if character == '"' {
            in_quotes = !in_quotes;
            continue;
        }
        if !in_quotes && input[index..].starts_with(needle) {
            return Some(index);
        }
    }

    None
}

pub(super) fn unquote_or_trimmed(input: &str) -> String {
    take_value(input)
        .filter(|(_, trailing)| trailing.trim().is_empty())
        .map(|(value, _)| value)
        .unwrap_or_else(|| input.trim().to_owned())
}

pub(super) fn decode_plantuml_text(input: &str) -> String {
    input.replace("\\n", "\n")
}

pub(super) fn non_empty(input: &str) -> Option<&str> {
    (!input.is_empty()).then_some(input)
}
