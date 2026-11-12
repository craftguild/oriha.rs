//! cargo run -p oriha-core --example responsive_images --release --locked
use std::collections::BTreeMap;

use oriha_core::render::RenderContext;
use oriha_core::{
    BlockOutput, BlockOutputMode, BlockOutputStatus, ImageCandidate, ImageLayout,
    ImagePresentation, ImageRenderRequest, OrihaEngine, RenderHook, RenderInput, RenderMode,
};

struct PreparedImages(BTreeMap<String, ImagePresentation>);
impl RenderHook for PreparedImages {
    fn image_presentation<'a>(
        &'a self,
        image: &ImageRenderRequest<'_>,
        _: &RenderContext<'_>,
    ) -> Option<&'a ImagePresentation> {
        self.0.get(image.destination)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fixed in-memory application data, prepared before any render call.
    let presentation = ImagePresentation::new(
        "/prepared/photo.png",
        vec![
            ImageCandidate::new("/prepared/photo-320.png", 320)?,
            ImageCandidate::new("/prepared/photo-960.png", 960)?,
        ],
        Some("(max-width: 40em) 100vw, 40em".into()),
        Some((960, 640)),
        ImageLayout::Responsive,
    )?;
    let images = BTreeMap::from([("/photo.png".into(), presentation)]);
    let engine = OrihaEngine::default().add_hook(PreparedImages(images));
    let input = RenderInput {
        markdown: "![Photo](/photo.png \"A caption\")\n\nInline ![Photo](/photo.png)".into(),
        mode: RenderMode::SemanticHtml,
        ..Default::default()
    };
    let parsed = engine.parse(input.clone());
    let document = engine.render_parsed(&parsed);
    assert!(document.diagnostics.is_empty());
    assert_eq!(document.html, engine.render(input.clone()).html);
    let output = engine.derive_parsed_blocks(&parsed, BlockOutputMode::Rendered);
    assert_eq!(
        serde_json::to_value(&output)?,
        serde_json::to_value(engine.derive_blocks(input, BlockOutputMode::Rendered))?
    );
    let BlockOutput::Rendered(output) = output else {
        unreachable!()
    };
    assert_eq!(output.status, BlockOutputStatus::Complete);
    let fragments: String = output.blocks.iter().filter_map(|b| b.html()).collect();
    assert_eq!(
        document
            .html
            .split_once('>')
            .unwrap()
            .1
            .strip_suffix("</article>")
            .unwrap(),
        fragments
    );
    assert_eq!(fragments.matches("srcset=\"").count(), 2);
    assert!(
        fragments.contains("srcset=\"/prepared/photo-320.png 320w, /prepared/photo-960.png 960w\"")
    );
    assert!(fragments.contains("width=\"960\" height=\"640\""));
    assert!(!fragments.contains("width: 300px"));
    println!("{fragments}");
    Ok(())
}
