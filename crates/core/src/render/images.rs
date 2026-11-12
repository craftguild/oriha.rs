use super::*;

pub(super) struct StandaloneImage<'a> {
    pub(super) destination: &'a str,
    pub(super) title: Option<&'a str>,
    pub(super) alt: &'a [InlineNode],
}

pub(super) fn standalone_image(children: &[InlineNode]) -> Option<StandaloneImage<'_>> {
    let mut image = None;
    for child in children {
        match &child.kind {
            InlineKind::Text(text) if text.trim().is_empty() => {}
            InlineKind::Image {
                destination,
                title,
                alt,
            } if image.is_none() => {
                image = Some(StandaloneImage {
                    destination,
                    title: title.as_deref(),
                    alt,
                });
            }
            _ => return None,
        }
    }
    image
}

pub(super) fn render_standalone_image(
    block_id: &str,
    image: StandaloneImage<'_>,
    state: &mut RenderState,
    context: &RenderContext,
    out: &mut String,
) {
    let resolved = ResolvedImage::new(
        ImageRenderRequest {
            destination: image.destination,
            title: image.title,
            alt: image.alt,
            placement: ImagePlacement::Standalone,
        },
        context,
    );
    let caption_class = if image.title.is_some() {
        " oriha-image--captioned"
    } else {
        ""
    };
    let caption_attr = if image.title.is_some() {
        r#" data-oriha-image-caption="true""#
    } else {
        ""
    };
    out.push_html(&format!(
        r#"<figure id="{}" class="oriha-image{caption_class}" data-oriha-block-kind="image"{caption_attr} style="{}">"#,
        escape_attr(block_id),
        resolved.container_style(),
    ));
    out.push_html(&resolved.render("oriha-image__img", true, context, state));
    if let Some(caption) = image.title {
        out.push_html(&format!(
            r#"<figcaption class="oriha-image__caption">{}</figcaption>"#,
            escape_text(caption),
        ));
    }
    out.push_html("</figure>");
}

pub(super) fn render_image_element(
    destination: &str,
    alt: &[InlineNode],
    title: Option<&str>,
    class_name: &str,
) -> String {
    let dimensions = image_dimensions_from_url(destination);
    render_image_element_with_dimensions(destination, alt, title, class_name, &dimensions, false)
}

pub(super) fn render_image_element_with_dimensions(
    destination: &str,
    alt: &[InlineNode],
    title: Option<&str>,
    class_name: &str,
    dimensions: &ImageDimensions,
    fill_container: bool,
) -> String {
    let title_attr = title
        .map(|value| format!(" title=\"{}\"", escape_attr(value)))
        .unwrap_or_default();
    let width_attr = dimensions
        .width
        .map(|width| format!(r#" width="{width}""#))
        .unwrap_or_default();
    let height_attr = dimensions
        .height
        .map(|height| format!(r#" height="{height}""#))
        .unwrap_or_default();
    format!(
        r#"<img class="{class_name}" src="{}" alt="{}"{title_attr}{width_attr}{height_attr} style="{}" />"#,
        escape_attr(&sanitize_url(destination)),
        escape_attr(&render_inline_plain_nodes(alt)),
        image_element_style(dimensions, fill_container),
    )
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ImageDimensions {
    pub(super) width: Option<u32>,
    pub(super) height: Option<u32>,
}

pub(super) fn image_dimensions_from_url(destination: &str) -> ImageDimensions {
    let mut dimensions = ImageDimensions::default();
    for component in image_dimension_components(destination) {
        let Some((name, value)) = component.split_once('=') else {
            continue;
        };
        let Some(value) = parse_positive_pixel_dimension(value) else {
            continue;
        };
        match name.trim().to_ascii_lowercase().as_str() {
            "w" | "width" if dimensions.width.is_none() => dimensions.width = Some(value),
            "h" | "height" if dimensions.height.is_none() => dimensions.height = Some(value),
            _ => {}
        }
    }
    dimensions
}

pub(super) fn image_dimension_components(destination: &str) -> Vec<&str> {
    let mut components = Vec::new();
    for marker in ['?', '#'] {
        let Some((_, rest)) = destination.split_once(marker) else {
            continue;
        };
        let end = rest.find(['?', '#']).unwrap_or(rest.len());
        components.extend(rest[..end].split('&'));
    }
    components
}

pub(super) fn parse_positive_pixel_dimension(value: &str) -> Option<u32> {
    let value = value.trim();
    if value.is_empty() || !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }
    value.parse::<u32>().ok().filter(|value| *value > 0)
}

pub(super) fn image_container_style(dimensions: &ImageDimensions) -> String {
    let width = dimensions.width.unwrap_or(300);
    format!("width: {width}px;")
}

pub(super) fn image_element_style(dimensions: &ImageDimensions, fill_container: bool) -> String {
    if fill_container {
        return "width: 100%; height: auto;".to_string();
    }
    let width = dimensions.width.unwrap_or(300);
    format!("width: {width}px; height: auto;")
}
