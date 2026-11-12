//! Validated, application-prepared image attributes. No image I/O occurs here.
use std::collections::BTreeSet;
use std::fmt;

use crate::ir::InlineNode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImagePlacement {
    Standalone,
    Inline,
}

/// Parsed semantic content, after parse hooks. Titles become captions in figures.
pub struct ImageRenderRequest<'a> {
    pub destination: &'a str,
    pub title: Option<&'a str>,
    pub alt: &'a [InlineNode],
    pub placement: ImagePlacement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageLayout {
    /// Retain legacy display sizing, including its unspecified 300px default.
    Fixed,
    /// Fit the parent, preserving aspect ratio and explicit authored size bounds.
    Responsive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImagePresentationError {
    EmptyCandidateUrl,
    ZeroCandidateWidth,
    DuplicateCandidateWidth(u32),
    InvalidIntrinsicDimensions,
    MissingSizes,
}

impl fmt::Display for ImagePresentationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCandidateUrl => f.write_str("candidate URL must not be empty"),
            Self::ZeroCandidateWidth => f.write_str("candidate width must be positive"),
            Self::DuplicateCandidateWidth(width) => write!(f, "duplicate candidate width: {width}"),
            Self::InvalidIntrinsicDimensions => {
                f.write_str("intrinsic dimensions must both be positive")
            }
            Self::MissingSizes => f.write_str("width candidates require explicit nonblank sizes"),
        }
    }
}
impl std::error::Error for ImagePresentationError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageCandidate {
    url: String,
    width: u32,
}

impl ImageCandidate {
    pub fn new(url: impl Into<String>, width: u32) -> Result<Self, ImagePresentationError> {
        if width == 0 {
            return Err(ImagePresentationError::ZeroCandidateWidth);
        }
        let url = url.into();
        if url.is_empty() {
            return Err(ImagePresentationError::EmptyCandidateUrl);
        }
        Ok(Self { url, width })
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn width(&self) -> u32 {
        self.width
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImagePresentation {
    source: String,
    candidates: Vec<ImageCandidate>,
    sizes: Option<String>,
    intrinsic_dimensions: Option<(u32, u32)>,
    layout: ImageLayout,
}

impl ImagePresentation {
    /// Construct before rendering. Candidate order is preserved, never sorted.
    /// Pixel descriptors/dimensions are declarations; no file is inspected.
    pub fn new(
        source: impl Into<String>,
        candidates: Vec<ImageCandidate>,
        sizes: Option<String>,
        intrinsic_dimensions: Option<(u32, u32)>,
        layout: ImageLayout,
    ) -> Result<Self, ImagePresentationError> {
        let mut widths = BTreeSet::new();
        for candidate in &candidates {
            if !widths.insert(candidate.width) {
                return Err(ImagePresentationError::DuplicateCandidateWidth(
                    candidate.width,
                ));
            }
        }
        if intrinsic_dimensions.is_some_and(|(w, h)| w == 0 || h == 0) {
            return Err(ImagePresentationError::InvalidIntrinsicDimensions);
        }
        if !candidates.is_empty() && sizes.as_ref().is_none_or(|v| v.trim().is_empty()) {
            return Err(ImagePresentationError::MissingSizes);
        }
        Ok(Self {
            source: source.into(),
            candidates,
            sizes,
            intrinsic_dimensions,
            layout,
        })
    }
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn candidates(&self) -> &[ImageCandidate] {
        &self.candidates
    }
    pub fn sizes(&self) -> Option<&str> {
        self.sizes.as_deref()
    }
    pub fn intrinsic_dimensions(&self) -> Option<(u32, u32)> {
        self.intrinsic_dimensions
    }
    pub fn layout(&self) -> ImageLayout {
        self.layout
    }
}
