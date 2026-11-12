//! Programming-language lexical state, comments and contextual words.
use super::*;

pub(super) fn tokenize_programming_line(
    line: &str,
    language: Language,
    state: &mut HighlightState,
) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < line.len() {
        if let Some(end_marker) = state.block_comment_end {
            if let Some(relative_end) = line[index..].find(end_marker) {
                let end = index + relative_end + end_marker.len();
                push_token(&mut tokens, TokenKind::Comment, &line[index..end]);
                state.block_comment_end = None;
                index = end;
            } else {
                push_token(&mut tokens, TokenKind::Comment, &line[index..]);
                return tokens;
            }
            continue;
        }

        let rest = &line[index..];
        if line_comment_start(rest, language).is_some() {
            push_token(&mut tokens, TokenKind::Comment, &line[index..]);
            break;
        }
        if let Some((start_marker, end_marker)) = block_comment_markers(rest, language) {
            if let Some(relative_end) = rest[start_marker.len()..].find(end_marker) {
                let end = index + start_marker.len() + relative_end + end_marker.len();
                push_token(&mut tokens, TokenKind::Comment, &line[index..end]);
                index = end;
            } else {
                push_token(&mut tokens, TokenKind::Comment, &line[index..]);
                state.block_comment_end = Some(end_marker);
                return tokens;
            }
            continue;
        }
        if language == Language::Python {
            if let Some(end) = python_string_literal_end(line, index) {
                push_token(&mut tokens, TokenKind::String, &line[index..end]);
                index = end;
                continue;
            }
        }
        if language == Language::Shell {
            if let Some(end) = shell_variable_end(line, index) {
                push_token(&mut tokens, TokenKind::Variable, &line[index..end]);
                index = end;
                continue;
            }
        }
        if language == Language::Css {
            if let Some(end) = css_color_end(line, index) {
                push_token(&mut tokens, TokenKind::Constant, &line[index..end]);
                index = end;
                continue;
            }
        }

        let Some(character) = rest.chars().next() else {
            break;
        };
        if is_code_quote(character, language) {
            if language == Language::Rust && character == '\'' {
                if let Some(end) = rust_char_literal_end(line, index) {
                    push_token(&mut tokens, TokenKind::String, &line[index..end]);
                    index = end;
                } else {
                    push_token(
                        &mut tokens,
                        TokenKind::Symbol,
                        &line[index..index + character.len_utf8()],
                    );
                    index += character.len_utf8();
                }
                continue;
            }
            let end = quoted_string_end(line, index, character);
            push_token(&mut tokens, TokenKind::String, &line[index..end]);
            index = end;
            continue;
        }
        if character.is_ascii_digit() {
            let end = number_end(line, index);
            push_token(&mut tokens, TokenKind::Number, &line[index..end]);
            index = end;
            continue;
        }
        if is_identifier_start(character) {
            let end = identifier_end_for_language(line, index, language);
            let word = &line[index..end];
            if let Some(token_kind) = word_token_kind(line, index, end, word, language) {
                push_token(&mut tokens, token_kind, word);
            } else {
                push_plain(&mut tokens, word);
            }
            index = end;
            continue;
        }
        if is_operator_character(character) {
            push_token(
                &mut tokens,
                TokenKind::Operator,
                &line[index..index + character.len_utf8()],
            );
            index += character.len_utf8();
            continue;
        }
        if is_punctuation_character(character) {
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

pub(super) fn line_comment_start(rest: &str, language: Language) -> Option<&'static str> {
    match language {
        Language::Javascript | Language::Rust => rest.starts_with("//").then_some("//"),
        Language::Python | Language::Shell => rest.starts_with('#').then_some("#"),
        Language::Sql => rest.starts_with("--").then_some("--"),
        Language::PlantUml => rest.starts_with('\'').then_some("'"),
        _ => None,
    }
}

pub(super) fn block_comment_markers(
    rest: &str,
    language: Language,
) -> Option<(&'static str, &'static str)> {
    match language {
        Language::Javascript | Language::Rust | Language::Css | Language::Sql => {
            rest.starts_with("/*").then_some(("/*", "*/"))
        }
        Language::PlantUml => rest.starts_with("/'").then_some(("/'", "'/")),
        _ => None,
    }
}

pub(super) fn word_token_kind(
    line: &str,
    start: usize,
    end: usize,
    word: &str,
    language: Language,
) -> Option<TokenKind> {
    let spec = language.spec();
    if contains_word(spec.keywords, word, spec.case_insensitive) {
        Some(TokenKind::Keyword)
    } else if contains_word(spec.literals, word, spec.case_insensitive) {
        Some(TokenKind::Constant)
    } else if contains_word(spec.builtins, word, spec.case_insensitive) {
        Some(TokenKind::Builtin)
    } else if language == Language::Css && next_non_space_char(line, end) == Some(':') {
        Some(TokenKind::Property)
    } else if matches!(
        language,
        Language::Javascript | Language::Rust | Language::Python
    ) && previous_significant_word(line, start).is_some_and(is_type_introducer)
    {
        Some(TokenKind::ClassName)
    } else if is_function_call_after_word(line, end, language) {
        Some(TokenKind::Function)
    } else if word.chars().next().is_some_and(char::is_uppercase)
        && matches!(
            language,
            Language::Javascript | Language::Rust | Language::Python
        )
    {
        Some(TokenKind::ClassName)
    } else {
        None
    }
}

pub(super) fn contains_word(words: &[&str], word: &str, case_insensitive: bool) -> bool {
    if case_insensitive {
        words
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(word))
    } else {
        words.contains(&word)
    }
}

pub(super) fn is_type_introducer(word: &str) -> bool {
    matches!(
        word,
        "class" | "enum" | "extends" | "impl" | "interface" | "struct" | "trait" | "type"
    )
}

pub(super) fn is_function_call_after_word(line: &str, end: usize, language: Language) -> bool {
    if next_non_space_char(line, end) == Some('(') {
        return true;
    }

    language == Language::Rust && rust_generic_call_open_paren(line, end)
}

pub(super) fn rust_generic_call_open_paren(line: &str, end: usize) -> bool {
    let mut index = skip_whitespace(line, end);
    if !line[index..].starts_with('<') {
        return false;
    }

    let mut depth = 0usize;
    while index < line.len() {
        let Some(character) = line[index..].chars().next() else {
            break;
        };
        match character {
            '<' => depth += 1,
            '>' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let after = index + character.len_utf8();
                    return next_non_space_char(line, after) == Some('(');
                }
            }
            _ => {}
        }
        index += character.len_utf8();
    }

    false
}

pub(super) fn previous_significant_word(line: &str, start: usize) -> Option<&str> {
    let before = line[..start].trim_end();
    let end = before
        .rfind(|ch: char| is_identifier_continue(ch))
        .map(|position| position + before[position..].chars().next().unwrap().len_utf8())?;
    let start = before[..end]
        .rfind(|ch: char| !is_identifier_continue(ch))
        .map_or(0, |position| {
            position + before[position..].chars().next().unwrap().len_utf8()
        });
    Some(&before[start..end])
}

pub(super) fn python_string_literal_end(line: &str, start: usize) -> Option<usize> {
    let rest = &line[start..];
    let lower = rest.to_ascii_lowercase();
    let prefix_len = ["fr", "rf", "br", "rb", "ur", "r", "u", "f", "b"]
        .into_iter()
        .find_map(|prefix| {
            lower
                .starts_with(prefix)
                .then_some(prefix.len())
                .filter(|prefix_len| {
                    rest[*prefix_len..]
                        .chars()
                        .next()
                        .is_some_and(|ch| ch == '\'' || ch == '"')
                })
        })?;
    let quote = rest[prefix_len..].chars().next()?;
    Some(quoted_string_end(line, start + prefix_len, quote))
}

pub(super) fn rust_char_literal_end(line: &str, start: usize) -> Option<usize> {
    let rest = &line[start..];
    if !rest.starts_with('\'') {
        return None;
    }
    let end = quoted_string_end(line, start, '\'');
    let content = &line[start + 1..end.saturating_sub(1)];
    (!content.is_empty() && content.len() <= 4 && end <= line.len() && line[..end].ends_with('\''))
        .then_some(end)
}

pub(super) fn shell_variable_end(line: &str, start: usize) -> Option<usize> {
    let rest = &line[start..];
    if rest.starts_with("${") {
        return rest.find('}').map(|position| start + position + 1);
    }
    if !rest.starts_with('$') {
        return None;
    }
    let after = start + 1;
    line[after..]
        .chars()
        .next()
        .filter(|ch| is_identifier_start(*ch))?;
    Some(identifier_end(line, after))
}

pub(super) fn css_color_end(line: &str, start: usize) -> Option<usize> {
    let rest = &line[start..];
    if !rest.starts_with('#') {
        return None;
    }
    let mut end = start + 1;
    while end < line.len() {
        let Some(ch) = line[end..].chars().next() else {
            break;
        };
        if ch.is_ascii_hexdigit() {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    matches!(end - start, 4 | 5 | 7 | 9).then_some(end)
}
