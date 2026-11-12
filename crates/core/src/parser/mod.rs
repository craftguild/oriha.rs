mod core;
mod cursor;
mod inline;
mod normalize;
mod rules;
mod traits;

pub use core::parse_document;
pub(crate) use inline::{decode_html_entities, decode_markdown, parse_inlines};

mod probe_defaults {
    #[inline]
    pub(super) fn probe_block_parse() {}
    #[inline]
    pub(super) fn probe_inline_parse() {}
}

// The explicit test imports replace these zero-cost production defaults.
#[allow(unused_imports)]
use probe_defaults::*;
#[cfg(test)]
use probe_tests::{probe_block_parse, probe_inline_parse};

#[cfg(test)]
mod probe_tests {
    pub(super) fn probe_block_parse() {
        super::probe_defaults::probe_block_parse();
        super::PARSE_CALLS.with(|calls| {
            let (blocks, inlines) = calls.get();
            calls.set((blocks + 1, inlines));
        });
    }

    pub(super) fn probe_inline_parse() {
        super::probe_defaults::probe_inline_parse();
        super::PARSE_CALLS.with(|calls| {
            let (blocks, inlines) = calls.get();
            calls.set((blocks, inlines + 1));
        });
    }
}

#[cfg(test)]
thread_local! {
    pub(crate) static PARSE_CALLS: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}
