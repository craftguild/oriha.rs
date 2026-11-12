//! Text measurement interfaces, spans and layout data model.
mod markup;
pub(crate) use markup::parse_inline_text_spans;
#[allow(unused_imports)]
use markup::*;
mod measurement;
#[allow(unused_imports)]
use measurement::*;
pub use measurement::{
    approximate_inline_text_measurement, inline_text_measurement_request, text_measurement_id,
};
mod layout;
#[cfg(test)]
pub(crate) use layout::layout_inline_text_line;
pub use layout::visible_inline_text;
#[allow(unused_imports)]
use layout::*;
#[allow(unused_imports)]
pub(crate) use layout::{
    inline_style_font_size, layout_inline_text_line_with_measurer, measure_inline_text_with,
    participant_display_line, participant_display_lines, participant_label_lines, text_lines,
};

use std::cell::RefCell;
use std::collections::HashSet;

use crate::sequence::ParticipantDisplay;

const DEFAULT_FONT_SIZE: f32 = 14.0;

pub trait TextMeasurer {
    fn base_inline_text_style(&self) -> InlineTextMeasurementStyle {
        InlineTextMeasurementStyle::default()
    }

    fn measure_inline_text(&self, line: &str) -> f32 {
        let request = inline_text_measurement_request(line, &self.base_inline_text_style());
        self.measure_inline_text_request(&request).width
    }

    fn measure_inline_text_request(
        &self,
        request: &InlineTextMeasurementRequest,
    ) -> InlineTextMeasurementResult {
        approximate_inline_text_measurement(request)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ApproxTextMeasurer;

impl TextMeasurer for ApproxTextMeasurer {}

#[derive(Debug, Default)]
pub struct CollectingTextMeasurer {
    seen: RefCell<HashSet<String>>,
    lines: RefCell<Vec<String>>,
    requests: RefCell<Vec<InlineTextMeasurementRequest>>,
}

impl CollectingTextMeasurer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lines(&self) -> Vec<String> {
        self.lines.borrow().clone()
    }

    pub fn requests(&self) -> Vec<InlineTextMeasurementRequest> {
        self.requests.borrow().clone()
    }

    pub fn into_lines(self) -> Vec<String> {
        self.lines.into_inner()
    }

    pub fn into_requests(self) -> Vec<InlineTextMeasurementRequest> {
        self.requests.into_inner()
    }
}

impl TextMeasurer for CollectingTextMeasurer {
    fn measure_inline_text_request(
        &self,
        request: &InlineTextMeasurementRequest,
    ) -> InlineTextMeasurementResult {
        let mut seen = self.seen.borrow_mut();
        if seen.insert(request.id.clone()) {
            self.lines.borrow_mut().push(request.source.clone());
            self.requests.borrow_mut().push(request.clone());
        }
        approximate_inline_text_measurement(request)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InlineTextMeasurementStyle {
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<String>,
    pub font_style: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineTextMeasurementRun {
    pub id: String,
    pub text: String,
    pub style: InlineTextMeasurementStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineTextMeasurementRequest {
    pub id: String,
    pub source: String,
    pub text: String,
    pub style: InlineTextMeasurementStyle,
    pub runs: Vec<InlineTextMeasurementRun>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineTextRunMeasurement {
    pub id: String,
    pub index: usize,
    pub x: f32,
    pub width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineTextMeasurementResult {
    pub id: String,
    pub width: f32,
    pub runs: Vec<InlineTextRunMeasurement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextAnchor {
    Start,
    Middle,
    End,
}

impl TextAnchor {
    pub(crate) fn from_svg_anchor(anchor: &str) -> Self {
        match anchor {
            "middle" => Self::Middle,
            "end" => Self::End,
            _ => Self::Start,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct InlineTextStyle {
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) underline: bool,
    pub(crate) strike: bool,
    pub(crate) wavy: bool,
    pub(crate) monospace: bool,
    pub(crate) font_size: Option<u16>,
    pub(crate) background_color: Option<String>,
    pub(crate) decoration_color: Option<String>,
    pub(crate) color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InlineTextSpan {
    pub(crate) text: String,
    pub(crate) style: InlineTextStyle,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InlineTextRun {
    pub(crate) text: String,
    pub(crate) style: InlineTextStyle,
    pub(crate) x: f32,
    pub(crate) width: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InlineTextLineLayout {
    pub(crate) width: f32,
    pub(crate) runs: Vec<InlineTextRun>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InlineTag {
    BoldStart,
    BoldEnd,
    ItalicStart,
    ItalicEnd,
    UnderlineStart { color: Option<String> },
    UnderlineEnd,
    StrikeStart { color: Option<String> },
    StrikeEnd,
    WavyStart { color: Option<String> },
    WavyEnd,
    FontStart { color: Option<String> },
    FontEnd,
    ColorStart { color: Option<String> },
    ColorEnd,
    BackgroundStart { color: Option<String> },
    BackgroundEnd,
    SizeStart { size: Option<u16> },
    SizeEnd,
    ImageFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParticipantDisplayLineKind {
    Title,
    Subtitle,
    Text,
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParticipantDisplayLine {
    pub(crate) kind: ParticipantDisplayLineKind,
    pub(crate) text: String,
}

#[cfg(test)]
mod tests {
    use super::{
        layout_inline_text_line, visible_inline_text, ApproxTextMeasurer, TextAnchor, TextMeasurer,
    };

    #[test]
    fn approximate_measurement_treats_cjk_as_wide_characters() {
        let measurer = ApproxTextMeasurer;
        assert!(measurer.measure_inline_text("日本語") > measurer.measure_inline_text("abc"));
    }

    #[test]
    fn inline_text_layout_positions_runs_from_anchor() {
        let layout = layout_inline_text_line("This is ~~waved~~", 24.0, TextAnchor::Start);
        let waved = layout
            .runs
            .iter()
            .find(|run| run.text == "waved")
            .expect("waved run should exist");

        assert!((68.0..72.0).contains(&waved.x));
    }

    #[test]
    fn visible_inline_text_strips_supported_markup() {
        assert_eq!(
            visible_inline_text(
                "<back:lavender>背景色つき note text</back> と <w:#FF33FF>波線 mixed text</w>"
            ),
            "背景色つき note text と 波線 mixed text"
        );
        assert_eq!(
            visible_inline_text("This is **bold** and <color purple>purple</color>"),
            "This is bold and purple"
        );
    }
}
