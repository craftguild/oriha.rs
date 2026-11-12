//! LaTeX command, atom, group and script parsing.
mod environments;
#[allow(unused_imports)]
use environments::*;

use crate::ast::{LatexAst, MathAtomClass, MathEnvironment, MathNode, MathSpace};
use crate::LatexRenderError;

pub(crate) fn parse_latex(source: &str) -> Result<LatexAst, LatexRenderError> {
    let source = strip_math_delimiters(source.trim());
    if source.trim().is_empty() {
        return Err(LatexRenderError::EmptySource);
    }

    let mut parser = Parser::new(source);
    let nodes = parser.parse_nodes(false)?;
    if parser.peek().is_some() {
        return parser.error("unexpected trailing input");
    }

    Ok(LatexAst { nodes })
}

fn strip_math_delimiters(source: &str) -> &str {
    if source.len() >= 4 && source.starts_with("$$") && source.ends_with("$$") {
        &source[2..source.len() - 2]
    } else if source.len() >= 2 && source.starts_with('$') && source.ends_with('$') {
        &source[1..source.len() - 1]
    } else if source.len() >= 4 && source.starts_with("\\[") && source.ends_with("\\]") {
        &source[2..source.len() - 2]
    } else {
        source
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
        }
    }

    fn parse_nodes(&mut self, stop_at_group_end: bool) -> Result<Vec<MathNode>, LatexRenderError> {
        let mut nodes = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                '}' if stop_at_group_end => break,
                '}' => return self.error("unexpected closing brace"),
                '{' => {
                    self.bump();
                    let mut group = self.parse_nodes(true)?;
                    self.expect('}')?;
                    nodes.append(&mut group);
                }
                '^' => {
                    self.bump();
                    let superscript = self.parse_script_argument()?;
                    self.attach_script(&mut nodes, None, Some(superscript))?;
                }
                '_' => {
                    self.bump();
                    let subscript = self.parse_script_argument()?;
                    self.attach_script(&mut nodes, Some(subscript), None)?;
                }
                '\\' => {
                    let node = self.parse_command()?;
                    nodes.push(node);
                }
                '&' => return self.error("alignment marker is only supported inside environments"),
                ch if ch.is_whitespace() => {
                    self.consume_whitespace();
                    push_space(&mut nodes, MathSpace::Normal);
                }
                ch if ch.is_ascii_digit() => {
                    let number = self.read_number();
                    nodes.push(MathNode::Number(number));
                }
                ch if ch.is_ascii_alphabetic() => {
                    self.bump();
                    nodes.push(MathNode::Identifier(ch.to_string()));
                }
                _ => nodes.push(self.parse_single_character_atom()?),
            }
        }

        Ok(nodes)
    }

    fn parse_command(&mut self) -> Result<MathNode, LatexRenderError> {
        self.expect('\\')?;
        let command = self.read_command_name()?;

        match command.as_str() {
            "," => Ok(MathNode::Space(MathSpace::Thin)),
            "\\" => self.error("row separator is only supported inside environments"),
            "frac" => Ok(MathNode::Fraction {
                numerator: self.parse_required_group_nodes()?,
                denominator: self.parse_required_group_nodes()?,
            }),
            "sqrt" => Ok(MathNode::Radical {
                body: self.parse_required_group_nodes()?,
            }),
            "begin" => self.parse_environment(),
            "end" => self.error("unexpected environment end"),
            command => {
                if let Some(node) = command_node(command) {
                    Ok(node)
                } else {
                    Err(LatexRenderError::UnsupportedCommand(command.to_owned()))
                }
            }
        }
    }

    fn parse_environment(&mut self) -> Result<MathNode, LatexRenderError> {
        let name = self.read_required_braced_text()?;
        if name != "aligned" && name != "bmatrix" {
            return Err(LatexRenderError::UnsupportedEnvironment(name));
        }

        let body = self.read_environment_body(&name)?;
        let rows = parse_environment_rows(&body)?;
        let environment = match name.as_str() {
            "aligned" => MathEnvironment::Aligned(rows),
            "bmatrix" => MathEnvironment::BMatrix(rows),
            _ => unreachable!("unsupported environments returned earlier"),
        };

        Ok(MathNode::Environment(environment))
    }

    fn parse_script_argument(&mut self) -> Result<Vec<MathNode>, LatexRenderError> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_required_group_nodes(),
            Some('\\') => Ok(vec![self.parse_command()?]),
            Some('^' | '_' | '}' | '&') | None => self.error("expected script argument"),
            Some(ch) if ch.is_ascii_digit() => Ok(vec![MathNode::Number(self.read_number())]),
            Some(ch) if ch.is_ascii_alphabetic() => {
                self.bump();
                Ok(vec![MathNode::Identifier(ch.to_string())])
            }
            Some(ch) => {
                let node = self.parse_single_character_atom_for(ch)?;
                Ok(vec![node])
            }
        }
    }

    fn parse_required_group_nodes(&mut self) -> Result<Vec<MathNode>, LatexRenderError> {
        self.skip_whitespace();
        self.expect('{')?;
        let nodes = self.parse_nodes(true)?;
        self.expect('}')?;
        Ok(nodes)
    }

    fn attach_script(
        &self,
        nodes: &mut Vec<MathNode>,
        subscript: Option<Vec<MathNode>>,
        superscript: Option<Vec<MathNode>>,
    ) -> Result<(), LatexRenderError> {
        let Some(previous) = nodes.pop() else {
            return Err(LatexRenderError::Parse {
                message: "script does not have a base".to_owned(),
                position: self.pos,
            });
        };

        let next = match previous {
            MathNode::Script {
                base,
                subscript: existing_subscript,
                superscript: existing_superscript,
            } => {
                if subscript.is_some() && existing_subscript.is_some() {
                    return Err(LatexRenderError::Parse {
                        message: "duplicate subscript".to_owned(),
                        position: self.pos,
                    });
                }
                if superscript.is_some() && existing_superscript.is_some() {
                    return Err(LatexRenderError::Parse {
                        message: "duplicate superscript".to_owned(),
                        position: self.pos,
                    });
                }
                MathNode::Script {
                    base,
                    subscript: subscript.or(existing_subscript),
                    superscript: superscript.or(existing_superscript),
                }
            }
            base => MathNode::Script {
                base: Box::new(base),
                subscript,
                superscript,
            },
        };

        nodes.push(next);
        Ok(())
    }

    fn read_command_name(&mut self) -> Result<String, LatexRenderError> {
        let Some(ch) = self.peek() else {
            return self.error("expected command name");
        };

        if ch.is_ascii_alphabetic() {
            let start = self.pos;
            while matches!(self.peek(), Some(ch) if ch.is_ascii_alphabetic()) {
                self.bump();
            }
            Ok(self.chars[start..self.pos].iter().collect())
        } else {
            self.bump();
            Ok(ch.to_string())
        }
    }

    fn read_number(&mut self) -> String {
        let start = self.pos;
        while matches!(self.peek(), Some(ch) if ch.is_ascii_digit()) {
            self.bump();
        }
        if self.peek() == Some('.')
            && matches!(self.chars.get(self.pos + 1), Some(ch) if ch.is_ascii_digit())
        {
            self.bump();
            while matches!(self.peek(), Some(ch) if ch.is_ascii_digit()) {
                self.bump();
            }
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn parse_single_character_atom(&mut self) -> Result<MathNode, LatexRenderError> {
        let Some(ch) = self.peek() else {
            return self.error("expected math atom");
        };
        self.parse_single_character_atom_for(ch)
    }

    fn parse_single_character_atom_for(&mut self, ch: char) -> Result<MathNode, LatexRenderError> {
        self.bump();
        if let Some(class) = character_operator_class(ch) {
            Ok(MathNode::Operator {
                text: ch.to_string(),
                class,
            })
        } else {
            Ok(MathNode::Identifier(ch.to_string()))
        }
    }

    fn read_required_braced_text(&mut self) -> Result<String, LatexRenderError> {
        self.skip_whitespace();
        self.expect('{')?;
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch == '}' {
                let text: String = self.chars[start..self.pos].iter().collect();
                self.bump();
                return Ok(text);
            }
            if ch == '{' {
                return self.error("nested braces are not supported in environment names");
            }
            self.bump();
        }
        self.error("unterminated braced text")
    }

    fn read_environment_body(&mut self, name: &str) -> Result<String, LatexRenderError> {
        let end = format!("\\end{{{name}}}");
        let start = self.pos;
        let mut brace_depth = 0usize;

        while self.pos < self.chars.len() {
            if brace_depth == 0 && self.starts_with(&end) {
                let body = self.chars[start..self.pos].iter().collect();
                self.pos += end.chars().count();
                return Ok(body);
            }

            match self.bump() {
                Some('{') => brace_depth += 1,
                Some('}') if brace_depth > 0 => brace_depth -= 1,
                _ => {}
            }
        }

        Err(LatexRenderError::Parse {
            message: format!("missing \\end{{{name}}}"),
            position: self.pos,
        })
    }

    fn starts_with(&self, pattern: &str) -> bool {
        for (index, expected) in (self.pos..).zip(pattern.chars()) {
            if self.chars.get(index) != Some(&expected) {
                return false;
            }
        }
        true
    }

    fn consume_whitespace(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
            self.bump();
        }
    }

    fn skip_whitespace(&mut self) {
        self.consume_whitespace();
    }

    fn expect(&mut self, expected: char) -> Result<(), LatexRenderError> {
        match self.bump() {
            Some(ch) if ch == expected => Ok(()),
            _ => self.error(format!("expected `{expected}`")),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn error<T>(&self, message: impl Into<String>) -> Result<T, LatexRenderError> {
        Err(LatexRenderError::Parse {
            message: message.into(),
            position: self.pos,
        })
    }
}

fn push_space(nodes: &mut Vec<MathNode>, space: MathSpace) {
    if matches!(nodes.last(), Some(MathNode::Space(_))) {
        return;
    }
    nodes.push(MathNode::Space(space));
}

fn command_node(command: &str) -> Option<MathNode> {
    match command {
        "alpha" => Some(MathNode::Identifier("\u{03b1}".to_owned())),
        "beta" => Some(MathNode::Identifier("\u{03b2}".to_owned())),
        "gamma" => Some(MathNode::Identifier("\u{03b3}".to_owned())),
        "delta" => Some(MathNode::Identifier("\u{03b4}".to_owned())),
        "pi" => Some(MathNode::Identifier("\u{03c0}".to_owned())),
        "theta" => Some(MathNode::Identifier("\u{03b8}".to_owned())),
        "lambda" => Some(MathNode::Identifier("\u{03bb}".to_owned())),
        "mu" => Some(MathNode::Identifier("\u{03bc}".to_owned())),
        "cos" => Some(MathNode::TextOperator("cos".to_owned())),
        "sin" => Some(MathNode::TextOperator("sin".to_owned())),
        "tan" => Some(MathNode::TextOperator("tan".to_owned())),
        "sum" => Some(MathNode::Operator {
            text: "\u{2211}".to_owned(),
            class: MathAtomClass::Op,
        }),
        "int" => Some(MathNode::Operator {
            text: "\u{222b}".to_owned(),
            class: MathAtomClass::Op,
        }),
        "cdot" => Some(MathNode::Operator {
            text: "\u{22c5}".to_owned(),
            class: MathAtomClass::Bin,
        }),
        "times" => Some(MathNode::Operator {
            text: "\u{00d7}".to_owned(),
            class: MathAtomClass::Bin,
        }),
        "le" => Some(MathNode::Operator {
            text: "\u{2264}".to_owned(),
            class: MathAtomClass::Rel,
        }),
        "ge" => Some(MathNode::Operator {
            text: "\u{2265}".to_owned(),
            class: MathAtomClass::Rel,
        }),
        "neq" => Some(MathNode::Operator {
            text: "\u{2260}".to_owned(),
            class: MathAtomClass::Rel,
        }),
        _ => None,
    }
}

fn character_operator_class(ch: char) -> Option<MathAtomClass> {
    match ch {
        '+' | '-' | '*' | '/' => Some(MathAtomClass::Bin),
        '=' | '<' | '>' => Some(MathAtomClass::Rel),
        '(' | '[' => Some(MathAtomClass::Open),
        ')' | ']' => Some(MathAtomClass::Close),
        ',' | '.' | ';' | ':' => Some(MathAtomClass::Punct),
        _ => None,
    }
}
