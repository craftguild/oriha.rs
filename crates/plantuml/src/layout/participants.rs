//! Participant dimensions, spacing and horizontal positions.
use super::*;

pub(super) fn participant_head_width(
    participant: &crate::sequence::Participant,
    options: &LayoutOptions,
    measurer: &dyn TextMeasurer,
) -> f32 {
    let display_width = participant
        .declaration
        .as_ref()
        .and_then(|declaration| declaration.display.as_ref())
        .map(|display| participant_display_width(display, measurer));
    let label_width =
        display_width.unwrap_or_else(|| participant_label_width(&participant.name, measurer));
    let icon_min_width = if matches!(participant.kind, ParticipantKind::Participant) {
        0.0
    } else {
        72.0
    };

    options
        .participant_head_width
        .max(label_width + PARTICIPANT_TEXT_PADDING_X)
        .max(icon_min_width + PARTICIPANT_TEXT_PADDING_X)
}

pub(super) fn participant_head_height(
    participant: &crate::sequence::Participant,
    options: &LayoutOptions,
) -> f32 {
    let display_height = participant
        .declaration
        .as_ref()
        .and_then(|declaration| declaration.display.as_ref())
        .map(participant_display_height)
        .unwrap_or(options.participant_head_height);
    let label_height = participant_label_height(&participant.name);
    let kind_height = if matches!(participant.kind, ParticipantKind::Participant) {
        options.participant_head_height
    } else {
        ICON_PARTICIPANT_HEAD_HEIGHT
    };

    options
        .participant_head_height
        .max(display_height)
        .max(label_height)
        .max(kind_height)
}

pub(super) fn participant_label_width(name: &str, measurer: &dyn TextMeasurer) -> f32 {
    participant_label_lines(name)
        .into_iter()
        .map(|line| estimate_text_width(&line, measurer))
        .fold(0.0, f32::max)
}

pub(super) fn participant_label_height(name: &str) -> f32 {
    let line_count = participant_label_lines(name).len().max(1) as f32;
    line_count * DISPLAY_TEXT_LINE_HEIGHT + PARTICIPANT_LABEL_PADDING_Y
}

pub(super) fn participant_display_width(
    display: &ParticipantDisplay,
    measurer: &dyn TextMeasurer,
) -> f32 {
    participant_display_lines(display)
        .into_iter()
        .map(|line| {
            if matches!(line.kind, ParticipantDisplayLineKind::Separator) || line.text.is_empty() {
                48.0
            } else {
                estimate_text_width(&line.text, measurer)
            }
        })
        .fold(0.0, f32::max)
}

pub(super) fn participant_display_height(display: &ParticipantDisplay) -> f32 {
    let content_height = participant_display_lines(display)
        .into_iter()
        .map(|line| {
            if matches!(line.kind, ParticipantDisplayLineKind::Separator) {
                DISPLAY_SEPARATOR_HEIGHT
            } else {
                DISPLAY_TEXT_LINE_HEIGHT
            }
        })
        .sum::<f32>();

    content_height + 20.0
}

pub(super) fn participant_gaps(
    diagram: &SequenceDiagram,
    participants: &[ParticipantLayout],
    options: &LayoutOptions,
    autonumbered_labels: &HashMap<usize, String>,
    measurer: &dyn TextMeasurer,
) -> Vec<f32> {
    let mut gaps = vec![options.participant_gap; participants.len().saturating_sub(1)];
    if gaps.is_empty() {
        return gaps;
    }

    let participant_index: HashMap<&str, usize> = participants
        .iter()
        .enumerate()
        .map(|(index, participant)| (participant.id.as_str(), index))
        .collect();

    for event in &diagram.events {
        if let SequenceEvent::Message(message) = event {
            let label = autonumbered_labels
                .get(&message.line)
                .map_or(message.label.as_str(), String::as_str);
            expand_gaps_for_message_label(
                &mut gaps,
                &participant_index,
                &message.from.id,
                &message.to.id,
                label,
                measurer,
            );
        }
    }

    for message in &diagram.messages {
        expand_gaps_for_message_label(
            &mut gaps,
            &participant_index,
            &message.from,
            &message.to,
            &message.label,
            measurer,
        );
    }

    for (index, gap) in gaps.iter_mut().enumerate() {
        let min_box_gap = participants[index].head_width / 2.0
            + participants[index + 1].head_width / 2.0
            + PARTICIPANT_HEAD_GUTTER;
        *gap = gap.max(min_box_gap);
    }

    gaps
}

pub(super) fn expand_gaps_for_message_label(
    gaps: &mut [f32],
    participant_index: &HashMap<&str, usize>,
    from: &str,
    to: &str,
    label: &str,
    measurer: &dyn TextMeasurer,
) {
    let Some(&from_index) = participant_index.get(from) else {
        return;
    };
    let Some(&to_index) = participant_index.get(to) else {
        return;
    };
    if from_index == to_index {
        return;
    }

    let left = from_index.min(to_index);
    let right = from_index.max(to_index);
    let span_gap_count = right - left;
    let required_span = estimate_text_block_width(label, measurer) + MESSAGE_LABEL_PADDING_X;
    let required_gap = required_span / (span_gap_count as f32);

    for gap in &mut gaps[left..right] {
        *gap = gap.max(required_gap);
    }
}

pub(super) fn position_participants(
    participants: &mut [ParticipantLayout],
    participant_gaps: &[f32],
    options: &LayoutOptions,
) {
    let Some(first) = participants.first_mut() else {
        return;
    };
    first.x = options.margin_x + first.head_width / 2.0;

    for index in 1..participants.len() {
        participants[index].x = participants[index - 1].x + participant_gaps[index - 1];
    }
}

pub(super) fn sequence_width(participants: &[ParticipantLayout], options: &LayoutOptions) -> f32 {
    let Some(last) = participants.last() else {
        return options
            .margin_x
            .mul_add(2.0, options.participant_head_width);
    };

    last.x + last.head_width / 2.0 + options.margin_x
}

pub(super) fn participant_svg_color(color: &str) -> String {
    let color = color.trim();
    match color.strip_prefix('#') {
        Some(named_color) if !is_hex_color_body(named_color) => named_color.to_owned(),
        _ => color.to_owned(),
    }
}

pub(super) fn is_hex_color_body(value: &str) -> bool {
    matches!(value.len(), 3 | 4 | 6 | 8)
        && value.chars().all(|character| character.is_ascii_hexdigit())
}
