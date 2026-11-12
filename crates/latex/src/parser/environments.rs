//! Matrix environment row boundaries and recursive cell parsing.
use super::*;

pub(super) fn parse_environment_rows(
    body: &str,
) -> Result<Vec<Vec<Vec<MathNode>>>, LatexRenderError> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut brace_depth = 0usize;
    let chars: Vec<char> = body.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        if brace_depth == 0 && ch == '&' {
            row.push(parse_cell(&cell)?);
            cell.clear();
            index += 1;
            continue;
        }
        if brace_depth == 0 && ch == '\\' && chars.get(index + 1) == Some(&'\\') {
            row.push(parse_cell(&cell)?);
            push_row(&mut rows, &mut row);
            cell.clear();
            index += 2;
            continue;
        }

        match ch {
            '{' => brace_depth += 1,
            '}' if brace_depth > 0 => brace_depth -= 1,
            _ => {}
        }
        cell.push(ch);
        index += 1;
    }

    row.push(parse_cell(&cell)?);
    push_row(&mut rows, &mut row);

    if rows.is_empty() {
        return Err(LatexRenderError::Parse {
            message: "environment did not contain any rows".to_owned(),
            position: 0,
        });
    }

    Ok(rows)
}

pub(super) fn parse_cell(cell: &str) -> Result<Vec<MathNode>, LatexRenderError> {
    let cell = cell.trim();
    if cell.is_empty() {
        return Ok(Vec::new());
    }

    let mut parser = Parser::new(cell);
    parser.parse_nodes(false)
}

pub(super) fn push_row(rows: &mut Vec<Vec<Vec<MathNode>>>, row: &mut Vec<Vec<MathNode>>) {
    if row.iter().any(|cell| !cell.is_empty()) {
        rows.push(std::mem::take(row));
    } else {
        row.clear();
    }
}
