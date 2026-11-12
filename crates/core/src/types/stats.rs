use super::*;

const BLOCK_KIND_ORDER: &[&str] = &[
    "document",
    "paragraph",
    "heading",
    "fencedCode",
    "indentedCode",
    "mathBlock",
    "thematicBreak",
    "htmlBlock",
    "linkReferenceDefinition",
    "blockQuote",
    "list",
    "listItem",
    "gfmTable",
    "blankLine",
    "callout",
    "foreignHtml",
];

const HEADING_LEVEL_ORDER: &[&str] = &["h1", "h2", "h3", "h4", "h5", "h6"];

const CODE_LANGUAGE_ORDER: &[&str] = &[
    "plain",
    "javascript",
    "jsx",
    "typescript",
    "tsx",
    "rust",
    "python",
    "html",
    "xml",
    "svg",
    "json",
    "jsonc",
    "css",
    "scss",
    "sass",
    "shell",
    "console",
    "yaml",
    "toml",
    "sql",
    "mysql",
    "postgresql",
    "markdown",
    "diff",
    "plantuml",
    "mermaid",
    "latex",
    "other",
];

const CALLOUT_KIND_ORDER: &[&str] = &["note", "tip", "important", "warning", "caution", "other"];
const LOCKED_FEATURE_ORDER: &[&str] = &[
    "syntaxTheme",
    "headingFont",
    "bodyFont",
    "codeFont",
    "callout",
    "mermaid",
    "plantuml",
    "math",
];
const UNSUPPORTED_KIND_ORDER: &[&str] = &["plantumlDiagram", "unknownUnsupported"];
const ISSUE_KIND_ORDER: &[&str] = &[
    "unclosedFencedCodeBlock",
    "unclosedCallout",
    "unclosedCodeBlock",
    "mathRenderFailed",
    "mermaidRenderFailed",
    "plantumlRenderFailed",
    "plantumlUnsupportedDiagram",
    "maxBlocksExceeded",
    "maxSourceBytesExceeded",
    "maxOutputBytesExceeded",
    "invalidInputJson",
    "invalidPreparedJson",
    "invalidTextMetricsJson",
    "unknownError",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RenderStats {
    #[serde(default = "render_stats_schema_version")]
    pub schema_version: u32,
    pub total_blocks: u32,
    pub blocks_by_kind: Vec<KindCount>,
    pub headings_by_level: Vec<KindCount>,
    pub code_blocks_by_language_kind: Vec<KindCount>,
    pub callouts_by_kind: Vec<KindCount>,
    pub locked_features_by_kind: Vec<KindCount>,
    pub unsupported_by_kind: Vec<KindCount>,
    pub issues_by_kind: Vec<KindCount>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderStatsBucket {
    BlocksByKind,
    HeadingsByLevel,
    CodeBlocksByLanguageKind,
    CalloutsByKind,
    LockedFeaturesByKind,
    UnsupportedByKind,
    IssuesByKind,
}

impl RenderStatsBucket {
    pub const fn kind_order(self) -> &'static [&'static str] {
        match self {
            Self::BlocksByKind => BLOCK_KIND_ORDER,
            Self::HeadingsByLevel => HEADING_LEVEL_ORDER,
            Self::CodeBlocksByLanguageKind => CODE_LANGUAGE_ORDER,
            Self::CalloutsByKind => CALLOUT_KIND_ORDER,
            Self::LockedFeaturesByKind => LOCKED_FEATURE_ORDER,
            Self::UnsupportedByKind => UNSUPPORTED_KIND_ORDER,
            Self::IssuesByKind => ISSUE_KIND_ORDER,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderStatsKindCatalog {
    pub block_kinds: &'static [&'static str],
    pub heading_levels: &'static [&'static str],
    pub code_language_kinds: &'static [&'static str],
    pub callout_kinds: &'static [&'static str],
    pub locked_feature_kinds: &'static [&'static str],
    pub unsupported_kinds: &'static [&'static str],
    pub issue_kinds: &'static [&'static str],
}

pub const fn render_stats_kind_catalog() -> RenderStatsKindCatalog {
    RenderStatsKindCatalog {
        block_kinds: BLOCK_KIND_ORDER,
        heading_levels: HEADING_LEVEL_ORDER,
        code_language_kinds: CODE_LANGUAGE_ORDER,
        callout_kinds: CALLOUT_KIND_ORDER,
        locked_feature_kinds: LOCKED_FEATURE_ORDER,
        unsupported_kinds: UNSUPPORTED_KIND_ORDER,
        issue_kinds: ISSUE_KIND_ORDER,
    }
}

pub fn is_valid_render_stats_kind(bucket: RenderStatsBucket, kind: &str) -> bool {
    bucket.kind_order().contains(&kind)
}

pub fn sort_render_stats_kind_counts(bucket: RenderStatsBucket, counts: &mut Vec<KindCount>) {
    sort_kind_counts(counts, bucket.kind_order());
}

impl Default for RenderStats {
    fn default() -> Self {
        Self {
            schema_version: render_stats_schema_version(),
            total_blocks: 0,
            blocks_by_kind: Vec::new(),
            headings_by_level: Vec::new(),
            code_blocks_by_language_kind: Vec::new(),
            callouts_by_kind: Vec::new(),
            locked_features_by_kind: Vec::new(),
            unsupported_by_kind: Vec::new(),
            issues_by_kind: Vec::new(),
        }
    }
}

impl RenderStats {
    pub fn record_block_kind(&mut self, kind: RenderBlockKind) {
        increment_kind_count(&mut self.blocks_by_kind, kind.as_str());
    }

    pub fn record_heading_level(&mut self, level: u8) {
        let kind = match level.clamp(1, 6) {
            1 => "h1",
            2 => "h2",
            3 => "h3",
            4 => "h4",
            5 => "h5",
            _ => "h6",
        };
        increment_kind_count(&mut self.headings_by_level, kind);
    }

    pub fn record_code_block_language(&mut self, language: &str) {
        increment_kind_count(
            &mut self.code_blocks_by_language_kind,
            canonical_code_language_kind(language),
        );
    }

    pub fn record_callout_kind(&mut self, kind: &str) {
        increment_kind_count(&mut self.callouts_by_kind, canonical_callout_kind(kind));
    }

    pub fn record_locked_feature(&mut self, kind: LockedFeatureKind) {
        increment_kind_count(&mut self.locked_features_by_kind, kind.as_str());
    }

    pub fn record_unsupported(&mut self, kind: UnsupportedKind) {
        increment_kind_count(&mut self.unsupported_by_kind, kind.as_str());
    }

    pub fn record_issue(&mut self, kind: RenderIssueKind) {
        increment_kind_count(&mut self.issues_by_kind, kind.as_str());
    }

    pub fn record_issue_code(&mut self, code: &str, severity: Severity) {
        if let Some(kind) = RenderIssueKind::from_diagnostic_code(code, severity) {
            self.record_issue(kind);
        }
    }

    pub fn merge(&mut self, other: &RenderStats) {
        self.total_blocks += other.total_blocks;
        merge_kind_counts(&mut self.blocks_by_kind, &other.blocks_by_kind);
        merge_kind_counts(&mut self.headings_by_level, &other.headings_by_level);
        merge_kind_counts(
            &mut self.code_blocks_by_language_kind,
            &other.code_blocks_by_language_kind,
        );
        merge_kind_counts(&mut self.callouts_by_kind, &other.callouts_by_kind);
        merge_kind_counts(
            &mut self.locked_features_by_kind,
            &other.locked_features_by_kind,
        );
        merge_kind_counts(&mut self.unsupported_by_kind, &other.unsupported_by_kind);
        merge_kind_counts(&mut self.issues_by_kind, &other.issues_by_kind);
        self.finalize();
    }

    pub fn finalize(&mut self) {
        sort_kind_counts(&mut self.blocks_by_kind, BLOCK_KIND_ORDER);
        sort_kind_counts(&mut self.headings_by_level, HEADING_LEVEL_ORDER);
        sort_kind_counts(&mut self.code_blocks_by_language_kind, CODE_LANGUAGE_ORDER);
        sort_kind_counts(&mut self.callouts_by_kind, CALLOUT_KIND_ORDER);
        sort_kind_counts(&mut self.locked_features_by_kind, LOCKED_FEATURE_ORDER);
        sort_kind_counts(&mut self.unsupported_by_kind, UNSUPPORTED_KIND_ORDER);
        sort_kind_counts(&mut self.issues_by_kind, ISSUE_KIND_ORDER);
    }
}

pub const fn render_stats_schema_version() -> u32 {
    1
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KindCount {
    pub kind: String,
    pub count: u32,
}

fn increment_kind_count(counts: &mut Vec<KindCount>, kind: &str) {
    if let Some(entry) = counts.iter_mut().find(|entry| entry.kind == kind) {
        entry.count += 1;
    } else {
        counts.push(KindCount {
            kind: kind.to_string(),
            count: 1,
        });
    }
}

fn merge_kind_counts(target: &mut Vec<KindCount>, source: &[KindCount]) {
    for entry in source {
        if entry.count == 0 {
            continue;
        }
        if let Some(target_entry) = target.iter_mut().find(|item| item.kind == entry.kind) {
            target_entry.count += entry.count;
        } else {
            target.push(entry.clone());
        }
    }
}

fn sort_kind_counts(counts: &mut Vec<KindCount>, order: &[&str]) {
    counts.retain(|entry| entry.count > 0);
    counts.sort_by(|left, right| {
        let left_index = order
            .iter()
            .position(|kind| *kind == left.kind)
            .unwrap_or(usize::MAX);
        let right_index = order
            .iter()
            .position(|kind| *kind == right.kind)
            .unwrap_or(usize::MAX);
        left_index
            .cmp(&right_index)
            .then_with(|| left.kind.cmp(&right.kind))
    });
}

fn canonical_code_language_kind(language: &str) -> &'static str {
    let normalized = language.trim().trim_start_matches('.').to_ascii_lowercase();
    match normalized.as_str() {
        "" | "plain" | "text" | "txt" => "plain",
        "js" | "javascript" | "mjs" | "cjs" => "javascript",
        "jsx" => "jsx",
        "ts" | "typescript" => "typescript",
        "tsx" => "tsx",
        "rs" | "rust" => "rust",
        "py" | "py3" | "python" | "python3" => "python",
        "html" | "htm" => "html",
        "xml" => "xml",
        "svg" => "svg",
        "json" => "json",
        "jsonc" => "jsonc",
        "css" => "css",
        "scss" => "scss",
        "sass" => "sass",
        "sh" | "bash" | "zsh" | "shell" => "shell",
        "console" | "terminal" => "console",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "sql" => "sql",
        "mysql" => "mysql",
        "postgres" | "postgresql" => "postgresql",
        "md" | "markdown" => "markdown",
        "diff" | "patch" => "diff",
        "plantuml" | "puml" => "plantuml",
        "mermaid" => "mermaid",
        "latex" | "tex" => "latex",
        _ => "other",
    }
}

fn canonical_callout_kind(kind: &str) -> &'static str {
    match kind.trim().to_ascii_lowercase().as_str() {
        "note" => "note",
        "tip" => "tip",
        "important" => "important",
        "warning" => "warning",
        "caution" => "caution",
        _ => "other",
    }
}

#[cfg(test)]
#[path = "stats_tests.rs"]
mod stats_tests;
