//! Bound recursive labels, link validation and combined formatting depth.
use std::cell::Cell;

thread_local! {
    static DEPTH: Cell<usize> = const { Cell::new(0) };
    static NESTING: Cell<usize> = const { Cell::new(0) };
}

pub(super) struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        DEPTH.with(|d| d.set(d.get() - 1));
        NESTING.with(|d| d.set(d.get() - 1));
    }
}

pub(super) fn enter() -> Guard {
    check_formatting(1);
    DEPTH.with(|d| {
        assert!(
            d.get() < 64,
            "Inline label recursion exceeds the engine safety limit (64)"
        );
        d.set(d.get() + 1);
    });
    NESTING.with(|d| d.set(d.get() + 1));
    Guard
}

pub(super) fn check_formatting(depth: usize) {
    NESTING.with(|d| {
        assert!(
            d.get().saturating_add(depth) <= 256,
            "Inline formatting exceeds the engine safety limit (256)"
        )
    });
}

pub(super) struct ParentGuard(usize);
impl Drop for ParentGuard {
    fn drop(&mut self) {
        NESTING.with(|d| d.set(d.get() - self.0));
    }
}
pub(super) fn parent_formatting(depth: usize) -> ParentGuard {
    check_formatting(depth);
    NESTING.with(|d| d.set(d.get() + depth));
    ParentGuard(depth)
}
