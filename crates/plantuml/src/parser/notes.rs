//! Note headers, target lists and multiline termination.
use super::*;

pub(super) fn parse_note(
    line_number: usize,
    line: &str,
) -> Result<Option<StatementParse>, PlantUmlError> {
    let line = strip_note_prefix_decoration(line);
    let Some((shape, rest)) = parse_note_keyword(line) else {
        return Ok(None);
    };

    let (header, text) = rest
        .split_once(':')
        .map_or((rest.trim(), None), |(head, body)| {
            (head.trim(), Some(body.trim().to_owned()))
        });
    let (placement, targets, color) = parse_note_header(header);

    if let Some(text) = text {
        return Ok(Some(StatementParse::Event(SequenceEvent::Note(Note {
            placement,
            targets,
            text: decode_plantuml_text(&text),
            color,
            shape,
            multiline: false,
            line: line_number,
        }))));
    }

    Ok(Some(StatementParse::StartNote(ActiveNote {
        placement,
        targets,
        color,
        shape,
        lines: Vec::new(),
        line: line_number,
    })))
}

pub(super) fn strip_note_prefix_decoration(line: &str) -> &str {
    let trimmed = line.trim_start();
    for marker in ["&", "/", "\\"] {
        let Some(rest) = trimmed.strip_prefix(marker) else {
            continue;
        };
        let rest = rest.trim_start();
        if parse_note_keyword(rest).is_some() {
            return rest;
        }
    }

    trimmed
}

pub(super) fn parse_note_keyword(line: &str) -> Option<(NoteShape, &str)> {
    let (keyword, rest) = split_first_word(line);
    match keyword.to_ascii_lowercase().as_str() {
        "note" => Some((NoteShape::Normal, rest)),
        "hnote" => Some((NoteShape::Hexagon, rest)),
        "rnote" => Some((NoteShape::Rectangle, rest)),
        _ => None,
    }
}

pub(super) fn parse_note_side_targets_and_color(rest: &str) -> (Vec<String>, Option<String>) {
    let rest = rest.trim();
    let rest = strip_keyword(rest, "of").unwrap_or(rest).trim();
    let (rest, color) = split_trailing_color(rest);
    (parse_targets(strip_leading_color_tokens(rest)), color)
}

pub(super) fn parse_note_targets_and_color(rest: &str) -> (Vec<String>, Option<String>) {
    let (rest, color) = split_trailing_color(rest);
    (parse_targets(strip_leading_color_tokens(rest)), color)
}

pub(super) fn parse_note_header(header: &str) -> (NotePlacement, Vec<String>, Option<String>) {
    let (keyword, rest) = split_first_word(header.trim());
    match keyword.to_ascii_lowercase().as_str() {
        "left" => {
            let (targets, color) = parse_note_side_targets_and_color(rest);
            (NotePlacement::Left, targets, color)
        }
        "right" => {
            let (targets, color) = parse_note_side_targets_and_color(rest);
            (NotePlacement::Right, targets, color)
        }
        "over" => {
            let (targets, color) = parse_note_targets_and_color(rest);
            (NotePlacement::Over, targets, color)
        }
        "across" => {
            let (_, color) = split_trailing_color(rest);
            (NotePlacement::Across, Vec::new(), color)
        }
        "" => (NotePlacement::Unknown(String::new()), Vec::new(), None),
        other => {
            let (targets, color) = parse_note_targets_and_color(rest);
            (NotePlacement::Unknown(other.to_owned()), targets, color)
        }
    }
}

pub(super) fn is_note_end(line: &str) -> bool {
    matches!(
        line.to_ascii_lowercase().as_str(),
        "end note" | "endnote" | "endhnote" | "endrnote" | "end hnote" | "end rnote"
    )
}

pub(super) fn parse_targets(input: &str) -> Vec<String> {
    input
        .split(',')
        .filter_map(|target| {
            let target = target.trim();
            if target.is_empty() {
                return None;
            }
            take_value(target).map(|(value, _)| value)
        })
        .collect()
}
