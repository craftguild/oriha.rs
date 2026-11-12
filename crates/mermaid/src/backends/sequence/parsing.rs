//! Sequence participant declarations and message grammar.
use super::*;

pub(super) fn parse_sequence_diagram(source: &str) -> Result<SequenceDiagram, MermaidParseError> {
    let lines = significant_lines(source);
    if lines.is_empty() {
        return Err(empty_parse_error());
    }
    parse_sequence(&lines)
}

pub(super) fn significant_lines(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("%%") {
                None
            } else {
                Some((index + 1, trimmed.trim_end_matches(';').trim().to_owned()))
            }
        })
        .collect()
}

pub(super) fn parse_sequence(
    lines: &[(usize, String)],
) -> Result<SequenceDiagram, MermaidParseError> {
    let mut participants = Vec::new();
    let mut participant_index = BTreeMap::new();
    let mut messages = Vec::new();
    let mut autonumber = false;

    for (line_no, line) in lines.iter().skip(1) {
        if line == "autonumber" {
            autonumber = true;
            continue;
        }
        if let Some(participant) = parse_participant(line) {
            upsert_participant(&mut participants, &mut participant_index, participant);
            continue;
        }
        if let Some(message) = parse_sequence_message(line) {
            upsert_participant(
                &mut participants,
                &mut participant_index,
                SequenceParticipant {
                    id: message.from.clone(),
                    label: message.from.clone(),
                },
            );
            upsert_participant(
                &mut participants,
                &mut participant_index,
                SequenceParticipant {
                    id: message.to.clone(),
                    label: message.to.clone(),
                },
            );
            messages.push(message);
            continue;
        }
        return Err(unsupported_syntax_error(*line_no, line));
    }

    if participants.is_empty() && messages.is_empty() {
        return Err(empty_parse_error());
    }

    Ok(SequenceDiagram {
        autonumber,
        participants,
        messages,
    })
}

pub(super) fn parse_participant(line: &str) -> Option<SequenceParticipant> {
    let body = line
        .strip_prefix("participant ")
        .or_else(|| line.strip_prefix("actor "))?
        .trim();
    if body.is_empty() {
        return None;
    }
    if let Some(index) = body.find(" as ") {
        let id = clean_identifier(body[..index].trim())?;
        let label = clean_label(body[index + 4..].trim());
        return Some(SequenceParticipant { id, label });
    }
    let id = clean_identifier(body)?;
    Some(SequenceParticipant {
        label: id.clone(),
        id,
    })
}

pub(super) fn parse_sequence_message(line: &str) -> Option<SequenceMessage> {
    let (route, label) = line.split_once(':')?;
    let (arrow_index, arrow) = find_sequence_arrow(route)?;
    let from = clean_identifier(route[..arrow_index].trim())?;
    let to = clean_identifier(route[arrow_index + arrow.len()..].trim())?;
    Some(SequenceMessage {
        from,
        to,
        label: clean_label(label),
        dashed: arrow.starts_with("--"),
    })
}

pub(super) fn find_sequence_arrow(route: &str) -> Option<(usize, &'static str)> {
    ["-->>", "->>", "-->", "->"]
        .iter()
        .filter_map(|arrow| route.find(arrow).map(|index| (index, *arrow)))
        .min_by_key(|(index, _)| *index)
}

pub(super) fn upsert_participant(
    participants: &mut Vec<SequenceParticipant>,
    index: &mut BTreeMap<String, usize>,
    participant: SequenceParticipant,
) {
    if let Some(existing) = index.get(&participant.id).copied() {
        if participants[existing].label == participants[existing].id {
            participants[existing].label = participant.label;
        }
        return;
    }
    index.insert(participant.id.clone(), participants.len());
    participants.push(participant);
}

pub(super) fn clean_identifier(raw: &str) -> Option<String> {
    let cleaned = clean_label(raw);
    if cleaned.is_empty()
        || cleaned.contains(' ')
        || cleaned.contains('\t')
        || cleaned.contains('[')
        || cleaned.contains(']')
        || cleaned.contains('{')
        || cleaned.contains('}')
        || cleaned.contains('(')
        || cleaned.contains(')')
    {
        None
    } else {
        Some(cleaned)
    }
}

pub(super) fn clean_label(raw: &str) -> String {
    raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_owned()
}

pub(super) fn empty_parse_error() -> MermaidParseError {
    MermaidParseError {
        line: None,
        message: "Mermaid source did not contain a diagram".to_owned(),
    }
}

pub(super) fn unsupported_syntax_error(line: usize, syntax: &str) -> MermaidParseError {
    MermaidParseError {
        line: Some(line),
        message: format!("Unsupported Mermaid syntax in subset renderer: `{syntax}`"),
    }
}
