//! Sequence layout data model and shared geometry constants.
mod composition;
#[allow(unused_imports)]
use composition::*;
pub use composition::{layout_sequence, layout_sequence_with_text_measurer};
mod vertical;
#[allow(unused_imports)]
use vertical::*;
mod notes;
#[allow(unused_imports)]
use notes::*;
pub(crate) use notes::{note_box_height, note_box_width_with_text_measurer};
mod participants;
#[allow(unused_imports)]
use participants::*;
mod autonumber;
#[allow(unused_imports)]
use autonumber::*;
mod messages;
#[allow(unused_imports)]
use messages::*;
mod semantic_events;
#[allow(unused_imports)]
use semantic_events::*;

use std::collections::HashMap;

use crate::sequence::{
    ActivationKind, AutonumberCommand, GroupKind, HeaderFooterKind, LineStyle, Message,
    MessageActivationShortcut, MessageArrowFeatures, MessageDirection, MessageEndpoint,
    MessageEndpointKind, Note, NotePlacement, NoteShape, ParticipantDisplay, ParticipantKind,
    SequenceDiagram, SequenceEvent, SequenceMessage, SequenceSemanticEvent,
};
use crate::text_layout::{
    measure_inline_text_with, participant_display_lines, participant_label_lines, text_lines,
    ApproxTextMeasurer, ParticipantDisplayLineKind, TextMeasurer,
};

const PARTICIPANT_TEXT_PADDING_X: f32 = 24.0;
const MESSAGE_LABEL_PADDING_X: f32 = 48.0;
const PARTICIPANT_HEAD_GUTTER: f32 = 40.0;
const ICON_PARTICIPANT_HEAD_HEIGHT: f32 = 70.0;
const DISPLAY_TEXT_LINE_HEIGHT: f32 = 16.0;
const DISPLAY_SEPARATOR_HEIGHT: f32 = 10.0;
const PARTICIPANT_LABEL_PADDING_Y: f32 = 20.0;
const EVENT_VERTICAL_GAP: f32 = 12.0;
pub(crate) const NOTE_TEXT_PADDING_X: f32 = 12.0;
pub(crate) const NOTE_TARGET_GAP: f32 = 12.0;
pub(crate) const NOTE_CANVAS_MARGIN: f32 = 12.0;
pub(crate) const NOTE_MIN_WIDTH: f32 = 120.0;
pub(crate) const NOTE_BASELINE_OFFSET: f32 = 18.0;
pub(crate) const NOTE_BOTTOM_PADDING: f32 = 8.0;
const PAGE_HEADER_BASELINE_Y: f32 = 16.0;
const PAGE_TITLE_BASELINE_Y: f32 = 42.0;
const PAGE_TITLE_ONLY_BASELINE_Y: f32 = 28.0;
const PAGE_DECORATION_BOTTOM_GAP: f32 = 14.0;
const PAGE_FOOTER_RESERVED_HEIGHT: f32 = 28.0;
const PAGE_BREAK_GAP: f32 = 8.0;
const PAGE_BREAK_LABEL_GAP: f32 = 18.0;

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutOptions {
    pub margin_x: f32,
    pub margin_y: f32,
    pub participant_gap: f32,
    pub participant_head_width: f32,
    pub participant_head_height: f32,
    pub message_start_y: f32,
    pub message_gap: f32,
    pub tail_gap: f32,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            margin_x: 32.0,
            margin_y: 24.0,
            participant_gap: 160.0,
            participant_head_width: 104.0,
            participant_head_height: 36.0,
            message_start_y: 92.0,
            message_gap: 48.0,
            tail_gap: 40.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SequenceLayout {
    pub width: f32,
    pub height: f32,
    pub view_box: String,
    pub participants: Vec<ParticipantLayout>,
    pub messages: Vec<MessageLayout>,
    pub timeline: Vec<SequenceTimelineEventLayout>,
    pub(crate) title: Option<TextEventLayout>,
    pub(crate) header: Option<HeaderFooterLayout>,
    pub(crate) footer: Option<HeaderFooterLayout>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticipantLayout {
    pub id: String,
    pub name: String,
    pub kind: ParticipantKind,
    pub color: Option<String>,
    pub display: Option<ParticipantDisplay>,
    pub x: f32,
    pub head_y: f32,
    pub tail_y: f32,
    pub head_width: f32,
    pub head_height: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageLayout {
    pub from: String,
    pub to: String,
    pub label: String,
    pub line_style: LineStyle,
    pub arrow_raw: String,
    pub arrow_features: MessageArrowFeatures,
    pub activation_shortcut: Option<MessageActivationShortcut>,
    pub from_endpoint_kind: MessageEndpointKind,
    pub to_endpoint_kind: MessageEndpointKind,
    pub direction: MessageDirection,
    pub line: usize,
    pub start_x: f32,
    pub end_x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SequenceTimelineEventLayout {
    Message(MessageLayout),
    Title(TextEventLayout),
    Note(NoteLayout),
    GroupStart(GroupStartLayout),
    GroupElse(GroupElseLayout),
    GroupEnd(GroupEndLayout),
    Divider(TextEventLayout),
    Delay(DelayLayout),
    Activation(ActivationEventLayout),
    Return(ReturnEventLayout),
    BoxStart(BoxStartLayout),
    BoxEnd(BoxEndLayout),
    Semantic(SemanticEventLayout),
}

impl SequenceTimelineEventLayout {
    pub fn y(&self) -> f32 {
        match self {
            Self::Message(event) => event.y,
            Self::Title(event) => event.y,
            Self::Note(event) => event.y,
            Self::GroupStart(event) => event.y,
            Self::GroupElse(event) => event.y,
            Self::GroupEnd(event) => event.y,
            Self::Divider(event) => event.y,
            Self::Delay(event) => event.y,
            Self::Activation(event) => event.y,
            Self::Return(event) => event.y,
            Self::BoxStart(event) => event.y,
            Self::BoxEnd(event) => event.y,
            Self::Semantic(event) => event.y(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteLayout {
    pub placement: NotePlacement,
    pub targets: Vec<String>,
    pub explicit_targets: bool,
    pub text: String,
    pub color: Option<String>,
    pub shape: NoteShape,
    pub multiline: bool,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupStartLayout {
    pub kind: GroupKind,
    pub label: Option<String>,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupElseLayout {
    pub label: Option<String>,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupEndLayout {
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextEventLayout {
    pub text: String,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelayLayout {
    pub text: Option<String>,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivationEventLayout {
    pub kind: ActivationKind,
    pub participant: String,
    pub color: Option<String>,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnEventLayout {
    pub label: Option<String>,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxStartLayout {
    pub title: Option<String>,
    pub color: Option<String>,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoxEndLayout {
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticEventLayout {
    Autonumber(AutonumberLayout),
    PageBreak(PageBreakLayout),
    HeaderFooter(HeaderFooterLayout),
    Reference(ReferenceLayout),
    Space(SpaceLayout),
    Mainframe(MainframeLayout),
    StyleBlock(StyleBlockLayout),
    HideUnlinked(HideUnlinkedLayout),
    Autoactivate(AutoactivateLayout),
}

impl SemanticEventLayout {
    pub fn y(&self) -> f32 {
        match self {
            Self::Autonumber(event) => event.y,
            Self::PageBreak(event) => event.y,
            Self::HeaderFooter(event) => event.y,
            Self::Reference(event) => event.y,
            Self::Space(event) => event.y,
            Self::Mainframe(event) => event.y,
            Self::StyleBlock(event) => event.y,
            Self::HideUnlinked(event) => event.y,
            Self::Autoactivate(event) => event.y,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutonumberLayout {
    pub command: AutonumberCommand,
    pub args: Vec<String>,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBreakLayout {
    pub label: Option<String>,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderFooterLayout {
    pub kind: HeaderFooterKind,
    pub text: String,
    pub multiline: bool,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReferenceLayout {
    pub targets: Vec<String>,
    pub text: Option<String>,
    pub multiline: bool,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpaceLayout {
    pub size: Option<String>,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MainframeLayout {
    pub title: Option<String>,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleBlockLayout {
    pub content: String,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HideUnlinkedLayout {
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutoactivateLayout {
    pub enabled: bool,
    pub line: usize,
    pub y: f32,
}

#[derive(Debug, Clone)]
struct AutonumberState {
    enabled: bool,
    next: i64,
    increment: i64,
    format: Option<String>,
    current: Option<i64>,
}

#[derive(Debug, Clone, Default)]
struct AutonumberedText {
    message_labels: HashMap<usize, String>,
    note_texts: HashMap<usize, String>,
}

fn estimate_text_width(text: &str, measurer: &dyn TextMeasurer) -> f32 {
    measure_inline_text_with(text, measurer)
}
