//! Inline span positioning and participant display line composition.
use super::*;

#[cfg(test)]
pub(crate) fn layout_inline_text_line(
    line: &str,
    anchor_x: f32,
    anchor: TextAnchor,
) -> InlineTextLineLayout {
    layout_inline_text_line_with_measurer(line, anchor_x, anchor, &ApproxTextMeasurer)
}

pub(crate) fn layout_inline_text_line_with_measurer(
    line: &str,
    anchor_x: f32,
    anchor: TextAnchor,
    measurer: &dyn TextMeasurer,
) -> InlineTextLineLayout {
    let spans = parse_inline_text_spans(line);
    let request = inline_text_measurement_request_from_spans(
        line,
        &spans,
        &measurer.base_inline_text_style(),
    );
    let measurement = measurer.measure_inline_text_request(&request);
    layout_inline_text_spans(spans, &measurement, anchor_x, anchor)
}

pub(crate) fn measure_inline_text_with(line: &str, measurer: &dyn TextMeasurer) -> f32 {
    measurer.measure_inline_text(line)
}

pub fn visible_inline_text(line: &str) -> String {
    parse_inline_text_spans(line)
        .into_iter()
        .map(|span| span.text)
        .collect()
}

pub(crate) fn text_lines(text: &str) -> Vec<String> {
    let normalized = text.replace("\\n", "\n");
    let lines = normalized
        .lines()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if lines.is_empty() {
        vec![String::new()]
    } else {
        lines
    }
}

pub(crate) fn participant_label_lines(name: &str) -> Vec<String> {
    let normalized = name.replace("\\n", "\n");
    let lines = normalized
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    if lines.is_empty() {
        vec![name.to_owned()]
    } else {
        lines
    }
}

pub(crate) fn participant_display_lines(
    display: &ParticipantDisplay,
) -> Vec<ParticipantDisplayLine> {
    display
        .lines
        .iter()
        .map(|line| participant_display_line(line))
        .collect()
}

pub(crate) fn participant_display_line(line: &str) -> ParticipantDisplayLine {
    let trimmed = line.trim();
    if is_participant_display_separator(trimmed) {
        return ParticipantDisplayLine {
            kind: ParticipantDisplayLineKind::Separator,
            text: String::new(),
        };
    }

    if let Some(title) = trimmed.strip_prefix('=') {
        return ParticipantDisplayLine {
            kind: ParticipantDisplayLineKind::Title,
            text: title.trim().to_owned(),
        };
    }

    if trimmed.starts_with("\"\"") && trimmed.ends_with("\"\"") && trimmed.len() >= 4 {
        return ParticipantDisplayLine {
            kind: ParticipantDisplayLineKind::Subtitle,
            text: trimmed[2..trimmed.len() - 2].trim().to_owned(),
        };
    }

    ParticipantDisplayLine {
        kind: ParticipantDisplayLineKind::Text,
        text: trimmed.to_owned(),
    }
}

pub(super) fn is_participant_display_separator(line: &str) -> bool {
    line.len() >= 3 && line.chars().all(|character| character == '-')
}

pub(crate) fn inline_style_font_size(style: &InlineTextStyle) -> f32 {
    style.font_size.map(f32::from).unwrap_or(DEFAULT_FONT_SIZE)
}

pub(super) fn layout_inline_text_spans(
    spans: Vec<InlineTextSpan>,
    measurement: &InlineTextMeasurementResult,
    anchor_x: f32,
    anchor: TextAnchor,
) -> InlineTextLineLayout {
    let width = measurement.width.max(0.0);
    let mut cursor_x = match anchor {
        TextAnchor::Start => anchor_x,
        TextAnchor::Middle => anchor_x - width / 2.0,
        TextAnchor::End => anchor_x - width,
    };

    let runs = spans
        .into_iter()
        .enumerate()
        .map(|(index, span)| {
            let measured = measurement
                .runs
                .iter()
                .find(|run| run.index == index)
                .filter(|run| run.width.is_finite() && run.x.is_finite());
            let (x, width) = measured
                .map(|run| {
                    let x = anchor_start_x(anchor_x, anchor, measurement.width) + run.x;
                    cursor_x = x + run.width;
                    (x, run.width)
                })
                .unwrap_or_else(|| {
                    let width = measure_inline_span(&span);
                    let x = cursor_x;
                    cursor_x += width;
                    (x, width)
                });
            InlineTextRun {
                text: span.text,
                style: span.style,
                x,
                width,
            }
        })
        .collect();

    InlineTextLineLayout { width, runs }
}

pub(super) fn anchor_start_x(anchor_x: f32, anchor: TextAnchor, width: f32) -> f32 {
    match anchor {
        TextAnchor::Start => anchor_x,
        TextAnchor::Middle => anchor_x - width / 2.0,
        TextAnchor::End => anchor_x - width,
    }
}
