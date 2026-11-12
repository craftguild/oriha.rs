//! cargo run -p oriha-core --example derive_blocks --release --locked
use oriha_core::{
    derive_blocks, derive_parsed_blocks, parse, BlockOutput, BlockOutputMode, BlockOutputStatus,
    RenderInput, StructuredBlockProperties,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = RenderInput {
        markdown: "# Hello **Rust**\n\nA [shared reference][r] and footnote[^n].\n\n- nested `code`\n- ![image alt](/image.png)\n\n[r]: https://example.com \"Reference\"\n[^n]: A retained note.\n".into(),
        ..RenderInput::default()
    };
    // One parse serves both projections. The input-based API is also available.
    let parsed = parse(input.clone());
    let BlockOutput::Structured(structured) =
        derive_parsed_blocks(&parsed, BlockOutputMode::Structured)
    else {
        unreachable!()
    };
    assert_eq!(structured.status, BlockOutputStatus::Complete);
    let heading = structured.blocks[0].as_ref().ok_or("missing heading")?;
    let StructuredBlockProperties::Heading { level, rich_text } = &heading.content else {
        return Err("expected typed heading".into());
    };
    assert_eq!(*level, 1);
    assert_eq!(heading.text.as_deref(), Some("Hello Rust"));
    assert_eq!(rich_text.len(), 2);
    assert_eq!(
        structured.references["r"].destination,
        "https://example.com"
    );
    assert_eq!(structured.footnotes[0].label, "n");
    println!("{}", serde_json::to_string_pretty(&structured)?);

    let BlockOutput::Rendered(rendered) = derive_parsed_blocks(&parsed, BlockOutputMode::Rendered)
    else {
        unreachable!()
    };
    assert_eq!(rendered.status, BlockOutputStatus::Complete);
    // A consumer can preserve all entries, or deliberately choose its Null policy.
    let body = rendered
        .blocks
        .iter()
        .filter_map(|block| block.html())
        .collect::<String>();
    assert!(body.contains("<strong>Rust</strong>"));
    assert!(body.contains("href=\"#fn-n\""));
    assert_eq!(heading.id, rendered.blocks[0].id);
    println!("{}", serde_json::to_string_pretty(&rendered)?);
    assert_eq!(
        serde_json::to_value(derive_blocks(input, BlockOutputMode::Rendered))?,
        serde_json::to_value(&rendered)?,
    );
    Ok(())
}
