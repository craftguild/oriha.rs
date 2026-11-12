//! STIX math metrics and nested display/text/script contexts.
use super::*;

impl LayoutContext {
    pub(super) fn fraction_child(self) -> Self {
        let next_style = self.style.fraction_child();
        Self {
            style: next_style,
            font_size: self.font_size * self.metrics.fraction_child_scale(self.style),
            metrics: self.metrics,
        }
    }

    pub(super) fn script_child(self) -> Self {
        let next_style = self.style.script_child();
        Self {
            style: next_style,
            font_size: self.font_size * self.metrics.script_child_scale(self.style),
            metrics: self.metrics,
        }
    }

    pub(super) fn grid_cell(self) -> Self {
        Self {
            style: self.style,
            font_size: self.font_size * self.metrics.grid_cell_scale(self.style),
            metrics: self.metrics,
        }
    }
}

impl MathStyle {
    pub(super) fn fraction_child(self) -> Self {
        match self {
            Self::Display => Self::Text,
            Self::Text => Self::Script,
            Self::Script | Self::ScriptScript => Self::ScriptScript,
        }
    }

    pub(super) fn script_child(self) -> Self {
        match self {
            Self::Display | Self::Text => Self::Script,
            Self::Script | Self::ScriptScript => Self::ScriptScript,
        }
    }
}

impl MathMetrics {
    pub(super) fn stix() -> Self {
        Self {
            font: stix_math_metrics(),
        }
    }

    pub(super) fn default_ascent(self, font_size: f32) -> f32 {
        self.font_units_to_px(self.font.ascender(), font_size)
    }

    pub(super) fn default_descent(self, font_size: f32) -> f32 {
        self.font_units_to_px(self.font.descender().saturating_abs(), font_size)
    }

    pub(super) fn axis_height(self, font_size: f32) -> f32 {
        self.font_units_to_px(self.font.constants.axis_height, font_size)
    }

    pub(super) fn rule_thickness(self, font_size: f32) -> f32 {
        f32::max(
            1.0,
            self.font_units_to_px(self.font.constants.fraction_rule_thickness, font_size),
        )
    }

    pub(super) fn fraction_horizontal_padding(self, font_size: f32) -> f32 {
        self.font_units_to_px(STIX_FRACTION_HORIZONTAL_PADDING_UNITS as i16, font_size)
    }

    pub(super) fn radical_body_gap(self, font_size: f32) -> f32 {
        self.font_units_to_px(STIX_RADICAL_BODY_GAP_UNITS as i16, font_size)
    }

    pub(super) fn radical_vertical_gap(self, style: MathStyle, font_size: f32) -> f32 {
        let units = match style {
            MathStyle::Display => self.font.constants.radical_display_style_vertical_gap,
            MathStyle::Text | MathStyle::Script | MathStyle::ScriptScript => {
                self.font.constants.radical_vertical_gap
            }
        };
        self.font_units_to_px(units, font_size)
    }

    pub(super) fn radical_rule_thickness(self, font_size: f32) -> f32 {
        f32::max(
            1.0,
            self.font_units_to_px(self.font.constants.radical_rule_thickness, font_size),
        )
    }

    pub(super) fn radical_extra_ascender(self, font_size: f32) -> f32 {
        self.font_units_to_px(self.font.constants.radical_extra_ascender, font_size)
    }

    pub(super) fn fraction_numerator_gap(self, style: MathStyle, font_size: f32) -> f32 {
        let units = match style {
            MathStyle::Display => self.font.constants.fraction_num_display_style_gap_min,
            MathStyle::Text | MathStyle::Script | MathStyle::ScriptScript => {
                self.font.constants.fraction_numerator_gap_min
            }
        };
        self.font_units_to_px(units, font_size)
    }

    pub(super) fn fraction_denominator_gap(self, style: MathStyle, font_size: f32) -> f32 {
        let units = match style {
            MathStyle::Display => self.font.constants.fraction_denom_display_style_gap_min,
            MathStyle::Text | MathStyle::Script | MathStyle::ScriptScript => {
                self.font.constants.fraction_denominator_gap_min
            }
        };
        self.font_units_to_px(units, font_size)
    }

    pub(super) fn fraction_numerator_shift(self, style: MathStyle, font_size: f32) -> f32 {
        let units = match style {
            MathStyle::Display => {
                self.font
                    .constants
                    .fraction_numerator_display_style_shift_up
            }
            MathStyle::Text | MathStyle::Script | MathStyle::ScriptScript => {
                self.font.constants.fraction_numerator_shift_up
            }
        };
        self.font_units_to_px(units, font_size)
    }

    pub(super) fn fraction_denominator_shift(self, style: MathStyle, font_size: f32) -> f32 {
        let units = match style {
            MathStyle::Display => {
                self.font
                    .constants
                    .fraction_denominator_display_style_shift_down
            }
            MathStyle::Text | MathStyle::Script | MathStyle::ScriptScript => {
                self.font.constants.fraction_denominator_shift_down
            }
        };
        self.font_units_to_px(units, font_size)
    }

    pub(super) fn script_gap(self, font_size: f32) -> f32 {
        self.font_units_to_px(self.font.constants.space_after_script, font_size)
    }

    pub(super) fn fraction_child_scale(self, style: MathStyle) -> f32 {
        match style {
            MathStyle::Display => TEXT_FONT_SIZE / DISPLAY_FONT_SIZE,
            MathStyle::Text => self.script_scale(),
            MathStyle::Script => self.script_script_scale() / self.script_scale(),
            MathStyle::ScriptScript => 1.0,
        }
    }

    pub(super) fn script_child_scale(self, style: MathStyle) -> f32 {
        match style {
            MathStyle::Display | MathStyle::Text => self.script_scale(),
            MathStyle::Script => self.script_script_scale() / self.script_scale(),
            MathStyle::ScriptScript => 1.0,
        }
    }

    pub(super) fn grid_cell_scale(self, style: MathStyle) -> f32 {
        match style {
            MathStyle::Display => TEXT_FONT_SIZE / DISPLAY_FONT_SIZE,
            MathStyle::Text => 1.0,
            MathStyle::Script => self.script_scale(),
            MathStyle::ScriptScript => self.script_script_scale(),
        }
    }

    pub(super) fn superscript_shift(
        self,
        base: &LayoutBox,
        superscript: &LayoutBox,
        font_size: f32,
    ) -> f32 {
        let shift_up = self.font_units_to_px(self.font.constants.superscript_shift_up, font_size);
        let baseline_drop = base.ascent
            - self.font_units_to_px(self.font.constants.superscript_baseline_drop_max, font_size);
        let bottom_min = superscript.descent
            + self.font_units_to_px(self.font.constants.superscript_bottom_min, font_size);
        f32::max(shift_up, f32::max(baseline_drop, bottom_min))
    }

    pub(super) fn subscript_shift(
        self,
        base: &LayoutBox,
        subscript: &LayoutBox,
        font_size: f32,
    ) -> f32 {
        let shift_down = self.font_units_to_px(self.font.constants.subscript_shift_down, font_size);
        let baseline_drop = base.descent
            + self.font_units_to_px(self.font.constants.subscript_baseline_drop_min, font_size);
        let top_max = subscript.ascent
            - self.font_units_to_px(self.font.constants.subscript_top_max, font_size);
        f32::max(shift_down, f32::max(baseline_drop, top_max))
    }

    pub(super) fn sub_superscript_gap(self, font_size: f32) -> f32 {
        self.font_units_to_px(self.font.constants.sub_superscript_gap_min, font_size)
    }

    pub(super) fn italic_correction(self, text: &str, font_size: f32, kind: TextRunKind) -> f32 {
        match kind {
            TextRunKind::Identifier | TextRunKind::Operator(MathAtomClass::Op)
                if text.chars().count() == 1 =>
            {
                text.chars()
                    .next()
                    .and_then(|ch| self.glyph_italic_correction_px(ch, font_size))
                    .unwrap_or(0.0)
            }
            _ => 0.0,
        }
    }

    pub(super) fn glyph_advance_px(
        self,
        ch: char,
        font_size: f32,
        kind: TextRunKind,
    ) -> Option<f32> {
        let advance = if kind == TextRunKind::Identifier {
            self.font
                .glyph_id(ch)
                .and_then(|glyph_id| self.font.glyph_advance_by_id(glyph_id))
        } else {
            self.font.glyph_advance(ch)
        }?;

        Some(advance as f32 * font_size / self.font.units_per_em() as f32)
    }

    pub(super) fn glyph_italic_correction_px(self, ch: char, font_size: f32) -> Option<f32> {
        let glyph_id = self.font.glyph_id(ch)?;
        let correction = self.font.glyph_italic_correction(glyph_id)?;

        Some(self.font_units_to_px(correction, font_size))
    }

    pub(super) fn glyph_y_max(self, text: &str, font_size: f32) -> Option<f32> {
        let mut chars = text.chars();
        let ch = chars.next()?;
        if chars.next().is_some() {
            return None;
        }
        let glyph_id = self.font.glyph_id(ch)?;
        let bounds = self.font.glyph_bounds(glyph_id)?;

        Some(self.font_units_to_px(bounds.y_max, font_size))
    }

    pub(super) fn atom_spacing(
        self,
        left: MathAtomClass,
        right: MathAtomClass,
        style: MathStyle,
        font_size: f32,
    ) -> f32 {
        let em = match (left, right) {
            (MathAtomClass::Open, _) | (_, MathAtomClass::Close) => 0.0,
            (_, MathAtomClass::Punct) => 0.0,
            (MathAtomClass::Punct, _) => 0.166,
            (MathAtomClass::Rel, _) | (_, MathAtomClass::Rel) => 0.277,
            (MathAtomClass::Bin, _) | (_, MathAtomClass::Bin) => 0.222,
            (MathAtomClass::Op, MathAtomClass::Ord)
            | (MathAtomClass::Op, MathAtomClass::Inner)
            | (MathAtomClass::Ord, MathAtomClass::Op)
            | (MathAtomClass::Inner, MathAtomClass::Op) => 0.166,
            _ => 0.0,
        };
        em * font_size * self.spacing_scale(style)
    }

    pub(super) fn text_width(self, text: &str, font_size: f32, kind: TextRunKind) -> f32 {
        font_size
            * text
                .chars()
                .map(|ch| {
                    self.glyph_advance_px(ch, font_size, kind)
                        .map(|advance| advance / font_size)
                        .unwrap_or_else(|| fallback_advance_em(ch, kind))
                })
                .sum::<f32>()
    }

    pub(super) fn spacing_scale(self, style: MathStyle) -> f32 {
        match style {
            MathStyle::Display | MathStyle::Text => 1.0,
            MathStyle::Script => self.script_scale(),
            MathStyle::ScriptScript => self.script_script_scale(),
        }
    }

    pub(super) fn script_scale(self) -> f32 {
        self.font.constants.script_percent_scale_down as f32 / 100.0
    }

    pub(super) fn script_script_scale(self) -> f32 {
        self.font.constants.script_script_percent_scale_down as f32 / 100.0
    }

    pub(super) fn font_units_to_px(self, value: i16, font_size: f32) -> f32 {
        value as f32 * font_size / self.font.units_per_em() as f32
    }
}
