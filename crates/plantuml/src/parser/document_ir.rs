//! Sequence IR assembly and participant identity insertion.
use super::*;

pub(super) fn build_sequence_ir(parse_tree: ParsedSequenceDocument) -> SequenceDiagram {
    let ParsedSequenceDocument {
        events,
        semantic_events,
    } = parse_tree;
    let mut participants = Vec::new();
    let mut messages = Vec::new();
    let mut participant_declarations = Vec::new();
    let mut title = None;
    let mut hide_footbox = false;
    let mut hide_unlinked = false;
    let mut autoactivate = None;
    let mut skinparams = Vec::new();

    for event in &semantic_events {
        match event {
            SequenceSemanticEvent::HideUnlinked(_) => hide_unlinked = true,
            SequenceSemanticEvent::Autoactivate(event) => autoactivate = Some(event.enabled),
            _ => {}
        }
    }

    for event in &events {
        match event {
            SequenceEvent::ParticipantDeclaration(declaration) => {
                participant_declarations.push(declaration.clone());
                insert_declared_participant(&mut participants, declaration);
            }
            SequenceEvent::Message(message) => {
                if message.from.is_participant() {
                    insert_implicit_participant(&mut participants, &message.from.id);
                }
                if message.to.is_participant() {
                    insert_implicit_participant(&mut participants, &message.to.id);
                }

                messages.push(Message {
                    from: message.from.id.clone(),
                    to: message.to.id.clone(),
                    label: message.label.clone(),
                    line_style: message.arrow.line_style,
                });
            }
            SequenceEvent::Note(note) => {
                for target in &note.targets {
                    insert_implicit_participant(&mut participants, target);
                }
            }
            SequenceEvent::Title(event_title) => title = Some(event_title.text.clone()),
            SequenceEvent::HideFootbox(_) => hide_footbox = true,
            SequenceEvent::SkinParam(skinparam) => skinparams.push(skinparam.clone()),
            SequenceEvent::Activation(activation) => {
                insert_implicit_participant(&mut participants, &activation.participant);
            }
            _ => {}
        }
    }

    SequenceDiagram {
        participants,
        messages,
        participant_declarations,
        events,
        semantic_events,
        title,
        hide_footbox,
        hide_unlinked,
        autoactivate,
        skinparams,
    }
}

pub(super) fn insert_declared_participant(
    participants: &mut Vec<Participant>,
    declaration: &ParticipantDeclaration,
) {
    if let Some(participant) = participants
        .iter_mut()
        .find(|participant| participant.id == declaration.id)
    {
        participant.name = declaration.name.clone();
        participant.kind = declaration.kind;
        participant.declaration = Some(declaration.clone());
        return;
    }

    participants.push(Participant {
        id: declaration.id.clone(),
        name: declaration.name.clone(),
        kind: declaration.kind,
        declaration: Some(declaration.clone()),
    });
}

pub(super) fn insert_implicit_participant(participants: &mut Vec<Participant>, id: &str) {
    if participants.iter().any(|participant| participant.id == id) {
        return;
    }

    participants.push(Participant {
        id: id.to_owned(),
        name: id.to_owned(),
        kind: ParticipantKind::Participant,
        declaration: None,
    });
}

pub(super) fn append_metadata(metadata: &mut String, value: &str) {
    if value.is_empty() {
        return;
    }
    if !metadata.is_empty() {
        metadata.push(' ');
    }
    metadata.push_str(value);
}
