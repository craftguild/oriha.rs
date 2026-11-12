//! Participant dispatch, display text and lifelines.
use super::*;

pub(super) fn render_lifeline(
    svg: &mut String,
    participant: &ParticipantLayout,
    theme: &PlantUmlTheme,
) {
    let x = format_number(participant.x);
    let y1 = format_number(participant.head_y + participant.head_height);
    let y2 = format_number(participant.tail_y);
    write!(
        svg,
        r#"<g class="participant-lifeline" data-participant="{}"><line x1="{x}" y1="{y1}" x2="{x}" y2="{y2}" stroke="{}" stroke-dasharray="4 4"/></g>"#,
        escape_attr(&participant.id),
        escape_attr(&theme.lifeline_color),
    )
    .expect("writing to String cannot fail");
}

pub(super) fn render_participant_box(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
) {
    let fill = participant_fill(participant, theme);
    write!(
        svg,
        r#"<g class="participant" data-participant="{}" data-participant-kind="{}">"#,
        escape_attr(&participant.id),
        participant_kind_name(participant.kind),
    )
    .expect("writing to String cannot fail");

    match participant.kind {
        ParticipantKind::Participant => {
            render_participant_rectangle(svg, participant, y, fill, theme, style);
        }
        ParticipantKind::Actor => {
            render_actor_participant(svg, participant, y, fill, theme);
        }
        ParticipantKind::Boundary => {
            render_boundary_participant(svg, participant, y, fill, theme);
        }
        ParticipantKind::Control => {
            render_control_participant(svg, participant, y, fill, theme);
        }
        ParticipantKind::Entity => {
            render_entity_participant(svg, participant, y, fill, theme);
        }
        ParticipantKind::Database => {
            render_database_participant(svg, participant, y, fill, theme);
        }
        ParticipantKind::Collections => {
            render_collections_participant(svg, participant, y, fill, theme);
        }
        ParticipantKind::Queue => {
            render_queue_participant(svg, participant, y, fill, theme);
        }
    }

    svg.push_str("</g>");
}

pub(super) fn render_participant_label(
    svg: &mut String,
    participant: &ParticipantLayout,
    center_y: f32,
    theme: &PlantUmlTheme,
) {
    render_participant_label_at(svg, participant, center_y, "participant-label", theme);
}

pub(super) fn render_participant_label_at(
    svg: &mut String,
    participant: &ParticipantLayout,
    center_y: f32,
    class_name: &str,
    theme: &PlantUmlTheme,
) {
    let lines = participant_label_lines(&participant.name);
    let first_y =
        center_y - PARTICIPANT_LABEL_LINE_HEIGHT * ((lines.len().saturating_sub(1)) as f32) / 2.0;

    for (index, line) in lines.iter().enumerate() {
        write!(
            svg,
            r#"<text class="{}" x="{}" y="{}" text-anchor="middle" fill="{}">"#,
            class_name,
            format_number(participant.x),
            format_number(first_y + PARTICIPANT_LABEL_LINE_HEIGHT * (index as f32)),
            escape_attr(&theme.text_color),
        )
        .expect("writing to String cannot fail");
        render_text_line_tspans(svg, participant.x, 0, line);
        svg.push_str("</text>");
    }
}

pub(super) fn render_participant_display(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    display: &ParticipantDisplay,
    theme: &PlantUmlTheme,
) {
    let lines = participant_display_lines(display);
    let content_height = lines
        .iter()
        .map(|line| {
            if matches!(line.kind, ParticipantDisplayLineKind::Separator) {
                10.0
            } else {
                16.0
            }
        })
        .sum::<f32>();
    let mut current_y = y + (participant.head_height - content_height) / 2.0 + 12.0;
    let left = participant.x - participant.head_width / 2.0 + 12.0;
    let right = participant.x + participant.head_width / 2.0 - 12.0;

    for line in lines {
        if matches!(line.kind, ParticipantDisplayLineKind::Separator) {
            let separator_y = current_y - 5.0;
            write!(
                svg,
                r#"<line class="participant-display-separator" x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/>"#,
                format_number(left),
                format_number(separator_y),
                format_number(right),
                format_number(separator_y),
                escape_attr(&theme.participant_stroke),
            )
            .expect("writing to String cannot fail");
            current_y += 10.0;
            continue;
        }

        let (class_name, attrs) = participant_display_text_attrs(line.kind);
        write!(
            svg,
            r#"<text class="{}" x="{}" y="{}" text-anchor="middle" fill="{}"{}>{}</text>"#,
            class_name,
            format_number(participant.x),
            format_number(current_y),
            escape_attr(&theme.text_color),
            attrs,
            escape_text(&line.text),
        )
        .expect("writing to String cannot fail");
        current_y += 16.0;
    }
}

pub(super) fn participant_fill<'a>(
    participant: &'a ParticipantLayout,
    theme: &'a PlantUmlTheme,
) -> &'a str {
    participant
        .color
        .as_deref()
        .unwrap_or(theme.participant_fill.as_str())
}

pub(super) fn participant_display_text_attrs(
    kind: ParticipantDisplayLineKind,
) -> (&'static str, &'static str) {
    match kind {
        ParticipantDisplayLineKind::Title => ("participant-display-title", r#" font-weight="600""#),
        ParticipantDisplayLineKind::Subtitle => {
            ("participant-display-subtitle", r#" font-style="italic""#)
        }
        ParticipantDisplayLineKind::Text => ("participant-display-text", ""),
        ParticipantDisplayLineKind::Separator => ("participant-display-separator", ""),
    }
}
