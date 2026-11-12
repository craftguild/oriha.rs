//! Group-frame nesting, labels and horizontal bounds.
use super::*;

pub(super) fn render_group_frames(
    svg: &mut String,
    layout: &SequenceLayout,
    theme: &PlantUmlTheme,
    measurer: &dyn TextMeasurer,
) {
    for frame in group_frames(layout) {
        render_group_frame(svg, layout, frame, theme, measurer);
    }
}

pub(super) fn group_frames(layout: &SequenceLayout) -> Vec<RenderGroupFrame<'_>> {
    let mut stack = Vec::<(&GroupStartLayout, usize, usize)>::new();
    let mut frames = Vec::new();

    for (index, event) in layout.timeline.iter().enumerate() {
        match event {
            SequenceTimelineEventLayout::GroupStart(group) => {
                stack.push((group, stack.len(), index));
            }
            SequenceTimelineEventLayout::GroupEnd(group_end) => {
                if let Some((start, depth, start_index)) = stack.pop() {
                    frames.push(RenderGroupFrame {
                        start,
                        end_y: group_end.y + GROUP_FRAME_BOTTOM_PADDING,
                        end_line: group_end.line,
                        depth,
                        start_index,
                        end_index: index,
                    });
                }
            }
            _ => {}
        }
    }

    let fallback_end_y = layout
        .timeline
        .last()
        .map_or(layout.height - 24.0, SequenceTimelineEventLayout::y)
        + GROUP_FRAME_BOTTOM_PADDING;
    while let Some((start, depth, start_index)) = stack.pop() {
        frames.push(RenderGroupFrame {
            start,
            end_y: fallback_end_y,
            end_line: usize::MAX,
            depth,
            start_index,
            end_index: layout.timeline.len(),
        });
    }

    frames
}

pub(super) fn render_group_frame(
    svg: &mut String,
    layout: &SequenceLayout,
    frame: RenderGroupFrame<'_>,
    theme: &PlantUmlTheme,
    measurer: &dyn TextMeasurer,
) {
    let (x, width) = group_frame_x_width(layout, frame, measurer);
    let y = frame.start.y - GROUP_FRAME_TOP_OFFSET;
    let height = (frame.end_y - y).max(GROUP_LABEL_BOX_HEIGHT);
    let (label, condition) = group_header_parts(frame.start);
    let label_width = (approx_text_width(&label, measurer) + GROUP_LABEL_PADDING_X * 2.0)
        .max(GROUP_LABEL_BOX_HEIGHT);
    let condition_x = x + label_width + GROUP_LABEL_GAP;

    write!(
        svg,
        r##"<g class="sequence-event group-frame group-frame-{}" data-event="group-frame" data-group-kind="{}" data-line="{}" data-depth="{}"><rect class="group-frame-box" x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" stroke-width="1"/><rect class="group-label-box" x="{}" y="{}" width="{}" height="{}" fill="#f8fafc" stroke="{}" stroke-width="1"/>"##,
        group_kind_name(frame.start.kind),
        group_kind_name(frame.start.kind),
        frame.start.line,
        frame.depth,
        format_number(x),
        format_number(y),
        format_number(width),
        format_number(height),
        escape_attr(&theme.participant_stroke),
        format_number(x),
        format_number(y),
        format_number(label_width),
        format_number(GROUP_LABEL_BOX_HEIGHT),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
    render_positioned_text(
        svg,
        PositionedText {
            class_name: "group-label",
            data_event: "group-label",
            data_line: frame.start.line,
            x: x + GROUP_LABEL_PADDING_X,
            y: y + 17.0,
            anchor: "start",
            fill: &theme.text_color,
            text: &label,
            attrs: r#" font-weight="700""#,
        },
    );
    if let Some(condition) = condition {
        let font_size = sequence_small_font_size(theme);
        let attrs = format!(r#" font-size="{font_size}" font-weight="700""#);
        render_positioned_text(
            svg,
            PositionedText {
                class_name: "group-condition",
                data_event: "group-condition",
                data_line: frame.start.line,
                x: condition_x,
                y: y + 17.0,
                anchor: "start",
                fill: &theme.text_color,
                text: &condition,
                attrs: &attrs,
            },
        );
    }
    svg.push_str("</g>");
}

pub(super) fn sequence_small_font_size(theme: &PlantUmlTheme) -> u16 {
    (f32::from(theme.font_size) * SEQUENCE_SMALL_FONT_SIZE_RATIO)
        .round()
        .max(1.0) as u16
}

pub(super) fn group_header_parts(group: &GroupStartLayout) -> (String, Option<String>) {
    match group.kind {
        GroupKind::Group => {
            let label = group
                .label
                .as_deref()
                .unwrap_or_else(|| group_kind_name(group.kind));
            let (label, condition) = split_group_secondary_label(label);
            (label.to_owned(), condition.map(str::to_owned))
        }
        _ => (
            group_kind_name(group.kind).to_owned(),
            group.label.as_ref().map(|label| format!("[{label}]")),
        ),
    }
}

pub(super) fn split_group_secondary_label(label: &str) -> (&str, Option<&str>) {
    let label = label.trim();
    if !label.ends_with(']') {
        return (label, None);
    }

    let Some(condition_start) = label.rfind('[') else {
        return (label, None);
    };

    let primary = label[..condition_start].trim_end();
    let condition = label[condition_start..].trim();
    if primary.is_empty() || condition.len() <= 2 {
        return (label, None);
    }

    (primary, Some(condition))
}

pub(super) fn group_frame_x_width(
    layout: &SequenceLayout,
    frame: RenderGroupFrame<'_>,
    measurer: &dyn TextMeasurer,
) -> (f32, f32) {
    let fallback = {
        let (x, width) = event_frame_x_width(layout);
        HorizontalBounds::new(x, x + width)
    };
    let start_index = frame
        .start_index
        .saturating_add(1)
        .min(layout.timeline.len());
    let end_index = frame.end_index.min(layout.timeline.len());
    let mut bounds = None::<HorizontalBounds>;

    for event in &layout.timeline[start_index..end_index] {
        let Some(event_bounds) = timeline_event_horizontal_bounds(layout, event, measurer) else {
            continue;
        };
        match &mut bounds {
            Some(bounds) => bounds.include(event_bounds),
            None => bounds = Some(event_bounds),
        }
    }

    let mut bounds = bounds.unwrap_or(fallback);
    let inset = GROUP_FRAME_INDENT_X * (frame.depth as f32);
    bounds.left = (bounds.left - GROUP_FRAME_INDENT_X + inset).max(GROUP_FRAME_MIN_X);
    bounds.right += GROUP_FRAME_INDENT_X;
    let header_width = group_header_width(frame.start, measurer);
    bounds.right = bounds.right.max(bounds.left + header_width);

    (bounds.left, (bounds.right - bounds.left).max(1.0))
}

pub(super) fn group_frame_x_width_containing_line(
    layout: &SequenceLayout,
    line: usize,
    measurer: &dyn TextMeasurer,
) -> (f32, f32) {
    group_frames(layout)
        .into_iter()
        .filter(|frame| frame.start.line < line && line <= frame.end_line)
        .max_by_key(|frame| frame.depth)
        .map_or_else(
            || {
                let (x, width) = event_frame_x_width(layout);
                (x, width)
            },
            |frame| group_frame_x_width(layout, frame, measurer),
        )
}

pub(super) fn group_header_width(group: &GroupStartLayout, measurer: &dyn TextMeasurer) -> f32 {
    let (label, condition) = group_header_parts(group);
    let label_width = (approx_text_width(&label, measurer) + GROUP_LABEL_PADDING_X * 2.0)
        .max(GROUP_LABEL_BOX_HEIGHT);
    let condition_width = condition.as_deref().map_or(0.0, |condition| {
        GROUP_LABEL_GAP + approx_text_width(condition, measurer)
    });
    label_width + condition_width
}

pub(super) fn timeline_event_horizontal_bounds(
    layout: &SequenceLayout,
    event: &SequenceTimelineEventLayout,
    measurer: &dyn TextMeasurer,
) -> Option<HorizontalBounds> {
    match event {
        SequenceTimelineEventLayout::Message(message) => {
            let mut bounds = HorizontalBounds::new(
                message.start_x.min(message.end_x),
                message.start_x.max(message.end_x),
            );

            if matches!(message.direction, MessageDirection::SelfMessage) {
                bounds.right = bounds.right.max(message.start_x + SELF_MESSAGE_WIDTH);
            }

            let label_width = approx_text_width(&message.label, measurer);
            let label_center = (message.start_x + message.end_x) / 2.0;
            bounds.include(HorizontalBounds::new(
                label_center - label_width / 2.0,
                label_center + label_width / 2.0,
            ));

            if let Some(participant_bounds) = participant_horizontal_bounds(layout, &message.from) {
                bounds.include(participant_bounds);
            }
            if let Some(participant_bounds) = participant_horizontal_bounds(layout, &message.to) {
                bounds.include(participant_bounds);
            }

            Some(bounds)
        }
        SequenceTimelineEventLayout::Note(note) => {
            let width = note_box_width_with_text_measurer(&note.text, measurer);
            let rect_x = note_rect_x(layout, note, width);
            Some(HorizontalBounds::new(rect_x, rect_x + width))
        }
        SequenceTimelineEventLayout::Activation(activation) => {
            participant_horizontal_bounds(layout, &activation.participant)
        }
        _ => None,
    }
}

pub(super) fn participant_horizontal_bounds(
    layout: &SequenceLayout,
    id: &str,
) -> Option<HorizontalBounds> {
    layout
        .participants
        .iter()
        .find(|participant| participant.id == id)
        .map(|participant| {
            HorizontalBounds::new(
                participant.x - participant.head_width / 2.0,
                participant.x + participant.head_width / 2.0,
            )
        })
}

pub(super) fn group_depth_at_line(layout: &SequenceLayout, line: usize) -> usize {
    let mut stack_depth = 0_usize;

    for event in &layout.timeline {
        match event {
            SequenceTimelineEventLayout::GroupStart(group) => {
                if group.line >= line {
                    return stack_depth.saturating_sub(1);
                }
                stack_depth += 1;
            }
            SequenceTimelineEventLayout::GroupElse(group_else) if group_else.line == line => {
                return stack_depth.saturating_sub(1);
            }
            SequenceTimelineEventLayout::GroupEnd(group_end) => {
                if group_end.line >= line {
                    return stack_depth.saturating_sub(1);
                }
                stack_depth = stack_depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    0
}

pub(super) fn approx_text_width(text: &str, measurer: &dyn TextMeasurer) -> f32 {
    measure_inline_text_with(text, measurer)
}
