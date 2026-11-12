//! Arrow feature classification and visual endpoint semantics.
use super::*;

pub(super) fn message_arrow_features(
    raw_operator: &str,
    operator: MessageOperator,
    left: &MessageEndpoint,
    right: &MessageEndpoint,
) -> MessageArrowFeatures {
    let (visual_left, visual_right) = visual_arrow_sides(raw_operator, operator);
    let (start_side, end_side) = match operator {
        MessageOperator::RightSolid | MessageOperator::RightDashed => (visual_left, visual_right),
        MessageOperator::LeftSolid | MessageOperator::LeftDashed => (visual_right, visual_left),
        MessageOperator::BidirectionalSolid | MessageOperator::BidirectionalDashed => {
            (visual_left, visual_right)
        }
    };
    let mut features = MessageArrowFeatures {
        lost: start_side.cross || end_side.cross,
        thin: matches!(
            (start_side.head, end_side.head),
            (Some(MessageArrowHead::Open), _) | (_, Some(MessageArrowHead::Open))
        ),
        half: message_arrow_half(start_side.head, end_side.head),
        open_circle: start_side.circle || end_side.circle,
        bidirectional: raw_operator.contains("<->") || raw_operator.contains("<-->"),
        start_head: start_side.head,
        end_head: end_side.head,
        start_circle: start_side.circle,
        end_circle: end_side.circle,
        start_cross: start_side.cross,
        end_cross: end_side.cross,
        color: message_arrow_color(raw_operator),
        slant: message_arrow_slant(raw_operator),
        incoming: matches!(left.kind, MessageEndpointKind::Incoming)
            || matches!(right.kind, MessageEndpointKind::Incoming),
        outgoing: matches!(left.kind, MessageEndpointKind::Outgoing)
            || matches!(right.kind, MessageEndpointKind::Outgoing),
        unknown_endpoint: matches!(left.kind, MessageEndpointKind::Unknown)
            || matches!(right.kind, MessageEndpointKind::Unknown),
        short_endpoint: matches!(left.kind, MessageEndpointKind::Short)
            || matches!(right.kind, MessageEndpointKind::Short),
        ..MessageArrowFeatures::default()
    };

    if matches!(left.kind, MessageEndpointKind::Incoming)
        && matches!(right.kind, MessageEndpointKind::Outgoing)
    {
        features.bidirectional = true;
    }

    features
}

pub(super) fn visual_arrow_sides(
    raw_operator: &str,
    operator: MessageOperator,
) -> (VisualArrowSide, VisualArrowSide) {
    let mut left = VisualArrowSide {
        circle: raw_operator.starts_with('o'),
        cross: raw_operator.starts_with('x'),
        ..VisualArrowSide::default()
    };
    let mut right = VisualArrowSide {
        circle: raw_operator.ends_with('o'),
        cross: raw_operator.ends_with('x'),
        ..VisualArrowSide::default()
    };

    match operator {
        MessageOperator::RightSolid | MessageOperator::RightDashed => {
            right.head = right_arrow_head(raw_operator);
        }
        MessageOperator::LeftSolid | MessageOperator::LeftDashed => {
            left.head = left_arrow_head(raw_operator);
        }
        MessageOperator::BidirectionalSolid | MessageOperator::BidirectionalDashed => {
            left.head = Some(MessageArrowHead::Filled);
            right.head = Some(MessageArrowHead::Filled);
        }
    }

    if left.cross {
        left.head = None;
    }
    if right.cross {
        right.head = None;
    }

    (left, right)
}

pub(super) fn right_arrow_head(raw_operator: &str) -> Option<MessageArrowHead> {
    if raw_operator == "--" {
        return None;
    }
    if raw_operator.contains(">>") {
        return Some(MessageArrowHead::Open);
    }
    if raw_operator.contains("-\\") {
        return Some(MessageArrowHead::HalfBottom);
    }
    if raw_operator.contains("-/") {
        return Some(MessageArrowHead::HalfTop);
    }

    Some(MessageArrowHead::Filled)
}

pub(super) fn left_arrow_head(raw_operator: &str) -> Option<MessageArrowHead> {
    if raw_operator.contains("\\-") {
        return Some(MessageArrowHead::HalfBottom);
    }
    if raw_operator.contains("//") {
        return Some(MessageArrowHead::HalfTop);
    }
    if raw_operator.contains("<-") {
        return Some(MessageArrowHead::Filled);
    }

    Some(MessageArrowHead::Filled)
}

pub(super) fn message_arrow_half(
    start_head: Option<MessageArrowHead>,
    end_head: Option<MessageArrowHead>,
) -> Option<MessageArrowHalf> {
    let top = matches!(
        (start_head, end_head),
        (Some(MessageArrowHead::HalfTop), _) | (_, Some(MessageArrowHead::HalfTop))
    );
    let bottom = matches!(
        (start_head, end_head),
        (Some(MessageArrowHead::HalfBottom), _) | (_, Some(MessageArrowHead::HalfBottom))
    );
    match (top, bottom) {
        (true, true) => Some(MessageArrowHalf::Both),
        (true, false) => Some(MessageArrowHalf::Top),
        (false, true) => Some(MessageArrowHalf::Bottom),
        (false, false) => None,
    }
}

pub(super) fn message_arrow_color(raw_operator: &str) -> Option<String> {
    let start = raw_operator.find('[')?;
    let end = raw_operator[start + 1..].find(']')? + start + 1;
    let value = raw_operator[start + 1..end].trim();
    value.starts_with('#').then(|| value.to_owned())
}

pub(super) fn message_arrow_slant(raw_operator: &str) -> Option<i32> {
    let start = raw_operator.find('(')?;
    let end = raw_operator[start + 1..].find(')')? + start + 1;
    raw_operator[start + 1..end].trim().parse().ok()
}
