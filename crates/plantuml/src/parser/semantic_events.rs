//! Semantic block assembly and activation-style events.
use super::*;

pub(super) fn parse_divider(line_number: usize, line: &str) -> Option<SequenceEvent> {
    if line.len() >= 4 && line.starts_with("==") && line.ends_with("==") {
        let text = line
            .trim_start_matches('=')
            .trim_end_matches('=')
            .trim()
            .to_owned();
        return Some(SequenceEvent::Divider(Divider {
            text,
            line: line_number,
        }));
    }

    None
}

pub(super) fn parse_delay(line_number: usize, line: &str) -> Option<SequenceEvent> {
    if !line.starts_with("...") {
        return None;
    }

    if line == "..." {
        return Some(SequenceEvent::Delay(Delay {
            text: None,
            line: line_number,
        }));
    }

    if line.ends_with("...") {
        let text = line
            .trim_start_matches("...")
            .trim_end_matches("...")
            .trim();
        return Some(SequenceEvent::Delay(Delay {
            text: non_empty(text).map(ToOwned::to_owned),
            line: line_number,
        }));
    }

    None
}

pub(super) fn parse_activation(
    line_number: usize,
    line: &str,
) -> Result<Option<SequenceEvent>, PlantUmlError> {
    let (keyword, rest) = split_first_word(line);
    let kind = match keyword.to_ascii_lowercase().as_str() {
        "create" => ActivationKind::Create,
        "activate" => ActivationKind::Activate,
        "deactivate" => ActivationKind::Deactivate,
        "destroy" => ActivationKind::Destroy,
        _ => return Ok(None),
    };

    let (participant, remaining) =
        take_value(rest.trim()).ok_or_else(|| PlantUmlError::InvalidStatement {
            line: line_number,
            text: line.to_owned(),
        })?;

    Ok(Some(SequenceEvent::Activation(Activation {
        kind,
        participant,
        color: non_empty(remaining.trim()).map(ToOwned::to_owned),
        line: line_number,
    })))
}

pub(super) fn parse_return(line_number: usize, line: &str) -> Option<SequenceEvent> {
    let rest = strip_keyword(line, "return")?;
    Some(SequenceEvent::Return(Return {
        label: non_empty(rest.trim()).map(ToOwned::to_owned),
        line: line_number,
    }))
}

pub(super) fn semantic_block_end_matches(kind: &ActiveSemanticBlockKind, line: &str) -> bool {
    match kind {
        ActiveSemanticBlockKind::HeaderFooter(HeaderFooterKind::Header) => {
            line.eq_ignore_ascii_case("endheader") || line.eq_ignore_ascii_case("end header")
        }
        ActiveSemanticBlockKind::HeaderFooter(HeaderFooterKind::Footer) => {
            line.eq_ignore_ascii_case("endfooter") || line.eq_ignore_ascii_case("end footer")
        }
        ActiveSemanticBlockKind::Reference { .. } => {
            line.eq_ignore_ascii_case("end ref") || line.eq_ignore_ascii_case("endref")
        }
        ActiveSemanticBlockKind::Style => line.eq_ignore_ascii_case("endstyle"),
    }
}

pub(super) fn finish_semantic_block(block: ActiveSemanticBlock) -> SequenceSemanticEvent {
    let text = block.lines.join("\n");
    match block.kind {
        ActiveSemanticBlockKind::HeaderFooter(kind) => {
            SequenceSemanticEvent::HeaderFooter(HeaderFooter {
                kind,
                text,
                multiline: true,
                line: block.line,
                raw: block.raw_start,
            })
        }
        ActiveSemanticBlockKind::Reference { targets } => {
            SequenceSemanticEvent::Reference(Reference {
                targets,
                text: if text.trim().is_empty() {
                    None
                } else {
                    Some(text)
                },
                multiline: true,
                line: block.line,
                raw: block.raw_start,
            })
        }
        ActiveSemanticBlockKind::Style => SequenceSemanticEvent::StyleBlock(StyleBlock {
            content: text,
            line: block.line,
            raw: block.raw_start,
        }),
    }
}

pub(super) fn semantic_block_kind(kind: &ActiveSemanticBlockKind) -> PlantUmlBlockKind {
    match kind {
        ActiveSemanticBlockKind::HeaderFooter(HeaderFooterKind::Header) => {
            PlantUmlBlockKind::Header
        }
        ActiveSemanticBlockKind::HeaderFooter(HeaderFooterKind::Footer) => {
            PlantUmlBlockKind::Footer
        }
        ActiveSemanticBlockKind::Reference { .. } => PlantUmlBlockKind::Ref,
        ActiveSemanticBlockKind::Style => PlantUmlBlockKind::Style,
    }
}

pub(super) fn semantic_event_as_pragma(event: &SequenceSemanticEvent) -> SequenceEvent {
    let (key, value, line, raw) = match event {
        SequenceSemanticEvent::Autonumber(event) => (
            "autonumber",
            (!event.args.is_empty()).then(|| event.args.join(" ")),
            event.line,
            event.raw.clone(),
        ),
        SequenceSemanticEvent::PageBreak(event) => (
            "newpage",
            event.label.clone(),
            event.line,
            event.raw.clone(),
        ),
        SequenceSemanticEvent::HeaderFooter(event) => (
            match event.kind {
                HeaderFooterKind::Header => "header",
                HeaderFooterKind::Footer => "footer",
            },
            non_empty(event.text.trim()).map(ToOwned::to_owned),
            event.line,
            event.raw.clone(),
        ),
        SequenceSemanticEvent::Reference(event) => {
            ("ref", event.text.clone(), event.line, event.raw.clone())
        }
        SequenceSemanticEvent::Space(event) => {
            ("space", event.size.clone(), event.line, event.raw.clone())
        }
        SequenceSemanticEvent::Mainframe(event) => (
            "mainframe",
            event.title.clone(),
            event.line,
            event.raw.clone(),
        ),
        SequenceSemanticEvent::StyleBlock(event) => (
            "style",
            non_empty(event.content.trim()).map(ToOwned::to_owned),
            event.line,
            event.raw.clone(),
        ),
        SequenceSemanticEvent::HideUnlinked(event) => {
            ("hide unlinked", None, event.line, event.raw.clone())
        }
        SequenceSemanticEvent::Autoactivate(event) => (
            "autoactivate",
            Some(if event.enabled { "on" } else { "off" }.to_owned()),
            event.line,
            event.raw.clone(),
        ),
    };

    SequenceEvent::Pragma(Pragma {
        key: key.to_owned(),
        value,
        line,
        raw,
    })
}
