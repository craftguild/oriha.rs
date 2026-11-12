//! Public message, endpoint, arrow and activation-shortcut IR types.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub label: String,
    pub line_style: LineStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dashed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceMessage {
    pub from: MessageEndpoint,
    pub to: MessageEndpoint,
    pub label: String,
    pub arrow: MessageArrow,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageEndpoint {
    pub id: String,
    pub raw: String,
    pub kind: MessageEndpointKind,
}

impl MessageEndpoint {
    pub fn is_participant(&self) -> bool {
        matches!(self.kind, MessageEndpointKind::Participant)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageEndpointKind {
    Participant,
    Incoming,
    Outgoing,
    Unknown,
    Short,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageArrow {
    pub raw: String,
    pub direction: MessageDirection,
    pub line_style: LineStyle,
    pub features: MessageArrowFeatures,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MessageArrowFeatures {
    pub lost: bool,
    pub thin: bool,
    pub half: Option<MessageArrowHalf>,
    pub open_circle: bool,
    pub bidirectional: bool,
    pub start_head: Option<MessageArrowHead>,
    pub end_head: Option<MessageArrowHead>,
    pub start_circle: bool,
    pub end_circle: bool,
    pub start_cross: bool,
    pub end_cross: bool,
    pub color: Option<String>,
    pub slant: Option<i32>,
    pub incoming: bool,
    pub outgoing: bool,
    pub unknown_endpoint: bool,
    pub short_endpoint: bool,
    pub parallel: bool,
    pub activation_shortcut: Option<MessageActivationShortcut>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageArrowHalf {
    Top,
    Bottom,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageArrowHead {
    Filled,
    Open,
    HalfTop,
    HalfBottom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageActivationShortcut {
    pub kind: ActivationKind,
    pub target: MessageActivationTarget,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageActivationTarget {
    From,
    To,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    LeftToRight,
    RightToLeft,
    Bidirectional,
    SelfMessage,
}
