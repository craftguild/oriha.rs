//! Transient autonumber state and inline label formatting.
use super::*;

impl Default for AutonumberState {
    fn default() -> Self {
        Self {
            enabled: false,
            next: 1,
            increment: 1,
            format: None,
            current: None,
        }
    }
}

pub(super) fn autonumbered_text(diagram: &SequenceDiagram) -> AutonumberedText {
    let mut text = AutonumberedText::default();
    let mut state = AutonumberState::default();

    for event in &diagram.events {
        match event {
            SequenceEvent::Pragma(pragma) => {
                if let Some(SequenceSemanticEvent::Autonumber(autonumber)) =
                    semantic_event_at_line(&diagram.semantic_events, pragma.line)
                {
                    apply_autonumber_command(&mut state, autonumber);
                }
            }
            SequenceEvent::Message(message) => {
                let prefix = next_autonumber_label(&mut state);
                let label = decode_plantuml_unicode_escapes(&substitute_autonumber_variable(
                    &message.label,
                    &state,
                ));
                let label = prefix.as_deref().map_or(label.clone(), |prefix| {
                    message_label_with_autonumber(prefix, &label)
                });
                if label != message.label {
                    text.message_labels.insert(message.line, label);
                }
            }
            SequenceEvent::Note(note) => {
                let note_text = decode_plantuml_unicode_escapes(&substitute_autonumber_variable(
                    &note.text, &state,
                ));
                if note_text != note.text {
                    text.note_texts.insert(note.line, note_text);
                }
            }
            _ => {}
        }
    }

    text
}

pub(super) fn semantic_event_at_line(
    semantic_events: &[SequenceSemanticEvent],
    line: usize,
) -> Option<&SequenceSemanticEvent> {
    semantic_events
        .iter()
        .find(|event| semantic_event_line(event) == line)
}

pub(super) fn apply_autonumber_command(
    state: &mut AutonumberState,
    autonumber: &crate::sequence::Autonumber,
) {
    match autonumber.command {
        AutonumberCommand::Start => {
            let mut index = 0;
            state.next = autonumber
                .args
                .first()
                .and_then(|arg| parse_autonumber_i64(arg))
                .unwrap_or(1);
            if parse_autonumber_i64(autonumber.args.first().map_or("", String::as_str)).is_some() {
                index = 1;
            }
            if let Some(increment) = autonumber
                .args
                .get(index)
                .and_then(|arg| parse_autonumber_i64(arg))
            {
                state.increment = increment;
                index += 1;
            } else {
                state.increment = 1;
            }
            state.format = autonumber_format_arg(&autonumber.args, index);
            state.current = None;
            state.enabled = true;
        }
        AutonumberCommand::Stop | AutonumberCommand::Off => {
            state.enabled = false;
        }
        AutonumberCommand::Resume => {
            let mut index = 0;
            if let Some(increment) = autonumber
                .args
                .first()
                .and_then(|arg| parse_autonumber_i64(arg))
            {
                state.increment = increment;
                index = 1;
            }
            if let Some(format) = autonumber_format_arg(&autonumber.args, index) {
                state.format = Some(format);
            }
            state.enabled = true;
        }
        AutonumberCommand::Increment => {
            state.next += state.increment;
        }
    }
}

pub(super) fn parse_autonumber_i64(value: &str) -> Option<i64> {
    value.parse::<i64>().ok()
}

pub(super) fn autonumber_format_arg(args: &[String], start: usize) -> Option<String> {
    (start < args.len()).then(|| args[start..].join(" "))
}

pub(super) fn next_autonumber_label(state: &mut AutonumberState) -> Option<String> {
    if !state.enabled {
        return None;
    }

    let current = state.next;
    let label = state.format.as_deref().map_or_else(
        || format!("<b>{current}</b>"),
        |format| format_autonumber_value(format, current),
    );
    state.current = Some(current);
    state.next += state.increment;
    Some(label)
}

pub(super) fn substitute_autonumber_variable(text: &str, state: &AutonumberState) -> String {
    let Some(current) = state.current else {
        return text.to_owned();
    };
    text.replace("%autonumber%", &current.to_string())
}

pub(super) fn message_label_with_autonumber(number: &str, label: &str) -> String {
    let number = number.trim_end();
    if label.is_empty() {
        number.to_owned()
    } else {
        format!("{number} {label}")
    }
}

pub(super) fn format_autonumber_value(format: &str, value: i64) -> String {
    let Some((start, end, placeholder)) = autonumber_placeholder_range(format) else {
        let value = format!("{format}{value}");
        return close_open_plantuml_inline_tags(value.trim_end());
    };

    let width = end - start;
    let value = match placeholder {
        '0' => format!("{value:0width$}"),
        '#' => value.to_string(),
        _ => unreachable!("autonumber placeholder should be zero or hash"),
    };
    let value = format!("{}{}{}", &format[..start], value, &format[end..]);
    close_open_plantuml_inline_tags(value.trim_end())
}

pub(super) fn autonumber_placeholder_range(value: &str) -> Option<(usize, usize, char)> {
    let mut start = None;
    let mut end = 0;
    let mut placeholder = '\0';

    for (index, character) in value.char_indices() {
        if character == '0' || character == '#' {
            if start.is_none() {
                start = Some(index);
                placeholder = character;
            } else if character != placeholder {
                return start.map(|start| (start, end, placeholder));
            }
            end = index + character.len_utf8();
        } else if let Some(start) = start {
            return Some((start, end, placeholder));
        }
    }

    start.map(|start| (start, end, placeholder))
}

pub(super) fn close_open_plantuml_inline_tags(value: &str) -> String {
    let mut open_tags = Vec::new();
    let mut chars = value.chars().peekable();

    while let Some(character) = chars.next() {
        if character != '<' {
            continue;
        }

        let mut tag = String::new();
        for character in chars.by_ref() {
            if character == '>' {
                break;
            }
            tag.push(character);
        }

        let tag_name = tag
            .trim()
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if !matches!(tag_name.as_str(), "b" | "u" | "font") {
            continue;
        }

        if tag.trim_start().starts_with('/') {
            if let Some(position) = open_tags.iter().rposition(|open_tag| open_tag == &tag_name) {
                open_tags.remove(position);
            }
        } else {
            open_tags.push(tag_name);
        }
    }

    let mut output = value.to_owned();
    for tag in open_tags.into_iter().rev() {
        output.push_str("</");
        output.push_str(&tag);
        output.push('>');
    }
    output
}

pub(super) fn decode_plantuml_unicode_escapes(value: &str) -> String {
    let mut output = String::new();
    let mut rest = value;

    while let Some(start) = rest.find("<U+") {
        output.push_str(&rest[..start]);
        let code_start = start + 3;
        let Some(end) = rest[code_start..].find('>') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let code_end = code_start + end;
        let code = &rest[code_start..code_end];
        if let Ok(value) = u32::from_str_radix(code, 16) {
            if let Some(character) = char::from_u32(value) {
                output.push(character);
                rest = &rest[code_end + 1..];
                continue;
            }
        }

        output.push_str(&rest[start..code_end + 1]);
        rest = &rest[code_end + 1..];
    }

    output.push_str(rest);
    output
}
