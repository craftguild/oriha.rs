use serde::Serialize;
use std::io::{self, Write};

use crate::types::{DiagnosticEngine, RenderDiagnostic, Severity};

#[derive(Debug)]
pub(crate) struct DerivationError {
    pub code: &'static str,
    pub message: &'static str,
}

impl DerivationError {
    pub fn limit() -> Self {
        Self {
            code: "blocks.output_limit",
            message: "Block output exceeds max_output_bytes",
        }
    }
    pub fn diagnostic(&self) -> RenderDiagnostic {
        RenderDiagnostic {
            severity: Severity::Error,
            code: self.code.into(),
            message: self.message.into(),
            source_range: None,
            block_id: None,
            engine: DiagnosticEngine::Core,
        }
    }
}

/// Early lower-bound accounting before copying owned payloads. A final counting
/// serializer checks the exact aggregate, including JSON escapes and envelope.
/// Neither pass allocates a second serialized document.
pub(crate) struct Budget {
    remaining: usize,
}
impl Budget {
    pub fn new(limit: usize) -> Self {
        Self { remaining: limit }
    }
    pub fn remaining(&self) -> usize {
        self.remaining
    }
    pub fn reserve(&mut self, bytes: usize) -> Result<(), DerivationError> {
        self.remaining = self
            .remaining
            .checked_sub(bytes)
            .ok_or_else(DerivationError::limit)?;
        Ok(())
    }
    pub fn copy(&mut self, value: &str) -> Result<String, DerivationError> {
        self.reserve(value.len())?;
        Ok(value.to_owned())
    }
    pub fn optional(&mut self, value: Option<&str>) -> Result<Option<String>, DerivationError> {
        value.map(|s| self.copy(s)).transpose()
    }
}

struct Counter {
    remaining: usize,
}
impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.remaining = self
            .remaining
            .checked_sub(bytes.len())
            .ok_or_else(|| io::Error::other("output limit"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) fn fits(value: &impl Serialize, limit: usize) -> bool {
    size(value, limit).is_ok()
}

pub(crate) fn size(value: &impl Serialize, limit: usize) -> Result<usize, DerivationError> {
    let mut counter = Counter { remaining: limit };
    serde_json::to_writer(&mut counter, value).map_err(|_| DerivationError::limit())?;
    Ok(limit - counter.remaining)
}
