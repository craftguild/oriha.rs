use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oriha_core::{render, FeatureFlags, RenderInput, RenderMode};

fn bench_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("oriha_core_render");

    let cases = vec![
        ("small_note", "# Note\n\nhello\n".to_string()),
        (
            "deep_nested_list",
            (0..200)
                .map(|i| format!("{}- item {}", "  ".repeat(i % 8), i))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        ("many_code_blocks", many_code_blocks(300)),
        ("large_table", large_table(200, 10)),
        ("100kb_document", repeated_paragraph(100 * 1024)),
        ("1mb_document", repeated_paragraph(1024 * 1024)),
        ("math_and_diagrams", math_and_diagrams(50)),
    ];

    for (name, markdown) in cases {
        group.bench_with_input(
            BenchmarkId::new("render", name),
            &markdown,
            |b, markdown| {
                b.iter(|| {
                    let _ = render(RenderInput {
                        markdown: markdown.clone(),
                        mode: RenderMode::SemanticHtml,
                        features: FeatureFlags {
                            math: true,
                            mermaid: true,
                            ..FeatureFlags::default()
                        },
                        ..RenderInput::default()
                    });
                });
            },
        );
    }

    group.finish();
}

fn repeated_paragraph(target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + 128);
    let line = "This is a benchmark paragraph with **markdown** and `code`.\n\n";
    while out.len() < target_bytes {
        out.push_str(line);
    }
    out
}

fn math_and_diagrams(count: usize) -> String {
    let mut out = String::new();
    for i in 0..count {
        out.push_str(&format!(
            "Inline math $x_{{{i}}}^2 + y_{{{i}}}$ in text.\n\n"
        ));
        out.push_str("$$\\sum_{n=1}^{\\infty} \\frac{1}{n^2} = \\frac{\\pi^2}{6}$$\n\n");
        out.push_str("```mermaid\nflowchart TD\n");
        out.push_str(&format!("A{i}[start] --> B{i}{{check}}\n"));
        out.push_str(&format!("B{i} --> C{i}[done]\n"));
        out.push_str("```\n\n");
    }
    out
}

fn many_code_blocks(count: usize) -> String {
    let mut out = String::new();
    for i in 0..count {
        out.push_str("```rust\n");
        out.push_str(&format!("let value_{} = {};\n", i, i));
        out.push_str("```\n\n");
    }
    out
}

fn large_table(rows: usize, cols: usize) -> String {
    let mut out = String::new();
    let header = (0..cols)
        .map(|i| format!("h{i}"))
        .collect::<Vec<_>>()
        .join(" | ");
    out.push_str(&format!("| {header} |\n"));
    let delim = (0..cols).map(|_| "---").collect::<Vec<_>>().join(" | ");
    out.push_str(&format!("| {delim} |\n"));
    for r in 0..rows {
        let row = (0..cols)
            .map(|c| format!("{r}:{c}"))
            .collect::<Vec<_>>()
            .join(" | ");
        out.push_str(&format!("| {row} |\n"));
    }
    out
}

criterion_group!(benches, bench_render);
criterion_main!(benches);
