//! Vertical placement, page chrome and timeline event extents.
use super::*;

pub(super) fn has_page_header(diagram: &SequenceDiagram) -> bool {
    diagram.semantic_events.iter().any(|event| {
        matches!(
            event,
            SequenceSemanticEvent::HeaderFooter(event)
                if matches!(event.kind, HeaderFooterKind::Header)
        )
    })
}

pub(super) fn has_page_footer(diagram: &SequenceDiagram) -> bool {
    diagram.semantic_events.iter().any(|event| {
        matches!(
            event,
            SequenceSemanticEvent::HeaderFooter(event)
                if matches!(event.kind, HeaderFooterKind::Footer)
        )
    })
}

pub(super) fn has_page_title(diagram: &SequenceDiagram) -> bool {
    diagram
        .events
        .iter()
        .any(|event| matches!(event, SequenceEvent::Title(_)))
}

pub(super) fn page_header_baseline_y(options: &LayoutOptions) -> f32 {
    (options.margin_y - 8.0).max(PAGE_HEADER_BASELINE_Y)
}

pub(super) fn page_title_baseline_y(options: &LayoutOptions, has_page_header: bool) -> f32 {
    if has_page_header {
        (page_header_baseline_y(options) + 26.0).max(PAGE_TITLE_BASELINE_Y)
    } else {
        (options.margin_y + 4.0).max(PAGE_TITLE_ONLY_BASELINE_Y)
    }
}

pub(super) fn participant_head_y(
    options: &LayoutOptions,
    has_page_header: bool,
    has_page_title: bool,
) -> f32 {
    if has_page_title {
        return page_title_baseline_y(options, has_page_header) + PAGE_DECORATION_BOTTOM_GAP;
    }
    if has_page_header {
        return page_header_baseline_y(options) + 18.0;
    }
    options.margin_y
}

pub(super) fn place_timeline_event_y(
    event: &SequenceTimelineEventLayout,
    next_y: f32,
    last_event_bottom: &mut f32,
) -> f32 {
    let y = next_y.max(*last_event_bottom + event_top_extent(event) + EVENT_VERTICAL_GAP);
    *last_event_bottom = y + event_bottom_extent(event);
    y
}

pub(super) fn place_attached_note_y(
    event: &SequenceTimelineEventLayout,
    message_y: f32,
    last_event_bottom: &mut f32,
    allow_message_overlap: bool,
) -> f32 {
    let mut y = match event {
        SequenceTimelineEventLayout::Note(note) => {
            message_y + NOTE_BASELINE_OFFSET - note_box_height(&note.text) / 2.0
        }
        _ => message_y,
    };
    if !allow_message_overlap {
        y = y.max(*last_event_bottom + EVENT_VERTICAL_GAP + event_top_extent(event));
    }
    *last_event_bottom = (*last_event_bottom).max(y + event_bottom_extent(event));
    y
}

pub(super) fn place_page_break_y(
    event: &SequenceTimelineEventLayout,
    last_event_bottom: &mut f32,
) -> f32 {
    let y = *last_event_bottom + PAGE_BREAK_GAP;
    *last_event_bottom = y + event_bottom_extent(event);
    y
}

pub(super) fn set_timeline_event_y(event: &mut SequenceTimelineEventLayout, y: f32) {
    match event {
        SequenceTimelineEventLayout::Message(event) => event.y = y,
        SequenceTimelineEventLayout::Title(event) => event.y = y,
        SequenceTimelineEventLayout::Note(event) => event.y = y,
        SequenceTimelineEventLayout::GroupStart(event) => event.y = y,
        SequenceTimelineEventLayout::GroupElse(event) => event.y = y,
        SequenceTimelineEventLayout::GroupEnd(event) => event.y = y,
        SequenceTimelineEventLayout::Divider(event) => event.y = y,
        SequenceTimelineEventLayout::Delay(event) => event.y = y,
        SequenceTimelineEventLayout::Activation(event) => event.y = y,
        SequenceTimelineEventLayout::Return(event) => event.y = y,
        SequenceTimelineEventLayout::BoxStart(event) => event.y = y,
        SequenceTimelineEventLayout::BoxEnd(event) => event.y = y,
        SequenceTimelineEventLayout::Semantic(event) => set_semantic_event_y(event, y),
    }
}

pub(super) fn set_semantic_event_y(event: &mut SemanticEventLayout, y: f32) {
    match event {
        SemanticEventLayout::Autonumber(event) => event.y = y,
        SemanticEventLayout::PageBreak(event) => event.y = y,
        SemanticEventLayout::HeaderFooter(event) => event.y = y,
        SemanticEventLayout::Reference(event) => event.y = y,
        SemanticEventLayout::Space(event) => event.y = y,
        SemanticEventLayout::Mainframe(event) => event.y = y,
        SemanticEventLayout::StyleBlock(event) => event.y = y,
        SemanticEventLayout::HideUnlinked(event) => event.y = y,
        SemanticEventLayout::Autoactivate(event) => event.y = y,
    }
}

pub(super) fn event_top_extent(event: &SequenceTimelineEventLayout) -> f32 {
    match event {
        SequenceTimelineEventLayout::Message(event) => label_top_extent(&event.label),
        SequenceTimelineEventLayout::Title(event) => marker_top_extent(&event.text),
        SequenceTimelineEventLayout::Note(_) => NOTE_BASELINE_OFFSET,
        SequenceTimelineEventLayout::GroupStart(event) => {
            22.0_f32.max(label_top_extent(&group_start_label(event)))
        }
        SequenceTimelineEventLayout::GroupElse(_) => 0.0,
        SequenceTimelineEventLayout::GroupEnd(_) => 0.0,
        SequenceTimelineEventLayout::Divider(event) => label_top_extent(&event.text),
        SequenceTimelineEventLayout::Delay(event) => {
            event.text.as_deref().map(label_top_extent).unwrap_or(8.0)
        }
        SequenceTimelineEventLayout::Activation(_) => 14.0,
        SequenceTimelineEventLayout::Return(event) => {
            event.label.as_deref().map(label_top_extent).unwrap_or(8.0)
        }
        SequenceTimelineEventLayout::BoxStart(event) => event
            .title
            .as_deref()
            .map(|title| 22.0_f32.max(label_top_extent(title)))
            .unwrap_or(22.0),
        SequenceTimelineEventLayout::BoxEnd(_) => 8.0,
        SequenceTimelineEventLayout::Semantic(SemanticEventLayout::PageBreak(_)) => 0.0,
        SequenceTimelineEventLayout::Semantic(event) => {
            marker_top_extent(&semantic_event_label(event))
        }
    }
}

pub(super) fn event_bottom_extent(event: &SequenceTimelineEventLayout) -> f32 {
    match event {
        SequenceTimelineEventLayout::Message(event) => {
            if matches!(event.direction, MessageDirection::SelfMessage) {
                36.0
            } else if event.activation_shortcut.is_some() {
                28.0
            } else {
                12.0
            }
        }
        SequenceTimelineEventLayout::Title(event) => marker_bottom_extent(&event.text),
        SequenceTimelineEventLayout::Note(event) => {
            note_box_height(&event.text) - NOTE_BASELINE_OFFSET
        }
        SequenceTimelineEventLayout::GroupStart(event) => {
            marker_bottom_extent(&group_start_label(event)).max(12.0)
        }
        SequenceTimelineEventLayout::GroupElse(_) => 22.0,
        SequenceTimelineEventLayout::GroupEnd(_) => 0.0,
        SequenceTimelineEventLayout::Divider(_) => 8.0,
        SequenceTimelineEventLayout::Delay(_) => 8.0,
        SequenceTimelineEventLayout::Activation(_) => 18.0,
        SequenceTimelineEventLayout::Return(_) => 8.0,
        SequenceTimelineEventLayout::BoxStart(event) => event
            .title
            .as_deref()
            .map(marker_bottom_extent)
            .unwrap_or(12.0)
            .max(12.0),
        SequenceTimelineEventLayout::BoxEnd(_) => 8.0,
        SequenceTimelineEventLayout::Semantic(SemanticEventLayout::PageBreak(event)) => {
            page_break_bottom_extent(event)
        }
        SequenceTimelineEventLayout::Semantic(event) => {
            marker_bottom_extent(&semantic_event_label(event))
        }
    }
}

pub(super) fn label_top_extent(text: &str) -> f32 {
    20.0 + DISPLAY_TEXT_LINE_HEIGHT * (text_line_count(text).saturating_sub(1) as f32)
}

pub(super) fn marker_top_extent(text: &str) -> f32 {
    18.0_f32.max(label_top_extent(text))
}

pub(super) fn marker_bottom_extent(text: &str) -> f32 {
    10.0 + DISPLAY_TEXT_LINE_HEIGHT * (text_line_count(text).saturating_sub(1) as f32)
}

pub(super) fn page_break_bottom_extent(event: &PageBreakLayout) -> f32 {
    event.label.as_deref().map_or(8.0, |label| {
        PAGE_BREAK_LABEL_GAP + DISPLAY_TEXT_LINE_HEIGHT * (text_line_count(label) as f32)
    })
}
