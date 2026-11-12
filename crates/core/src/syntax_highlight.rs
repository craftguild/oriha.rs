//! Language-neutral highlighting coordination and escaped token emission.
mod languages;
#[allow(unused_imports)]
use languages::*;
mod programming;
#[allow(unused_imports)]
use programming::*;
mod structured;
#[allow(unused_imports)]
use structured::*;
mod html;
#[allow(unused_imports)]
use html::*;
mod lexemes;
#[allow(unused_imports)]
use lexemes::*;

use crate::html::escape_text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Language {
    Plain,
    Javascript,
    Rust,
    Python,
    Html,
    Json,
    Jsonc,
    Css,
    Shell,
    Yaml,
    Toml,
    Sql,
    Markdown,
    Diff,
    PlantUml,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenKind {
    Plain,
    AttrName,
    Builtin,
    ClassName,
    Comment,
    Constant,
    Deleted,
    Doctype,
    Function,
    Inserted,
    Keyword,
    Number,
    Operator,
    Property,
    Punctuation,
    Section,
    String,
    Symbol,
    Tag,
    Url,
    Variable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    text: String,
}

#[derive(Clone, Copy)]
struct LanguageSpec {
    keywords: &'static [&'static str],
    builtins: &'static [&'static str],
    literals: &'static [&'static str],
    case_insensitive: bool,
}

#[derive(Default)]
struct HighlightState {
    block_comment_end: Option<&'static str>,
}

pub(crate) fn highlight_lines(code: &str, language: &str) -> Vec<String> {
    tokenize_lines(code, Language::from_info(language))
        .into_iter()
        .map(|tokens| emit_html(&tokens))
        .collect()
}

fn tokenize_lines(code: &str, language: Language) -> Vec<Vec<Token>> {
    if code.is_empty() {
        return vec![vec![plain_token("")]];
    }

    let mut state = HighlightState::default();
    code.split('\n')
        .map(|line| tokenize_line(line, language, &mut state))
        .collect()
}

fn tokenize_line(line: &str, language: Language, state: &mut HighlightState) -> Vec<Token> {
    match language {
        Language::Plain => vec![plain_token(line)],
        Language::Html => tokenize_html_line(line),
        Language::Json | Language::Jsonc => tokenize_json_line(line, language),
        Language::Yaml => tokenize_yaml_line(line),
        Language::Toml => tokenize_toml_line(line),
        Language::Markdown => tokenize_markdown_line(line),
        Language::Diff => tokenize_diff_line(line),
        Language::Css
        | Language::Javascript
        | Language::Rust
        | Language::Python
        | Language::Shell
        | Language::Sql
        | Language::PlantUml => tokenize_programming_line(line, language, state),
    }
}

fn emit_html(tokens: &[Token]) -> String {
    let mut out = String::new();
    for token in tokens {
        if token.kind == TokenKind::Plain {
            out.push_str(&escape_text(&token.text));
        } else {
            out.push_str("<span class=\"token ");
            out.push_str(token.kind.css_class());
            out.push_str("\">");
            out.push_str(&escape_text(&token.text));
            out.push_str("</span>");
        }
    }
    out
}

impl TokenKind {
    fn css_class(self) -> &'static str {
        match self {
            Self::Plain => "",
            Self::AttrName => "attr-name",
            Self::Builtin => "builtin",
            Self::ClassName => "class-name",
            Self::Comment => "comment",
            Self::Constant => "constant",
            Self::Deleted => "deleted",
            Self::Doctype => "doctype",
            Self::Function => "function",
            Self::Inserted => "inserted",
            Self::Keyword => "keyword",
            Self::Number => "number",
            Self::Operator => "operator",
            Self::Property => "property",
            Self::Punctuation => "punctuation",
            Self::Section => "section",
            Self::String => "string",
            Self::Symbol => "symbol",
            Self::Tag => "tag",
            Self::Url => "url",
            Self::Variable => "variable",
        }
    }
}

fn token(kind: TokenKind, text: &str) -> Token {
    Token {
        kind,
        text: text.to_string(),
    }
}

fn plain_token(text: &str) -> Token {
    token(TokenKind::Plain, text)
}

fn push_token(tokens: &mut Vec<Token>, kind: TokenKind, text: &str) {
    if !text.is_empty() {
        tokens.push(token(kind, text));
    }
}

fn push_plain(tokens: &mut Vec<Token>, text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = tokens.last_mut() {
        if last.kind == TokenKind::Plain {
            last.text.push_str(text);
            return;
        }
    }
    tokens.push(plain_token(text));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot_line(line: &str, language: &str) -> String {
        tokenize_lines(line, Language::from_info(language))[0]
            .iter()
            .map(|token| format!("{}:{:?}", token.kind.css_class(), token.text))
            .collect::<Vec<_>>()
            .join("|")
    }

    #[test]
    fn javascript_snapshot_keeps_template_literal_as_string() {
        assert_eq!(
            snapshot_line("const message = `Hello ${name}`; // greet", "javascript"),
            r#"keyword:"const"|:" message "|operator:"="|:" "|string:"`Hello ${name}`"|punctuation:";"|:" "|comment:"// greet""#
        );
    }

    #[test]
    fn rust_snapshot_distinguishes_lifetime_and_char_literal() {
        assert_eq!(
            snapshot_line("fn borrow<'a>(c: char) -> &'a str { 'x' }", "rust"),
            r#"keyword:"fn"|:" "|function:"borrow"|operator:"<"|symbol:"'"|:"a"|operator:">"|punctuation:"("|:"c"|operator:":"|:" "|builtin:"char"|punctuation:")"|:" "|operator:"-"|operator:">"|:" "|operator:"&"|symbol:"'"|:"a "|builtin:"str"|:" "|punctuation:"{"|:" "|string:"'x'"|:" "|punctuation:"}""#
        );
    }

    #[test]
    fn html_snapshot_covers_tag_attributes_and_strings() {
        assert_eq!(
            snapshot_line(r#"<div class="container">Hello</div>"#, "html"),
            r#"punctuation:"<"|tag:"div"|:" "|attr-name:"class"|operator:"="|string:"\"container\""|punctuation:">"|:"Hello"|punctuation:"<"|punctuation:"/"|tag:"div"|punctuation:">""#
        );
    }

    #[test]
    fn json_snapshot_marks_keys_values_and_literals() {
        assert_eq!(
            snapshot_line(r#"{ "enabled": true, "name": "oriha" }"#, "json"),
            r#"punctuation:"{"|:" "|property:"\"enabled\""|punctuation:":"|:" "|constant:"true"|punctuation:","|:" "|property:"\"name\""|punctuation:":"|:" "|string:"\"oriha\""|:" "|punctuation:"}""#
        );
    }

    #[test]
    fn css_snapshot_marks_properties_colors_and_comments() {
        assert_eq!(
            snapshot_line("background-color: #ff00aa; /* brand */", "css"),
            r##"property:"background-color"|operator:":"|:" "|constant:"#ff00aa"|punctuation:";"|:" "|comment:"/* brand */""##
        );
    }

    #[test]
    fn yaml_and_toml_do_not_treat_quoted_hash_as_comment() {
        assert_eq!(
            snapshot_line(r#"name: "value # not comment" # comment"#, "yaml"),
            r##"property:"name"|punctuation:":"|:" "|string:"\"value # not comment\""|:" "|comment:"# comment""##
        );
        assert_eq!(
            snapshot_line(r#"name = "value # not comment" # comment"#, "toml"),
            r##"property:"name"|:" "|operator:"="|:" "|string:"\"value # not comment\""|:" "|comment:"# comment""##
        );
    }

    #[test]
    fn diff_snapshot_marks_added_removed_and_hunk_lines() {
        assert_eq!(
            snapshot_line("@@ -1 +1 @@", "diff"),
            r#"attr-name:"@@ -1 +1 @@""#
        );
        assert_eq!(snapshot_line("+new", "diff"), r#"inserted:"+new""#);
        assert_eq!(snapshot_line("-old", "diff"), r#"deleted:"-old""#);
    }

    #[test]
    fn carries_multiline_block_comment_state() {
        let html = highlight_lines(
            "const a = 1; /* start\nstill comment\n*/ const b = 2;",
            "js",
        );
        assert!(html[0].contains(r#"<span class="token comment">/* start</span>"#));
        assert_eq!(
            html[1],
            r#"<span class="token comment">still comment</span>"#
        );
        assert!(html[2].contains(r#"<span class="token comment">*/</span>"#));
        assert!(html[2].contains(r#"<span class="token keyword">const</span>"#));
    }

    #[test]
    fn plain_text_only_escapes_html_without_spans() {
        let html = highlight_lines("<script>", "").join("\n");
        assert_eq!(html, "&lt;script&gt;");
    }
}
