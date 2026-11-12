use oriha_core::{render, FeatureFlags, RenderInput, RenderMode, RenderOutput};
use serde::Deserialize;
use std::collections::BTreeMap;

fn render_with_features(markdown: &str, features: FeatureFlags) -> RenderOutput {
    render(RenderInput {
        markdown: markdown.to_string(),
        mode: RenderMode::SemanticHtml,
        features,
        ..RenderInput::default()
    })
}

#[test]
fn callout_extension_fixture_examples_must_match() {
    let fixture = include_str!("fixtures/callout_extension_spec.json");
    let examples: Vec<CalloutExample> =
        serde_json::from_str(fixture).expect("valid callout extension fixture");

    for example in examples {
        let mut features = FeatureFlags::default();
        if let Some(callouts_enabled) = example.callouts_enabled {
            features.callouts = callouts_enabled;
        }
        let output = render_with_features(&example.markdown, features);

        for expected in &example.html_contains {
            assert_contains_named(&example.name, &output.html, expected);
        }
        for unexpected in &example.html_not_contains {
            assert_not_contains_named(&example.name, &output.html, unexpected);
        }
        let actual_callout_counts: BTreeMap<String, u32> = output
            .stats
            .callouts_by_kind
            .iter()
            .map(|entry| (entry.kind.clone(), entry.count))
            .collect();
        assert_eq!(
            actual_callout_counts, example.callout_counts,
            "callout stats mismatch for fixture example {:?}\n\n{}",
            example.name, output.html
        );
    }
}

#[derive(Deserialize)]
struct CalloutExample {
    name: String,
    markdown: String,
    #[serde(default)]
    callouts_enabled: Option<bool>,
    #[serde(default)]
    html_contains: Vec<String>,
    #[serde(default)]
    html_not_contains: Vec<String>,
    #[serde(default)]
    callout_counts: BTreeMap<String, u32>,
}

fn assert_contains_named(name: &str, haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "fixture {name:?} expected output to contain {needle:?}\n\n{haystack}"
    );
}

fn assert_not_contains_named(name: &str, haystack: &str, needle: &str) {
    assert!(
        !haystack.contains(needle),
        "fixture {name:?} expected output not to contain {needle:?}\n\n{haystack}"
    );
}
