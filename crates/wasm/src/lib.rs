mod distribution;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = renderMarkdownJson)]
pub fn render_markdown_json(input_json: &str) -> String {
    oriha_core::render_json(input_json)
}
