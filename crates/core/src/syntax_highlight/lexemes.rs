//! Shared string, number, identifier and punctuation boundaries.
use super::*;

pub(super) fn quoted_string_end(line: &str, start: usize, quote: char) -> usize {
    let mut escaped = false;
    let mut index = start + quote.len_utf8();
    while index < line.len() {
        let Some(character) = line[index..].chars().next() else {
            break;
        };
        index += character.len_utf8();
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == quote {
            break;
        }
    }
    index
}

pub(super) fn number_end(line: &str, start: usize) -> usize {
    let mut index = start;
    while index < line.len() {
        let Some(character) = line[index..].chars().next() else {
            break;
        };
        if character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-' | '+') {
            index += character.len_utf8();
        } else {
            break;
        }
    }
    index
}

pub(super) fn identifier_end_for_language(line: &str, start: usize, language: Language) -> usize {
    if language == Language::Css {
        css_identifier_end(line, start)
    } else {
        identifier_end(line, start)
    }
}

pub(super) fn identifier_end(line: &str, start: usize) -> usize {
    let mut index = start;
    while index < line.len() {
        let Some(character) = line[index..].chars().next() else {
            break;
        };
        if is_identifier_continue(character) {
            index += character.len_utf8();
        } else {
            break;
        }
    }
    index
}

pub(super) fn css_identifier_end(line: &str, start: usize) -> usize {
    let mut index = start;
    while index < line.len() {
        let Some(character) = line[index..].chars().next() else {
            break;
        };
        if is_identifier_continue(character) || character == '-' {
            index += character.len_utf8();
        } else {
            break;
        }
    }
    index
}

pub(super) fn next_non_space_char(line: &str, index: usize) -> Option<char> {
    line[index..]
        .chars()
        .find(|character| !character.is_whitespace())
}

pub(super) fn skip_whitespace(line: &str, mut index: usize) -> usize {
    while index < line.len() {
        let Some(character) = line[index..].chars().next() else {
            break;
        };
        if !character.is_whitespace() {
            break;
        }
        index += character.len_utf8();
    }
    index
}

pub(super) fn is_code_quote(character: char, language: Language) -> bool {
    match language {
        Language::Javascript => matches!(character, '\'' | '"' | '`'),
        Language::Rust
        | Language::Python
        | Language::Css
        | Language::Shell
        | Language::Sql
        | Language::PlantUml => matches!(character, '\'' | '"'),
        _ => false,
    }
}

pub(super) fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_' || character == '@'
}

pub(super) fn is_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '@')
}

pub(super) fn is_operator_character(character: char) -> bool {
    matches!(
        character,
        '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~' | '?' | ':'
    )
}

pub(super) fn is_punctuation_character(character: char) -> bool {
    matches!(
        character,
        '{' | '}' | '[' | ']' | '(' | ')' | ',' | ';' | '.'
    )
}
