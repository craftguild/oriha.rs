//! Sequence document parsing state and source-line progression.
mod statements;
#[allow(unused_imports)]
use statements::*;
mod directives;
#[allow(unused_imports)]
use directives::*;
mod participants;
#[allow(unused_imports)]
use participants::*;
mod endpoints;
#[allow(unused_imports)]
use endpoints::*;
mod notes;
#[allow(unused_imports)]
use notes::*;
mod semantic_events;
#[allow(unused_imports)]
use semantic_events::*;
mod messages;
#[allow(unused_imports)]
use messages::*;
mod arrows;
#[allow(unused_imports)]
use arrows::*;
mod document_ir;
#[allow(unused_imports)]
use document_ir::*;
mod text;
#[allow(unused_imports)]
use text::*;

use crate::sequence::{
    Activation, ActivationKind, Autoactivate, Autonumber, AutonumberCommand, BoxBlock, BoxEnd,
    Delay, Divider, Group, GroupElse, GroupEnd, GroupKind, HeaderFooter, HeaderFooterKind,
    HideFootbox, HideUnlinked, LineStyle, Mainframe, Message, MessageActivationShortcut,
    MessageActivationTarget, MessageArrow, MessageArrowFeatures, MessageArrowHalf,
    MessageArrowHead, MessageDirection, MessageEndpoint, MessageEndpointKind, Note, NotePlacement,
    NoteShape, PageBreak, Participant, ParticipantDeclaration, ParticipantDisplay, ParticipantKind,
    ParticipantSpot, PlantUmlBlockKind, PlantUmlError, Pragma, Reference, Return, SequenceDiagram,
    SequenceEvent, SequenceMessage, SequenceSemanticEvent, SkinParam, Space, StyleBlock, Title,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSequenceDocument {
    events: Vec<SequenceEvent>,
    semantic_events: Vec<SequenceSemanticEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveNote {
    placement: NotePlacement,
    targets: Vec<String>,
    color: Option<String>,
    shape: NoteShape,
    lines: Vec<String>,
    line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveTitle {
    lines: Vec<String>,
    line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ActiveIgnoredBlock {
    kind: ActiveIgnoredBlockKind,
    line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveIgnoredBlockKind {
    SkinParam,
    Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OpenBlock {
    kind: PlantUmlBlockKind,
    line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveSemanticBlock {
    kind: ActiveSemanticBlockKind,
    lines: Vec<String>,
    line: usize,
    raw_start: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ActiveSemanticBlockKind {
    HeaderFooter(HeaderFooterKind),
    Reference { targets: Vec<String> },
    Style,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MessageOperator {
    RightSolid,
    RightDashed,
    LeftSolid,
    LeftDashed,
    BidirectionalSolid,
    BidirectionalDashed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveParticipantDeclaration {
    kind: ParticipantKind,
    line: usize,
    header: String,
    lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ParticipantMetadata {
    color: Option<String>,
    order: Option<i32>,
    stereotype: Option<String>,
    spot: Option<ParticipantSpot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedMessage {
    message: SequenceMessage,
    declarations: Vec<ParticipantDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedMessageEndpoint {
    endpoint: MessageEndpoint,
    declaration: Option<ParticipantDeclaration>,
    color: Option<String>,
    activation_shortcut: Option<MessageActivationShortcut>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct VisualArrowSide {
    head: Option<MessageArrowHead>,
    circle: bool,
    cross: bool,
}

pub fn parse_sequence(source: &str) -> Result<SequenceDiagram, PlantUmlError> {
    let parse_tree = parse_document(source)?;
    Ok(build_sequence_ir(parse_tree))
}

fn parse_document(source: &str) -> Result<ParsedSequenceDocument, PlantUmlError> {
    let mut saw_start = false;
    let mut saw_end = false;
    let mut events = Vec::new();
    let mut semantic_events = Vec::new();
    let mut active_note: Option<ActiveNote> = None;
    let mut active_title: Option<ActiveTitle> = None;
    let mut active_ignored_block: Option<ActiveIgnoredBlock> = None;
    let mut active_semantic_block: Option<ActiveSemanticBlock> = None;
    let mut active_participant: Option<ActiveParticipantDeclaration> = None;
    let mut in_block_comment = false;
    let mut group_stack: Vec<OpenBlock> = Vec::new();
    let mut box_stack: Vec<OpenBlock> = Vec::new();
    let mut nested_diagram_depth = 0usize;

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();

        if in_block_comment {
            if is_block_comment_end(line) {
                in_block_comment = false;
            }
            continue;
        }

        if is_block_comment_start(line) {
            if !is_inline_block_comment(line) {
                in_block_comment = true;
            }
            continue;
        }

        if let Some(block) = active_ignored_block {
            if ignored_block_end_matches(block.kind, line) {
                active_ignored_block = None;
            } else if line.eq_ignore_ascii_case("@enduml") {
                return Err(PlantUmlError::UnclosedBlock {
                    line: block.line,
                    block: PlantUmlBlockKind::Style,
                });
            }
            continue;
        }

        if let Some(participant) = active_participant.as_mut() {
            if let Some(before_close) = line.strip_suffix(']') {
                if !before_close.trim().is_empty() {
                    participant.lines.push(before_close.trim_end().to_owned());
                }
                let participant = active_participant
                    .take()
                    .expect("active participant declaration should close");
                events.push(SequenceEvent::ParticipantDeclaration(
                    finish_multiline_participant_declaration(participant),
                ));
                continue;
            }

            if line.eq_ignore_ascii_case("@enduml") {
                return Err(PlantUmlError::UnclosedBlock {
                    line: participant.line,
                    block: PlantUmlBlockKind::Participant,
                });
            }

            participant.lines.push(raw_line.trim_end().to_owned());
            continue;
        }

        if let Some(title) = active_title.as_mut() {
            if is_title_end(line) {
                let title = active_title
                    .take()
                    .expect("active title should be present before closing");
                events.push(SequenceEvent::Title(Title {
                    text: decode_plantuml_text(&title.lines.join("\n")),
                    line: title.line,
                }));
                continue;
            }

            if line.eq_ignore_ascii_case("@enduml") {
                return Err(PlantUmlError::InvalidStatement {
                    line: title.line,
                    text: "title".to_owned(),
                });
            }

            title.lines.push(raw_line.trim_end().to_owned());
            continue;
        }

        if let Some(block) = active_semantic_block.as_mut() {
            if semantic_block_end_matches(&block.kind, line) {
                let block = active_semantic_block
                    .take()
                    .expect("active semantic block should close");
                let semantic_event = finish_semantic_block(block);
                events.push(semantic_event_as_pragma(&semantic_event));
                semantic_events.push(semantic_event);
                continue;
            }

            if line.eq_ignore_ascii_case("@enduml") {
                return Err(PlantUmlError::UnclosedBlock {
                    line: block.line,
                    block: semantic_block_kind(&block.kind),
                });
            }

            block.lines.push(raw_line.trim_end().to_owned());
            continue;
        }

        if let Some(note) = active_note.as_mut() {
            if is_note_end(line) {
                let note = active_note
                    .take()
                    .expect("active note should be present before closing");
                events.push(SequenceEvent::Note(Note {
                    placement: note.placement,
                    targets: note.targets,
                    text: decode_plantuml_text(&note.lines.join("\n")),
                    color: note.color,
                    shape: note.shape,
                    multiline: true,
                    line: note.line,
                }));
                continue;
            }

            if line.eq_ignore_ascii_case("@enduml") {
                return Err(PlantUmlError::UnclosedBlock {
                    line: note.line,
                    block: PlantUmlBlockKind::Note,
                });
            }

            note.lines.push(raw_line.trim_end().to_owned());
            continue;
        }

        if line.is_empty() {
            continue;
        }

        if is_comment(line) {
            continue;
        }

        if !saw_start {
            if is_startuml_directive(line) {
                saw_start = true;
                continue;
            }
            return Err(PlantUmlError::MissingStart);
        }

        if saw_end {
            return Err(PlantUmlError::ContentAfterEnd { line: line_number });
        }

        if is_startuml_directive(line) {
            nested_diagram_depth += 1;
            continue;
        }

        if line.eq_ignore_ascii_case("@enduml") {
            if nested_diagram_depth > 0 {
                nested_diagram_depth -= 1;
                continue;
            }

            if let Some(block) = box_stack.last().copied() {
                return Err(PlantUmlError::UnclosedBlock {
                    line: block.line,
                    block: block.kind,
                });
            }
            if let Some(block) = group_stack.last().copied() {
                return Err(PlantUmlError::UnclosedBlock {
                    line: block.line,
                    block: block.kind,
                });
            }

            saw_end = true;
            continue;
        }

        match parse_statement(line_number, line, &mut group_stack, &mut box_stack)? {
            StatementParse::Event(event) => events.push(event),
            StatementParse::Events(parsed_events) => events.extend(parsed_events),
            StatementParse::Semantic(semantic_event) => {
                events.push(semantic_event_as_pragma(&semantic_event));
                semantic_events.push(semantic_event);
            }
            StatementParse::StartNote(note) => active_note = Some(note),
            StatementParse::StartTitle(title) => active_title = Some(title),
            StatementParse::StartIgnoredBlock(block) => active_ignored_block = Some(block),
            StatementParse::StartSemanticBlock(block) => active_semantic_block = Some(block),
            StatementParse::StartParticipantDeclaration(participant) => {
                active_participant = Some(participant);
            }
        }
    }

    if let Some(participant) = active_participant {
        return Err(PlantUmlError::UnclosedBlock {
            line: participant.line,
            block: PlantUmlBlockKind::Participant,
        });
    }

    if let Some(title) = active_title {
        return Err(PlantUmlError::InvalidStatement {
            line: title.line,
            text: "title".to_owned(),
        });
    }

    if let Some(block) = active_ignored_block {
        return Err(PlantUmlError::UnclosedBlock {
            line: block.line,
            block: PlantUmlBlockKind::Style,
        });
    }

    if let Some(block) = active_semantic_block {
        return Err(PlantUmlError::UnclosedBlock {
            line: block.line,
            block: semantic_block_kind(&block.kind),
        });
    }

    if let Some(note) = active_note {
        return Err(PlantUmlError::UnclosedBlock {
            line: note.line,
            block: PlantUmlBlockKind::Note,
        });
    }

    if !saw_start {
        return Err(PlantUmlError::MissingStart);
    }
    if !saw_end {
        return Err(PlantUmlError::MissingEnd);
    }

    Ok(ParsedSequenceDocument {
        events,
        semantic_events,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StatementParse {
    Event(SequenceEvent),
    Events(Vec<SequenceEvent>),
    Semantic(SequenceSemanticEvent),
    StartNote(ActiveNote),
    StartTitle(ActiveTitle),
    StartIgnoredBlock(ActiveIgnoredBlock),
    StartSemanticBlock(ActiveSemanticBlock),
    StartParticipantDeclaration(ActiveParticipantDeclaration),
}

fn ignored_block_end_matches(kind: ActiveIgnoredBlockKind, line: &str) -> bool {
    match kind {
        ActiveIgnoredBlockKind::SkinParam => {
            line == "}" || line.eq_ignore_ascii_case("endskinparam")
        }
        ActiveIgnoredBlockKind::Style => {
            line.eq_ignore_ascii_case("</style>") || line.eq_ignore_ascii_case("endstyle")
        }
    }
}

fn is_comment(line: &str) -> bool {
    line.starts_with('\'') || line.starts_with("//")
}

fn is_block_comment_start(line: &str) -> bool {
    line.starts_with("/'") || line.starts_with("/*")
}

fn is_block_comment_end(line: &str) -> bool {
    line.ends_with("'/") || line.ends_with("*/")
}

fn is_inline_block_comment(line: &str) -> bool {
    is_block_comment_start(line) && is_block_comment_end(line)
}

fn is_startuml_directive(line: &str) -> bool {
    let (directive, _) = split_first_word(line);
    directive.eq_ignore_ascii_case("@startuml")
}
