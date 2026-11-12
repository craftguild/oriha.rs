//! Note frames, anchors and event-span bounds.
use super::*;

pub(super) fn render_note(
    svg: &mut String,
    layout: &SequenceLayout,
    note: &NoteLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) {
    let width = note_box_width_with_text_measurer(&note.text, measurer);
    let height = note_box_height(&note.text);
    let rect_x = note_rect_x(layout, note, width);
    let rect_y = note.y - NOTE_BASELINE_OFFSET;
    let text_x = rect_x + NOTE_TEXT_PADDING_X;
    let text_y = rect_y + NOTE_BASELINE_OFFSET;
    let fill = note
        .color
        .as_deref()
        .map(svg_color_value)
        .unwrap_or_else(|| Cow::Borrowed("#fff7cc"));
    let corner_radius = style
        .corner_radius
        .svg_px()
        .map(|px| format!(r#" rx="{px}" ry="{px}""#))
        .unwrap_or_default();

    write!(
        svg,
        r##"<g class="sequence-event note" data-event="note" data-placement="{}" data-targets="{}" data-note-shape="{}" data-multiline="{}" data-line="{}">"##,
        note_placement_name(&note.placement),
        escape_attr(&note.targets.join(",")),
        note_shape_name(note.shape),
        note.multiline,
        note.line,
    )
    .expect("writing to String cannot fail");
    let frame = NoteFrame {
        shape: note.shape,
        x: rect_x,
        y: rect_y,
        width,
        height,
        fill: fill.as_ref(),
        stroke: &theme.participant_stroke,
        corner_radius: &corner_radius,
    };
    render_note_frame(svg, &frame);
    render_text_element(
        svg,
        text_x,
        text_y,
        "start",
        &theme.text_color,
        &note.text,
        measurer,
    );
    svg.push_str("</g>");
}

pub(super) fn render_note_frame(svg: &mut String, frame: &NoteFrame<'_>) {
    match frame.shape {
        NoteShape::Hexagon => {
            let notch = 10.0_f32.min(frame.width / 4.0).min(frame.height / 2.0);
            write!(
                svg,
                r##"<polygon class="note-frame note-frame-hexagon" points="{},{} {},{} {},{} {},{} {},{} {},{}" fill="{}" stroke="{}"/>"##,
                format_number(frame.x + notch),
                format_number(frame.y),
                format_number(frame.x + frame.width - notch),
                format_number(frame.y),
                format_number(frame.x + frame.width),
                format_number(frame.y + frame.height / 2.0),
                format_number(frame.x + frame.width - notch),
                format_number(frame.y + frame.height),
                format_number(frame.x + notch),
                format_number(frame.y + frame.height),
                format_number(frame.x),
                format_number(frame.y + frame.height / 2.0),
                escape_attr(frame.fill),
                escape_attr(frame.stroke),
            )
            .expect("writing to String cannot fail");
        }
        NoteShape::Normal | NoteShape::Rectangle => {
            write!(
                svg,
                r##"<rect class="note-frame note-frame-rect" x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}"{}/>"##,
                format_number(frame.x),
                format_number(frame.y),
                format_number(frame.width),
                format_number(frame.height),
                escape_attr(frame.fill),
                escape_attr(frame.stroke),
                frame.corner_radius,
            )
            .expect("writing to String cannot fail");
        }
    }
}

pub(super) fn note_rect_x(layout: &SequenceLayout, note: &NoteLayout, width: f32) -> f32 {
    let anchor_x = note_anchor_x(layout, note);
    match note.placement {
        NotePlacement::Left => anchor_x - width - NOTE_TARGET_GAP,
        NotePlacement::Right => anchor_x + NOTE_TARGET_GAP,
        NotePlacement::Over | NotePlacement::Across | NotePlacement::Unknown(_) => {
            anchor_x - width / 2.0
        }
    }
    .clamp(
        NOTE_CANVAS_MARGIN,
        (layout.width - width - NOTE_CANVAS_MARGIN).max(NOTE_CANVAS_MARGIN),
    )
}

pub(super) fn participant_x(layout: &SequenceLayout, id: &str) -> Option<f32> {
    layout
        .participants
        .iter()
        .find(|participant| participant.id == id)
        .map(|participant| participant.x)
}

pub(super) fn note_anchor_x(layout: &SequenceLayout, note: &NoteLayout) -> f32 {
    if matches!(note.placement, NotePlacement::Across) {
        return layout.width / 2.0;
    }

    let target_positions: Vec<f32> = note
        .targets
        .iter()
        .filter_map(|target| participant_x(layout, target))
        .collect();

    if target_positions.is_empty() {
        return layout.width / 2.0;
    }

    target_positions.iter().sum::<f32>() / (target_positions.len() as f32)
}

pub(super) fn participant_span_x_width(layout: &SequenceLayout) -> (f32, f32) {
    let Some(first) = layout.participants.first() else {
        return (32.0, (layout.width - 64.0).max(0.0));
    };
    let Some(last) = layout.participants.last() else {
        return (32.0, (layout.width - 64.0).max(0.0));
    };

    let left = first.x.min(last.x);
    let right = first.x.max(last.x);
    (left, (right - left).max(1.0))
}

pub(super) fn event_frame_x_width(layout: &SequenceLayout) -> (f32, f32) {
    let Some(first) = layout.participants.first() else {
        return (32.0, (layout.width - 64.0).max(1.0));
    };
    let Some(last) = layout.participants.last() else {
        return (32.0, (layout.width - 64.0).max(1.0));
    };

    let left = first.x.min(last.x) - first.head_width / 2.0;
    let right = first.x.max(last.x) + last.head_width / 2.0;
    (left, (right - left).max(1.0))
}
