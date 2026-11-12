use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MermaidParseError {
    pub(crate) line: Option<usize>,
    pub(crate) message: String,
}

impl MermaidParseError {
    pub(crate) fn empty() -> Self {
        Self {
            line: None,
            message: "Mermaid source did not contain a diagram".to_owned(),
        }
    }

    pub(crate) fn unsupported_syntax(line: usize, syntax: &str) -> Self {
        Self {
            line: Some(line),
            message: format!("Unsupported Mermaid syntax in subset renderer: `{syntax}`"),
        }
    }
}

impl fmt::Display for MermaidParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line}: {}", self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for MermaidParseError {}

pub(crate) fn significant_lines(source: &str) -> Vec<(usize, String)> {
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

pub(crate) fn clean_identifier(raw: &str) -> Option<String> {
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

pub(crate) fn clean_label(raw: &str) -> String {
    raw.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_owned()
}

pub(crate) fn non_empty(value: &str) -> Option<String> {
    let value = clean_label(value);
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
