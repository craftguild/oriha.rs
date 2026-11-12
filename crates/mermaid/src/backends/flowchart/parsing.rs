//! Flowchart declarations, node grammar and edge parsing.
use super::*;

pub(super) fn parse_flowchart_diagram(
    source: &str,
) -> Result<FlowchartDiagram, FlowchartParseError> {
    let lines = significant_lines(source);
    if lines.is_empty() {
        return Err(FlowchartParseError::empty());
    }
    parse_flowchart(&lines)
}

pub(super) fn significant_lines(source: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("%%") {
                None
            } else {
                Some((index + 1, trimmed.trim_end_matches(';').trim().to_owned()))
            }
        })
        .collect()
}

pub(super) fn parse_flowchart(
    lines: &[(usize, String)],
) -> Result<FlowchartDiagram, FlowchartParseError> {
    let (line_no, header) = &lines[0];
    let mut header_parts = header.split_whitespace();
    let _keyword = header_parts.next();
    let direction = match header_parts.next().unwrap_or("TD") {
        "TD" | "TB" => FlowDirection::TopDown,
        "LR" => FlowDirection::LeftRight,
        other => return Err(FlowchartParseError::unsupported_diagram(other, *line_no)),
    };
    if header_parts.next().is_some() {
        return Err(FlowchartParseError::unsupported_syntax(*line_no, header));
    }

    let mut nodes = Vec::new();
    let mut node_index = BTreeMap::new();
    let mut edges = Vec::new();

    for (line_no, line) in lines.iter().skip(1) {
        reject_flowchart_keywords(*line_no, line)?;
        if let Some(edge) = parse_flow_edge(line) {
            let from = parse_flow_node_token(&edge.from)
                .ok_or_else(|| FlowchartParseError::unsupported_syntax(*line_no, line))?;
            let to = parse_flow_node_token(&edge.to)
                .ok_or_else(|| FlowchartParseError::unsupported_syntax(*line_no, line))?;
            upsert_flow_node(&mut nodes, &mut node_index, from);
            upsert_flow_node(&mut nodes, &mut node_index, to);
            edges.push(FlowEdge {
                from: parse_flow_node_token(&edge.from)
                    .expect("from node already parsed")
                    .id,
                to: parse_flow_node_token(&edge.to)
                    .expect("to node already parsed")
                    .id,
                label: edge.label,
            });
            continue;
        }

        let node = parse_flow_node_token(line)
            .ok_or_else(|| FlowchartParseError::unsupported_syntax(*line_no, line))?;
        upsert_flow_node(&mut nodes, &mut node_index, node);
    }

    if nodes.is_empty() {
        return Err(FlowchartParseError::empty());
    }

    Ok(FlowchartDiagram {
        direction,
        nodes,
        edges,
    })
}

pub(super) fn reject_flowchart_keywords(
    line_no: usize,
    line: &str,
) -> Result<(), FlowchartParseError> {
    let first = line.split_whitespace().next().unwrap_or_default();
    if matches!(
        first,
        "subgraph" | "end" | "classDef" | "class" | "style" | "linkStyle" | "click"
    ) {
        return Err(FlowchartParseError::unsupported_syntax(line_no, line));
    }
    Ok(())
}

pub(super) fn parse_flow_edge(line: &str) -> Option<ParsedFlowEdge> {
    let arrow_index = line.find("-->")?;
    let before_arrow = line[..arrow_index].trim();
    let after_arrow = line[arrow_index + 3..].trim();
    let (from, before_label) = if let Some(label_index) = before_arrow.rfind("--") {
        (
            before_arrow[..label_index].trim(),
            non_empty(before_arrow[label_index + 2..].trim()),
        )
    } else {
        (before_arrow, None)
    };
    let (after_label, to) = parse_bar_label(after_arrow);
    let label = before_label.or(after_label);

    if from.is_empty() || to.trim().is_empty() {
        return None;
    }

    Some(ParsedFlowEdge {
        from: from.to_owned(),
        to: to.trim().to_owned(),
        label,
    })
}

pub(super) fn parse_bar_label(input: &str) -> (Option<String>, &str) {
    let input = input.trim();
    if let Some(rest) = input.strip_prefix('|') {
        if let Some(end) = rest.find('|') {
            let label = non_empty(rest[..end].trim());
            return (label, rest[end + 1..].trim());
        }
    }
    (None, input)
}

pub(super) fn parse_flow_node_token(token: &str) -> Option<ParsedFlowNode> {
    let token = token.trim();
    if token.is_empty() || token.contains("-->") {
        return None;
    }

    if let Some((id, label)) = parse_enclosed(token, "([", "])") {
        return Some(ParsedFlowNode {
            id,
            label,
            shape: FlowNodeShape::Stadium,
            explicit: true,
        });
    }
    if let Some((id, label)) = parse_enclosed(token, "((", "))") {
        return Some(ParsedFlowNode {
            id,
            label,
            shape: FlowNodeShape::Round,
            explicit: true,
        });
    }
    if let Some((id, label)) = parse_enclosed(token, "[", "]") {
        return Some(ParsedFlowNode {
            id,
            label,
            shape: FlowNodeShape::Rectangle,
            explicit: true,
        });
    }
    if let Some((id, label)) = parse_enclosed(token, "{", "}") {
        return Some(ParsedFlowNode {
            id,
            label,
            shape: FlowNodeShape::Diamond,
            explicit: true,
        });
    }
    if let Some((id, label)) = parse_enclosed(token, "(", ")") {
        return Some(ParsedFlowNode {
            id,
            label,
            shape: FlowNodeShape::Round,
            explicit: true,
        });
    }

    let id = clean_identifier(token)?;
    Some(ParsedFlowNode {
        label: id.clone(),
        id,
        shape: FlowNodeShape::Rectangle,
        explicit: false,
    })
}

pub(super) fn parse_enclosed(token: &str, open: &str, close: &str) -> Option<(String, String)> {
    let start = token.find(open)?;
    if !token.ends_with(close) {
        return None;
    }
    let id = clean_identifier(token[..start].trim())?;
    let label_start = start + open.len();
    let label_end = token.len().checked_sub(close.len())?;
    if label_start > label_end {
        return None;
    }
    let label = clean_label(&token[label_start..label_end]);
    Some((id, label))
}

pub(super) fn upsert_flow_node(
    nodes: &mut Vec<FlowNode>,
    index: &mut BTreeMap<String, usize>,
    parsed: ParsedFlowNode,
) {
    if let Some(existing) = index.get(&parsed.id).copied() {
        if parsed.explicit {
            nodes[existing].label = parsed.label;
            nodes[existing].shape = parsed.shape;
        }
        return;
    }
    index.insert(parsed.id.clone(), nodes.len());
    nodes.push(FlowNode {
        id: parsed.id,
        label: parsed.label,
        shape: parsed.shape,
    });
}

pub(super) fn clean_identifier(raw: &str) -> Option<String> {
    let cleaned = clean_label(raw);
    if cleaned.is_empty()
        || cleaned.contains(' ')
        || cleaned.contains('\t')
        || cleaned.contains('[')
        || cleaned.contains(']')
        || cleaned.contains('{')
        || cleaned.contains('}')
        || cleaned.contains('(')
        || cleaned.contains(')')
    {
        None
    } else {
        Some(cleaned)
    }
}

pub(super) fn clean_label(raw: &str) -> String {
    raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_owned()
}

pub(super) fn non_empty(value: &str) -> Option<String> {
    let value = clean_label(value);
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
