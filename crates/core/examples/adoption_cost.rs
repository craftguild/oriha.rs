//! ADR-0001 consumer-profile pipeline probe; emits one JSON object per sample.
//! Allocator counters are live requested bytes, never RSS. Single-threaded runs.
use oriha_core::render::{RenderContext, RenderState};
use oriha_core::{
    AnnotationAttributes, AnnotationMetadata, EngineLimits, FeatureFlags, OrihaEngine, ParseStatus,
    RenderHook, RenderInput, RenderMode, TextAnnotation,
};
use serde_json::json;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering::Relaxed};
use std::sync::Arc;
use std::time::Instant;

struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);
static PEAK: AtomicIsize = AtomicIsize::new(0);
static TOTAL: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static BINS: [AtomicUsize; 25] = [const { AtomicUsize::new(0) }; 25];
fn allocated(n: usize) {
    let live = LIVE.fetch_add(n as isize, Relaxed) + n as isize;
    PEAK.fetch_max(live, Relaxed);
    TOTAL.fetch_add(n, Relaxed);
    CALLS.fetch_add(1, Relaxed);
    let bin = (usize::BITS - n.max(1).leading_zeros() - 1) as usize;
    BINS[bin.min(BINS.len() - 1)].fetch_add(n, Relaxed);
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc(layout);
        if !p.is_null() {
            allocated(layout.size());
        }
        p
    }
    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as isize, Relaxed);
        System.dealloc(p, layout);
    }
    unsafe fn realloc(&self, p: *mut u8, layout: Layout, n: usize) -> *mut u8 {
        let q = System.realloc(p, layout, n);
        if !q.is_null() {
            // Account a realloc as a new requested buffer and release the old one.
            // High water uses the post-realloc size, not allocator-internal overlap.
            LIVE.fetch_sub(layout.size() as isize, Relaxed);
            allocated(n);
        }
        q
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;

struct CountParses(Arc<AtomicUsize>);
impl RenderHook for CountParses {
    fn before_document_parse(
        &self,
        _: &mut RenderInput,
        _: &mut RenderState,
        _: &mut RenderContext,
    ) {
        self.0.fetch_add(1, Relaxed);
    }
}

struct Case {
    name: String,
    text: String,
    annotations: usize,
    reject: bool,
}
fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for n in [500, 1000, 2000, 4000, 5040] {
        for (suffix, label) in [("", "absent"), ("@tail", "mention"), ("late:", "colon")] {
            cases.push(Case {
                name: format!("{label}_{n}"),
                text: "simple words\n".repeat(n) + suffix,
                annotations: usize::MAX,
                reject: false,
            });
        }
    }
    for (label, cap) in [("zero", 0), ("low", 1), ("high", usize::MAX)] {
        cases.push(Case {
            name: format!("annotations_{label}"),
            text: "Hello @alice and @ali**ce**, 日本語.\n\n".repeat(400),
            annotations: cap,
            reject: false,
        });
    }
    for (label, text) in [
        ("unbroken", "a".repeat(65000)),
        ("many_ats", "a@".repeat(32000)),
        ("invalid_token", "{".repeat(32000) + "a@bad"),
        ("rejected_source", "x".repeat(65537)),
        ("rejected_blocks", "# @tail\n".repeat(5000)),
        ("rejected_nesting", "> ".repeat(65) + "@tail\n"),
    ] {
        cases.push(Case {
            name: label.into(),
            text,
            annotations: 0,
            reject: label.starts_with("rejected_"),
        });
    }
    cases
}

fn run(case: &Case, sample: i32) -> serde_json::Value {
    let calls = Arc::new(AtomicUsize::new(0));
    let engine = OrihaEngine::default()
        .with_limits(EngineLimits {
            max_source_bytes: 65536,
            max_blocks: 4096,
            max_nesting_depth: 64,
            max_output_bytes: 262144,
        })
        .add_hook(CountParses(calls.clone()));
    let before = LIVE.load(Relaxed);
    let total_before = TOTAL.load(Relaxed);
    let calls_before = CALLS.load(Relaxed);
    let bins_before = BINS.each_ref().map(|bin| bin.load(Relaxed));
    PEAK.store(before, Relaxed);
    let pipeline = Instant::now();
    let start = Instant::now();
    let parsed = engine.parse(RenderInput {
        markdown: case.text.clone(),
        mode: RenderMode::SemanticHtml,
        features: FeatureFlags {
            callouts: false,
            ..FeatureFlags::default()
        },
        ..Default::default()
    });
    let parse_ns = start.elapsed().as_nanos();
    let retained_parsed = LIVE.load(Relaxed) - before;
    let parse_allocated = TOTAL.load(Relaxed) - total_before;
    let parse_calls = CALLS.load(Relaxed) - calls_before;
    let parse_bins = BINS.each_ref().map(|bin| bin.load(Relaxed));
    // Compatible with the baseline API. These workloads require empty diagnostics;
    // a source rejection is a separate sample, never a full-input timing result.
    if case.reject {
        let (status, code) = match case.name.as_str() {
            "rejected_source" => (ParseStatus::Failed, "engine.max_source_bytes_exceeded"),
            "rejected_blocks" => (ParseStatus::Incomplete, "engine.max_blocks_exceeded"),
            "rejected_nesting" => (ParseStatus::Incomplete, "engine.max_nesting_depth_exceeded"),
            _ => unreachable!(),
        };
        assert_eq!(parsed.parse_status(), status);
        assert!(parsed.diagnostics().iter().any(|d| d.code == code));
        return json!({"case":case.name,"sample":sample,"outcome":format!("{status:?}"),
            "input_bytes":case.text.len(),"rejection_ns":parse_ns,
            "retained_bytes":retained_parsed,"peak_bytes":PEAK.load(Relaxed)-before});
    }
    assert!(
        parsed.diagnostics().is_empty(),
        "{}: {:?}",
        case.name,
        parsed.diagnostics()
    );
    assert_eq!(parsed.parse_status(), ParseStatus::Complete);
    assert_eq!(parsed.source(), case.text);
    let start = Instant::now();
    let regions: Vec<_> = parsed.text_regions().collect();
    let regions_ns = start.elapsed().as_nanos();
    let regions_live = LIVE.load(Relaxed) - before - retained_parsed;
    let start = Instant::now();
    // Probe lexer only: ASCII @words, with no recipient/delivery semantics.
    let mut annotations = Vec::new();
    let mut candidates = 0;
    for region in &regions {
        for (at, _) in region.text.match_indices('@') {
            let end = at
                + 1
                + region.text[at + 1..]
                    .bytes()
                    .take_while(u8::is_ascii_alphanumeric)
                    .count();
            if end == at + 1 {
                continue;
            }
            candidates += 1;
            if annotations.len() < case.annotations {
                annotations.push(TextAnnotation {
                    region: region.id,
                    range: at..end,
                    metadata: AnnotationMetadata {
                        kind: "probe".into(),
                        id: format!("occurrence-{candidates}"),
                        attributes: AnnotationAttributes::default(),
                    },
                });
            }
        }
    }
    let scan_ns = start.elapsed().as_nanos();
    let region_count = regions.len();
    drop(regions);
    let annotation_count = annotations.len();
    let start = Instant::now();
    let annotated = parsed
        .apply_annotations(&annotations)
        .expect("valid ranges");
    let annotation_ns = start.elapsed().as_nanos();
    drop(annotations);
    let retained_both = LIVE.load(Relaxed) - before;
    let start = Instant::now();
    let html = engine.render_parsed(&annotated);
    let render_ns = start.elapsed().as_nanos();
    let pipeline_ns = pipeline.elapsed().as_nanos();
    let peak = PEAK.load(Relaxed) - before;
    let start = Instant::now();
    let repeat = engine.render_parsed(&annotated);
    let repeat_render_ns = start.elapsed().as_nanos();
    assert_eq!(html.html, repeat.html);
    assert_eq!(
        serde_json::to_value(&html.diagnostics).unwrap(),
        serde_json::to_value(&repeat.diagnostics).unwrap()
    );
    assert!(!html
        .diagnostics
        .iter()
        .any(|d| d.code == "engine.max_output_bytes_exceeded"));
    assert_eq!(calls.load(Relaxed), 1, "rendering must reuse parsing");
    std::hint::black_box((&parsed, &annotated, &html, &repeat));
    json!({"case":case.name,"sample":sample,"outcome":"complete",
        "profile":"consumer-64k","input_bytes":case.text.len(),
        "annotations":annotation_count,"candidates":candidates,"regions":region_count,
        "parse_ns":parse_ns,"regions_ns":regions_ns,"scan_ns":scan_ns,
        "annotation_ns":annotation_ns,"render_ns":render_ns,
        "repeat_render_ns":repeat_render_ns,"pipeline_ns":pipeline_ns,
        "retained_parsed_bytes":retained_parsed,"regions_bytes":regions_live,
        "retained_both_bytes":retained_both,"peak_pipeline_bytes":peak,
        "parse_allocated_bytes":parse_allocated,"parse_allocation_calls":parse_calls,
        "parse_allocated_size_bins":parse_bins.iter().zip(bins_before).map(|(a,b)|a-b).collect::<Vec<_>>(),
        "parse_hook_calls":calls.load(Relaxed)})
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let samples = std::env::var("ORIHA_SAMPLES")
        .map(|n| n.parse::<i32>().expect("integer ORIHA_SAMPLES"))
        .unwrap_or(7);
    for case in cases() {
        if !args.is_empty() && !args.iter().any(|a| case.name.contains(a)) {
            continue;
        }
        let _ = run(&case, -1);
        for sample in 0..samples {
            println!("{}", run(&case, sample));
        }
    }
}
