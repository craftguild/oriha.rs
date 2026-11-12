//! One resolver/serializer for contextual semantic images, including figures.
use super::*;

pub(super) struct ResolvedImage<'a, 'h> {
    pub request: ImageRenderRequest<'a>,
    pub dimensions: ImageDimensions,
    presentation: Option<&'h ImagePresentation>,
}

impl<'a, 'h> ResolvedImage<'a, 'h> {
    pub fn new(request: ImageRenderRequest<'a>, context: &'h RenderContext<'_>) -> Self {
        let presentation = context
            .hooks
            .iter()
            .find_map(|h| h.image_presentation(&request, context));
        let dimensions = image_dimensions_from_url(request.destination);
        Self {
            request,
            dimensions,
            presentation,
        }
    }

    fn responsive(&self) -> bool {
        self.presentation
            .is_some_and(|p| p.layout() == ImageLayout::Responsive)
    }

    pub fn container_style(&self) -> String {
        if self.responsive() {
            match self.dimensions.width {
                Some(width) => format!("width: 100%; max-width: {width}px;"),
                None => "width: 100%; max-width: 100%;".into(),
            }
        } else {
            image_container_style(&self.dimensions)
        }
    }

    pub fn render(
        &self,
        class_name: &str,
        fill_container: bool,
        context: &RenderContext,
        state: &mut RenderState,
    ) -> String {
        let title = if fill_container {
            None
        } else {
            self.request.title
        };
        let Some(presentation) = self.presentation else {
            return render_image_element_with_dimensions(
                self.request.destination,
                self.request.alt,
                title,
                class_name,
                &self.dimensions,
                fill_container,
            );
        };
        // Borrowed prepared data is never cloned. Bound both temporary escaped
        // values and the combined element, including in ordinary document APIs.
        let result = crate::blocks::html_budget::scoped(context.max_output_bytes, || {
            let mut out = String::new();
            out.push_html("<img class=\"");
            out.push_html(class_name);
            attribute_start(&mut out, "src");
            emit_url(&mut out, presentation.source(), false);
            attribute_start(&mut out, "alt");
            out.push_html(&escape_attr(&render_inline_plain_nodes(self.request.alt)));
            if let Some(title) = title {
                attribute_start(&mut out, "title");
                out.push_html(&escape_attr(title));
            }
            if !presentation.candidates().is_empty() {
                attribute_start(&mut out, "srcset");
                for (i, candidate) in presentation.candidates().iter().enumerate() {
                    if i != 0 {
                        out.push_html(", ");
                    }
                    emit_url(&mut out, candidate.url(), true);
                    out.push_html(&format!(" {}w", candidate.width()));
                }
            }
            if let Some(sizes) = presentation.sizes() {
                attribute_start(&mut out, "sizes");
                out.push_html(&escape_attr(sizes));
            }
            if let Some((width, height)) = presentation.intrinsic_dimensions() {
                out.push_html(&format!("\" width=\"{width}\" height=\"{height}"));
            }
            attribute_start(&mut out, "style");
            if self.responsive() {
                out.push_html("width: auto; height: auto; ");
                let intrinsic = presentation.intrinsic_dimensions();
                match display_bound(self.dimensions.width, intrinsic.map(|d| d.0)) {
                    Some(w) => out.push_html(&format!("max-width: min(100%, {w}px);")),
                    None => out.push_html("max-width: 100%;"),
                }
                if let Some(h) = display_bound(self.dimensions.height, intrinsic.map(|d| d.1)) {
                    out.push_html(&format!(" max-height: {h}px;"));
                }
            } else {
                out.push_html(&image_element_style(&self.dimensions, fill_container));
            }
            out.push_html("\" />");
            out
        });
        match result {
            Ok(html) => html,
            Err(_) => {
                state.diagnostics.push(RenderDiagnostic {
                    severity: Severity::Error,
                    code: "engine.max_output_bytes_exceeded".into(),
                    message: "Image presentation exceeds max_output_bytes".into(),
                    source_range: None,
                    block_id: None,
                    engine: DiagnosticEngine::Core,
                });
                String::new()
            }
        }
    }
}

fn display_bound(authored: Option<u32>, intrinsic: Option<u32>) -> Option<u32> {
    match (authored, intrinsic) {
        (Some(a), Some(i)) => Some(a.min(i)),
        (a, i) => a.or(i),
    }
}

fn attribute_start(out: &mut String, name: &str) {
    out.push_html("\" ");
    out.push_html(name);
    out.push_html("=\"");
}

fn emit_url(out: &mut String, url: &str, candidate: bool) {
    crate::blocks::html_budget::check_size(url.len());
    let safe = sanitize_url(url);
    // A data: URL needs its first comma to separate metadata from payload
    // (RFC 2397). The srcset URL token ends at ASCII whitespace, so that
    // internal syntax comma is safe. Encode all other candidate commas,
    // including trailing payload commas that srcset would otherwise strip.
    let data_url = safe
        .get(..5)
        .is_some_and(|s| s.eq_ignore_ascii_case("data:"));
    for (index, part) in safe.split_inclusive(',').enumerate() {
        if candidate && part.ends_with(',') && !(data_url && index == 0) {
            out.push_html(&escape_attr(&part[..part.len() - 1]));
            out.push_html("%2C");
        } else {
            out.push_html(&escape_attr(part));
        }
    }
}
