//! Message operators, timing annotations and activation shortcuts.
use super::*;

pub(super) fn parse_message(
    line_number: usize,
    line: &str,
) -> Result<Option<ParsedMessage>, PlantUmlError> {
    let (prefix, label) = split_message_label(line);
    let (parallel, prefix) = strip_parallel_marker(prefix);
    let Some((left, operator, right, raw_operator)) = find_message_operator(prefix) else {
        return Ok(None);
    };

    let left = parse_message_endpoint(line_number, line, left, MessageActivationTarget::From)?;
    let right = parse_message_endpoint(line_number, line, right, MessageActivationTarget::To)?;
    let left_endpoint = left.endpoint.clone();
    let right_endpoint = right.endpoint.clone();

    let mut features =
        message_arrow_features(raw_operator, operator, &left_endpoint, &right_endpoint);
    features.parallel = parallel;
    features.activation_shortcut = right
        .activation_shortcut
        .clone()
        .or_else(|| left.activation_shortcut.clone());
    if features.color.is_none() {
        features.color = right.color.clone().or_else(|| left.color.clone());
    }

    let line_style = match operator {
        MessageOperator::RightSolid
        | MessageOperator::LeftSolid
        | MessageOperator::BidirectionalSolid => LineStyle::Solid,
        MessageOperator::RightDashed
        | MessageOperator::LeftDashed
        | MessageOperator::BidirectionalDashed => LineStyle::Dashed,
    };

    let (from, to, visual_direction) = match operator {
        MessageOperator::RightSolid | MessageOperator::RightDashed => {
            let direction = if left_endpoint.id == right_endpoint.id {
                MessageDirection::SelfMessage
            } else {
                MessageDirection::LeftToRight
            };
            (left_endpoint, right_endpoint, direction)
        }
        MessageOperator::LeftSolid | MessageOperator::LeftDashed => {
            let direction = if left_endpoint.id == right_endpoint.id {
                MessageDirection::SelfMessage
            } else {
                MessageDirection::RightToLeft
            };
            (right_endpoint, left_endpoint, direction)
        }
        MessageOperator::BidirectionalSolid | MessageOperator::BidirectionalDashed => {
            let direction = if left_endpoint.id == right_endpoint.id {
                MessageDirection::SelfMessage
            } else {
                MessageDirection::Bidirectional
            };
            (left_endpoint, right_endpoint, direction)
        }
    };

    let declarations = [left.declaration, right.declaration]
        .into_iter()
        .flatten()
        .collect();

    Ok(Some(ParsedMessage {
        message: SequenceMessage {
            from,
            to,
            label: decode_plantuml_text(label),
            arrow: MessageArrow {
                raw: raw_operator.to_owned(),
                direction: visual_direction,
                line_style,
                features,
            },
            line: line_number,
        },
        declarations,
    }))
}

pub(super) fn split_message_label(line: &str) -> (&str, &str) {
    find_outside_quotes(line, ":").map_or((line, ""), |index| {
        (&line[..index], line[index + 1..].trim())
    })
}

pub(super) fn find_message_operator(prefix: &str) -> Option<(&str, MessageOperator, &str, &str)> {
    let mut in_quotes = false;
    let mut escaped = false;

    for (index, character) in prefix.char_indices() {
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
        if in_quotes {
            continue;
        }

        if let Some((operator_len, operator)) = message_operator_at(&prefix[index..]) {
            return Some((
                prefix[..index].trim(),
                operator,
                prefix[index + operator_len..].trim(),
                &prefix[index..index + operator_len],
            ));
        }
    }

    None
}

pub(super) fn message_operator_at(input: &str) -> Option<(usize, MessageOperator)> {
    if let Some(timing_len) = timing_annotation_len(input) {
        if let Some((operator_len, operator)) =
            message_operator_without_leading_timing(&input[timing_len..])
        {
            return Some((timing_len + operator_len, operator));
        }
    }

    message_operator_without_leading_timing(input)
}

pub(super) fn message_operator_without_leading_timing(
    input: &str,
) -> Option<(usize, MessageOperator)> {
    let mut best = colored_message_operator_at(input);

    if let Some((len, operator)) = base_message_operator_at(input) {
        let len = len + timing_annotation_len(&input[len..]).unwrap_or(0);
        if best.is_none_or(|(best_len, _)| len > best_len) {
            best = Some((len, operator));
        }
    }

    best
}

pub(super) fn colored_message_operator_at(input: &str) -> Option<(usize, MessageOperator)> {
    let rest = input.strip_prefix("-[")?;
    let color_end = rest.find(']')? + 2;
    let after_color = &input[color_end + 1..];

    for (token, operator) in [
        ("-->", MessageOperator::RightDashed),
        ("->", MessageOperator::RightDashed),
        (">", MessageOperator::RightSolid),
    ] {
        if after_color.starts_with(token) {
            let len = color_end + 1 + token.len();
            let len = len + timing_annotation_len(&input[len..]).unwrap_or(0);
            return Some((len, operator));
        }
    }

    None
}

pub(super) fn base_message_operator_at(input: &str) -> Option<(usize, MessageOperator)> {
    const OPERATORS: &[(&str, MessageOperator)] = &[
        ("<-->", MessageOperator::BidirectionalDashed),
        ("o<->o", MessageOperator::BidirectionalSolid),
        ("x<->x", MessageOperator::BidirectionalSolid),
        ("<->o", MessageOperator::BidirectionalSolid),
        ("<->", MessageOperator::BidirectionalSolid),
        ("o\\\\--", MessageOperator::LeftDashed),
        ("o\\--", MessageOperator::LeftDashed),
        ("//--", MessageOperator::LeftDashed),
        ("\\\\-", MessageOperator::LeftSolid),
        ("x->o", MessageOperator::RightSolid),
        ("o->o", MessageOperator::RightSolid),
        ("->>o", MessageOperator::RightSolid),
        ("->x", MessageOperator::RightSolid),
        ("->>", MessageOperator::RightSolid),
        ("->o", MessageOperator::RightSolid),
        ("x->", MessageOperator::RightSolid),
        ("o->", MessageOperator::RightSolid),
        ("->", MessageOperator::RightSolid),
        ("-\\\\o", MessageOperator::RightSolid),
        ("-\\o", MessageOperator::RightSolid),
        ("-//o", MessageOperator::RightSolid),
        ("-/o", MessageOperator::RightSolid),
        ("-\\\\", MessageOperator::RightSolid),
        ("-\\", MessageOperator::RightSolid),
        ("-//", MessageOperator::RightSolid),
        ("-/", MessageOperator::RightSolid),
        ("-->", MessageOperator::RightDashed),
        ("<--", MessageOperator::LeftDashed),
        ("x<-", MessageOperator::LeftSolid),
        ("<-", MessageOperator::LeftSolid),
        ("--", MessageOperator::RightDashed),
    ];

    OPERATORS
        .iter()
        .copied()
        .find(|(token, _)| input.starts_with(token))
        .map(|(token, operator)| (token.len(), operator))
}

pub(super) fn timing_annotation_len(input: &str) -> Option<usize> {
    let rest = input.strip_prefix('(')?;
    let end = rest.find(')')?;
    let value = rest[..end].trim();
    (!value.is_empty() && value.chars().all(|character| character.is_ascii_digit()))
        .then_some(end + 2)
}

pub(super) fn strip_parallel_marker(prefix: &str) -> (bool, &str) {
    let trimmed = prefix.trim_start();
    trimmed
        .strip_prefix('&')
        .map(|rest| (true, rest.trim_start()))
        .unwrap_or((false, prefix))
}

pub(super) fn split_activation_shortcuts(
    input: &str,
    target: MessageActivationTarget,
    color: Option<String>,
) -> (&str, Option<MessageActivationShortcut>) {
    let mut rest = input.trim_end();
    let mut shortcut = None;

    loop {
        let mut matched = false;
        for (token, kind) in [
            ("++", ActivationKind::Activate),
            ("--", ActivationKind::Deactivate),
            ("**", ActivationKind::Create),
            ("!!", ActivationKind::Destroy),
        ] {
            if let Some(before) = rest.strip_suffix(token) {
                if shortcut.is_none() {
                    shortcut = Some(MessageActivationShortcut {
                        kind,
                        target,
                        color: color.clone(),
                    });
                }
                rest = before.trim_end();
                matched = true;
                break;
            }
        }

        if !matched {
            break;
        }
    }

    if shortcut.is_some() {
        (rest, shortcut)
    } else {
        (input.trim_end(), None)
    }
}

pub(super) fn strip_leading_teoz_anchors(input: &str) -> &str {
    let mut rest = input.trim();
    loop {
        let Some(after_open) = rest.strip_prefix('{') else {
            return rest;
        };
        let Some(end) = after_open.find('}') else {
            return rest;
        };
        rest = after_open[end + 1..].trim_start();
    }
}
