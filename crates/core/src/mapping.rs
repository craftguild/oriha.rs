//! UTF-8 correspondence carried by parser transformations.
use std::ops::Range;

/// Origin of visible inline content, distinct from block source-map coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextProvenance {
    Literal,
    Escape,
    CharacterReference,
    Normalized,
    Generated,
}

/// A coalesced mapping from visible UTF-8 bytes to original input bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextMapping {
    pub display: Range<usize>,
    pub source: Vec<Range<usize>>,
    pub provenance: TextProvenance,
    /// Whether subranges have byte-for-byte correspondence. False for entities,
    /// escapes, expanded tabs, and other indivisible transformations.
    pub exact: bool,
}

impl TextMapping {
    pub fn literal(display: Range<usize>, source: Range<usize>) -> Self {
        assert_eq!(display.len(), source.len());
        Self {
            display,
            source: vec![source],
            provenance: TextProvenance::Literal,
            exact: true,
        }
    }

    pub fn generated(display: Range<usize>) -> Self {
        Self {
            display,
            source: Vec::new(),
            provenance: TextProvenance::Generated,
            exact: false,
        }
    }

    pub(crate) fn clipped(&self, range: Range<usize>) -> Option<Self> {
        let start = self.display.start.max(range.start);
        let end = self.display.end.min(range.end);
        if start >= end {
            return None;
        }
        let source = if self.exact && self.source.len() == 1 {
            std::iter::once(
                self.source[0].start + start - self.display.start
                    ..self.source[0].start + end - self.display.start,
            )
            .collect()
        } else {
            self.source.clone()
        };
        Some(Self {
            display: start - range.start..end - range.start,
            source,
            provenance: self.provenance,
            exact: self.exact,
        })
    }
}

pub(crate) fn push_mapping(out: &mut Vec<TextMapping>, next: TextMapping) {
    if next.display.is_empty() {
        return;
    }
    if let Some(last) = out.last_mut() {
        if last.display.end == next.display.start && last.provenance == next.provenance {
            if last.exact
                && next.exact
                && last.source.len() == 1
                && next.source.len() == 1
                && last.source[0].end == next.source[0].start
            {
                last.display.end = next.display.end;
                last.source[0].end = next.source[0].end;
                return;
            }
            if !last.exact && !next.exact && last.source == next.source {
                last.display.end = next.display.end;
                return;
            }
        }
    }
    out.push(next);
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MappedText {
    pub text: String,
    pub mapping: Vec<TextMapping>,
}

impl MappedText {
    pub fn identity(text: &str, offset: usize) -> Self {
        let mapping = if text.is_empty() {
            Vec::new()
        } else {
            vec![TextMapping::literal(
                0..text.len(),
                offset..offset + text.len(),
            )]
        };
        Self {
            text: text.to_string(),
            mapping,
        }
    }

    pub fn generated(text: &str) -> Self {
        Self {
            text: text.to_string(),
            mapping: if text.is_empty() {
                Vec::new()
            } else {
                vec![TextMapping::generated(0..text.len())]
            },
        }
    }

    pub fn slice(&self, range: Range<usize>) -> Self {
        Self {
            text: self.text[range.clone()].to_string(),
            mapping: self
                .overlapping(&range)
                .iter()
                .filter_map(|m| m.clipped(range.clone()))
                .collect(),
        }
    }

    // Internal mappings are ordered and nonoverlapping in display coordinates.
    // Restrict both clipping paths to the intersecting window: normalization
    // may leave thousands of segments in one multiline paragraph.
    fn overlapping(&self, range: &Range<usize>) -> &[TextMapping] {
        let start = self
            .mapping
            .partition_point(|m| m.display.end <= range.start);
        let end = start + self.mapping[start..].partition_point(|m| m.display.start < range.end);
        &self.mapping[start..end]
    }

    pub fn append(&mut self, other: Self) {
        let offset = self.text.len();
        self.text.push_str(&other.text);
        for mut m in other.mapping {
            m.display = m.display.start + offset..m.display.end + offset;
            push_mapping(&mut self.mapping, m);
        }
    }

    pub fn transformed(&self, range: Range<usize>, text: &str, provenance: TextProvenance) -> Self {
        let mut source: Vec<Range<usize>> = Vec::new();
        for m in self
            .overlapping(&range)
            .iter()
            .filter_map(|m| m.clipped(range.clone()))
        {
            for r in m.source {
                if let Some(last) = source.last_mut() {
                    if *last == r {
                        continue;
                    }
                    if last.end == r.start {
                        last.end = r.end;
                        continue;
                    }
                }
                source.push(r);
            }
        }
        let provenance = if source.is_empty() {
            TextProvenance::Generated
        } else {
            provenance
        };
        Self {
            text: text.to_string(),
            mapping: vec![TextMapping {
                display: 0..text.len(),
                source,
                provenance,
                exact: false,
            }],
        }
    }

    /// Compose a mapping expressed in parent-input coordinates with its parent.
    pub fn compose(text: &str, mapping: &[TextMapping], parent: &Self) -> Self {
        let mut out = Self::default();
        for m in mapping {
            let value = &text[m.display.clone()];
            if m.exact && m.source.len() == 1 {
                let mut part = parent.slice(m.source[0].clone());
                part.text = value.to_string();
                for seg in &mut part.mapping {
                    if m.provenance != TextProvenance::Literal
                        && seg.provenance != TextProvenance::Generated
                    {
                        seg.provenance = m.provenance;
                    }
                }
                out.append(part);
            } else {
                let mut combined = Self::default();
                for r in &m.source {
                    combined.append(parent.slice(r.clone()));
                }
                out.append(combined.transformed(0..combined.text.len(), value, m.provenance));
            }
        }
        if out.text != text {
            return Self::generated(text);
        }
        out
    }

    pub fn trim(&self) -> Self {
        let start = self.text.len() - self.text.trim_start().len();
        let end = self.text.trim_end().len().max(start);
        self.slice(start..end)
    }
}
