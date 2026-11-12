//! Message endpoint references and inline declarations.
use super::*;

pub(super) fn parse_message_endpoint(
    line_number: usize,
    line: &str,
    input: &str,
    target: MessageActivationTarget,
) -> Result<ParsedMessageEndpoint, PlantUmlError> {
    let input = strip_leading_teoz_anchors(input);
    let (without_color, trailing_color) = split_trailing_color(input);
    let (endpoint_source, activation_shortcut) =
        split_activation_shortcuts(without_color, target, trailing_color.clone());
    let color = activation_shortcut
        .is_none()
        .then_some(trailing_color)
        .flatten();
    let endpoint_source = strip_leading_teoz_anchors(endpoint_source);
    let (endpoint, declaration) =
        parse_endpoint_with_declaration(line_number, line, endpoint_source)?;

    Ok(ParsedMessageEndpoint {
        endpoint,
        declaration,
        color,
        activation_shortcut,
    })
}

pub(super) fn parse_endpoint_with_declaration(
    line_number: usize,
    line: &str,
    input: &str,
) -> Result<(MessageEndpoint, Option<ParticipantDeclaration>), PlantUmlError> {
    let raw = input.trim();
    if let Some((name_source, alias_source)) = split_as_alias(raw) {
        let (name, name_trailing) =
            take_value(name_source.trim()).ok_or_else(|| PlantUmlError::InvalidMessage {
                line: line_number,
                text: line.to_owned(),
            })?;
        let (alias, alias_trailing) =
            take_value(alias_source.trim()).ok_or_else(|| PlantUmlError::InvalidMessage {
                line: line_number,
                text: line.to_owned(),
            })?;

        if !name_trailing.trim().is_empty() || !alias_trailing.trim().is_empty() {
            return Err(PlantUmlError::InvalidMessage {
                line: line_number,
                text: line.to_owned(),
            });
        }

        let display = ParticipantDisplay {
            lines: name.lines().map(ToOwned::to_owned).collect(),
            raw: name.clone(),
        };
        let declaration = ParticipantDeclaration {
            id: alias.clone(),
            name: name.clone(),
            alias: Some(alias.clone()),
            kind: ParticipantKind::Participant,
            color: None,
            order: None,
            stereotype: None,
            spot: None,
            display: Some(display),
            line: line_number,
            raw: raw.to_owned(),
        };
        return Ok((
            MessageEndpoint {
                id: alias,
                raw: raw.to_owned(),
                kind: MessageEndpointKind::Participant,
            },
            Some(declaration),
        ));
    }

    let endpoint = parse_endpoint(raw).ok_or_else(|| PlantUmlError::InvalidMessage {
        line: line_number,
        text: line.to_owned(),
    })?;
    Ok((endpoint, None))
}

pub(super) fn parse_endpoint(input: &str) -> Option<MessageEndpoint> {
    let raw = input.trim();
    if raw.is_empty() {
        return Some(MessageEndpoint {
            id: String::new(),
            raw: String::new(),
            kind: MessageEndpointKind::Short,
        });
    }
    if matches!(raw, "[" | "]" | "?") {
        let kind = match raw {
            "[" => MessageEndpointKind::Incoming,
            "]" => MessageEndpointKind::Outgoing,
            "?" => MessageEndpointKind::Unknown,
            _ => MessageEndpointKind::Participant,
        };
        return Some(MessageEndpoint {
            id: raw.to_owned(),
            raw: raw.to_owned(),
            kind,
        });
    }

    let (id, trailing) = take_value(raw)?;
    trailing.trim().is_empty().then(|| MessageEndpoint {
        id,
        raw: raw.to_owned(),
        kind: MessageEndpointKind::Participant,
    })
}

pub(super) fn split_trailing_color(input: &str) -> (&str, Option<String>) {
    let rest = input.trim_end();
    if let Some(index) = rest.rfind(char::is_whitespace) {
        let tail = rest[index..].trim();
        if tail.starts_with('#') {
            return (rest[..index].trim_end(), Some(tail.to_owned()));
        }
    }

    (rest, None)
}
