//! STIX OpenType font model and shared metric lookup.
mod cmap;
#[allow(unused_imports)]
use cmap::*;
mod cff;
#[allow(unused_imports)]
use cff::*;
mod type2;
#[allow(unused_imports)]
use type2::*;
mod geometry;
#[allow(unused_imports)]
use geometry::*;
mod math_tables;
#[allow(unused_imports)]
use math_tables::*;
mod binary;
#[allow(unused_imports)]
use binary::*;

use std::sync::OnceLock;

use crate::font::math_font_bytes;

#[derive(Debug)]
pub(crate) struct OpenTypeMathFont {
    data: &'static [u8],
    units_per_em: u16,
    ascender: i16,
    descender: i16,
    number_of_h_metrics: u16,
    num_glyphs: u16,
    hmtx_offset: usize,
    cmap: Cmap,
    cff: Option<CffOutlines>,
    italic_corrections: Vec<GlyphItalicCorrection>,
    pub(crate) constants: MathConstants,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MathConstants {
    pub script_percent_scale_down: u16,
    pub script_script_percent_scale_down: u16,
    pub axis_height: i16,
    pub subscript_shift_down: i16,
    pub subscript_top_max: i16,
    pub subscript_baseline_drop_min: i16,
    pub superscript_shift_up: i16,
    pub superscript_bottom_min: i16,
    pub superscript_baseline_drop_max: i16,
    pub sub_superscript_gap_min: i16,
    pub space_after_script: i16,
    pub fraction_numerator_shift_up: i16,
    pub fraction_numerator_display_style_shift_up: i16,
    pub fraction_denominator_shift_down: i16,
    pub fraction_denominator_display_style_shift_down: i16,
    pub fraction_numerator_gap_min: i16,
    pub fraction_num_display_style_gap_min: i16,
    pub fraction_rule_thickness: i16,
    pub fraction_denominator_gap_min: i16,
    pub fraction_denom_display_style_gap_min: i16,
    pub radical_vertical_gap: i16,
    pub radical_display_style_vertical_gap: i16,
    pub radical_rule_thickness: i16,
    pub radical_extra_ascender: i16,
    #[allow(dead_code)]
    pub radical_kern_before_degree: i16,
    #[allow(dead_code)]
    pub radical_kern_after_degree: i16,
    #[allow(dead_code)]
    pub radical_degree_bottom_raise_percent: u16,
}

#[derive(Debug, Clone)]
enum Cmap {
    Format4 { segments: Vec<Format4Segment> },
    Format12 { groups: Vec<Format12Group> },
}

#[derive(Debug, Clone)]
struct Format4Segment {
    start_code: u16,
    end_code: u16,
    id_delta: i16,
    id_range_offset: u16,
    id_range_offset_offset: usize,
}

#[derive(Debug, Clone)]
struct Format12Group {
    start_char_code: u32,
    end_char_code: u32,
    start_glyph_id: u32,
}

#[derive(Debug, Clone, Copy)]
struct GlyphItalicCorrection {
    glyph_id: u32,
    value: i16,
}

#[derive(Debug, Clone)]
struct CffOutlines {
    charstrings: Vec<ByteRange>,
    global_subrs: Vec<ByteRange>,
    local_subrs: Vec<ByteRange>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GlyphBounds {
    pub y_max: i16,
}

#[derive(Debug, Clone, Copy)]
struct ByteRange {
    offset: usize,
    length: usize,
}

#[derive(Debug, Clone, Copy)]
struct TableRef {
    offset: usize,
    length: usize,
}

pub(crate) fn stix_math_metrics() -> &'static OpenTypeMathFont {
    static METRICS: OnceLock<OpenTypeMathFont> = OnceLock::new();
    METRICS.get_or_init(|| {
        OpenTypeMathFont::parse(math_font_bytes())
            .expect("bundled STIX Two Math font should expose required OpenType tables")
    })
}

impl OpenTypeMathFont {
    fn parse(data: &'static [u8]) -> Result<Self, String> {
        let head = find_table(data, b"head")?;
        let hhea = find_table(data, b"hhea")?;
        let maxp = find_table(data, b"maxp")?;
        let hmtx = find_table(data, b"hmtx")?;
        let cmap = find_table(data, b"cmap")?;
        let math = find_table(data, b"MATH")?;
        let cff = find_table(data, b"CFF ").ok();

        let units_per_em = read_u16(data, head.offset + 18)?;
        let ascender = read_i16(data, hhea.offset + 4)?;
        let descender = read_i16(data, hhea.offset + 6)?;
        let number_of_h_metrics = read_u16(data, hhea.offset + 34)?;
        let num_glyphs = read_u16(data, maxp.offset + 4)?;
        if number_of_h_metrics == 0 {
            return Err("hhea.numberOfHMetrics must be greater than zero".to_owned());
        }
        let hmtx_min_len = number_of_h_metrics as usize * 4;
        if hmtx.length < hmtx_min_len {
            return Err("hmtx table is shorter than hhea.numberOfHMetrics".to_owned());
        }

        let constants = parse_math_constants(data, math)?;
        let italic_corrections = parse_math_italic_corrections(data, math)?;

        Ok(Self {
            data,
            units_per_em,
            ascender,
            descender,
            number_of_h_metrics,
            num_glyphs,
            hmtx_offset: hmtx.offset,
            cmap: parse_cmap(data, cmap)?,
            cff: cff
                .map(|table| parse_cff_outlines(data, table))
                .transpose()?,
            italic_corrections,
            constants,
        })
    }

    pub(crate) fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    pub(crate) fn ascender(&self) -> i16 {
        self.ascender
    }

    pub(crate) fn descender(&self) -> i16 {
        self.descender
    }

    pub(crate) fn glyph_id(&self, ch: char) -> Option<u32> {
        let glyph_id = self.cmap.glyph_id(self.data, ch as u32)?;
        (glyph_id < self.num_glyphs as u32).then_some(glyph_id)
    }

    pub(crate) fn glyph_advance_by_id(&self, glyph_id: u32) -> Option<u16> {
        if glyph_id >= self.num_glyphs as u32 {
            return None;
        }
        let metric_index = usize::min(glyph_id as usize, self.number_of_h_metrics as usize - 1);
        read_u16_opt(self.data, self.hmtx_offset + metric_index * 4)
    }

    pub(crate) fn glyph_advance(&self, ch: char) -> Option<u16> {
        self.glyph_advance_by_id(self.glyph_id(ch)?)
    }

    pub(crate) fn glyph_italic_correction(&self, glyph_id: u32) -> Option<i16> {
        if glyph_id >= self.num_glyphs as u32 {
            return None;
        }
        self.italic_corrections
            .binary_search_by_key(&glyph_id, |correction| correction.glyph_id)
            .ok()
            .map(|index| self.italic_corrections[index].value)
    }

    pub(crate) fn glyph_bounds(&self, glyph_id: u32) -> Option<GlyphBounds> {
        let cff = self.cff.as_ref()?;
        let charstring = cff.charstrings.get(glyph_id as usize)?;
        let mut interpreter = Type2BoundsInterpreter::new(self.data, cff);
        interpreter.run(*charstring, 0)
    }
}

#[derive(Debug)]
struct CffDictEntry {
    operator: Vec<u8>,
    operands: Vec<i32>,
}

struct Type2BoundsInterpreter<'a> {
    data: &'a [u8],
    cff: &'a CffOutlines,
    stack: Vec<f32>,
    x: f32,
    y: f32,
    bounds: Option<GlyphBoundsF32>,
    stem_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct GlyphBoundsF32 {
    x_min: f32,
    y_min: f32,
    x_max: f32,
    y_max: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_stix_math_constants_and_advances_from_bundled_font() {
        let font = OpenTypeMathFont::parse(math_font_bytes()).expect("STIX font should parse");

        assert_eq!(font.units_per_em(), 1000);
        assert_eq!(font.ascender(), 762);
        assert_eq!(font.descender(), -238);
        assert_eq!(font.glyph_advance('x'), Some(479));
        assert_eq!(font.glyph_advance('1'), Some(495));
        assert_eq!(font.glyph_advance('+'), Some(720));
        assert_eq!(font.glyph_advance('\u{2211}'), Some(936));
        assert_eq!(font.glyph_advance('\u{222b}'), Some(684));
        assert_eq!(font.glyph_advance('\u{03b8}'), Some(514));

        let mathematical_x = font
            .glyph_id('\u{1d465}')
            .expect("mathematical x maps to a glyph");
        assert_eq!(font.glyph_advance_by_id(mathematical_x), Some(559));
        assert_eq!(font.glyph_italic_correction(mathematical_x), Some(10));

        let mathematical_f = font
            .glyph_id('\u{1d453}')
            .expect("mathematical f maps to a glyph");
        assert_eq!(font.glyph_italic_correction(mathematical_f), Some(20));

        let mathematical_theta = font
            .glyph_id('\u{1d703}')
            .expect("mathematical theta maps to a glyph");
        assert_eq!(font.glyph_italic_correction(mathematical_theta), Some(25));

        let integral = font.glyph_id('\u{222b}').expect("integral maps to a glyph");
        assert_eq!(font.glyph_italic_correction(integral), Some(230));

        let ascii_x = font.glyph_id('x').expect("ASCII x maps to a glyph");
        assert_eq!(font.glyph_italic_correction(ascii_x), None);

        let sqrt = font.glyph_id('\u{221a}').expect("sqrt maps to a glyph");
        assert_eq!(
            font.glyph_bounds(sqrt).map(|bounds| bounds.y_max),
            Some(922)
        );

        let constants = font.constants;
        assert_eq!(constants.script_percent_scale_down, 70);
        assert_eq!(constants.script_script_percent_scale_down, 55);
        assert_eq!(constants.axis_height, 258);
        assert_eq!(constants.subscript_shift_down, 210);
        assert_eq!(constants.subscript_top_max, 368);
        assert_eq!(constants.subscript_baseline_drop_min, 160);
        assert_eq!(constants.superscript_shift_up, 360);
        assert_eq!(constants.superscript_bottom_min, 120);
        assert_eq!(constants.superscript_baseline_drop_max, 230);
        assert_eq!(constants.sub_superscript_gap_min, 150);
        assert_eq!(constants.space_after_script, 40);
        assert_eq!(constants.fraction_rule_thickness, 68);
        assert_eq!(constants.fraction_numerator_gap_min, 68);
        assert_eq!(constants.fraction_num_display_style_gap_min, 150);
        assert_eq!(constants.radical_vertical_gap, 85);
        assert_eq!(constants.radical_display_style_vertical_gap, 170);
        assert_eq!(constants.radical_rule_thickness, 68);
        assert_eq!(constants.radical_extra_ascender, 78);
        assert_eq!(constants.radical_kern_before_degree, 65);
        assert_eq!(constants.radical_kern_after_degree, -335);
        assert_eq!(constants.radical_degree_bottom_raise_percent, 55);
    }
}
