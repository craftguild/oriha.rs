#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LatexAst {
    pub nodes: Vec<MathNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MathNode {
    Identifier(String),
    Number(String),
    Operator {
        text: String,
        class: MathAtomClass,
    },
    TextOperator(String),
    Space(MathSpace),
    Fraction {
        numerator: Vec<MathNode>,
        denominator: Vec<MathNode>,
    },
    Radical {
        body: Vec<MathNode>,
    },
    Script {
        base: Box<MathNode>,
        subscript: Option<Vec<MathNode>>,
        superscript: Option<Vec<MathNode>>,
    },
    Environment(MathEnvironment),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MathSpace {
    Normal,
    Thin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MathAtomClass {
    Ord,
    Op,
    Bin,
    Rel,
    Open,
    Close,
    Punct,
    Inner,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MathEnvironment {
    Aligned(Vec<Vec<Vec<MathNode>>>),
    BMatrix(Vec<Vec<Vec<MathNode>>>),
}
