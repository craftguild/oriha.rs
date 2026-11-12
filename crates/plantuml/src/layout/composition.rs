//! Sequence layout orchestration with explicit text measurements.
use super::*;

pub fn layout_sequence(diagram: &SequenceDiagram, options: &LayoutOptions) -> SequenceLayout {
    layout_sequence_with_text_measurer(diagram, options, &ApproxTextMeasurer)
}

pub fn layout_sequence_with_text_measurer(
    diagram: &SequenceDiagram,
    options: &LayoutOptions,
    measurer: &dyn TextMeasurer,
) -> SequenceLayout {
    let has_page_header = has_page_header(diagram);
    let has_page_title = has_page_title(diagram);
    let has_page_footer = has_page_footer(diagram);
    let page_header_y = page_header_baseline_y(options);
    let page_title_y = page_title_baseline_y(options, has_page_header);
    let participant_head_y = participant_head_y(options, has_page_header, has_page_title);

    let mut participants: Vec<ParticipantLayout> = diagram
        .participants
        .iter()
        .map(|participant| {
            let declaration = participant.declaration.as_ref();
            ParticipantLayout {
                id: participant.id.clone(),
                name: participant.name.clone(),
                kind: participant.kind,
                color: declaration
                    .and_then(|declaration| declaration.color.as_deref())
                    .map(participant_svg_color),
                display: declaration.and_then(|declaration| declaration.display.clone()),
                x: 0.0,
                head_y: participant_head_y,
                tail_y: 0.0,
                head_width: participant_head_width(participant, options, measurer),
                head_height: participant_head_height(participant, options),
            }
        })
        .collect();
    let max_head_height = participants
        .iter()
        .map(|participant| participant.head_height)
        .fold(options.participant_head_height, f32::max);
    let message_start_y = options
        .message_start_y
        .max(participant_head_y + max_head_height + 32.0);
    let autonumbered_text = autonumbered_text(diagram);
    let participant_gaps = participant_gaps(
        diagram,
        &participants,
        options,
        &autonumbered_text.message_labels,
        measurer,
    );
    position_participants(&mut participants, &participant_gaps, options);
    let mut width = sequence_width(&participants, options);

    let participant_x: HashMap<&str, f32> = participants
        .iter()
        .map(|participant| (participant.id.as_str(), participant.x))
        .collect();

    let mut messages = Vec::new();
    let mut timeline = Vec::new();
    let mut next_y = message_start_y;
    let mut last_event_bottom = message_start_y - EVENT_VERTICAL_GAP - 24.0;
    let mut semantic_index = 0;
    let mut last_message_for_note: Option<SequenceMessage> = None;
    let mut last_message_y: Option<f32> = None;
    let mut title_layout = None;
    let mut header_layout = None;
    let mut footer_layout = None;

    for event in &diagram.events {
        let timeline_event = match event {
            SequenceEvent::Message(message) => {
                let label = autonumbered_text
                    .message_labels
                    .get(&message.line)
                    .map_or_else(
                        || decode_plantuml_unicode_escapes(&message.label),
                        Clone::clone,
                    );
                let layout =
                    layout_sequence_message(message, &label, &participant_x, options, width, 0.0);
                last_message_for_note = Some(message.clone());
                Some(SequenceTimelineEventLayout::Message(layout))
            }
            SequenceEvent::Title(title) => {
                title_layout = Some(TextEventLayout {
                    text: title.text.clone(),
                    line: title.line,
                    y: page_title_y,
                });
                None
            }
            SequenceEvent::Note(note) => {
                let text = autonumbered_text
                    .note_texts
                    .get(&note.line)
                    .cloned()
                    .unwrap_or_else(|| decode_plantuml_unicode_escapes(&note.text));
                let text = normalize_note_text(&text);
                let explicit_targets = !note.targets.is_empty();
                let targets =
                    note_targets_for_layout(note, last_message_for_note.as_ref(), &participant_x);
                Some(SequenceTimelineEventLayout::Note(NoteLayout {
                    placement: note.placement.clone(),
                    targets,
                    explicit_targets,
                    text,
                    color: note.color.clone(),
                    shape: note.shape,
                    multiline: note.multiline,
                    line: note.line,
                    y: 0.0,
                }))
            }
            SequenceEvent::GroupStart(group) => {
                Some(SequenceTimelineEventLayout::GroupStart(GroupStartLayout {
                    kind: group.kind,
                    label: group.label.clone(),
                    line: group.line,
                    y: 0.0,
                }))
            }
            SequenceEvent::GroupElse(group_else) => {
                Some(SequenceTimelineEventLayout::GroupElse(GroupElseLayout {
                    label: group_else.label.clone(),
                    line: group_else.line,
                    y: 0.0,
                }))
            }
            SequenceEvent::GroupEnd(group_end) => {
                Some(SequenceTimelineEventLayout::GroupEnd(GroupEndLayout {
                    line: group_end.line,
                    y: 0.0,
                }))
            }
            SequenceEvent::Divider(divider) => {
                Some(SequenceTimelineEventLayout::Divider(TextEventLayout {
                    text: divider.text.clone(),
                    line: divider.line,
                    y: 0.0,
                }))
            }
            SequenceEvent::Delay(delay) => Some(SequenceTimelineEventLayout::Delay(DelayLayout {
                text: delay.text.clone(),
                line: delay.line,
                y: 0.0,
            })),
            SequenceEvent::Activation(activation) => Some(SequenceTimelineEventLayout::Activation(
                ActivationEventLayout {
                    kind: activation.kind,
                    participant: activation.participant.clone(),
                    color: activation.color.clone(),
                    line: activation.line,
                    y: 0.0,
                },
            )),
            SequenceEvent::Return(return_event) => {
                Some(SequenceTimelineEventLayout::Return(ReturnEventLayout {
                    label: return_event.label.clone(),
                    line: return_event.line,
                    y: 0.0,
                }))
            }
            SequenceEvent::BoxStart(box_block) => {
                Some(SequenceTimelineEventLayout::BoxStart(BoxStartLayout {
                    title: box_block.title.clone(),
                    color: box_block.color.clone(),
                    line: box_block.line,
                    y: 0.0,
                }))
            }
            SequenceEvent::BoxEnd(box_end) => {
                Some(SequenceTimelineEventLayout::BoxEnd(BoxEndLayout {
                    line: box_end.line,
                    y: 0.0,
                }))
            }
            SequenceEvent::Pragma(pragma) => {
                let semantic_event = layout_semantic_event_at_line(
                    &diagram.semantic_events,
                    &mut semantic_index,
                    pragma.line,
                    0.0,
                );
                match semantic_event {
                    Some(SemanticEventLayout::HeaderFooter(mut event)) => {
                        match event.kind {
                            HeaderFooterKind::Header => {
                                event.y = page_header_y;
                                header_layout = Some(event);
                            }
                            HeaderFooterKind::Footer => {
                                footer_layout = Some(event);
                            }
                        }
                        None
                    }
                    other => other.map(SequenceTimelineEventLayout::Semantic),
                }
            }
            SequenceEvent::ParticipantDeclaration(_)
            | SequenceEvent::HideFootbox(_)
            | SequenceEvent::SkinParam(_) => None,
        };

        if let Some(mut timeline_event) = timeline_event {
            let y = match &timeline_event {
                SequenceTimelineEventLayout::Semantic(SemanticEventLayout::PageBreak(_)) => {
                    place_page_break_y(&timeline_event, &mut last_event_bottom)
                }
                SequenceTimelineEventLayout::Note(note)
                    if matches!(note.placement, NotePlacement::Left | NotePlacement::Right)
                        && !note.targets.is_empty()
                        && last_message_y.is_some() =>
                {
                    let previous_event_is_attached_message = !note.explicit_targets
                        && matches!(
                            timeline.last(),
                            Some(SequenceTimelineEventLayout::Message(message))
                                if Some(message.y) == last_message_y
                        );
                    place_attached_note_y(
                        &timeline_event,
                        last_message_y.expect("checked last message y"),
                        &mut last_event_bottom,
                        previous_event_is_attached_message,
                    )
                }
                _ => place_timeline_event_y(&timeline_event, next_y, &mut last_event_bottom),
            };
            set_timeline_event_y(&mut timeline_event, y);
            if let SequenceTimelineEventLayout::Message(message) = &timeline_event {
                messages.push(message.clone());
                last_message_y = Some(message.y);
            }
            timeline.push(timeline_event);
            next_y = y + options.message_gap;
        }
    }

    if timeline.is_empty() && !diagram.messages.is_empty() {
        for message in &diagram.messages {
            let layout = layout_legacy_message(message, &participant_x, options, next_y);
            let mut timeline_event = SequenceTimelineEventLayout::Message(layout);
            let y = place_timeline_event_y(&timeline_event, next_y, &mut last_event_bottom);
            set_timeline_event_y(&mut timeline_event, y);
            if let SequenceTimelineEventLayout::Message(message) = &timeline_event {
                messages.push(message.clone());
            }
            timeline.push(timeline_event);
            next_y = y + options.message_gap;
        }
    }

    let left_note_shift = left_note_shift_for_notes(&timeline, &participants, measurer);
    if left_note_shift > 0.0 {
        shift_horizontal_layout(
            &mut participants,
            &mut messages,
            &mut timeline,
            left_note_shift,
        );
        width += left_note_shift;
    }
    width = expand_width_for_notes(width, &timeline, &participants, options, measurer);

    let last_event_bottom = timeline
        .iter()
        .map(|event| event.y() + event_bottom_extent(event))
        .fold(message_start_y, f32::max);
    let tail_y = last_event_bottom + options.tail_gap;
    for participant in &mut participants {
        participant.tail_y = tail_y;
    }
    let footer_reserved_height = if has_page_footer {
        PAGE_FOOTER_RESERVED_HEIGHT
    } else {
        0.0
    };
    let height = tail_y + max_head_height + options.margin_y + footer_reserved_height;
    if let Some(footer) = &mut footer_layout {
        footer.y = height - 12.0;
    }

    SequenceLayout {
        width,
        height,
        view_box: format!("0 0 {} {}", width.ceil() as u32, height.ceil() as u32),
        participants,
        messages,
        timeline,
        title: title_layout,
        header: header_layout,
        footer: footer_layout,
    }
}
