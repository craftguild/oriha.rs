//! Reproducible parsing time and live allocator-byte measurements for ADR-0001.
#![allow(dead_code, unused_imports)]
use types::{EngineLimits, FeatureFlags};
#[path = "../src/types.rs"] mod types;
#[path = "../src/ir.rs"] mod ir;
#[path = "../src/source.rs"] mod source;
#[path = "../src/emoji/mod.rs"] mod emoji;
#[path = "../src/parser/mod.rs"] mod parser;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::Instant;
struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let p = System.alloc(l);
        if !p.is_null() {
            LIVE.fetch_add(l.size() as isize, Ordering::Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size() as isize, Ordering::Relaxed);
        System.dealloc(p, l);
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let q = System.realloc(p, l, n);
        if !q.is_null() {
            LIVE.fetch_add(n as isize - l.size() as isize, Ordering::Relaxed);
        }
        q
    }
}
#[global_allocator]
static ALLOC: Counting = Counting;
fn main() {
    let cases = [
        ("unannotated", "# Heading\n\nPlain **formatted** text and [link](https://example.com).\n".repeat(10)),
        ("short_message", "Hello @alice, @ali**ce** and 日本語 🙂!".to_string()),
        ("large_document", "Paragraph with **formatting**, literal @alice, 日本語 and [link](https://example.com).\n\n".repeat(1300)),
        ("deep_nesting", (0..64).map(|i| format!("{}- item {i} @alice\n", "  ".repeat(i % 8))).collect::<String>()),
        ("escapes_entities", r"\@alice \\@alice &#64;alice &ngE; &amp; \*escaped\* ".repeat(1000)),
    ];
    println!("case,input_bytes,median_parse_ns,retained_bytes");
    for (name, text) in cases {
        let features = FeatureFlags::default();
        let parse = || (text.clone(), parser::parse_document(&text, &features, EngineLimits::default()));
        drop(parse());
        let before = LIVE.load(Ordering::Relaxed);
        let retained = parse();
        let bytes = LIVE.load(Ordering::Relaxed) - before;
        std::hint::black_box(&retained);
        drop(retained);
        let mut times = Vec::new();
        for _ in 0..31 {
            let start = Instant::now();
            let doc = parse();
            times.push(start.elapsed().as_nanos());
            std::hint::black_box(&doc);
            drop(doc);
        }
        times.sort_unstable();
        println!("{name},{},{},{bytes}", text.len(), times[15]);
    }
}
