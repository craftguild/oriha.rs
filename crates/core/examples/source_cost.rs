//! Reproducible parsing time and live allocator-byte measurements for ADR-0001.
use oriha_core::{parse, OrihaEngine, ParseStatus, RenderHook, RenderInput};
struct Noop;
impl RenderHook for Noop {}
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
    let scaling = std::env::args().any(|arg| arg == "--scaling");
    let samples = if scaling { 7 } else { 31 };
    let cases: Vec<(String, String, bool)> = if scaling {
        [500, 1000, 2000, 4000, 8000]
            .into_iter()
            .flat_map(|lines| {
                [
                    (
                        format!("multiline_{lines}"),
                        "simple words\n".repeat(lines),
                        false,
                    ),
                    (
                        format!("noop_hook_{lines}"),
                        "paragraph\n\n".repeat(lines),
                        true,
                    ),
                ]
            })
            .collect()
    } else {
        [
        ("unannotated", "# Heading\n\nPlain **formatted** text and [link](https://example.com).\n".repeat(10)),
        ("short_message", "Hello @alice, @ali**ce** and 日本語 🙂!".to_string()),
        ("large_document", "Paragraph with **formatting**, literal @alice, 日本語 and [link](https://example.com).\n\n".repeat(1300)),
        ("deep_nesting", (0..64).map(|i| format!("{}- item {i} @alice\n", "  ".repeat(i % 8))).collect::<String>()),
        ("escapes_entities", r"\@alice \\@alice &#64;alice &ngE; &amp; \*escaped\* ".repeat(1000)),
    ].into_iter().map(|(name, text)| (name.to_owned(), text, false)).collect()
    };
    println!("case,input_bytes,median_parse_ns,retained_bytes");
    for (name, text, hook) in cases {
        let engine = OrihaEngine::default().add_hook(Noop);
        let parse = || {
            let input = RenderInput {
                markdown: text.clone(),
                ..RenderInput::default()
            };
            let document = if hook {
                engine.parse(input)
            } else {
                parse(input)
            };
            assert_eq!(document.parse_status(), ParseStatus::Complete, "{name}");
            document
        };
        drop(parse());
        let before = LIVE.load(Ordering::Relaxed);
        let retained = parse();
        let bytes = LIVE.load(Ordering::Relaxed) - before;
        std::hint::black_box(&retained);
        drop(retained);
        let mut times = Vec::new();
        for _ in 0..samples {
            let start = Instant::now();
            let doc = parse();
            times.push(start.elapsed().as_nanos());
            std::hint::black_box(&doc);
            drop(doc);
        }
        times.sort_unstable();
        println!("{name},{},{},{bytes}", text.len(), times[samples / 2]);
    }
}
