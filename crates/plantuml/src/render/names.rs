//! Stable serialized names for diagram event attributes.
use super::*;

pub(super) fn note_placement_name(placement: &NotePlacement) -> &str {
    match placement {
        NotePlacement::Left => "left",
        NotePlacement::Right => "right",
        NotePlacement::Over => "over",
        NotePlacement::Across => "across",
        NotePlacement::Unknown(_) => "unknown",
    }
}

pub(super) fn note_shape_name(shape: NoteShape) -> &'static str {
    match shape {
        NoteShape::Normal => "normal",
        NoteShape::Rectangle => "rectangle",
        NoteShape::Hexagon => "hexagon",
    }
}

pub(super) fn group_kind_name(kind: GroupKind) -> &'static str {
    match kind {
        GroupKind::Alt => "alt",
        GroupKind::Opt => "opt",
        GroupKind::Loop => "loop",
        GroupKind::Par => "par",
        GroupKind::Break => "break",
        GroupKind::Critical => "critical",
        GroupKind::Group => "group",
        GroupKind::Partition => "partition",
    }
}

pub(super) fn participant_kind_name(kind: ParticipantKind) -> &'static str {
    match kind {
        ParticipantKind::Participant => "participant",
        ParticipantKind::Actor => "actor",
        ParticipantKind::Boundary => "boundary",
        ParticipantKind::Control => "control",
        ParticipantKind::Entity => "entity",
        ParticipantKind::Database => "database",
        ParticipantKind::Collections => "collections",
        ParticipantKind::Queue => "queue",
    }
}

pub(super) fn activation_kind_name(kind: ActivationKind) -> &'static str {
    match kind {
        ActivationKind::Create => "create",
        ActivationKind::Activate => "activate",
        ActivationKind::Deactivate => "deactivate",
        ActivationKind::Destroy => "destroy",
    }
}

pub(super) fn activation_shortcut_target_name(target: MessageActivationTarget) -> &'static str {
    match target {
        MessageActivationTarget::From => "from",
        MessageActivationTarget::To => "to",
    }
}

pub(super) fn message_arrow_half_name(half: MessageArrowHalf) -> &'static str {
    match half {
        MessageArrowHalf::Top => "top",
        MessageArrowHalf::Bottom => "bottom",
        MessageArrowHalf::Both => "both",
    }
}

pub(super) fn arrow_head_name(head: MessageArrowHead) -> &'static str {
    match head {
        MessageArrowHead::Filled => "filled",
        MessageArrowHead::Open => "open",
        MessageArrowHead::HalfTop => "half-top",
        MessageArrowHead::HalfBottom => "half-bottom",
    }
}

pub(super) fn message_endpoint_kind_name(kind: MessageEndpointKind) -> &'static str {
    match kind {
        MessageEndpointKind::Participant => "participant",
        MessageEndpointKind::Incoming => "incoming",
        MessageEndpointKind::Outgoing => "outgoing",
        MessageEndpointKind::Unknown => "unknown",
        MessageEndpointKind::Short => "short",
    }
}

pub(super) fn autonumber_command_name(command: AutonumberCommand) -> &'static str {
    match command {
        AutonumberCommand::Start => "start",
        AutonumberCommand::Stop => "stop",
        AutonumberCommand::Resume => "resume",
        AutonumberCommand::Increment => "increment",
        AutonumberCommand::Off => "off",
    }
}

pub(super) fn header_footer_kind_name(kind: HeaderFooterKind) -> &'static str {
    match kind {
        HeaderFooterKind::Header => "header",
        HeaderFooterKind::Footer => "footer",
    }
}

pub(super) fn message_direction_name(direction: MessageDirection) -> &'static str {
    match direction {
        MessageDirection::LeftToRight => "left-to-right",
        MessageDirection::RightToLeft => "right-to-left",
        MessageDirection::Bidirectional => "bidirectional",
        MessageDirection::SelfMessage => "self",
    }
}
