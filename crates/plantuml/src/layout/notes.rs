//! Note targeting, note dimensions and horizontal expansion.
use super::*;

pub(super) fn note_targets_for_layout(
    note: &Note,
    last_message: Option<&SequenceMessage>,
    participant_x: &HashMap<&str, f32>,
) -> Vec<String> {
    if !note.targets.is_empty() {
        return note.targets.clone();
    }

    let Some(message) = last_message else {
        return Vec::new();
    };

    let choose_right = match note.placement {
        NotePlacement::Left => false,
        NotePlacement::Right => true,
        NotePlacement::Over | NotePlacement::Across | NotePlacement::Unknown(_) => {
            return Vec::new();
        }
    };

    side_message_target(message, participant_x, choose_right)
        .into_iter()
        .collect()
}

pub(super) fn side_message_target(
    message: &SequenceMessage,
    participant_x: &HashMap<&str, f32>,
    choose_right: bool,
) -> Option<String> {
    let mut endpoints = [&message.from, &message.to]
        .into_iter()
        .filter(|endpoint| endpoint.is_participant())
        .filter_map(|endpoint| {
            participant_x
                .get(endpoint.id.as_str())
                .map(|x| (endpoint.id.clone(), *x))
        })
        .collect::<Vec<_>>();

    endpoints.sort_by(|(_, left_x), (_, right_x)| left_x.total_cmp(right_x));

    if choose_right {
        endpoints.pop().map(|(id, _)| id)
    } else {
        endpoints.into_iter().next().map(|(id, _)| id)
    }
}

pub(super) fn normalize_note_text(text: &str) -> String {
    let lines = text
        .lines()
        .map(str::trim)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if lines.is_empty() {
        text.trim().to_owned()
    } else {
        lines.join("\n")
    }
}

pub(super) fn expand_width_for_notes(
    width: f32,
    timeline: &[SequenceTimelineEventLayout],
    participants: &[ParticipantLayout],
    options: &LayoutOptions,
    measurer: &dyn TextMeasurer,
) -> f32 {
    let participant_x: HashMap<&str, f32> = participants
        .iter()
        .map(|participant| (participant.id.as_str(), participant.x))
        .collect();

    timeline.iter().fold(width, |width, event| {
        let SequenceTimelineEventLayout::Note(note) = event else {
            return width;
        };

        let note_width = note_box_width_with_text_measurer(&note.text, measurer);
        match note.placement {
            NotePlacement::Right => {
                let anchor_x = note_anchor_x_from_positions(width, note, &participant_x);
                width.max(anchor_x + NOTE_TARGET_GAP + note_width + options.margin_x)
            }
            NotePlacement::Over | NotePlacement::Unknown(_) => {
                let anchor_x = note_anchor_x_from_positions(width, note, &participant_x);
                width
                    .max(note_width + options.margin_x * 2.0)
                    .max(anchor_x + note_width / 2.0 + NOTE_CANVAS_MARGIN)
            }
            NotePlacement::Across => width.max(note_width + options.margin_x * 2.0),
            NotePlacement::Left => width,
        }
    })
}

pub(super) fn left_note_shift_for_notes(
    timeline: &[SequenceTimelineEventLayout],
    participants: &[ParticipantLayout],
    measurer: &dyn TextMeasurer,
) -> f32 {
    let participant_x: HashMap<&str, f32> = participants
        .iter()
        .map(|participant| (participant.id.as_str(), participant.x))
        .collect();

    timeline.iter().fold(0.0_f32, |shift, event| {
        let SequenceTimelineEventLayout::Note(note) = event else {
            return shift;
        };
        if note.targets.is_empty() {
            return shift;
        }

        let note_width = note_box_width_with_text_measurer(&note.text, measurer);
        let anchor_x = note_anchor_x_from_positions(0.0, note, &participant_x);
        let rect_x = match note.placement {
            NotePlacement::Left => anchor_x - note_width - NOTE_TARGET_GAP,
            NotePlacement::Over | NotePlacement::Unknown(_) => anchor_x - note_width / 2.0,
            NotePlacement::Right | NotePlacement::Across => return shift,
        };
        shift.max((NOTE_CANVAS_MARGIN - rect_x).max(0.0))
    })
}

pub(super) fn shift_horizontal_layout(
    participants: &mut [ParticipantLayout],
    messages: &mut [MessageLayout],
    timeline: &mut [SequenceTimelineEventLayout],
    delta: f32,
) {
    for participant in participants {
        participant.x += delta;
    }
    for message in messages {
        shift_message_layout_x(message, delta);
    }
    for event in timeline {
        if let SequenceTimelineEventLayout::Message(message) = event {
            shift_message_layout_x(message, delta);
        }
    }
}

pub(super) fn shift_message_layout_x(message: &mut MessageLayout, delta: f32) {
    if matches!(message.from_endpoint_kind, MessageEndpointKind::Participant) {
        message.start_x += delta;
    }
    if matches!(message.to_endpoint_kind, MessageEndpointKind::Participant) {
        message.end_x += delta;
    }
}

pub(super) fn note_anchor_x_from_positions(
    width: f32,
    note: &NoteLayout,
    participant_x: &HashMap<&str, f32>,
) -> f32 {
    if matches!(note.placement, NotePlacement::Across) {
        return width / 2.0;
    }

    let target_positions = note
        .targets
        .iter()
        .filter_map(|target| participant_x.get(target.as_str()))
        .copied()
        .collect::<Vec<_>>();

    if target_positions.is_empty() {
        width / 2.0
    } else {
        target_positions.iter().sum::<f32>() / (target_positions.len() as f32)
    }
}

pub(crate) fn note_box_width_with_text_measurer(text: &str, measurer: &dyn TextMeasurer) -> f32 {
    (estimate_text_block_width(text, measurer) + NOTE_TEXT_PADDING_X * 2.0).max(NOTE_MIN_WIDTH)
}

pub(crate) fn note_box_height(text: &str) -> f32 {
    let extra_lines = text_line_count(text).saturating_sub(1) as f32;
    NOTE_BASELINE_OFFSET + NOTE_BOTTOM_PADDING + DISPLAY_TEXT_LINE_HEIGHT * extra_lines
}

pub(super) fn estimate_text_block_width(text: &str, measurer: &dyn TextMeasurer) -> f32 {
    text_lines(text)
        .into_iter()
        .map(|line| estimate_text_width(&line, measurer))
        .fold(0.0, f32::max)
}

pub(super) fn text_line_count(text: &str) -> usize {
    text_lines(text).len().max(1)
}
