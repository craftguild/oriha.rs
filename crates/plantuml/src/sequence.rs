//! Public sequence, participant, note and event IR.
mod messages;
#[allow(unused_imports)]
use messages::*;
pub use messages::{
    LineStyle, Message, MessageActivationShortcut, MessageActivationTarget, MessageArrow,
    MessageArrowFeatures, MessageArrowHalf, MessageArrowHead, MessageDirection, MessageEndpoint,
    MessageEndpointKind, SequenceMessage,
};

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceDiagram {
    pub participants: Vec<Participant>,
    pub messages: Vec<Message>,
    pub participant_declarations: Vec<ParticipantDeclaration>,
    pub events: Vec<SequenceEvent>,
    pub semantic_events: Vec<SequenceSemanticEvent>,
    pub title: Option<String>,
    pub hide_footbox: bool,
    pub hide_unlinked: bool,
    pub autoactivate: Option<bool>,
    pub skinparams: Vec<SkinParam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub kind: ParticipantKind,
    pub declaration: Option<ParticipantDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantDeclaration {
    pub id: String,
    pub name: String,
    pub alias: Option<String>,
    pub kind: ParticipantKind,
    pub color: Option<String>,
    pub order: Option<i32>,
    pub stereotype: Option<String>,
    pub spot: Option<ParticipantSpot>,
    pub display: Option<ParticipantDisplay>,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantDisplay {
    pub lines: Vec<String>,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantSpot {
    pub letter: char,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipantKind {
    Participant,
    Actor,
    Boundary,
    Control,
    Entity,
    Database,
    Collections,
    Queue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequenceEvent {
    ParticipantDeclaration(ParticipantDeclaration),
    Message(SequenceMessage),
    Note(Note),
    GroupStart(Group),
    GroupElse(GroupElse),
    GroupEnd(GroupEnd),
    Divider(Divider),
    Delay(Delay),
    Activation(Activation),
    Return(Return),
    BoxStart(BoxBlock),
    BoxEnd(BoxEnd),
    Title(Title),
    HideFootbox(HideFootbox),
    SkinParam(SkinParam),
    Pragma(Pragma),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    pub placement: NotePlacement,
    pub targets: Vec<String>,
    pub text: String,
    pub color: Option<String>,
    pub shape: NoteShape,
    pub multiline: bool,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteShape {
    Normal,
    Rectangle,
    Hexagon,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotePlacement {
    Left,
    Right,
    Over,
    Across,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub kind: GroupKind,
    pub label: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupKind {
    Alt,
    Opt,
    Loop,
    Par,
    Break,
    Critical,
    Group,
    Partition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupElse {
    pub label: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupEnd {
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divider {
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delay {
    pub text: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activation {
    pub kind: ActivationKind,
    pub participant: String,
    pub color: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationKind {
    Create,
    Activate,
    Deactivate,
    Destroy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Return {
    pub label: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxBlock {
    pub title: Option<String>,
    pub color: Option<String>,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxEnd {
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title {
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HideFootbox {
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkinParam {
    pub key: String,
    pub value: Option<String>,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pragma {
    pub key: String,
    pub value: Option<String>,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequenceSemanticEvent {
    Autonumber(Autonumber),
    PageBreak(PageBreak),
    HeaderFooter(HeaderFooter),
    Reference(Reference),
    Space(Space),
    Mainframe(Mainframe),
    StyleBlock(StyleBlock),
    HideUnlinked(HideUnlinked),
    Autoactivate(Autoactivate),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Autonumber {
    pub command: AutonumberCommand,
    pub args: Vec<String>,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonumberCommand {
    Start,
    Stop,
    Resume,
    Increment,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageBreak {
    pub label: Option<String>,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderFooter {
    pub kind: HeaderFooterKind,
    pub text: String,
    pub multiline: bool,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderFooterKind {
    Header,
    Footer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    pub targets: Vec<String>,
    pub text: Option<String>,
    pub multiline: bool,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Space {
    pub size: Option<String>,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mainframe {
    pub title: Option<String>,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleBlock {
    pub content: String,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HideUnlinked {
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Autoactivate {
    pub enabled: bool,
    pub line: usize,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantUmlBlockKind {
    Note,
    Group,
    Box,
    Header,
    Footer,
    Ref,
    Style,
    Participant,
}

impl PlantUmlBlockKind {
    fn label(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Group => "group",
            Self::Box => "box",
            Self::Header => "header",
            Self::Footer => "footer",
            Self::Ref => "ref",
            Self::Style => "style",
            Self::Participant => "participant",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlantUmlError {
    MissingStart,
    MissingEnd,
    ContentAfterEnd {
        line: usize,
    },
    InvalidMessage {
        line: usize,
        text: String,
    },
    InvalidStatement {
        line: usize,
        text: String,
    },
    UnsupportedStatement {
        line: usize,
        text: String,
    },
    UnexpectedBlockEnd {
        line: usize,
        text: String,
    },
    UnclosedBlock {
        line: usize,
        block: PlantUmlBlockKind,
    },
}

impl fmt::Display for PlantUmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStart => write!(f, "missing @startuml"),
            Self::MissingEnd => write!(f, "missing @enduml"),
            Self::ContentAfterEnd { line } => write!(f, "content after @enduml at line {line}"),
            Self::InvalidMessage { line, text } => {
                write!(f, "invalid sequence message at line {line}: {text}")
            }
            Self::InvalidStatement { line, text } => {
                write!(f, "invalid PlantUML statement at line {line}: {text}")
            }
            Self::UnsupportedStatement { line, text } => {
                write!(f, "unsupported PlantUML statement at line {line}: {text}")
            }
            Self::UnexpectedBlockEnd { line, text } => {
                write!(f, "unexpected block end at line {line}: {text}")
            }
            Self::UnclosedBlock { line, block } => {
                write!(
                    f,
                    "unclosed PlantUML {} block starting at line {line}",
                    block.label()
                )
            }
        }
    }
}

impl std::error::Error for PlantUmlError {}
