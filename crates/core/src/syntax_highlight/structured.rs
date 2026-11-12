//! JSON, YAML, TOML, Markdown and diff line tokenization.
use super::*;

pub(super) fn tokenize_json_line(line: &str, language: Language) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < line.len() {
        let rest = &line[index..];
        if language == Language::Jsonc && rest.starts_with("//") {
            push_token(&mut tokens, TokenKind::Comment, &line[index..]);
            break;
        }
        let Some(character) = rest.chars().next() else {
            break;
        };
        if character == '"' {
            let end = quoted_string_end(line, index, character);
            let token_kind = if next_non_space_char(line, end) == Some(':') {
                TokenKind::Property
            } else {
                TokenKind::String
            };
            push_token(&mut tokens, token_kind, &line[index..end]);
            index = end;
            continue;
        }
        if character == '-' || character.is_ascii_digit() {
            let end = number_end(line, index);
            push_token(&mut tokens, TokenKind::Number, &line[index..end]);
            index = end;
            continue;
        }
        if is_identifier_start(character) {
            let end = identifier_end(line, index);
            let word = &line[index..end];
            if matches!(word, "true" | "false" | "null") {
                push_token(&mut tokens, TokenKind::Constant, word);
            } else {
                push_plain(&mut tokens, word);
            }
            index = end;
            continue;
        }
        if is_punctuation_character(character) || character == ':' {
            push_token(
                &mut tokens,
                TokenKind::Punctuation,
                &line[index..index + character.len_utf8()],
            );
            index += character.len_utf8();
            continue;
        }
        push_plain(&mut tokens, &line[index..index + character.len_utf8()]);
        index += character.len_utf8();
    }
    tokens
}

pub(super) fn tokenize_yaml_line(line: &str) -> Vec<Token> {
    if let Some(comment_start) = find_unquoted_comment(line, '#') {
        let mut tokens = tokenize_yaml_line(&line[..comment_start]);
        push_token(&mut tokens, TokenKind::Comment, &line[comment_start..]);
        return tokens;
    }

    let mut tokens = Vec::new();
    let trimmed_start = line.trim_start();
    let leading = line.len() - trimmed_start.len();
    push_plain(&mut tokens, &line[..leading]);
    let mut index = leading;

    if line[index..].starts_with('-') {
        push_token(&mut tokens, TokenKind::Punctuation, "-");
        index += 1;
    }

    if let Some(colon) = yaml_key_colon(line, index) {
        let key = line[index..colon].trim_end();
        let trailing = colon - index - key.len();
        if !key.is_empty() {
            push_token(&mut tokens, TokenKind::Property, key);
        }
        push_plain(&mut tokens, &line[colon - trailing..colon]);
        push_token(&mut tokens, TokenKind::Punctuation, ":");
        tokenize_scalar_into(&mut tokens, &line[colon + 1..]);
    } else {
        tokenize_scalar_into(&mut tokens, &line[index..]);
    }
    tokens
}

pub(super) fn tokenize_toml_line(line: &str) -> Vec<Token> {
    if let Some(comment_start) = find_unquoted_comment(line, '#') {
        let mut tokens = tokenize_toml_line(&line[..comment_start]);
        push_token(&mut tokens, TokenKind::Comment, &line[comment_start..]);
        return tokens;
    }

    let mut tokens = Vec::new();
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();
    push_plain(&mut tokens, &line[..leading]);
    if trimmed.starts_with('[') {
        let end = trimmed
            .find(']')
            .map_or(line.len(), |position| leading + position + 1);
        push_token(&mut tokens, TokenKind::Section, &line[leading..end]);
        push_plain(&mut tokens, &line[end..]);
        return tokens;
    }
    if let Some(equal) = line[leading..].find('=') {
        let equal = leading + equal;
        let key_end = leading + line[leading..equal].trim_end().len();
        push_token(&mut tokens, TokenKind::Property, &line[leading..key_end]);
        push_plain(&mut tokens, &line[key_end..equal]);
        push_token(&mut tokens, TokenKind::Operator, "=");
        tokenize_scalar_into(&mut tokens, &line[equal + 1..]);
    } else {
        tokenize_scalar_into(&mut tokens, &line[leading..]);
    }
    tokens
}

pub(super) fn tokenize_markdown_line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let trimmed = line.trim_start();
    let leading = line.len() - trimmed.len();
    if trimmed.starts_with('#') {
        push_plain(&mut tokens, &line[..leading]);
        let marker_end = leading + trimmed.chars().take_while(|ch| *ch == '#').count();
        push_token(&mut tokens, TokenKind::Keyword, &line[leading..marker_end]);
        push_plain(&mut tokens, &line[marker_end..]);
    } else if trimmed.starts_with('>') || trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        push_plain(&mut tokens, &line[..leading]);
        push_token(
            &mut tokens,
            TokenKind::Punctuation,
            &line[leading..leading + 1],
        );
        push_plain(&mut tokens, &line[leading + 1..]);
    } else {
        tokenize_inline_markup_into(&mut tokens, line);
    }
    tokens
}

pub(super) fn tokenize_diff_line(line: &str) -> Vec<Token> {
    if line.starts_with("@@") {
        vec![token(TokenKind::AttrName, line)]
    } else if line.starts_with('+') {
        vec![token(TokenKind::Inserted, line)]
    } else if line.starts_with('-') {
        vec![token(TokenKind::Deleted, line)]
    } else if line.starts_with("diff ") || line.starts_with("index ") {
        vec![token(TokenKind::Keyword, line)]
    } else {
        vec![plain_token(line)]
    }
}

pub(super) fn tokenize_scalar_into(tokens: &mut Vec<Token>, value: &str) {
    let mut index = 0;
    while index < value.len() {
        let rest = &value[index..];
        let Some(character) = rest.chars().next() else {
            break;
        };
        if character == '"' || character == '\'' {
            let end = quoted_string_end(value, index, character);
            push_token(tokens, TokenKind::String, &value[index..end]);
            index = end;
            continue;
        }
        if character == '-' || character.is_ascii_digit() {
            let end = number_end(value, index);
            push_token(tokens, TokenKind::Number, &value[index..end]);
            index = end;
            continue;
        }
        if is_identifier_start(character) {
            let end = identifier_end(value, index);
            let word = &value[index..end];
            if matches!(word, "true" | "false" | "null" | "True" | "False" | "None") {
                push_token(tokens, TokenKind::Constant, word);
            } else {
                push_plain(tokens, word);
            }
            index = end;
            continue;
        }
        push_plain(tokens, &value[index..index + character.len_utf8()]);
        index += character.len_utf8();
    }
}

pub(super) fn tokenize_inline_markup_into(tokens: &mut Vec<Token>, line: &str) {
    let mut index = 0;
    while index < line.len() {
        let rest = &line[index..];
        if let Some(stripped) = rest.strip_prefix('`') {
            let end = stripped
                .find('`')
                .map_or(line.len(), |position| index + position + 2);
            push_token(tokens, TokenKind::String, &line[index..end]);
            index = end;
            continue;
        }
        if rest.starts_with('[') {
            let end = rest
                .find(')')
                .map_or(index + 1, |position| index + position + 1);
            push_token(tokens, TokenKind::Url, &line[index..end]);
            index = end;
            continue;
        }
        let Some(character) = rest.chars().next() else {
            break;
        };
        push_plain(tokens, &line[index..index + character.len_utf8()]);
        index += character.len_utf8();
    }
}

pub(super) fn find_unquoted_comment(line: &str, marker: char) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if Some(character) == quote {
            quote = None;
            continue;
        }
        if quote.is_none() && (character == '"' || character == '\'') {
            quote = Some(character);
            continue;
        }
        if quote.is_none() && character == marker {
            return Some(index);
        }
    }
    None
}

pub(super) fn yaml_key_colon(line: &str, start: usize) -> Option<usize> {
    let colon = line[start..].find(':')? + start;
    let key = line[start..colon].trim();
    (!key.is_empty() && !key.contains(' ')).then_some(colon)
}
