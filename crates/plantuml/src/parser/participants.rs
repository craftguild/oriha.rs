//! Participant declarations, multiline displays and metadata.
use super::*;

pub(super) fn parse_participant_declaration(
    line_number: usize,
    line: &str,
) -> Result<Option<SequenceEvent>, PlantUmlError> {
    parse_participant_declaration_from_line(line_number, line, None)
        .map(|declaration| declaration.map(SequenceEvent::ParticipantDeclaration))
}

pub(super) fn parse_multiline_participant_declaration_start(
    line_number: usize,
    line: &str,
) -> Option<ActiveParticipantDeclaration> {
    let (keyword, rest) = split_first_word(line);
    let kind = match keyword.to_ascii_lowercase().as_str() {
        "participant" => ParticipantKind::Participant,
        "actor" => ParticipantKind::Actor,
        "boundary" => ParticipantKind::Boundary,
        "control" => ParticipantKind::Control,
        "entity" => ParticipantKind::Entity,
        "database" => ParticipantKind::Database,
        "collections" => ParticipantKind::Collections,
        "queue" => ParticipantKind::Queue,
        _ => return None,
    };

    let header = rest.trim().strip_suffix('[')?.trim_end();
    Some(ActiveParticipantDeclaration {
        kind,
        line: line_number,
        header: format!("{keyword} {header}"),
        lines: Vec::new(),
    })
}

pub(super) fn parse_participant_declaration_from_line(
    line_number: usize,
    line: &str,
    display: Option<ParticipantDisplay>,
) -> Result<Option<ParticipantDeclaration>, PlantUmlError> {
    let (keyword, rest) = split_first_word(line);
    let kind = match keyword.to_ascii_lowercase().as_str() {
        "participant" => ParticipantKind::Participant,
        "actor" => ParticipantKind::Actor,
        "boundary" => ParticipantKind::Boundary,
        "control" => ParticipantKind::Control,
        "entity" => ParticipantKind::Entity,
        "database" => ParticipantKind::Database,
        "collections" => ParticipantKind::Collections,
        "queue" => ParticipantKind::Queue,
        _ => return Ok(None),
    };

    let mut rest = rest.trim();
    if rest.is_empty() {
        return Err(PlantUmlError::InvalidStatement {
            line: line_number,
            text: line.to_owned(),
        });
    }

    let mut inline_display = None;
    if let Some((before_display, display_body)) = split_inline_participant_display(rest) {
        rest = before_display.trim();
        let display_body = decode_plantuml_text(display_body);
        inline_display = Some(ParticipantDisplay {
            lines: display_body.lines().map(ToOwned::to_owned).collect(),
            raw: display_body,
        });
    }

    if matches!(find_message_operator(rest), Some((left, _, _, _)) if left.is_empty()) {
        return Ok(None);
    }

    let (name_source, alias_source) = split_as_alias(rest)
        .map(|(name, alias)| (name.trim(), Some(alias.trim())))
        .unwrap_or((rest, None));
    let (name, trailing) =
        take_value(name_source).ok_or_else(|| PlantUmlError::InvalidStatement {
            line: line_number,
            text: line.to_owned(),
        })?;

    let mut metadata_source = trailing.trim().to_owned();

    let alias = if let Some(alias_source) = alias_source {
        let (alias, alias_trailing) =
            take_value(alias_source).ok_or_else(|| PlantUmlError::InvalidStatement {
                line: line_number,
                text: line.to_owned(),
            })?;
        append_metadata(&mut metadata_source, alias_trailing.trim());
        Some(alias)
    } else {
        None
    };

    let id = alias.clone().unwrap_or_else(|| name.clone());
    let metadata = parse_participant_metadata(&metadata_source);
    let display = display.or(inline_display);

    Ok(Some(ParticipantDeclaration {
        id,
        name,
        alias,
        kind,
        color: metadata.color,
        order: metadata.order,
        stereotype: metadata.stereotype,
        spot: metadata.spot,
        display,
        line: line_number,
        raw: line.to_owned(),
    }))
}

pub(super) fn finish_multiline_participant_declaration(
    participant: ActiveParticipantDeclaration,
) -> ParticipantDeclaration {
    let raw = format!(
        "{} [\n{}\n]",
        participant.header,
        participant.lines.join("\n")
    );
    let display_raw = decode_plantuml_text(&participant.lines.join("\n"));
    let display = ParticipantDisplay {
        lines: display_raw.lines().map(ToOwned::to_owned).collect(),
        raw: display_raw,
    };
    parse_participant_declaration_from_line(participant.line, &participant.header, Some(display))
        .ok()
        .flatten()
        .unwrap_or_else(|| ParticipantDeclaration {
            id: participant.header.clone(),
            name: participant.header,
            alias: None,
            kind: participant.kind,
            color: None,
            order: None,
            stereotype: None,
            spot: None,
            display: None,
            line: participant.line,
            raw,
        })
}

pub(super) fn split_inline_participant_display(input: &str) -> Option<(&str, &str)> {
    let start = find_outside_quotes(input, "[")?;
    let rest = input[start + 1..].trim_end();
    let display = rest.strip_suffix(']')?;
    Some((&input[..start], display.trim()))
}

pub(super) fn parse_participant_metadata(input: &str) -> ParticipantMetadata {
    let mut metadata = ParticipantMetadata::default();
    let tokens = input.split_whitespace().collect::<Vec<_>>();
    let mut index = 0;
    while index < tokens.len() {
        let token = tokens[index];
        if token.starts_with('#') && metadata.color.is_none() {
            metadata.color = Some(token.to_owned());
            index += 1;
            continue;
        }

        if token.eq_ignore_ascii_case("order") {
            if let Some(value) = tokens.get(index + 1).and_then(|value| value.parse().ok()) {
                metadata.order = Some(value);
                index += 2;
                continue;
            }
        }

        if token.starts_with("<<") {
            let mut stereotype = token.to_owned();
            while !stereotype.ends_with(">>") && index + 1 < tokens.len() {
                index += 1;
                stereotype.push(' ');
                stereotype.push_str(tokens[index]);
            }
            metadata.spot = parse_participant_spot(&stereotype);
            metadata.stereotype = Some(
                stereotype
                    .trim_start_matches("<<")
                    .trim_end_matches(">>")
                    .trim()
                    .to_owned(),
            );
        }

        index += 1;
    }

    metadata
}

pub(super) fn parse_participant_spot(stereotype: &str) -> Option<ParticipantSpot> {
    let start = stereotype.find('(')?;
    let end = stereotype[start + 1..].find(')')? + start + 1;
    let mut parts = stereotype[start + 1..end].split(',');
    let letter = parts.next()?.trim().chars().next()?;
    let color = parts
        .next()
        .map(str::trim)
        .and_then(non_empty)
        .map(ToOwned::to_owned);

    Some(ParticipantSpot { letter, color })
}
