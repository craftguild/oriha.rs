//! Participant symbol geometry and icon labels.
use super::*;

pub(super) fn render_participant_rectangle(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    fill: &str,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
) {
    let x = participant.x - participant.head_width / 2.0;
    let text_y = y + participant.head_height / 2.0 + 5.0;
    let corner_radius = style
        .corner_radius
        .svg_px()
        .map(|px| format!(r#" rx="{px}" ry="{px}""#))
        .unwrap_or_default();
    write!(
        svg,
        r#"<rect class="participant-box participant-kind-participant" x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}"{} />"#,
        format_number(x),
        format_number(y),
        format_number(participant.head_width),
        format_number(participant.head_height),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
        corner_radius,
    )
    .expect("writing to String cannot fail");

    if let Some(display) = &participant.display {
        render_participant_display(svg, participant, y, display, theme);
    } else {
        render_participant_label(svg, participant, text_y, theme);
    }
}

pub(super) fn render_actor_participant(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    fill: &str,
    theme: &PlantUmlTheme,
) {
    let cx = participant.x;
    let head_y = y + 13.0;
    let body_top = y + 21.0;
    let body_bottom = y + 39.0;
    write!(
        svg,
        r#"<g class="participant-icon participant-icon-actor participant-kind-actor"><circle cx="{}" cy="{}" r="7" fill="{}" stroke="{}"/><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/></g>"#,
        format_number(cx),
        format_number(head_y),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
        format_number(cx),
        format_number(body_top),
        format_number(cx),
        format_number(body_bottom),
        escape_attr(&theme.participant_stroke),
        format_number(cx - 15.0),
        format_number(y + 28.0),
        format_number(cx + 15.0),
        format_number(y + 28.0),
        escape_attr(&theme.participant_stroke),
        format_number(cx),
        format_number(body_bottom),
        format_number(cx - 12.0),
        format_number(y + 52.0),
        escape_attr(&theme.participant_stroke),
        format_number(cx),
        format_number(body_bottom),
        format_number(cx + 12.0),
        format_number(y + 52.0),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
    render_icon_participant_label(svg, participant, y, theme);
}

pub(super) fn render_boundary_participant(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    fill: &str,
    theme: &PlantUmlTheme,
) {
    let cx = participant.x;
    let cy = y + 27.0;
    write!(
        svg,
        r#"<g class="participant-icon participant-icon-boundary participant-kind-boundary"><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/><circle cx="{}" cy="{}" r="14" fill="{}" stroke="{}"/><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/></g>"#,
        format_number(cx - 18.0),
        format_number(y + 10.0),
        format_number(cx - 18.0),
        format_number(y + 48.0),
        escape_attr(&theme.participant_stroke),
        format_number(cx + 3.0),
        format_number(cy),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
        format_number(cx - 18.0),
        format_number(cy),
        format_number(cx - 11.0),
        format_number(cy),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
    render_icon_participant_label(svg, participant, y, theme);
}

pub(super) fn render_control_participant(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    fill: &str,
    theme: &PlantUmlTheme,
) {
    let cx = participant.x;
    let cy = y + 27.0;
    write!(
        svg,
        r#"<g class="participant-icon participant-icon-control participant-kind-control"><circle cx="{}" cy="{}" r="17" fill="{}" stroke="{}"/><path d="M {},{} L {},{} L {},{}" fill="none" stroke="{}" stroke-linecap="round" stroke-linejoin="round"/></g>"#,
        format_number(cx),
        format_number(cy),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
        format_number(cx - 4.0),
        format_number(cy - 8.0),
        format_number(cx + 7.0),
        format_number(cy),
        format_number(cx - 4.0),
        format_number(cy + 8.0),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
    render_icon_participant_label(svg, participant, y, theme);
}

pub(super) fn render_entity_participant(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    fill: &str,
    theme: &PlantUmlTheme,
) {
    let cx = participant.x;
    let cy = y + 24.0;
    write!(
        svg,
        r#"<g class="participant-icon participant-icon-entity participant-kind-entity"><circle cx="{}" cy="{}" r="16" fill="{}" stroke="{}"/><line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/></g>"#,
        format_number(cx),
        format_number(cy),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
        format_number(cx - 17.0),
        format_number(y + 45.0),
        format_number(cx + 17.0),
        format_number(y + 45.0),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
    render_icon_participant_label(svg, participant, y, theme);
}

pub(super) fn render_database_participant(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    fill: &str,
    theme: &PlantUmlTheme,
) {
    let cx = participant.x;
    let top = y + 10.0;
    let bottom = y + 43.0;
    write!(
        svg,
        r#"<g class="participant-icon participant-icon-database participant-kind-database"><path d="M {},{} V {} C {},{} {},{} {},{} C {},{} {},{} {},{} V {} C {},{} {},{} {},{} Z" fill="{}" stroke="{}"/><ellipse cx="{}" cy="{}" rx="19" ry="7" fill="{}" stroke="{}"/></g>"#,
        format_number(cx - 19.0),
        format_number(top),
        format_number(bottom),
        format_number(cx - 19.0),
        format_number(bottom + 4.0),
        format_number(cx - 10.0),
        format_number(bottom + 7.0),
        format_number(cx),
        format_number(bottom + 7.0),
        format_number(cx + 10.0),
        format_number(bottom + 7.0),
        format_number(cx + 19.0),
        format_number(bottom + 4.0),
        format_number(cx + 19.0),
        format_number(bottom),
        format_number(top),
        format_number(cx + 19.0),
        format_number(top + 4.0),
        format_number(cx + 10.0),
        format_number(top + 7.0),
        format_number(cx),
        format_number(top + 7.0),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
        format_number(cx),
        format_number(top),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
    render_icon_participant_label(svg, participant, y, theme);
}

pub(super) fn render_collections_participant(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    fill: &str,
    theme: &PlantUmlTheme,
) {
    let x = participant.x - 17.0;
    write!(
        svg,
        r#"<g class="participant-icon participant-icon-collections participant-kind-collections"><rect x="{}" y="{}" width="32" height="24" fill="{}" stroke="{}"/><rect x="{}" y="{}" width="32" height="24" fill="{}" stroke="{}"/><rect x="{}" y="{}" width="32" height="24" fill="{}" stroke="{}"/></g>"#,
        format_number(x + 8.0),
        format_number(y + 10.0),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
        format_number(x + 4.0),
        format_number(y + 15.0),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
        format_number(x),
        format_number(y + 20.0),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
    render_icon_participant_label(svg, participant, y, theme);
}

pub(super) fn render_queue_participant(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    fill: &str,
    theme: &PlantUmlTheme,
) {
    let cx = participant.x;
    let cy = y + 27.0;
    write!(
        svg,
        r#"<g class="participant-icon participant-icon-queue participant-kind-queue"><rect x="{}" y="{}" width="42" height="22" fill="{}" stroke="{}"/><ellipse cx="{}" cy="{}" rx="7" ry="11" fill="{}" stroke="{}"/><ellipse cx="{}" cy="{}" rx="7" ry="11" fill="{}" stroke="{}"/></g>"#,
        format_number(cx - 21.0),
        format_number(cy - 11.0),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
        format_number(cx - 21.0),
        format_number(cy),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
        format_number(cx + 21.0),
        format_number(cy),
        escape_attr(fill),
        escape_attr(&theme.participant_stroke),
    )
    .expect("writing to String cannot fail");
    render_icon_participant_label(svg, participant, y, theme);
}

pub(super) fn render_icon_participant_label(
    svg: &mut String,
    participant: &ParticipantLayout,
    y: f32,
    theme: &PlantUmlTheme,
) {
    render_participant_label_at(
        svg,
        participant,
        y + participant.head_height - 7.0,
        "participant-icon-label",
        theme,
    );
}
