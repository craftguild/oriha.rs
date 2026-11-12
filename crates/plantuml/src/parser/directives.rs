//! Document, group, page and autonumber directive grammar.
use super::*;

pub(super) fn autonumber_command_arg(arg: &str) -> Option<AutonumberCommand> {
    match arg.to_ascii_lowercase().as_str() {
        "start" => Some(AutonumberCommand::Start),
        "stop" => Some(AutonumberCommand::Stop),
        "resume" => Some(AutonumberCommand::Resume),
        "inc" | "increment" => Some(AutonumberCommand::Increment),
        "off" => Some(AutonumberCommand::Off),
        _ => None,
    }
}

pub(super) fn split_autonumber_args(rest: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    let mut quoted = false;

    for character in rest.chars() {
        if in_quote {
            if character == '"' {
                args.push(current.clone());
                current.clear();
                in_quote = false;
                quoted = false;
            } else {
                current.push(character);
            }
            continue;
        }

        if character == '"' {
            if !current.is_empty() {
                args.push(current.clone());
                current.clear();
            }
            in_quote = true;
            quoted = true;
            continue;
        }

        if character.is_whitespace() {
            if !current.is_empty() {
                args.push(current.clone());
                current.clear();
            }
            continue;
        }

        current.push(character);
    }

    if !current.is_empty() || quoted {
        args.push(current);
    }

    args
}

pub(super) fn parse_space_separator(line: &str) -> Option<Option<String>> {
    if line == "|||" {
        return Some(None);
    }

    let inner = line.strip_prefix("||")?.strip_suffix("||")?.trim();
    if inner.is_empty() {
        return None;
    }

    Some(Some(inner.to_owned()))
}

pub(super) fn parse_header_footer_statement(
    line_number: usize,
    line: &str,
    rest: &str,
    kind: HeaderFooterKind,
) -> Result<StatementParse, PlantUmlError> {
    let text = rest.trim();
    if text.is_empty() {
        return Ok(StatementParse::StartSemanticBlock(ActiveSemanticBlock {
            kind: ActiveSemanticBlockKind::HeaderFooter(kind),
            lines: Vec::new(),
            line: line_number,
            raw_start: line.to_owned(),
        }));
    }

    Ok(StatementParse::Semantic(
        SequenceSemanticEvent::HeaderFooter(HeaderFooter {
            kind,
            text: text.to_owned(),
            multiline: false,
            line: line_number,
            raw: line.to_owned(),
        }),
    ))
}

pub(super) fn parse_reference_statement(
    line_number: usize,
    line: &str,
    rest: &str,
) -> Result<StatementParse, PlantUmlError> {
    let rest = rest.trim();
    let rest = strip_keyword(rest, "over").unwrap_or(rest).trim();
    let (target_source, text) = rest.split_once(':').map_or((rest, None), |(head, body)| {
        (head.trim(), non_empty(body.trim()).map(ToOwned::to_owned))
    });
    let targets = parse_targets(target_source);

    if text.is_some() {
        return Ok(StatementParse::Semantic(SequenceSemanticEvent::Reference(
            Reference {
                targets,
                text,
                multiline: false,
                line: line_number,
                raw: line.to_owned(),
            },
        )));
    }

    Ok(StatementParse::StartSemanticBlock(ActiveSemanticBlock {
        kind: ActiveSemanticBlockKind::Reference { targets },
        lines: Vec::new(),
        line: line_number,
        raw_start: line.to_owned(),
    }))
}

pub(super) fn parse_title(
    line_number: usize,
    line: &str,
) -> Result<Option<StatementParse>, PlantUmlError> {
    let Some(rest) = strip_keyword(line, "title") else {
        return Ok(None);
    };

    let text = rest.trim();
    if text.is_empty() {
        return Ok(Some(StatementParse::StartTitle(ActiveTitle {
            lines: Vec::new(),
            line: line_number,
        })));
    }

    Ok(Some(StatementParse::Event(SequenceEvent::Title(Title {
        text: decode_plantuml_text(text),
        line: line_number,
    }))))
}

pub(super) fn parse_ignored_block_start(
    line_number: usize,
    line: &str,
) -> Option<ActiveIgnoredBlock> {
    if line.eq_ignore_ascii_case("<style>") {
        return Some(ActiveIgnoredBlock {
            kind: ActiveIgnoredBlockKind::Style,
            line: line_number,
        });
    }

    let rest = strip_keyword(line, "skinparam")?.trim();
    rest.ends_with('{').then_some(ActiveIgnoredBlock {
        kind: ActiveIgnoredBlockKind::SkinParam,
        line: line_number,
    })
}

pub(super) fn parse_skinparam(
    line_number: usize,
    line: &str,
) -> Result<Option<SequenceEvent>, PlantUmlError> {
    let Some(rest) = strip_keyword(line, "skinparam") else {
        return Ok(None);
    };
    let rest = rest.trim();
    if rest.is_empty() {
        return Err(PlantUmlError::InvalidStatement {
            line: line_number,
            text: line.to_owned(),
        });
    }

    let (key, value) = split_first_word(rest);
    Ok(Some(SequenceEvent::SkinParam(SkinParam {
        key: key.to_owned(),
        value: non_empty(value.trim()).map(ToOwned::to_owned),
        line: line_number,
        raw: line.to_owned(),
    })))
}

pub(super) fn parse_pragma(
    line_number: usize,
    line: &str,
) -> Result<Option<SequenceEvent>, PlantUmlError> {
    let Some(rest) = strip_keyword(line, "!pragma") else {
        return Ok(None);
    };
    let rest = rest.trim();
    if rest.is_empty() {
        return Err(PlantUmlError::InvalidStatement {
            line: line_number,
            text: line.to_owned(),
        });
    }

    let (key, value) = split_first_word(rest);
    Ok(Some(SequenceEvent::Pragma(Pragma {
        key: key.to_owned(),
        value: non_empty(value.trim()).map(ToOwned::to_owned),
        line: line_number,
        raw: line.to_owned(),
    })))
}

pub(super) fn parse_group_else(
    line_number: usize,
    line: &str,
    group_stack: &[OpenBlock],
) -> Result<Option<SequenceEvent>, PlantUmlError> {
    let Some(rest) = strip_keyword(line, "else") else {
        return Ok(None);
    };

    if group_stack.is_empty() {
        return Err(PlantUmlError::InvalidStatement {
            line: line_number,
            text: line.to_owned(),
        });
    }

    Ok(Some(SequenceEvent::GroupElse(GroupElse {
        label: non_empty(strip_leading_color_tokens(rest)).map(ToOwned::to_owned),
        line: line_number,
    })))
}

pub(super) fn strip_group_keyword_color(keyword: &str) -> &str {
    keyword
        .split_once('#')
        .map(|(keyword, _)| keyword)
        .unwrap_or(keyword)
}

pub(super) fn strip_leading_color_tokens(input: &str) -> &str {
    let mut rest = input.trim();
    loop {
        let (token, trailing) = split_first_word(rest);
        if token.len() > 1 && token.starts_with('#') {
            rest = trailing.trim_start();
            continue;
        }

        return rest;
    }
}

pub(super) fn parse_group_start(
    line_number: usize,
    line: &str,
    group_stack: &mut Vec<OpenBlock>,
) -> Option<SequenceEvent> {
    let (keyword, rest) = split_first_word(line);
    let keyword = strip_group_keyword_color(keyword);
    let kind = match keyword.to_ascii_lowercase().as_str() {
        "alt" => GroupKind::Alt,
        "opt" => GroupKind::Opt,
        "loop" => GroupKind::Loop,
        "par" => GroupKind::Par,
        "break" => GroupKind::Break,
        "critical" => GroupKind::Critical,
        "group" => GroupKind::Group,
        "partition" => GroupKind::Partition,
        _ => return None,
    };

    group_stack.push(OpenBlock {
        kind: PlantUmlBlockKind::Group,
        line: line_number,
    });

    Some(SequenceEvent::GroupStart(Group {
        kind,
        label: non_empty(strip_leading_color_tokens(rest)).map(ToOwned::to_owned),
        line: line_number,
    }))
}

pub(super) fn parse_box_start(
    line_number: usize,
    line: &str,
) -> Result<Option<SequenceEvent>, PlantUmlError> {
    let Some(rest) = strip_keyword(line, "box") else {
        return Ok(None);
    };

    let rest = rest.trim();
    if rest.is_empty() {
        return Ok(Some(SequenceEvent::BoxStart(BoxBlock {
            title: None,
            color: None,
            line: line_number,
            raw: line.to_owned(),
        })));
    }

    let (title, remaining) = take_value(rest).ok_or_else(|| PlantUmlError::InvalidStatement {
        line: line_number,
        text: line.to_owned(),
    })?;
    let color = non_empty(remaining.trim()).map(ToOwned::to_owned);

    Ok(Some(SequenceEvent::BoxStart(BoxBlock {
        title: Some(title),
        color,
        line: line_number,
        raw: line.to_owned(),
    })))
}

pub(super) fn is_title_end(line: &str) -> bool {
    line.eq_ignore_ascii_case("end title") || line.eq_ignore_ascii_case("endtitle")
}
