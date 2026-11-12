//! Producer-owned repeated consumer-profile probe. No hooks or second parser.
//! Build twice with ORIHA_MEMORY_ALLOCATIONS=1/0 for requested bytes / sampled RSS.
//! The cache and early fallback model the consumer; HTML sanitization is excluded.
use oriha_core::{
    AnnotationAttributes, AnnotationMetadata, EngineLimits, FeatureFlags, OrihaEngine, ParseStatus,
    ParsedDocument, RegionId, RenderInput, RenderMode, TextAnnotation, TextProvenance,
};
use serde::Serialize;
use serde_json::json;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicIsize, Ordering::Relaxed};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

const TRACK: bool = match option_env!("ORIHA_MEMORY_ALLOCATIONS") {
    Some(s) => !s.is_empty() && s.as_bytes()[0] == b'1',
    None => false,
};
const MAX_HTML: usize = 262_144;
const MAX_CACHE: usize = 8_388_608;
const MIN_WRAPPER: usize =
    r#"<span data-oriha-annotation-kind="" data-oriha-annotation-id=""></span>"#.len();
static LIVE: AtomicIsize = AtomicIsize::new(0);
static PEAK: AtomicIsize = AtomicIsize::new(0);
#[derive(Clone, Copy, Default)]
struct Counters {
    live: isize,
    peak: isize,
    requested: usize,
}
thread_local! { static LOCAL: Cell<Counters> = Cell::new(Counters::default()); }
fn account(added: usize, removed: usize) {
    if !TRACK {
        return;
    }
    let delta = added as isize - removed as isize;
    let live = LIVE.fetch_add(delta, Relaxed) + delta;
    PEAK.fetch_max(live, Relaxed);
    let _ = LOCAL.try_with(|cell| {
        let mut c = cell.get();
        c.live += delta;
        c.peak = c.peak.max(c.live);
        c.requested += added;
        cell.set(c);
    });
}
struct Counting;
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc(layout);
        if !p.is_null() {
            account(layout.size(), 0);
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        account(0, layout.size());
        System.dealloc(p, layout);
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let q = System.realloc(p, layout, size);
        if !q.is_null() {
            account(size, layout.size());
        }
        q
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;

#[derive(Clone, Copy, Default, Serialize)]
struct Stage {
    ns: u64,
    live_delta: isize,
    peak_extra: isize,
    requested: usize,
}
fn stage<T>(f: impl FnOnce() -> T) -> (T, Stage) {
    let before = LOCAL.with(|c| {
        let mut v = c.get();
        v.peak = v.live;
        c.set(v);
        v
    });
    let start = Instant::now();
    let value = f();
    let ns = start.elapsed().as_nanos() as u64;
    let after = LOCAL.with(Cell::get);
    (
        value,
        Stage {
            ns,
            live_delta: after.live - before.live,
            peak_extra: after.peak - before.live,
            requested: after.requested - before.requested,
        },
    )
}
#[derive(Serialize)]
struct Trace {
    round: usize,
    item: usize,
    class: usize,
    input_bytes: usize,
    candidates: usize,
    fallback: bool,
    stages: [Stage; 6],
}
struct Occurrence {
    region: RegionId,
    range: std::ops::Range<usize>,
    handle: String,
}
struct Output {
    html: String,
    fallback: bool,
}
fn escaped(body: &str) -> Output {
    let mut html = String::from("<pre class=\"markdown-fallback\">");
    for c in body.chars() {
        match c {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            _ => html.push(c),
        }
    }
    html.push_str("</pre>");
    Output {
        html,
        fallback: true,
    }
}
fn engine() -> OrihaEngine {
    OrihaEngine::default()
        .with_limits(EngineLimits {
            max_source_bytes: 65_536,
            max_blocks: 4096,
            max_nesting_depth: 64,
            max_output_bytes: MAX_HTML,
        })
        .with_features(FeatureFlags {
            gfm: true,
            syntax_highlight: true,
            callouts: false,
            raw_html: false,
            math: false,
            mermaid: false,
            plantuml: false,
        })
}
fn run(body: &str, round: usize, item: usize) -> (Output, Trace) {
    // Consumer ownership: caller body, retained original, blocking-job input,
    // then RenderInput's owned source. No parsed-document cache is involved.
    let original = body.to_owned();
    let input = original.clone();
    let engine = engine();
    let (parsed, parse) = stage(|| {
        engine.parse(RenderInput {
            markdown: input.clone(),
            mode: RenderMode::SemanticHtml,
            ..Default::default()
        })
    });
    assert_eq!(parsed.parse_status(), ParseStatus::Complete);
    assert_eq!(parsed.source(), body);
    let (regions, regions_stage) = stage(|| parsed.text_regions().collect::<Vec<_>>());
    let (found, scan) = stage(|| {
        let mut found = Vec::new();
        for region in &regions {
            for (at, _) in region.text.match_indices('@') {
                if region.text[..at]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric() || "_-/.:@%".contains(c))
                {
                    continue;
                }
                let tail = &region.text[at + 1..];
                if !tail.starts_with(|c: char| c.is_ascii_alphabetic()) {
                    continue;
                }
                let length = tail
                    .bytes()
                    .take_while(|b| b.is_ascii_alphanumeric() || *b == b'-')
                    .count();
                let index = region
                    .mappings
                    .partition_point(|m| m.mapping.display.end <= at);
                if !region.mappings.get(index).is_some_and(|m| {
                    m.mapping.display.contains(&at)
                        && matches!(
                            m.mapping.provenance,
                            TextProvenance::Literal | TextProvenance::CharacterReference
                        )
                }) {
                    continue;
                }
                found.push(Occurrence {
                    region: region.id,
                    range: at..at + 1 + length,
                    handle: tail[..length].to_ascii_lowercase(),
                });
            }
        }
        found
    });
    drop(regions);
    let count = found.len();
    let mut stages = [
        parse,
        regions_stage,
        scan,
        Stage::default(),
        Stage::default(),
        Stage::default(),
    ];
    let output = if count > MAX_HTML / MIN_WRAPPER {
        // Existing consumer raw-output lower bound, not an engine mention limit.
        escaped(body)
    } else {
        let (annotations, plan) = stage(|| {
            found
                .into_iter()
                .enumerate()
                .map(|(i, o)| TextAnnotation {
                    region: o.region,
                    range: o.range,
                    metadata: AnnotationMetadata {
                        kind: "mention".into(),
                        id: format!("mention-{i}"),
                        attributes: AnnotationAttributes {
                            classes: vec!["mention".into()],
                            data: BTreeMap::from([(
                                "mention".into(),
                                format!("probe:{}", o.handle),
                            )]),
                        },
                    },
                })
                .collect::<Vec<_>>()
        });
        stages[3] = plan;
        let (decorated, apply) = stage(|| {
            if annotations.is_empty() {
                None
            } else {
                Some(
                    parsed
                        .apply_annotations(&annotations)
                        .expect("valid annotations"),
                )
            }
        });
        stages[4] = apply;
        drop(annotations);
        let (output, render) =
            stage(|| engine.render_parsed(decorated.as_ref().unwrap_or(&parsed)));
        stages[5] = render;
        if output.html.len() > MAX_HTML
            || output
                .diagnostics
                .iter()
                .any(|d| d.code.starts_with("engine."))
        {
            escaped(body)
        } else {
            Output {
                html: output.html,
                fallback: false,
            }
        }
    };
    let trace = Trace {
        round,
        item,
        class: round % 3,
        input_bytes: body.len(),
        candidates: count,
        fallback: output.fallback,
        stages,
    };
    (output, trace)
}
fn body(round: usize, item: usize) -> String {
    match round % 3 {
        0 => format!("{round}-{item} {}@alice", "simple words\n".repeat(5040)),
        1 => format!("{round}-{item} {}", "@all ".repeat(12000)),
        _ => format!("{round}-{item} {}", "@ali**ce** ".repeat(5000)),
    }
}
struct Cache {
    rows: VecDeque<([u8; 32], Arc<Output>)>,
    bytes: usize,
}
impl Cache {
    fn new() -> Self {
        let rows = VecDeque::with_capacity(256);
        let bytes = std::mem::size_of::<Self>()
            + rows.capacity() * std::mem::size_of::<([u8; 32], Arc<Output>)>();
        Self { rows, bytes }
    }
    fn charge(output: &Output) -> usize {
        output.html.capacity() + std::mem::size_of::<Output>() + 2 * std::mem::size_of::<usize>()
    }
    fn insert(&mut self, round: usize, item: usize, output: Output) {
        let charge = Self::charge(&output);
        while !self.rows.is_empty() && (self.bytes + charge > MAX_CACHE || self.rows.len() >= 256) {
            self.bytes -= Self::charge(&self.rows.pop_front().unwrap().1);
        }
        if charge <= MAX_CACHE {
            let mut key = [0; 32];
            key[..8].copy_from_slice(&(round as u64).to_le_bytes());
            key[8..16].copy_from_slice(&(item as u64).to_le_bytes());
            self.bytes += charge;
            self.rows.push_back((key, Arc::new(output)));
        }
    }
}
fn idle(phase: &str, cache: usize) {
    println!(
        "{}",
        json!({"type":"quiescence", "phase":phase, "cache_bytes":cache,
        "live_requested":LIVE.load(Relaxed)})
    );
    std::thread::sleep(Duration::from_millis(250));
}
fn main() {
    let schedule = std::env::args().nth(1).unwrap_or("queued".into());
    let rounds = std::env::var("ORIHA_ROUNDS")
        .ok()
        .map(|s| s.parse().unwrap())
        .unwrap_or(16);
    let (workers, queued) = match schedule.as_str() {
        "serial" => (1, 1),
        "two" => (2, 2),
        "queued" => (2, 16),
        _ => panic!("serial|two|queued"),
    };
    println!(
        "{}",
        json!({"type":"profile", "schedule":schedule, "workers":workers,
        "queued":queued, "rounds":rounds, "allocations":TRACK,
        "stages":["parse","regions","candidate_scan","annotation_plan","annotation_apply","render"],
        "sanitizer":false, "cache_cap":MAX_CACHE,
        "inline_node_size":std::mem::size_of::<oriha_core::ir::InlineNode>(),
        "mapping_size":std::mem::size_of::<oriha_core::TextMapping>()})
    );
    let (tx, rx) = mpsc::channel::<(String, usize, usize)>();
    let rx = Arc::new(Mutex::new(rx));
    let (results, done) = mpsc::channel();
    let mut threads = Vec::new();
    for _ in 0..workers {
        let rx = rx.clone();
        let results = results.clone();
        threads.push(std::thread::spawn(move || loop {
            let job = rx.lock().unwrap().recv();
            let Ok((body, round, item)) = job else {
                break;
            };
            results.send(run(&body, round, item)).unwrap();
        }));
    }
    drop(results);
    let mut cache = Cache::new();
    idle("start", cache.bytes);
    for round in 0..rounds {
        let before = LIVE.load(Relaxed);
        PEAK.store(before, Relaxed);
        let start = Instant::now();
        let mut traces = Vec::with_capacity(16);
        for start in (0..16).step_by(queued) {
            for item in start..(start + queued).min(16) {
                tx.send((body(round, item), round, item)).unwrap();
            }
            for _ in start..(start + queued).min(16) {
                let (output, trace) = done.recv().unwrap();
                cache.insert(round, trace.item, output);
                traces.push(trace);
            }
        }
        let ns = start.elapsed().as_nanos();
        println!(
            "{}",
            json!({"type":"batch", "round":round,"class":round%3,"ns":ns,
            "live_before":before,"live_after":LIVE.load(Relaxed),"peak_requested":PEAK.load(Relaxed),
            "cache_bytes":cache.bytes,"cache_items":cache.rows.len()})
        );
        for trace in traces {
            println!("{}", json!({"type":"body","trace":trace}));
        }
        assert!(cache.bytes <= MAX_CACHE);
    }
    idle("cache_retained_workers_idle", cache.bytes);
    drop(cache);
    idle("cache_dropped_workers_idle", 0);
    drop(tx);
    for thread in threads {
        thread.join().unwrap();
    }
    drop(done);
    drop(rx);
    idle("workers_joined", 0);
    // Keep ParsedDocument named in this harness's type-level dependencies.
    let _: Option<ParsedDocument> = None;
}
