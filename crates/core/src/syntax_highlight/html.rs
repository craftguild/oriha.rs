//! HTML tag, attribute and markup tokenization.
use super::*;

pub(super) fn tokenize_html_line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < line.len() {
        let rest = &line[index..];
        if rest.starts_with("<!--") {
            let end = rest
                .find("-->")
                .map_or(line.len(), |position| index + position + "-->".len());
            push_token(&mut tokens, TokenKind::Comment, &line[index..end]);
            index = end;
            continue;
        }
        if rest.starts_with("<!") || rest.starts_with("<?") {
            let end = rest
                .find('>')
                .map_or(line.len(), |position| index + position + 1);
            push_token(&mut tokens, TokenKind::Doctype, &line[index..end]);
            index = end;
            continue;
        }
        if rest.starts_with('<') {
            index = tokenize_html_tag(&mut tokens, line, index);
            continue;
        }

        let Some(character) = rest.chars().next() else {
            break;
        };
        push_plain(&mut tokens, &line[index..index + character.len_utf8()]);
        index += character.len_utf8();
    }
    tokens
}

pub(super) fn tokenize_html_tag(tokens: &mut Vec<Token>, line: &str, mut index: usize) -> usize {
    push_token(tokens, TokenKind::Punctuation, &line[index..index + 1]);
    index += 1;

    if line[index..].starts_with('/') {
        push_token(tokens, TokenKind::Punctuation, &line[index..index + 1]);
        index += 1;
    }

    if let Some(character) = line[index..].chars().next() {
        if is_identifier_start(character) {
            let end = html_name_end(line, index);
            push_token(tokens, TokenKind::Tag, &line[index..end]);
            index = end;
        }
    }

    while index < line.len() {
        let rest = &line[index..];
        let Some(character) = rest.chars().next() else {
            break;
        };
        if character == '>' {
            push_token(
                tokens,
                TokenKind::Punctuation,
                &line[index..index + character.len_utf8()],
            );
            return index + character.len_utf8();
        }
        if rest.starts_with("/>") {
            push_token(tokens, TokenKind::Punctuation, "/>");
            return index + "/>".len();
        }
        if character.is_whitespace() {
            push_plain(tokens, &line[index..index + character.len_utf8()]);
            index += character.len_utf8();
            continue;
        }
        if is_identifier_start(character) {
            let end = html_name_end(line, index);
            push_token(tokens, TokenKind::AttrName, &line[index..end]);
            index = end;
            continue;
        }
        if character == '"' || character == '\'' {
            let end = quoted_string_end(line, index, character);
            push_token(tokens, TokenKind::String, &line[index..end]);
            index = end;
            continue;
        }
        if character == '=' {
            push_token(
                tokens,
                TokenKind::Operator,
                &line[index..index + character.len_utf8()],
            );
            index += character.len_utf8();
            continue;
        }
        push_token(
            tokens,
            TokenKind::Punctuation,
            &line[index..index + character.len_utf8()],
        );
        index += character.len_utf8();
    }

    index
}

pub(super) fn html_name_end(line: &str, start: usize) -> usize {
    let mut index = start;
    while index < line.len() {
        let Some(character) = line[index..].chars().next() else {
            break;
        };
        if is_identifier_continue(character) || matches!(character, '-' | ':' | '.') {
            index += character.len_utf8();
        } else {
            break;
        }
    }
    index
}
