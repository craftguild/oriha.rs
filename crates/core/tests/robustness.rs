use oriha_core::{render, RenderInput, RenderMode};

#[test]
fn malformed_markdown_regression_no_panic_and_html_returned() {
    let markdown = "```rust\nfn x() {\n- [broken\n> >\n:::note\n";
    let output = render(RenderInput {
        markdown: markdown.to_string(),
        mode: RenderMode::SemanticHtml,
        ..RenderInput::default()
    });

    assert!(!output.html.is_empty());
    assert!(output.diagnostics.iter().any(
        |d| d.code.contains("unclosed") || matches!(d.severity, oriha_core::Severity::Warning)
    ));
}

#[test]
fn random_inputs_do_not_panic_and_source_ranges_are_monotonic() {
    for seed in 0_u64..200 {
        let markdown = pseudo_random_markdown(seed, 1024);
        let output = render(RenderInput {
            markdown,
            mode: RenderMode::SemanticHtml,
            ..RenderInput::default()
        });

        let mut prev = (0_u32, 0_u32, 0_u32, 0_u32);
        for block in &output.source_map {
            let now = (
                block.source_range.start_line,
                block.source_range.start_column,
                block.source_range.end_line,
                block.source_range.end_column,
            );
            assert!(
                now >= prev,
                "source range must be monotonic: prev={:?}, now={:?}",
                prev,
                now
            );
            prev = now;
        }
    }
}

#[test]
fn prepare_stage_failures_propagate_to_final_render_output() {
    let prepared = oriha_core::PreparedMarkdownLayout::with_error(
        RenderInput {
            markdown: "# original input".to_string(),
            ..RenderInput::default()
        },
        "engine.panic",
        "Markdown engine panicked: boom".to_string(),
    );

    let output = oriha_core::render_prepared_markdown(
        prepared,
        oriha_core::TextMeasurementResults::default(),
    );

    assert!(output.html.is_empty());
    assert!(
        output
            .diagnostics
            .iter()
            .any(|d| d.code == "engine.panic" && d.message.contains("boom")),
        "prepare-stage failure must surface in final diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(kind_count(&output.stats.issues_by_kind, "unknownError"), 1);
}

#[test]
fn engine_panics_are_converted_into_fallback_values() {
    let value = oriha_core::catch_engine_panic(
        || -> String { panic!("boom") },
        |message| {
            assert!(message.contains("boom"), "message was: {message}");
            "fallback".to_string()
        },
    );
    assert_eq!(value, "fallback");

    let value = oriha_core::catch_engine_panic(|| "ok".to_string(), |_| unreachable!());
    assert_eq!(value, "ok");
}

fn kind_count(entries: &[oriha_core::KindCount], kind: &str) -> u32 {
    entries
        .iter()
        .find(|entry| entry.kind == kind)
        .map(|entry| entry.count)
        .unwrap_or(0)
}

fn pseudo_random_markdown(mut seed: u64, len: usize) -> String {
    let alphabet =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_*#`[]()<>:\n \t";
    let mut out = String::with_capacity(len);
    for _ in 0..len {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = (seed % alphabet.len() as u64) as usize;
        out.push(alphabet[idx] as char);
    }
    out
}
