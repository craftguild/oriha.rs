//! Diagram-kind detection and parse-error translation.
use super::*;

pub fn detect_mermaid_diagram_kind(source: &str) -> MermaidDiagramKind {
    detect_mermaid_diagram(source).kind
}

pub fn detect_mermaid_diagram(source: &str) -> MermaidDiagramDetection {
    let Some((line, first)) = first_significant_line(source) else {
        return MermaidDiagramDetection {
            kind: MermaidDiagramKind::Unknown,
            reason: MermaidDiagramDetectionReason::NoEvidence,
            line: None,
        };
    };

    let first_token = first.split_whitespace().next().unwrap_or(first.as_str());
    let kind = mermaid_diagram_kind_from_token(first_token);
    MermaidDiagramDetection {
        reason: mermaid_detection_reason(&kind),
        kind,
        line: Some(line),
    }
}

pub(super) fn mermaid_diagram_kind_from_token(first_token: &str) -> MermaidDiagramKind {
    match first_token {
        "flowchart" | "graph" => MermaidDiagramKind::Flowchart,
        "sequenceDiagram" => MermaidDiagramKind::Sequence,
        "stateDiagram" | "stateDiagram-v2" => MermaidDiagramKind::State,
        "classDiagram" => MermaidDiagramKind::Class,
        "erDiagram" => MermaidDiagramKind::EntityRelationship,
        "gantt" => MermaidDiagramKind::Gantt,
        "gitGraph" => MermaidDiagramKind::GitGraph,
        "journey" => MermaidDiagramKind::UserJourney,
        "mindmap" => MermaidDiagramKind::MindMap,
        "pie" => MermaidDiagramKind::Pie,
        "quadrantChart" => MermaidDiagramKind::Quadrant,
        "requirementDiagram" => MermaidDiagramKind::Requirement,
        "timeline" => MermaidDiagramKind::Timeline,
        "C4Context" | "C4Container" | "C4Component" | "C4Dynamic" | "C4Deployment" => {
            MermaidDiagramKind::C4
        }
        "xychart-beta" => MermaidDiagramKind::XyChart,
        "zenuml" => MermaidDiagramKind::ZenUml,
        "block-beta" => MermaidDiagramKind::Block,
        "sankey-beta" => MermaidDiagramKind::Sankey,
        _ => MermaidDiagramKind::Unknown,
    }
}

pub(super) fn mermaid_detection_reason(kind: &MermaidDiagramKind) -> MermaidDiagramDetectionReason {
    match kind {
        MermaidDiagramKind::Flowchart
        | MermaidDiagramKind::Sequence
        | MermaidDiagramKind::State => MermaidDiagramDetectionReason::DiagramDirective,
        MermaidDiagramKind::Unknown => MermaidDiagramDetectionReason::UnknownDirective,
        _ => MermaidDiagramDetectionReason::UnsupportedDirective,
    }
}

pub(super) fn first_significant_line(source: &str) -> Option<(usize, String)> {
    source.lines().enumerate().find_map(|(index, line)| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            None
        } else {
            Some((index + 1, trimmed.trim_end_matches(';').trim().to_owned()))
        }
    })
}

pub(super) fn empty_diagram_error() -> MermaidError {
    MermaidError::Parse {
        kind: MermaidDiagramKind::Unknown,
        line: None,
        message: EMPTY_DIAGRAM_MESSAGE.to_owned(),
    }
}

pub(crate) fn parse_error_to_mermaid(
    kind: MermaidDiagramKind,
    error: MermaidParseError,
) -> MermaidError {
    MermaidError::Parse {
        kind,
        line: error.line,
        message: error.message,
    }
}
