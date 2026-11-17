use super::*;

impl Pool<'_> {
    pub(super) fn highlighting(
        &mut self,
        source: &str,
        language: Option<&str>,
    ) -> Result<Option<CodeHighlighting>> {
        if !self.options.syntax {
            return Ok(None);
        }
        let lines = crate::syntax_highlight::typed_lines(source, language.unwrap_or("plain"))
            .into_iter()
            .map(|tokens| CodeLine { tokens })
            .collect();
        let value = CodeHighlighting { lines };
        self.b
            .reserve(crate::blocks::budget::size(&value, self.b.remaining())?)?;
        Ok(Some(value))
    }

    pub(super) fn math_svg(
        &mut self,
        id: &str,
        source: &str,
        display: bool,
        closed: bool,
    ) -> Result<Option<SvgContent>> {
        if !self.options.math {
            return Ok(None);
        }
        if !closed {
            self.warning(id, "math.render_failed", "Math has no closing delimiter")?;
            return Ok(None);
        }
        let mode = if display {
            LatexRenderMode::Display
        } else {
            LatexRenderMode::Inline
        };
        match render_latex_source_svg(source, mode, &self.parsed.context.appearance) {
            Ok(rendered) => {
                if self.fonts.is_empty() {
                    // Reserve before copying the shared font. A successful SVG
                    // that exhausts the aggregate budget fails the whole body.
                    let font = DocumentFont {
                        id: oriha_latex::MATH_FONT_ASSET_ID.into(),
                        family: oriha_latex::MATH_FONT_FAMILY.into(),
                        mime_type: oriha_latex::MATH_FONT_MIME_TYPE.into(),
                        encoding: FontEncoding::Base64,
                        content: self.b.copy(oriha_latex::math_font_base64())?,
                    };
                    self.fonts.push(font);
                }
                let value = SvgContent {
                    markup: rendered.svg,
                    width: Some(rendered.width),
                    height: Some(rendered.height),
                    baseline: Some(rendered.baseline),
                    font_ids: vec![oriha_latex::MATH_FONT_ASSET_ID.into()],
                };
                self.charge_svg(value)
            }
            Err(_) => {
                self.warning(
                    id,
                    "math.render_failed",
                    "Math could not be rendered; source retained",
                )?;
                Ok(None)
            }
        }
    }

    pub(super) fn fence_svg(
        &mut self,
        id: &str,
        source: &str,
        language: Option<&str>,
    ) -> Result<Option<SvgContent>> {
        let language = language.unwrap_or("plain");
        if is_latex_language(language) {
            return self.math_svg(id, source, true, true);
        }
        let context = &self.parsed.context;
        if self.options.mermaid && is_mermaid_language(language) {
            match render_mermaid_source_svg(
                source,
                id,
                &context.appearance,
                &context.mermaid_text_metrics,
            ) {
                Ok(rendered) => self.charge_svg(SvgContent {
                    markup: rendered.svg,
                    width: Some(rendered.width as f32),
                    height: Some(rendered.height as f32),
                    baseline: None,
                    font_ids: Vec::new(),
                }),
                Err(_) => {
                    self.warning(
                        id,
                        "mermaid.render_failed",
                        "Mermaid could not be rendered; source retained",
                    )?;
                    Ok(None)
                }
            }
        } else if self.options.plantuml && is_plantuml_language(language) {
            match render_plantuml_source_svg(
                source,
                &context.appearance,
                &context.plantuml_text_metrics,
            ) {
                Ok(rendered) => self.charge_svg(SvgContent {
                    markup: rendered.svg,
                    width: Some(rendered.width as f32),
                    height: Some(rendered.height as f32),
                    baseline: None,
                    font_ids: Vec::new(),
                }),
                Err(PlantUmlError::UnsupportedDiagram { .. }) => {
                    self.warning(
                        id,
                        "plantuml.unsupported_diagram",
                        "PlantUML diagram is unsupported; source retained",
                    )?;
                    Ok(None)
                }
                Err(_) => {
                    self.warning(
                        id,
                        "plantuml.render_failed",
                        "PlantUML could not be rendered; source retained",
                    )?;
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    fn charge_svg(&mut self, value: SvgContent) -> Result<Option<SvgContent>> {
        self.b
            .reserve(crate::blocks::budget::size(&value, self.b.remaining())?)?;
        Ok(Some(value))
    }

    fn warning(&mut self, id: &str, code: &str, message: &str) -> Result<()> {
        let diagnostic = RenderDiagnostic {
            severity: Severity::Warning,
            code: self.b.copy(code)?,
            message: self.b.copy(message)?,
            block_id: Some(self.b.copy(id)?),
            source_range: None,
            engine: DiagnosticEngine::Core,
        };
        self.b.reserve(32)?;
        self.diagnostics.push(diagnostic);
        Ok(())
    }
}
