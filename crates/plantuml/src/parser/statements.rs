//! Statement classification and semantic statement dispatch.
use super::*;

pub(super) fn parse_statement(
    line_number: usize,
    line: &str,
    group_stack: &mut Vec<OpenBlock>,
    box_stack: &mut Vec<OpenBlock>,
) -> Result<StatementParse, PlantUmlError> {
    if is_note_end(line) {
        return Err(PlantUmlError::UnexpectedBlockEnd {
            line: line_number,
            text: line.to_owned(),
        });
    }

    if is_title_end(line) {
        return Err(PlantUmlError::UnexpectedBlockEnd {
            line: line_number,
            text: line.to_owned(),
        });
    }

    if let Some(statement) = parse_title(line_number, line)? {
        return Ok(statement);
    }

    if line.eq_ignore_ascii_case("hide footbox") {
        return Ok(StatementParse::Event(SequenceEvent::HideFootbox(
            HideFootbox { line: line_number },
        )));
    }

    if let Some(block) = parse_ignored_block_start(line_number, line) {
        return Ok(StatementParse::StartIgnoredBlock(block));
    }

    if let Some(statement) = parse_semantic_statement(line_number, line)? {
        return Ok(statement);
    }

    if let Some(event) = parse_skinparam(line_number, line)? {
        return Ok(StatementParse::Event(event));
    }

    if let Some(event) = parse_pragma(line_number, line)? {
        return Ok(StatementParse::Event(event));
    }

    if line.eq_ignore_ascii_case("end box") {
        if box_stack.pop().is_none() {
            return Err(PlantUmlError::UnexpectedBlockEnd {
                line: line_number,
                text: line.to_owned(),
            });
        }

        return Ok(StatementParse::Event(SequenceEvent::BoxEnd(BoxEnd {
            line: line_number,
        })));
    }

    if line.eq_ignore_ascii_case("end") {
        if group_stack.pop().is_none() {
            return Err(PlantUmlError::UnexpectedBlockEnd {
                line: line_number,
                text: line.to_owned(),
            });
        }

        return Ok(StatementParse::Event(SequenceEvent::GroupEnd(GroupEnd {
            line: line_number,
        })));
    }

    if let Some(event) = parse_group_else(line_number, line, group_stack)? {
        return Ok(StatementParse::Event(event));
    }

    if let Some(event) = parse_group_start(line_number, line, group_stack) {
        return Ok(StatementParse::Event(event));
    }

    if let Some(event) = parse_box_start(line_number, line)? {
        box_stack.push(OpenBlock {
            kind: PlantUmlBlockKind::Box,
            line: line_number,
        });
        return Ok(StatementParse::Event(event));
    }

    if let Some(participant) = parse_multiline_participant_declaration_start(line_number, line) {
        return Ok(StatementParse::StartParticipantDeclaration(participant));
    }

    if let Some(event) = parse_participant_declaration(line_number, line)? {
        return Ok(StatementParse::Event(event));
    }

    if let Some(note) = parse_note(line_number, line)? {
        return Ok(note);
    }

    if let Some(event) = parse_divider(line_number, line) {
        return Ok(StatementParse::Event(event));
    }

    if let Some(event) = parse_delay(line_number, line) {
        return Ok(StatementParse::Event(event));
    }

    if let Some(event) = parse_activation(line_number, line)? {
        return Ok(StatementParse::Event(event));
    }

    if let Some(event) = parse_return(line_number, line) {
        return Ok(StatementParse::Event(event));
    }

    if let Some(parsed) = parse_message(line_number, line)? {
        let mut events = parsed
            .declarations
            .into_iter()
            .map(SequenceEvent::ParticipantDeclaration)
            .collect::<Vec<_>>();
        events.push(SequenceEvent::Message(parsed.message));
        return Ok(StatementParse::Events(events));
    }

    Err(PlantUmlError::UnsupportedStatement {
        line: line_number,
        text: line.to_owned(),
    })
}

pub(super) fn parse_semantic_statement(
    line_number: usize,
    line: &str,
) -> Result<Option<StatementParse>, PlantUmlError> {
    if let Some(size) = parse_space_separator(line) {
        return Ok(Some(StatementParse::Semantic(
            SequenceSemanticEvent::Space(Space {
                size,
                line: line_number,
                raw: line.to_owned(),
            }),
        )));
    }

    if let Some(rest) = strip_keyword(line, "autonumber") {
        let rest = rest.trim();
        let mut args = split_autonumber_args(rest);
        let (command, consume_command) = args
            .first()
            .and_then(|arg| autonumber_command_arg(arg))
            .map_or((AutonumberCommand::Start, false), |command| (command, true));
        if consume_command {
            args.remove(0);
        }

        return Ok(Some(StatementParse::Semantic(
            SequenceSemanticEvent::Autonumber(Autonumber {
                command,
                args,
                line: line_number,
                raw: line.to_owned(),
            }),
        )));
    }

    if let Some(rest) = strip_keyword(line, "newpage") {
        return Ok(Some(StatementParse::Semantic(
            SequenceSemanticEvent::PageBreak(PageBreak {
                label: non_empty(rest.trim()).map(ToOwned::to_owned),
                line: line_number,
                raw: line.to_owned(),
            }),
        )));
    }

    if let Some(rest) = strip_keyword(line, "header") {
        return parse_header_footer_statement(line_number, line, rest, HeaderFooterKind::Header)
            .map(Some);
    }

    if let Some(rest) = strip_keyword(line, "footer") {
        return parse_header_footer_statement(line_number, line, rest, HeaderFooterKind::Footer)
            .map(Some);
    }

    if let Some(rest) = strip_keyword(line, "ref") {
        return parse_reference_statement(line_number, line, rest).map(Some);
    }

    if let Some(rest) = strip_keyword(line, "space") {
        return Ok(Some(StatementParse::Semantic(
            SequenceSemanticEvent::Space(Space {
                size: non_empty(rest.trim()).map(ToOwned::to_owned),
                line: line_number,
                raw: line.to_owned(),
            }),
        )));
    }

    if let Some(rest) = strip_keyword(line, "mainframe") {
        return Ok(Some(StatementParse::Semantic(
            SequenceSemanticEvent::Mainframe(Mainframe {
                title: non_empty(rest.trim()).map(unquote_or_trimmed),
                line: line_number,
                raw: line.to_owned(),
            }),
        )));
    }

    if line.eq_ignore_ascii_case("style") {
        return Ok(Some(StatementParse::StartSemanticBlock(
            ActiveSemanticBlock {
                kind: ActiveSemanticBlockKind::Style,
                lines: Vec::new(),
                line: line_number,
                raw_start: line.to_owned(),
            },
        )));
    }

    if line.eq_ignore_ascii_case("hide unlinked") {
        return Ok(Some(StatementParse::Semantic(
            SequenceSemanticEvent::HideUnlinked(HideUnlinked {
                line: line_number,
                raw: line.to_owned(),
            }),
        )));
    }

    if let Some(rest) = strip_keyword(line, "autoactivate") {
        let enabled = match rest.trim().to_ascii_lowercase().as_str() {
            "" | "on" | "true" => true,
            "off" | "false" => false,
            _ => {
                return Err(PlantUmlError::InvalidStatement {
                    line: line_number,
                    text: line.to_owned(),
                });
            }
        };

        return Ok(Some(StatementParse::Semantic(
            SequenceSemanticEvent::Autoactivate(Autoactivate {
                enabled,
                line: line_number,
                raw: line.to_owned(),
            }),
        )));
    }

    Ok(None)
}
