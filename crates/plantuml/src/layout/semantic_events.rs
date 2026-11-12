//! Semantic event positions and labels.
use super::*;

pub(super) fn group_start_label(group: &GroupStartLayout) -> String {
    match &group.label {
        Some(label) => format!("{} {}", group_kind_name(group.kind), label),
        None => group_kind_name(group.kind).to_owned(),
    }
}

pub(super) fn semantic_event_label(event: &SemanticEventLayout) -> String {
    match event {
        SemanticEventLayout::Autonumber(event) => {
            let mut label = format!("autonumber {}", autonumber_command_name(event.command));
            if !event.args.is_empty() {
                label.push(' ');
                label.push_str(&event.args.join(" "));
            }
            label.trim_end().to_owned()
        }
        SemanticEventLayout::PageBreak(event) => {
            event.label.clone().unwrap_or_else(|| "newpage".to_owned())
        }
        SemanticEventLayout::HeaderFooter(event) => {
            format!("{}: {}", header_footer_kind_name(event.kind), event.text)
        }
        SemanticEventLayout::Reference(event) => {
            event.text.clone().unwrap_or_else(|| "ref".to_owned())
        }
        SemanticEventLayout::Space(event) => event
            .size
            .as_deref()
            .map_or("space".to_owned(), |size| format!("space {size}")),
        SemanticEventLayout::Mainframe(event) => event
            .title
            .clone()
            .unwrap_or_else(|| "mainframe".to_owned()),
        SemanticEventLayout::StyleBlock(_) => "style block".to_owned(),
        SemanticEventLayout::HideUnlinked(_) => "hide unlinked".to_owned(),
        SemanticEventLayout::Autoactivate(event) => {
            if event.enabled {
                "autoactivate on".to_owned()
            } else {
                "autoactivate off".to_owned()
            }
        }
    }
}

pub(super) fn group_kind_name(kind: GroupKind) -> &'static str {
    match kind {
        GroupKind::Alt => "alt",
        GroupKind::Opt => "opt",
        GroupKind::Loop => "loop",
        GroupKind::Par => "par",
        GroupKind::Break => "break",
        GroupKind::Critical => "critical",
        GroupKind::Group => "group",
        GroupKind::Partition => "partition",
    }
}

pub(super) fn autonumber_command_name(command: AutonumberCommand) -> &'static str {
    match command {
        AutonumberCommand::Start => "start",
        AutonumberCommand::Stop => "stop",
        AutonumberCommand::Resume => "resume",
        AutonumberCommand::Increment => "increment",
        AutonumberCommand::Off => "off",
    }
}

pub(super) fn header_footer_kind_name(kind: HeaderFooterKind) -> &'static str {
    match kind {
        HeaderFooterKind::Header => "header",
        HeaderFooterKind::Footer => "footer",
    }
}

pub(super) fn layout_semantic_event_at_line(
    semantic_events: &[SequenceSemanticEvent],
    semantic_index: &mut usize,
    line: usize,
    y: f32,
) -> Option<SemanticEventLayout> {
    while let Some(event) = semantic_events.get(*semantic_index) {
        let event_line = semantic_event_line(event);
        if event_line < line {
            *semantic_index += 1;
            continue;
        }
        if event_line == line {
            *semantic_index += 1;
            return match event {
                SequenceSemanticEvent::Autonumber(_) => None,
                _ => Some(layout_semantic_event(event, y)),
            };
        }
        break;
    }

    None
}

pub(super) fn semantic_event_line(event: &SequenceSemanticEvent) -> usize {
    match event {
        SequenceSemanticEvent::Autonumber(event) => event.line,
        SequenceSemanticEvent::PageBreak(event) => event.line,
        SequenceSemanticEvent::HeaderFooter(event) => event.line,
        SequenceSemanticEvent::Reference(event) => event.line,
        SequenceSemanticEvent::Space(event) => event.line,
        SequenceSemanticEvent::Mainframe(event) => event.line,
        SequenceSemanticEvent::StyleBlock(event) => event.line,
        SequenceSemanticEvent::HideUnlinked(event) => event.line,
        SequenceSemanticEvent::Autoactivate(event) => event.line,
    }
}

pub(super) fn layout_semantic_event(event: &SequenceSemanticEvent, y: f32) -> SemanticEventLayout {
    match event {
        SequenceSemanticEvent::Autonumber(event) => {
            SemanticEventLayout::Autonumber(AutonumberLayout {
                command: event.command,
                args: event.args.clone(),
                line: event.line,
                y,
            })
        }
        SequenceSemanticEvent::PageBreak(event) => {
            SemanticEventLayout::PageBreak(PageBreakLayout {
                label: event.label.clone(),
                line: event.line,
                y,
            })
        }
        SequenceSemanticEvent::HeaderFooter(event) => {
            SemanticEventLayout::HeaderFooter(HeaderFooterLayout {
                kind: event.kind,
                text: event.text.clone(),
                multiline: event.multiline,
                line: event.line,
                y,
            })
        }
        SequenceSemanticEvent::Reference(event) => {
            SemanticEventLayout::Reference(ReferenceLayout {
                targets: event.targets.clone(),
                text: event.text.clone(),
                multiline: event.multiline,
                line: event.line,
                y,
            })
        }
        SequenceSemanticEvent::Space(event) => SemanticEventLayout::Space(SpaceLayout {
            size: event.size.clone(),
            line: event.line,
            y,
        }),
        SequenceSemanticEvent::Mainframe(event) => {
            SemanticEventLayout::Mainframe(MainframeLayout {
                title: event.title.clone(),
                line: event.line,
                y,
            })
        }
        SequenceSemanticEvent::StyleBlock(event) => {
            SemanticEventLayout::StyleBlock(StyleBlockLayout {
                content: event.content.clone(),
                line: event.line,
                y,
            })
        }
        SequenceSemanticEvent::HideUnlinked(event) => {
            SemanticEventLayout::HideUnlinked(HideUnlinkedLayout {
                line: event.line,
                y,
            })
        }
        SequenceSemanticEvent::Autoactivate(event) => {
            SemanticEventLayout::Autoactivate(AutoactivateLayout {
                enabled: event.enabled,
                line: event.line,
                y,
            })
        }
    }
}
