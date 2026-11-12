//! Scoped bounds for the existing String-based built-in renderer. Ordinary
//! document APIs have no active scope and retain their existing behavior.
use std::cell::Cell;

use super::budget::DerivationError;

thread_local! {
    static LIMIT: Cell<Option<usize>> = const { Cell::new(None) };
    static DEPTH: Cell<usize> = const { Cell::new(0) };
}

#[derive(Debug)]
struct OutputLimit;

pub(crate) fn check_size(bytes: usize) {
    if LIMIT.with(|v| v.get().is_some_and(|limit| bytes > limit)) {
        std::panic::resume_unwind(Box::new(OutputLimit));
    }
}

pub(crate) fn scoped<T>(limit: usize, op: impl FnOnce() -> T) -> Result<T, DerivationError> {
    struct Restore(Option<usize>, usize);
    impl Drop for Restore {
        fn drop(&mut self) {
            LIMIT.with(|v| v.set(self.0));
            DEPTH.with(|v| v.set(self.1));
        }
    }
    let _restore = Restore(
        LIMIT.with(|v| v.replace(Some(v.get().map_or(limit, |outer| outer.min(limit))))),
        DEPTH.with(|v| v.replace(0)),
    );
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(op)) {
        Ok(value) => Ok(value),
        Err(payload) if payload.is::<OutputLimit>() => Err(DerivationError::limit()),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

pub(crate) struct DepthGuard(bool);
impl Drop for DepthGuard {
    fn drop(&mut self) {
        if self.0 {
            DEPTH.with(|v| v.set(v.get() - 1));
        }
    }
}
pub(crate) fn enter() -> DepthGuard {
    let active = LIMIT.with(|v| v.get().is_some());
    if active {
        DEPTH.with(|v| {
            if v.get() >= 256 {
                std::panic::resume_unwind(Box::new(OutputLimit));
            }
            v.set(v.get() + 1);
        });
    }
    DepthGuard(active)
}

pub(crate) trait HtmlBuffer {
    fn push_html(&mut self, text: &str);
    fn push_html_char(&mut self, ch: char);
}
impl HtmlBuffer for String {
    fn push_html(&mut self, text: &str) {
        check_size(self.len().saturating_add(text.len()));
        self.push_str(text);
    }
    fn push_html_char(&mut self, ch: char) {
        check_size(self.len().saturating_add(ch.len_utf8()));
        self.push(ch);
    }
}
