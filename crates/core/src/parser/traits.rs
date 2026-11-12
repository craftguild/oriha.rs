use crate::ir::BlockNode;
use crate::source::{SourceCursor, SourceSpan};
use crate::types::{DiagnosticEngine, EngineLimits, FeatureFlags, RenderDiagnostic, Severity};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct ParseContext {
    pub features: FeatureFlags,
    pub limits: EngineLimits,
    pub depth: usize,
    pub omissions: Rc<RefCell<Vec<RenderDiagnostic>>>,
}

#[derive(Clone, Debug)]
pub struct ParseStart {
    #[allow(dead_code)]
    pub kind: BlockStartKind,
}

#[derive(Clone, Debug)]
pub enum BlockStartKind {
    Heading,
    FencedCode,
    MathBlock,
    BlockQuote,
    List,
    Callout,
    SetextHeading,
    ThematicBreak,
    IndentedCode,
    HtmlBlock,
    LinkReferenceDefinition,
    GfmTable,
    Paragraph,
}

#[derive(Clone, Debug)]
pub struct ParseResult {
    pub block: BlockNode,
    #[allow(dead_code)]
    pub span: SourceSpan,
}

pub trait BlockDetector {
    fn detect(&self, cursor: &SourceCursor<'_>, ctx: &ParseContext) -> Option<ParseStart>;
}

pub trait BlockParser {
    fn parse(
        &self,
        cursor: &mut SourceCursor<'_>,
        start: ParseStart,
        ctx: &mut ParseContext,
        id_gen: &mut usize,
    ) -> ParseResult;
}

pub trait BlockRule: BlockDetector + BlockParser {}

impl<T> BlockRule for T where T: BlockDetector + BlockParser {}

pub trait BlockRuleParser: BlockRule {}

impl<T> BlockRuleParser for T where T: BlockRule {}

impl ParseContext {
    pub fn child(&self) -> Option<Self> {
        if self.depth >= self.limits.max_nesting_depth {
            self.omissions.borrow_mut().push(RenderDiagnostic {
                severity: Severity::Warning,
                code: "engine.max_nesting_depth_exceeded".into(),
                message: "Maximum container nesting depth exceeded. Nested content was skipped."
                    .into(),
                source_range: None,
                block_id: None,
                engine: DiagnosticEngine::Core,
            });
            return None;
        }
        let mut child = self.clone();
        child.depth += 1;
        Some(child)
    }
}
