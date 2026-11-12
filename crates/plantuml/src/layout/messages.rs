//! Message endpoint and direction geometry.
use super::*;

pub(super) fn layout_sequence_message(
    message: &SequenceMessage,
    label: &str,
    participant_x: &HashMap<&str, f32>,
    options: &LayoutOptions,
    width: f32,
    y: f32,
) -> MessageLayout {
    let start_x = message_endpoint_x(
        &message.from,
        participant_x,
        options,
        width,
        true,
        message.arrow.direction,
    );
    let end_x = message_endpoint_x(
        &message.to,
        participant_x,
        options,
        width,
        false,
        message.arrow.direction,
    );

    MessageLayout {
        from: message.from.id.clone(),
        to: message.to.id.clone(),
        label: label.to_owned(),
        line_style: message.arrow.line_style,
        arrow_raw: message.arrow.raw.clone(),
        arrow_features: message.arrow.features.clone(),
        activation_shortcut: message.arrow.features.activation_shortcut.clone(),
        from_endpoint_kind: message.from.kind,
        to_endpoint_kind: message.to.kind,
        direction: positioned_message_direction(message.arrow.direction, start_x, end_x),
        line: message.line,
        start_x,
        end_x,
        y,
    }
}

pub(super) fn positioned_message_direction(
    direction: MessageDirection,
    start_x: f32,
    end_x: f32,
) -> MessageDirection {
    match direction {
        MessageDirection::SelfMessage | MessageDirection::Bidirectional => direction,
        MessageDirection::LeftToRight | MessageDirection::RightToLeft => {
            if (start_x - end_x).abs() < f32::EPSILON {
                MessageDirection::SelfMessage
            } else if end_x >= start_x {
                MessageDirection::LeftToRight
            } else {
                MessageDirection::RightToLeft
            }
        }
    }
}

pub(super) fn message_endpoint_x(
    endpoint: &MessageEndpoint,
    participant_x: &HashMap<&str, f32>,
    options: &LayoutOptions,
    width: f32,
    is_start: bool,
    direction: MessageDirection,
) -> f32 {
    if let Some(x) = participant_x.get(endpoint.id.as_str()) {
        return *x;
    }

    let left = (options.margin_x / 2.0).max(8.0);
    let right = (width - options.margin_x / 2.0).max(left + 1.0);
    match endpoint.kind {
        MessageEndpointKind::Incoming => left,
        MessageEndpointKind::Outgoing => right,
        MessageEndpointKind::Unknown | MessageEndpointKind::Short => match direction {
            MessageDirection::RightToLeft => {
                if is_start {
                    right
                } else {
                    left
                }
            }
            _ => {
                if is_start {
                    left
                } else {
                    right
                }
            }
        },
        MessageEndpointKind::Participant => {
            if is_start {
                left
            } else {
                right
            }
        }
    }
}

pub(super) fn layout_legacy_message(
    message: &Message,
    participant_x: &HashMap<&str, f32>,
    options: &LayoutOptions,
    y: f32,
) -> MessageLayout {
    let start_x = participant_x
        .get(message.from.as_str())
        .copied()
        .unwrap_or(options.margin_x);
    let end_x = participant_x
        .get(message.to.as_str())
        .copied()
        .unwrap_or(options.margin_x);
    let direction = if message.from == message.to {
        MessageDirection::SelfMessage
    } else if end_x >= start_x {
        MessageDirection::LeftToRight
    } else {
        MessageDirection::RightToLeft
    };

    MessageLayout {
        from: message.from.clone(),
        to: message.to.clone(),
        label: message.label.clone(),
        line_style: message.line_style,
        arrow_raw: String::new(),
        arrow_features: MessageArrowFeatures::default(),
        activation_shortcut: None,
        from_endpoint_kind: MessageEndpointKind::Participant,
        to_endpoint_kind: MessageEndpointKind::Participant,
        direction,
        line: 0,
        start_x,
        end_x,
        y,
    }
}
