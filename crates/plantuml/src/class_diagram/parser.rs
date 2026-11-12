//! Class declarations, relations, aliases and parse diagnostics.
use super::*;

pub(super) fn parse_class_diagram(source: &str) -> Result<ClassDiagram, crate::PlantUmlError> {
    let mut started = false;
    let mut ended = false;
    let mut classes = Vec::<ClassBox>::new();
    let mut relations = Vec::<Relation>::new();
    let mut current_block: Option<(usize, ClassBox)> = None;

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = strip_inline_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }

        if !started {
            if line.starts_with("@startuml") {
                started = true;
                continue;
            }
            return Err(parse_error(None, "missing @startuml"));
        }

        if ended {
            return Err(parse_error(Some(line_number), "content after @enduml"));
        }

        if let Some((start_line, block)) = &mut current_block {
            if line == "}" {
                upsert_class(&mut classes, block.clone());
                current_block = None;
                continue;
            }
            if line.contains('}') {
                let (member, _) = line.split_once('}').unwrap_or((line, ""));
                let member = member.trim();
                if !member.is_empty() {
                    block.members.push(member.to_owned());
                }
                upsert_class(&mut classes, block.clone());
                current_block = None;
                continue;
            }
            if line.starts_with("@enduml") {
                return Err(parse_error(Some(*start_line), "unclosed class block"));
            }
            block.members.push(line.to_owned());
            continue;
        }

        if line.starts_with("@enduml") {
            ended = true;
            continue;
        }
        if is_directive(line) {
            continue;
        }

        if let Some(parsed) = parse_class_declaration_header(line) {
            if parsed.opens_block {
                if parsed.inline_members.is_empty() && !parsed.closes_block {
                    current_block = Some((line_number, parsed.class));
                } else {
                    let mut class = parsed.class;
                    class.members = parsed.inline_members;
                    upsert_class(&mut classes, class);
                }
            } else {
                upsert_class(&mut classes, parsed.class);
            }
            continue;
        }

        if let Some(relation) = parse_relation_line(line_number, line) {
            ensure_class_exists(&mut classes, &relation.from);
            ensure_class_exists(&mut classes, &relation.to);
            relations.push(relation);
            continue;
        }

        return Err(parse_error(
            Some(line_number),
            format!("unsupported class statement: {line}"),
        ));
    }

    if !started {
        return Err(parse_error(None, "missing @startuml"));
    }
    if let Some((line, _)) = current_block {
        return Err(parse_error(Some(line), "unclosed class block"));
    }
    if !ended {
        return Err(parse_error(None, "missing @enduml"));
    }

    Ok(ClassDiagram { classes, relations })
}

pub(super) fn parse_class_declaration_header(line: &str) -> Option<ParsedClassHeader> {
    let (kind, rest) = if let Some(rest) = line.strip_prefix("abstract class ") {
        (ClassKind::AbstractClass, rest)
    } else if let Some(rest) = line.strip_prefix("interface ") {
        (ClassKind::Interface, rest)
    } else if let Some(rest) = line.strip_prefix("class ") {
        (ClassKind::Class, rest)
    } else {
        return None;
    };

    let (name, rest) = take_name(rest.trim())?;
    let rest = rest.trim();
    let opens_block = rest.contains('{');
    let closes_block = rest.contains('}');
    let inline_members = if opens_block && closes_block {
        rest.split_once('{')
            .and_then(|(_, after_open)| after_open.split_once('}'))
            .map(|(body, _)| {
                body.split(';')
                    .map(str::trim)
                    .filter(|member| !member.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    Some(ParsedClassHeader {
        class: ClassBox {
            name,
            kind,
            members: Vec::new(),
        },
        opens_block,
        closes_block,
        inline_members,
    })
}

pub(super) fn parse_relation_line(_line_number: usize, line: &str) -> Option<Relation> {
    for (operator, kind) in [
        ("<|--", RelationKind::Inheritance),
        ("..|>", RelationKind::Realization),
        ("-->", RelationKind::DirectedAssociation),
        ("--", RelationKind::Association),
    ] {
        if let Some((left, right)) = line.split_once(operator) {
            let left = relation_endpoint(left)?;
            let (right, label) = split_relation_label(right);
            let right = relation_endpoint(right)?;
            let (from, to) = match kind {
                RelationKind::Inheritance => (right, left),
                _ => (left, right),
            };
            return Some(Relation {
                from,
                to,
                kind,
                label,
            });
        }
    }
    None
}

pub(super) fn split_relation_label(value: &str) -> (&str, Option<String>) {
    if let Some((endpoint, label)) = value.split_once(':') {
        let label = label.trim();
        (
            endpoint,
            if label.is_empty() {
                None
            } else {
                Some(label.to_owned())
            },
        )
    } else {
        (value, None)
    }
}

pub(super) fn relation_endpoint(value: &str) -> Option<String> {
    let (name, _) = take_name(value.trim())?;
    Some(name)
}

pub(super) fn take_name(input: &str) -> Option<(String, &str)> {
    if let Some(rest) = input.strip_prefix('"') {
        let (name, rest) = rest.split_once('"')?;
        return Some((name.to_owned(), rest));
    }

    let end = input
        .char_indices()
        .find_map(|(index, ch)| {
            if ch.is_whitespace() || ch == '{' || ch == '}' || ch == ':' {
                Some(index)
            } else {
                None
            }
        })
        .unwrap_or(input.len());
    if end == 0 {
        None
    } else {
        Some((input[..end].to_owned(), &input[end..]))
    }
}

pub(super) fn ensure_class_exists(classes: &mut Vec<ClassBox>, name: &str) {
    if classes.iter().any(|class| class.name == name) {
        return;
    }
    classes.push(ClassBox {
        name: name.to_owned(),
        kind: ClassKind::Class,
        members: Vec::new(),
    });
}

pub(super) fn upsert_class(classes: &mut Vec<ClassBox>, class: ClassBox) {
    if let Some(existing) = classes
        .iter_mut()
        .find(|existing| existing.name == class.name)
    {
        *existing = class;
    } else {
        classes.push(class);
    }
}

pub(super) fn strip_inline_comment(line: &str) -> &str {
    let trimmed = line.trim_start();
    if trimmed.starts_with('\'') || trimmed.starts_with("//") {
        ""
    } else {
        line
    }
}

pub(super) fn is_directive(line: &str) -> bool {
    line.starts_with('@')
        || line.starts_with("skinparam ")
        || line.starts_with("title ")
        || line.starts_with("hide ")
        || line.starts_with("show ")
}

pub(super) fn parse_error(line: Option<usize>, message: impl Into<String>) -> crate::PlantUmlError {
    crate::PlantUmlError::Parse {
        kind: crate::PlantUmlDiagramKind::Class,
        line,
        message: message.into(),
    }
}
