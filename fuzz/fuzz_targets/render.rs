#![no_main]

use libfuzzer_sys::fuzz_target;
use oriha_core::{FeatureFlags, RenderInput};

fuzz_target!(|data: &[u8]| {
    let Ok(markdown) = std::str::from_utf8(data) else {
        return;
    };
    let output = oriha_core::render(RenderInput {
        markdown: markdown.to_string(),
        features: FeatureFlags {
            math: true,
            mermaid: true,
            plantuml: true,
            ..FeatureFlags::default()
        },
        ..RenderInput::default()
    });
    assert!(!output.diagnostics.iter().any(|d| d.code == "engine.panic"));
});
