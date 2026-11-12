//! Sequence SVG composition and shared render model.
mod measurements;
pub(crate) use measurements::collect_sequence_render_text_measurements;
#[allow(unused_imports)]
use measurements::*;
mod page;
#[allow(unused_imports)]
use page::*;
mod groups;
#[allow(unused_imports)]
use groups::*;
mod participant_shapes;
#[allow(unused_imports)]
use participant_shapes::*;
mod participants;
#[allow(unused_imports)]
use participants::*;
mod messages;
#[allow(unused_imports)]
use messages::*;
mod arrows;
#[allow(unused_imports)]
use arrows::*;
mod message_attributes;
#[allow(unused_imports)]
use message_attributes::*;
mod notes;
#[allow(unused_imports)]
use notes::*;
mod structural_events;
#[allow(unused_imports)]
use structural_events::*;
mod activity_events;
#[allow(unused_imports)]
use activity_events::*;
mod semantic_events;
#[allow(unused_imports)]
use semantic_events::*;
mod text;
#[allow(unused_imports)]
use text::*;
mod names;
#[allow(unused_imports)]
use names::*;

use std::{borrow::Cow, fmt::Write};

use crate::layout::{
    note_box_height, note_box_width_with_text_measurer, ActivationEventLayout, BoxEndLayout,
    BoxStartLayout, DelayLayout, GroupElseLayout, GroupEndLayout, GroupStartLayout,
    HeaderFooterLayout, MessageLayout, NoteLayout, PageBreakLayout, ParticipantLayout,
    ReturnEventLayout, SemanticEventLayout, SequenceLayout, SequenceTimelineEventLayout,
    TextEventLayout, NOTE_BASELINE_OFFSET, NOTE_CANVAS_MARGIN, NOTE_TARGET_GAP,
    NOTE_TEXT_PADDING_X,
};
use crate::sequence::{
    ActivationKind, AutonumberCommand, GroupKind, HeaderFooterKind, LineStyle,
    MessageActivationTarget, MessageArrowHalf, MessageArrowHead, MessageDirection,
    MessageEndpointKind, NotePlacement, NoteShape, ParticipantDisplay, ParticipantKind,
};
use crate::text_layout::{
    inline_style_font_size, layout_inline_text_line_with_measurer, measure_inline_text_with,
    parse_inline_text_spans, participant_display_lines, participant_label_lines, text_lines,
    ApproxTextMeasurer, InlineTextStyle, ParticipantDisplayLineKind, TextAnchor, TextMeasurer,
};

const PARTICIPANT_LABEL_LINE_HEIGHT: f32 = 16.0;
const TEXT_LINE_HEIGHT: f32 = 16.0;
const SELF_MESSAGE_WIDTH: f32 = 64.0;
const SELF_MESSAGE_HEIGHT: f32 = 28.0;
const ARROW_DECORATION_GAP: f32 = 8.0;
const ARROW_CIRCLE_RADIUS: f32 = 4.0;
const ARROW_CROSS_SIZE: f32 = 5.0;
const GROUP_FRAME_INDENT_X: f32 = 10.0;
const GROUP_FRAME_TOP_OFFSET: f32 = 18.0;
const GROUP_FRAME_BOTTOM_PADDING: f32 = 8.0;
const GROUP_FRAME_MIN_X: f32 = 2.0;
const GROUP_LABEL_BOX_HEIGHT: f32 = 24.0;
const GROUP_LABEL_PADDING_X: f32 = 12.0;
const GROUP_LABEL_GAP: f32 = 10.0;
const DEFAULT_SEQUENCE_FONT_SIZE: f32 = 14.0;
const SEQUENCE_SMALL_FONT_SIZE_RATIO: f32 = 12.0 / DEFAULT_SEQUENCE_FONT_SIZE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantUmlTheme {
    pub font_family: String,
    pub font_size: u16,
    pub background_color: String,
    pub participant_fill: String,
    pub participant_stroke: String,
    pub lifeline_color: String,
    pub message_color: String,
    pub text_color: String,
}

impl Default for PlantUmlTheme {
    fn default() -> Self {
        Self {
            font_family: r#"-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif"#.to_owned(),
            font_size: 14,
            background_color: "#ffffff".to_owned(),
            participant_fill: "#f8fafc".to_owned(),
            participant_stroke: "#94a3b8".to_owned(),
            lifeline_color: "#94a3b8".to_owned(),
            message_color: "#94a3b8".to_owned(),
            text_color: "#1e293b".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantUmlStyle {
    pub corner_radius: CornerRadius,
    pub arrowhead: ArrowheadStyle,
    pub stroke_width: StrokeWidth,
}

impl Default for PlantUmlStyle {
    fn default() -> Self {
        Self {
            corner_radius: CornerRadius::None,
            arrowhead: ArrowheadStyle::Triangle,
            stroke_width: StrokeWidth::Thin,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerRadius {
    None,
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
}

impl CornerRadius {
    fn svg_px(self) -> Option<u16> {
        match self {
            Self::None => None,
            Self::Xs => Some(2),
            Self::Sm => Some(4),
            Self::Md => Some(6),
            Self::Lg => Some(8),
            Self::Xl => Some(12),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowheadStyle {
    Triangle,
    Chevron,
    Kite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeWidth {
    Thin,
    Regular,
    Bold,
}

impl StrokeWidth {
    fn svg_px(self) -> u16 {
        match self {
            Self::Thin => 1,
            Self::Regular => 2,
            Self::Bold => 3,
        }
    }

    fn dasharray(self) -> &'static str {
        match self {
            Self::Thin => "6 4",
            Self::Regular => "8 5",
            Self::Bold => "10 6",
        }
    }

    fn arrowhead_length(self) -> f32 {
        10.0 + f32::from(self.svg_px() - 1) * 2.0
    }

    fn arrowhead_half_height(self) -> f32 {
        5.0 + f32::from(self.svg_px() - 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedPlantUml {
    pub svg: String,
    pub width: u32,
    pub height: u32,
    pub view_box: String,
}

pub fn render_sequence_svg(layout: &SequenceLayout, theme: &PlantUmlTheme) -> RenderedPlantUml {
    render_sequence_svg_with_style(layout, theme, &PlantUmlStyle::default())
}

pub fn render_sequence_svg_with_style(
    layout: &SequenceLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
) -> RenderedPlantUml {
    render_sequence_svg_with_style_and_text_measurer(layout, theme, style, &ApproxTextMeasurer)
}

pub fn render_sequence_svg_with_style_and_text_measurer(
    layout: &SequenceLayout,
    theme: &PlantUmlTheme,
    style: &PlantUmlStyle,
    measurer: &dyn TextMeasurer,
) -> RenderedPlantUml {
    let width = layout.width.ceil() as u32;
    let height = layout.height.ceil() as u32;
    let mut svg = String::new();

    write!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" data-diagram-type="SEQUENCE" viewBox="{}" width="{width}" height="{height}" font-family="{}" font-size="{}">"#,
        escape_attr(&layout.view_box),
        escape_attr(&theme.font_family),
        theme.font_size,
    )
    .expect("writing to String cannot fail");
    if let Some(title) = &layout.title {
        write!(svg, "<title>{}</title>", escape_text(&title.text))
            .expect("writing to String cannot fail");
    }
    write!(
        svg,
        r#"<rect width="100%" height="100%" fill="{}"/>"#,
        escape_attr(&theme.background_color),
    )
    .expect("writing to String cannot fail");
    render_page_chrome(&mut svg, layout, theme);
    render_group_frames(&mut svg, layout, theme, measurer);

    for participant in &layout.participants {
        render_lifeline(&mut svg, participant, theme);
    }

    if layout.timeline.is_empty() {
        for message in &layout.messages {
            render_message(&mut svg, message, theme, style, measurer);
        }
    } else {
        for event in &layout.timeline {
            render_timeline_event(&mut svg, layout, event, theme, style, measurer);
        }
    }

    for participant in &layout.participants {
        render_participant_box(&mut svg, participant, participant.head_y, theme, style);
        render_participant_box(&mut svg, participant, participant.tail_y, theme, style);
    }

    svg.push_str("</svg>");

    RenderedPlantUml {
        svg,
        width,
        height,
        view_box: layout.view_box.clone(),
    }
}

#[derive(Debug, Clone, Copy)]
struct RenderGroupFrame<'a> {
    start: &'a GroupStartLayout,
    end_y: f32,
    end_line: usize,
    depth: usize,
    start_index: usize,
    end_index: usize,
}

#[derive(Debug, Clone, Copy)]
struct HorizontalBounds {
    left: f32,
    right: f32,
}

impl HorizontalBounds {
    fn new(left: f32, right: f32) -> Self {
        Self { left, right }
    }

    fn include(&mut self, other: Self) {
        self.left = self.left.min(other.left);
        self.right = self.right.max(other.right);
    }
}

struct PositionedText<'a> {
    class_name: &'a str,
    data_event: &'a str,
    data_line: usize,
    x: f32,
    y: f32,
    anchor: &'a str,
    fill: &'a str,
    text: &'a str,
    attrs: &'a str,
}

fn svg_color_value(color: &str) -> Cow<'_, str> {
    let color = color.trim();
    match color.strip_prefix('#') {
        Some(named_color) if !is_hex_color_body(named_color) => Cow::Owned(named_color.to_owned()),
        _ => Cow::Borrowed(color),
    }
}

fn is_hex_color_body(value: &str) -> bool {
    matches!(value.len(), 3 | 4 | 6 | 8)
        && value.chars().all(|character| character.is_ascii_hexdigit())
}

struct NoteFrame<'a> {
    shape: NoteShape,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fill: &'a str,
    stroke: &'a str,
    corner_radius: &'a str,
}

struct SemanticMarker<'a> {
    class_name: &'a str,
    data_event: &'a str,
    line: usize,
    y: f32,
    label: &'a str,
    extra_attrs: &'a str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ArrowSide {
    head: Option<MessageArrowHead>,
    circle: bool,
    cross: bool,
}

struct InlineTextAnnotation<'a> {
    x: f32,
    y: f32,
    anchor: &'a str,
    fill: &'a str,
    line_index: usize,
    line: &'a str,
    measurer: &'a dyn TextMeasurer,
}

fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(value: &str) -> String {
    escape_text(value).replace('"', "&quot;")
}

fn format_number(value: f32) -> String {
    if value.fract().abs() < f32::EPSILON {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}
