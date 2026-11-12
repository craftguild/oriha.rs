//! Language aliases and keyword, builtin and literal vocabularies.
use super::*;

impl Language {
    pub(super) fn from_info(info: &str) -> Self {
        match info.trim().to_ascii_lowercase().as_str() {
            "js" | "jsx" | "javascript" | "mjs" | "cjs" | "ts" | "tsx" | "typescript" => {
                Self::Javascript
            }
            "rs" | "rust" => Self::Rust,
            "py" | "python" | "py3" => Self::Python,
            "html" | "htm" | "xml" | "svg" => Self::Html,
            "json" => Self::Json,
            "jsonc" => Self::Jsonc,
            "css" | "scss" | "sass" => Self::Css,
            "sh" | "bash" | "zsh" | "shell" | "console" => Self::Shell,
            "yaml" | "yml" => Self::Yaml,
            "toml" => Self::Toml,
            "sql" | "mysql" | "postgresql" | "postgres" => Self::Sql,
            "md" | "markdown" => Self::Markdown,
            "diff" | "patch" => Self::Diff,
            "plantuml" | "puml" => Self::PlantUml,
            _ => Self::Plain,
        }
    }

    pub(super) fn spec(self) -> LanguageSpec {
        match self {
            Self::Javascript => LanguageSpec {
                keywords: JS_KEYWORDS,
                builtins: JS_BUILTINS,
                literals: JS_LITERALS,
                case_insensitive: false,
            },
            Self::Rust => LanguageSpec {
                keywords: RUST_KEYWORDS,
                builtins: RUST_BUILTINS,
                literals: RUST_LITERALS,
                case_insensitive: false,
            },
            Self::Python => LanguageSpec {
                keywords: PYTHON_KEYWORDS,
                builtins: PYTHON_BUILTINS,
                literals: PYTHON_LITERALS,
                case_insensitive: false,
            },
            Self::Shell => LanguageSpec {
                keywords: SHELL_KEYWORDS,
                builtins: SHELL_BUILTINS,
                literals: &[],
                case_insensitive: false,
            },
            Self::Sql => LanguageSpec {
                keywords: SQL_KEYWORDS,
                builtins: SQL_BUILTINS,
                literals: SQL_LITERALS,
                case_insensitive: true,
            },
            Self::Css => LanguageSpec {
                keywords: CSS_KEYWORDS,
                builtins: &[],
                literals: &[],
                case_insensitive: false,
            },
            Self::PlantUml => LanguageSpec {
                keywords: PLANTUML_KEYWORDS,
                builtins: &[],
                literals: &[],
                case_insensitive: false,
            },
            _ => LanguageSpec {
                keywords: &[],
                builtins: &[],
                literals: &[],
                case_insensitive: false,
            },
        }
    }
}

pub(super) const JS_KEYWORDS: &[&str] = &[
    "abstract",
    "as",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "finally",
    "for",
    "from",
    "function",
    "get",
    "if",
    "implements",
    "import",
    "in",
    "instanceof",
    "interface",
    "is",
    "keyof",
    "let",
    "module",
    "namespace",
    "new",
    "of",
    "private",
    "protected",
    "public",
    "readonly",
    "return",
    "satisfies",
    "set",
    "static",
    "switch",
    "throw",
    "try",
    "type",
    "typeof",
    "var",
    "void",
    "while",
    "yield",
];

pub(super) const JS_BUILTINS: &[&str] = &[
    "Array", "Boolean", "Date", "Error", "JSON", "Map", "Math", "Number", "Object", "Promise",
    "Set", "String", "console", "document", "window",
];

pub(super) const JS_LITERALS: &[&str] = &["true", "false", "null", "undefined", "NaN", "Infinity"];

pub(super) const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "type", "unsafe", "use",
    "where", "while",
];

pub(super) const RUST_BUILTINS: &[&str] = &[
    "Box", "Option", "Result", "String", "Vec", "bool", "char", "str", "usize", "u32", "u64",
    "i32", "i64",
];

pub(super) const RUST_LITERALS: &[&str] = &["true", "false", "None", "Some", "Ok", "Err"];

pub(super) const PYTHON_KEYWORDS: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif",
    "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is", "lambda",
    "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with", "yield",
];

pub(super) const PYTHON_BUILTINS: &[&str] = &[
    "dict",
    "enumerate",
    "float",
    "int",
    "len",
    "list",
    "open",
    "print",
    "range",
    "set",
    "str",
    "tuple",
];

pub(super) const PYTHON_LITERALS: &[&str] =
    &["True", "False", "None", "NotImplemented", "Ellipsis"];

pub(super) const SHELL_KEYWORDS: &[&str] = &[
    "case", "do", "done", "elif", "else", "esac", "fi", "for", "function", "if", "in", "select",
    "then", "time", "until", "while",
];

pub(super) const SHELL_BUILTINS: &[&str] = &[
    "cd", "echo", "eval", "exec", "export", "printf", "pwd", "read", "set", "shift", "source",
    "test", "unset",
];

pub(super) const SQL_KEYWORDS: &[&str] = &[
    "ADD", "ALTER", "AND", "AS", "ASC", "BETWEEN", "BY", "CASE", "CREATE", "DELETE", "DESC",
    "DISTINCT", "DROP", "ELSE", "END", "EXISTS", "FROM", "GROUP", "HAVING", "IN", "INSERT", "INTO",
    "IS", "JOIN", "LEFT", "LIKE", "LIMIT", "NOT", "ON", "OR", "ORDER", "OUTER", "RIGHT", "SELECT",
    "SET", "TABLE", "THEN", "UPDATE", "VALUES", "WHEN", "WHERE", "WITH",
];

pub(super) const SQL_BUILTINS: &[&str] = &[
    "AVG", "CAST", "COALESCE", "COUNT", "DATE", "MAX", "MIN", "SUM",
];

pub(super) const SQL_LITERALS: &[&str] = &["TRUE", "FALSE", "NULL"];

pub(super) const CSS_KEYWORDS: &[&str] = &["important", "from", "to"];

pub(super) const PLANTUML_KEYWORDS: &[&str] = &[
    "actor",
    "activate",
    "alt",
    "autonumber",
    "boundary",
    "box",
    "collections",
    "control",
    "create",
    "database",
    "deactivate",
    "destroy",
    "else",
    "end",
    "entity",
    "footer",
    "group",
    "header",
    "hide",
    "loop",
    "newpage",
    "note",
    "of",
    "opt",
    "over",
    "par",
    "participant",
    "queue",
    "ref",
    "return",
    "show",
    "title",
];
