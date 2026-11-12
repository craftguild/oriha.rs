//! Supported diagram-kind inference and unsupported syntax detection.
use super::*;

pub fn detect_plantuml_diagram_kind(source: &str) -> PlantUmlDiagramKind {
    detect_plantuml_diagram(source).kind
}

pub fn detect_plantuml_diagram(source: &str) -> PlantUmlDiagramDetection {
    let candidate_lines = diagram_kind_candidate_lines(source);
    let start_kind = candidate_lines
        .iter()
        .find_map(|line| start_directive_diagram_kind(&line.text).map(|kind| (line.number, kind)));

    match start_kind {
        Some((_, StartDirectiveDiagramKind::GenericUml)) | None => {
            infer_uml_body_diagram_kind(&candidate_lines)
        }
        Some((line, StartDirectiveDiagramKind::Specific(kind))) => PlantUmlDiagramDetection {
            kind,
            reason: PlantUmlDiagramDetectionReason::StartDirective,
            line: Some(line),
        },
    }
}

pub(super) fn start_directive_diagram_kind(line: &str) -> Option<StartDirectiveDiagramKind> {
    let lower = line.to_ascii_lowercase();
    let directive = lower
        .split_whitespace()
        .next()
        .unwrap_or(lower.as_str())
        .trim_start_matches('@');

    Some(match directive {
        "startuml" => StartDirectiveDiagramKind::GenericUml,
        "startmindmap" => StartDirectiveDiagramKind::Specific(PlantUmlDiagramKind::MindMap),
        "startwbs" => StartDirectiveDiagramKind::Specific(PlantUmlDiagramKind::Wbs),
        "startgantt" => StartDirectiveDiagramKind::Specific(PlantUmlDiagramKind::Gantt),
        "startsalt" => StartDirectiveDiagramKind::Specific(PlantUmlDiagramKind::Salt),
        "startjson" => StartDirectiveDiagramKind::Specific(PlantUmlDiagramKind::Json),
        "startyaml" => StartDirectiveDiagramKind::Specific(PlantUmlDiagramKind::Yaml),
        "startdot" => StartDirectiveDiagramKind::Specific(PlantUmlDiagramKind::Dot),
        directive if directive.starts_with("start") => {
            StartDirectiveDiagramKind::Specific(PlantUmlDiagramKind::Unknown)
        }
        _ => return None,
    })
}

pub(super) fn diagram_kind_candidate_lines(source: &str) -> Vec<DiagramKindCandidateLine> {
    let mut lines = Vec::new();
    let mut in_block_comment = false;

    for (line_index, line) in source.lines().map(str::trim).enumerate() {
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

        if line.is_empty() || is_line_comment(line) {
            continue;
        }

        lines.push(DiagramKindCandidateLine {
            number: line_index + 1,
            text: line.to_owned(),
        });
    }

    lines
}

pub(super) fn infer_uml_body_diagram_kind(
    lines: &[DiagramKindCandidateLine],
) -> PlantUmlDiagramDetection {
    for candidate in lines {
        let line = candidate.text.as_str();

        if line.starts_with('@') {
            continue;
        }

        if is_class_strong_statement(line) {
            return PlantUmlDiagramDetection {
                kind: PlantUmlDiagramKind::Class,
                reason: PlantUmlDiagramDetectionReason::ClassStatement,
                line: Some(candidate.number),
            };
        }
        if is_sequence_statement(line) {
            return PlantUmlDiagramDetection {
                kind: PlantUmlDiagramKind::Sequence,
                reason: PlantUmlDiagramDetectionReason::SequenceStatement,
                line: Some(candidate.number),
            };
        }
        if is_class_statement(line) {
            return PlantUmlDiagramDetection {
                kind: PlantUmlDiagramKind::Class,
                reason: PlantUmlDiagramDetectionReason::ClassStatement,
                line: Some(candidate.number),
            };
        }
        if let Some(kind) = unsupported_statement_diagram_kind(line) {
            return PlantUmlDiagramDetection {
                kind,
                reason: PlantUmlDiagramDetectionReason::UnsupportedStatement,
                line: Some(candidate.number),
            };
        }
    }

    PlantUmlDiagramDetection {
        kind: PlantUmlDiagramKind::Sequence,
        reason: if lines.is_empty() {
            PlantUmlDiagramDetectionReason::NoEvidence
        } else {
            PlantUmlDiagramDetectionReason::DefaultToSequence
        },
        line: None,
    }
}

pub(super) fn is_line_comment(line: &str) -> bool {
    line.starts_with('\'') || line.starts_with("//")
}

pub(super) fn is_block_comment_start(line: &str) -> bool {
    line.starts_with("/'") || line.starts_with("/*")
}

pub(super) fn is_block_comment_end(line: &str) -> bool {
    line.ends_with("'/") || line.ends_with("*/")
}

pub(super) fn is_inline_block_comment(line: &str) -> bool {
    is_block_comment_start(line) && is_block_comment_end(line)
}

pub(super) fn is_sequence_statement(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let first = lower.split_whitespace().next().unwrap_or_default();

    matches!(
        first,
        "actor"
            | "autonumber"
            | "autoactivate"
            | "boundary"
            | "box"
            | "collections"
            | "control"
            | "database"
            | "deactivate"
            | "destroy"
            | "entity"
            | "footer"
            | "group"
            | "header"
            | "hide"
            | "hnote"
            | "participant"
            | "queue"
            | "rnote"
            | "return"
            | "skinparam"
            | "title"
            | "activate"
            | "alt"
            | "break"
            | "critical"
            | "else"
            | "end"
            | "loop"
            | "newpage"
            | "opt"
            | "par"
            | "ref"
            | "space"
    ) || lower.starts_with("note ")
        || lower.starts_with("==")
        || lower == "..."
        || lower == "|||"
        || lower.contains("->")
        || lower.contains("-->")
        || lower.contains("<-")
        || lower.contains("<--")
}

pub(super) fn is_class_strong_statement(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.starts_with("class ")
        || lower.starts_with("interface ")
        || lower.starts_with("abstract class ")
        || lower.contains("<|--")
        || lower.contains("..|>")
}

pub(super) fn unsupported_statement_diagram_kind(line: &str) -> Option<PlantUmlDiagramKind> {
    let lower = line.to_ascii_lowercase();
    let first = lower.split_whitespace().next().unwrap_or_default();

    match first.trim_matches('"') {
        "abstract" | "annotation" | "class" | "enum" | "interface" => {
            Some(PlantUmlDiagramKind::Class)
        }
        "object" => Some(PlantUmlDiagramKind::Object),
        "usecase" => Some(PlantUmlDiagramKind::UseCase),
        "state" => Some(PlantUmlDiagramKind::State),
        "component" => Some(PlantUmlDiagramKind::Component),
        "artifact" | "card" | "cloud" | "collections" | "file" | "folder" | "frame" | "node"
        | "package" | "queue" | "rectangle" | "stack" | "storage" => {
            Some(PlantUmlDiagramKind::Deployment)
        }
        "start" | "stop" | "while" | "if" | "fork" | "split" | "repeat" | "partition" => {
            Some(PlantUmlDiagramKind::Activity)
        }
        "clock" | "concise" | "robust" | "binary" => Some(PlantUmlDiagramKind::Timing),
        _ => None,
    }
}
