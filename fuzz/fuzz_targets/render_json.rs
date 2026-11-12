#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(input_json) = std::str::from_utf8(data) else {
        return;
    };
    let output = oriha_core::render_json(input_json);
    assert!(!output.is_empty());
});
