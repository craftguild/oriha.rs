use crate::ast::{LatexAst, MathAtomClass, MathEnvironment, MathNode, MathSpace};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LatexIr {
    pub nodes: Vec<IrNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IrNode {
    Identifier(String),
    Number(String),
    Operator {
        text: String,
        class: MathAtomClass,
    },
    TextOperator(String),
    Space(Spacing),
    Fraction {
        numerator: Vec<IrNode>,
        denominator: Vec<IrNode>,
    },
    Radical {
        body: Vec<IrNode>,
    },
    Script {
        base: Box<IrNode>,
        subscript: Option<Vec<IrNode>>,
        superscript: Option<Vec<IrNode>>,
    },
    Grid {
        kind: GridKind,
        rows: Vec<Vec<Vec<IrNode>>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Spacing {
    pub em: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GridKind {
    Aligned,
    BMatrix,
}

pub(crate) fn lower_latex(ast: &LatexAst) -> LatexIr {
    LatexIr {
        nodes: lower_nodes(&ast.nodes),
    }
}

fn lower_nodes(nodes: &[MathNode]) -> Vec<IrNode> {
    nodes.iter().map(lower_node).collect()
}

fn lower_node(node: &MathNode) -> IrNode {
    match node {
        MathNode::Identifier(identifier) => IrNode::Identifier(identifier.clone()),
        MathNode::Number(number) => IrNode::Number(number.clone()),
        MathNode::Operator { text, class } => IrNode::Operator {
            text: text.clone(),
            class: *class,
        },
        MathNode::TextOperator(operator) => IrNode::TextOperator(operator.clone()),
        MathNode::Space(space) => IrNode::Space(match space {
            MathSpace::Normal => Spacing { em: 0.28 },
            MathSpace::Thin => Spacing { em: 0.16 },
        }),
        MathNode::Fraction {
            numerator,
            denominator,
        } => IrNode::Fraction {
            numerator: lower_nodes(numerator),
            denominator: lower_nodes(denominator),
        },
        MathNode::Radical { body } => IrNode::Radical {
            body: lower_nodes(body),
        },
        MathNode::Script {
            base,
            subscript,
            superscript,
        } => IrNode::Script {
            base: Box::new(lower_node(base)),
            subscript: subscript.as_ref().map(|nodes| lower_nodes(nodes)),
            superscript: superscript.as_ref().map(|nodes| lower_nodes(nodes)),
        },
        MathNode::Environment(environment) => match environment {
            MathEnvironment::Aligned(rows) => IrNode::Grid {
                kind: GridKind::Aligned,
                rows: lower_grid(rows),
            },
            MathEnvironment::BMatrix(rows) => IrNode::Grid {
                kind: GridKind::BMatrix,
                rows: lower_grid(rows),
            },
        },
    }
}

fn lower_grid(rows: &[Vec<Vec<MathNode>>]) -> Vec<Vec<Vec<IrNode>>> {
    rows.iter()
        .map(|row| row.iter().map(|cell| lower_nodes(cell)).collect())
        .collect()
}

impl IrNode {
    pub(crate) fn atom_class(&self) -> Option<MathAtomClass> {
        match self {
            Self::Identifier(_) | Self::Number(_) => Some(MathAtomClass::Ord),
            Self::Operator { class, .. } => Some(*class),
            Self::TextOperator(_) => Some(MathAtomClass::Op),
            Self::Space(_) => None,
            Self::Fraction { .. } | Self::Radical { .. } | Self::Grid { .. } => {
                Some(MathAtomClass::Inner)
            }
            Self::Script { base, .. } => base.atom_class().or(Some(MathAtomClass::Ord)),
        }
    }
}
