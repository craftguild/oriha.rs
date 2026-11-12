//! Text measurement collection for participants and timeline events.
use super::*;

pub(crate) fn collect_sequence_render_text_measurements(
    layout: &SequenceLayout,
    measurer: &dyn TextMeasurer,
) {
    for participant in &layout.participants {
        collect_participant_render_text_measurements(participant, measurer);
    }

    if let Some(header) = &layout.header {
        measure_render_text_block(&page_counter_text(&header.text), measurer);
    }
    if let Some(title) = &layout.title {
        measure_render_text_block(&title.text, measurer);
    }
    if let Some(footer) = &layout.footer {
        measure_render_text_block(&page_counter_text(&footer.text), measurer);
    }

    for frame in group_frames(layout) {
        let (label, condition) = group_header_parts(frame.start);
        measure_render_text_block(&label, measurer);
        if let Some(condition) = condition {
            measure_render_text_block(&condition, measurer);
        }
    }

    for message in &layout.messages {
        measure_render_text_block(&message.label, measurer);
    }

    for event in &layout.timeline {
        collect_timeline_event_render_text_measurements(event, measurer);
    }
}

pub(super) fn collect_participant_render_text_measurements(
    participant: &ParticipantLayout,
    measurer: &dyn TextMeasurer,
) {
    if let Some(display) = &participant.display {
        for line in participant_display_lines(display) {
            if matches!(line.kind, ParticipantDisplayLineKind::Separator) {
                continue;
            }
            measure_render_text_block(&line.text, measurer);
        }
    } else {
        for line in participant_label_lines(&participant.name) {
            measure_render_text_block(&line, measurer);
        }
    }
}

pub(super) fn collect_timeline_event_render_text_measurements(
    event: &SequenceTimelineEventLayout,
    measurer: &dyn TextMeasurer,
) {
    match event {
        SequenceTimelineEventLayout::Message(message) => {
            measure_render_text_block(&message.label, measurer);
        }
        SequenceTimelineEventLayout::Title(title) => {
            measure_render_text_block(&title.text, measurer);
        }
        SequenceTimelineEventLayout::Note(note) => {
            measure_render_text_block(&note.text, measurer);
        }
        SequenceTimelineEventLayout::GroupStart(_) => {}
        SequenceTimelineEventLayout::GroupElse(group_else) => {
            measure_render_text_block(&group_else_label(group_else), measurer);
        }
        SequenceTimelineEventLayout::GroupEnd(_) => {}
        SequenceTimelineEventLayout::Divider(divider) => {
            measure_render_text_block(&divider.text, measurer);
        }
        SequenceTimelineEventLayout::Delay(delay) => {
            if let Some(text) = &delay.text {
                measure_render_text_block(text, measurer);
            }
        }
        SequenceTimelineEventLayout::Activation(_) => {}
        SequenceTimelineEventLayout::Return(return_event) => {
            if let Some(label) = &return_event.label {
                measure_render_text_block(label, measurer);
            }
        }
        SequenceTimelineEventLayout::BoxStart(_) | SequenceTimelineEventLayout::BoxEnd(_) => {}
        SequenceTimelineEventLayout::Semantic(semantic) => {
            collect_semantic_event_render_text_measurements(semantic, measurer);
        }
    }
}

pub(super) fn collect_semantic_event_render_text_measurements(
    event: &SemanticEventLayout,
    measurer: &dyn TextMeasurer,
) {
    match event {
        SemanticEventLayout::Autonumber(event) => {
            let mut label = format!("autonumber {}", autonumber_command_name(event.command));
            if !event.args.is_empty() {
                label.push(' ');
                label.push_str(&event.args.join(" "));
            }
            measure_render_text_block(label.trim_end(), measurer);
        }
        SemanticEventLayout::PageBreak(event) => {
            if let Some(label) = &event.label {
                measure_render_text_block(label, measurer);
            }
        }
        SemanticEventLayout::HeaderFooter(event) => {
            measure_render_text_block(&page_counter_text(&event.text), measurer);
        }
        SemanticEventLayout::Reference(event) => {
            measure_render_text_block(event.text.as_deref().unwrap_or("ref"), measurer);
        }
        SemanticEventLayout::Space(event) => {
            let label = event
                .size
                .as_deref()
                .map_or("space".to_owned(), |size| format!("space {size}"));
            measure_render_text_block(&label, measurer);
        }
        SemanticEventLayout::Mainframe(event) => {
            measure_render_text_block(event.title.as_deref().unwrap_or("mainframe"), measurer);
        }
        SemanticEventLayout::StyleBlock(_) => {
            measure_render_text_block("style block", measurer);
        }
        SemanticEventLayout::HideUnlinked(_) => {
            measure_render_text_block("hide unlinked", measurer);
        }
        SemanticEventLayout::Autoactivate(event) => {
            let label = if event.enabled {
                "autoactivate on"
            } else {
                "autoactivate off"
            };
            measure_render_text_block(label, measurer);
        }
    }
}

pub(super) fn measure_render_text_block(text: &str, measurer: &dyn TextMeasurer) {
    for line in text_lines(text) {
        measurer.measure_inline_text(&line);
    }
}
