# Prepared responsive images in Rust

`oriha-core` can emit responsive HTML from data prepared by your application.
It does not read images, select variants, generate URLs or serve files. Prepare
the lookup before rendering, then return borrowed data from `RenderHook`.

Run the complete in-memory consumer from the repository root:

```sh
cargo run -p oriha-core --example responsive_images --release --locked
```

The [example](../../crates/core/examples/responsive_images.rs) asserts document
and block HTML through direct and retained input. Its URLs are declarations;
the example needs no image files, network access or image-processing library.

## Typed API

The following public types are exported from `oriha_core` (and `oriha_core::render`):

```rust
ImageCandidate::new(url: impl Into<String>, width: u32)
    -> Result<ImageCandidate, ImagePresentationError>;

ImagePresentation::new(
    source: impl Into<String>,
    candidates: Vec<ImageCandidate>,
    sizes: Option<String>,
    intrinsic_dimensions: Option<(u32, u32)>,
    layout: ImageLayout,
) -> Result<ImagePresentation, ImagePresentationError>;
```

Implement the provided hook method; existing implementations inherit `None`:

```rust
use std::collections::BTreeMap;
use oriha_core::{ImagePresentation, ImageRenderRequest, RenderHook};
use oriha_core::render::RenderContext;

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
```

For example, populate the map with this prepared value before adding the hook:

```rust
use oriha_core::{ImageCandidate, ImageLayout, ImagePresentation};

let photo = ImagePresentation::new(
    "/prepared/photo.png",
    vec![
        ImageCandidate::new("/prepared/photo-320.png", 320)?,
        ImageCandidate::new("/prepared/photo-960.png", 960)?,
    ],
    Some("(max-width: 40em) 100vw, 40em".into()),
    Some((960, 640)),
    ImageLayout::Responsive,
)?;
```

The application chooses `sizes` for its own layout. The example's 40em is not
an engine policy. URLs may be relative or absolute; file paths and public-origin
policy belong to the application. Presentation getters expose source,
candidates, sizes, dimensions and layout without allowing mutation of validated
fields. Candidates expose `url()` and `width()`.

## Validation and encoding

Candidate URLs must be nonempty, with positive unique `u32` pixel widths.
Candidate order is preserved. Any width-candidate list requires explicit
nonblank `sizes`, including `Fixed` layout. Intrinsic dimensions are either
absent or one positive `(width, height)` pair. A partial pair cannot be expressed.
Zero, duplicate, empty-URL and missing-sizes failures are typed
`ImagePresentationError` variants; construction never repairs invalid data.
An empty candidate list is allowed. Dimensions and descriptors remain the
application's claim about actual pixels; the library cannot verify those pixels.

URLs pass through the existing unsafe-scheme rejection and URL encoder.
Candidate URL whitespace and data commas are percent-encoded. For `data:` URLs,
the first comma is required syntax separating the media-type/base64 header from
the payload; it remains literal. Later payload commas are encoded. This rule
also applies to mixed-case `data:` schemes and parameterized media types.
Existing percent escapes and validated IPv6 authority brackets are preserved;
path/query brackets and Unicode UTF-8 bytes are encoded. Each URL and `sizes`
value is attribute-escaped independently. Supply ordinary unescaped strings,
never a preassembled `srcset`, raw HTML or attribute map. `sizes` is escaped as
data; it is not a CSS-policy validator. Alt text, titles and captions keep the
existing escaping and semantics. Disabling raw HTML still escapes authored tags.

The producer's data-image regression checks unchanged release API output in
Chrome, including `decode()` and `currentSrc` for direct and retained document/
block APIs. It requires a local Chrome executable and can be rerun with:

```sh
python3 scripts/check-image-data-urls.py --chrome /path/to/chrome
```

This test-only command uses embedded PNG/SVG data and an isolated temporary
browser profile; it adds no browser or image-processing runtime dependency.

## Layout and compatibility

`ImageLayout::Responsive` uses automatic width/height and a maximum width of
100%, preserving aspect ratio without an unspecified 300px width. A figure fits
its parent. Supplied intrinsic width/height also cap CSS display size, so
`srcset` density and `sizes` do not stretch a small source to the viewport.
Explicit authored URL dimensions intersect those intrinsic maxima as display
bounds; they do not replace intrinsic pixel attributes or request cropping.
Both CSS dimensions remain automatic so a narrower width or height bound
preserves aspect ratio. Without intrinsic dimensions, the prior responsive
parent/author bounds remain. Paired intrinsic dimensions also become HTML
`width` and `height` attributes.
The styles work in semantic fragments without theme CSS.

`ImageLayout::Fixed` preserves the legacy display-sizing policy (including its
unspecified 300px default), while supplying the requested source/attributes.
No responding hook preserves existing HTML, styles and serialized results
exactly. The global legacy default, IR, structured DTOs, bridge schemas and
resource contracts are unchanged. Custom renderers retain ownership of their
output; the hook applies to built-in semantic images rather than arbitrary HTML.

## Hook lifecycle and bounds

`ImageRenderRequest` exposes post-parse `destination`, `title`, `alt` nodes and
`ImagePlacement::{Standalone, Inline}`. Standalone means the paragraph becomes
a figure; images in links, headings, table cells and footnotes are inline.
Nested paragraph figures retain standalone placement. The full render context
contains the retained effective settings. Hooks are queried in registration
order; the first `Some` wins. A figure's title is visible in its caption; inline
images keep a `title` attribute. IDs, source mapping, wrappers and classes are
preserved. Alt nodes are content, not a request to render nested images.

Both document and rendered-block APIs resolve at semantic image emission after
parse hooks. Rendering a retained `ParsedDocument` does not reparse, repeat parse
hooks or mutate the retained content. Each render uses its receiving engine's
hooks. Structured derivation never calls presentation hooks and remains equal
to the original structured output. Block derivation continues to exclude both
document finalization hooks. A hook should only look up immutable prepared data;
application-side interior mutation still requires application synchronization.

The returned reference avoids cloning the prepared source/candidate list.
Checked appends bound temporary URL/attribute encodings and the combined element
before allocation can grow beyond the retained output bound. Nested block
scopes use the smaller remaining bound. The existing exact serialized block
budget includes all expanded attributes and returns `Failed` with empty
collections on failure. Ordinary document output reports an output-limit error
and retains its existing aggregate truncation behavior. As with other extension
methods, allocations inside application hook code remain its responsibility.

See [ADR-0003](../adr/0003-responsive-image-presentation.md) and the
[Verification methods](verification.md) for the accepted
contract, source/artifact identity, commands, results and producer-only limits.
