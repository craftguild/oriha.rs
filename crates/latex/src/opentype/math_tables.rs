//! OpenType MATH constants, italic corrections and glyph coverage.
use super::*;

pub(super) fn parse_math_constants(data: &[u8], table: TableRef) -> Result<MathConstants, String> {
    let constants_offset = read_u16(data, table.offset + 4)? as usize;
    let mut cursor = checked_add(table.offset, constants_offset)?;

    let script_percent_scale_down = read_u16(data, cursor)?;
    cursor += 2;
    let script_script_percent_scale_down = read_u16(data, cursor)?;
    cursor += 2;

    cursor += 4; // DelimitedSubFormulaMinHeight, DisplayOperatorMinHeight.
    cursor += 4; // MathLeading.
    let axis_height = read_math_value(data, &mut cursor)?;
    cursor += 8; // AccentBaseHeight, FlattenedAccentBaseHeight.
    let subscript_shift_down = read_math_value(data, &mut cursor)?;
    let subscript_top_max = read_math_value(data, &mut cursor)?;
    let subscript_baseline_drop_min = read_math_value(data, &mut cursor)?;
    let superscript_shift_up = read_math_value(data, &mut cursor)?;
    cursor += 4; // SuperscriptShiftUpCramped.
    let superscript_bottom_min = read_math_value(data, &mut cursor)?;
    let superscript_baseline_drop_max = read_math_value(data, &mut cursor)?;
    let sub_superscript_gap_min = read_math_value(data, &mut cursor)?;
    cursor += 4; // SuperscriptBottomMaxWithSubscript.
    let space_after_script = read_math_value(data, &mut cursor)?;
    cursor += 16; // Upper/lower limit constants.
    cursor += 40; // Stack/stretch stack constants.
    let fraction_numerator_shift_up = read_math_value(data, &mut cursor)?;
    let fraction_numerator_display_style_shift_up = read_math_value(data, &mut cursor)?;
    let fraction_denominator_shift_down = read_math_value(data, &mut cursor)?;
    let fraction_denominator_display_style_shift_down = read_math_value(data, &mut cursor)?;
    let fraction_numerator_gap_min = read_math_value(data, &mut cursor)?;
    let fraction_num_display_style_gap_min = read_math_value(data, &mut cursor)?;
    let fraction_rule_thickness = read_math_value(data, &mut cursor)?;
    let fraction_denominator_gap_min = read_math_value(data, &mut cursor)?;
    let fraction_denom_display_style_gap_min = read_math_value(data, &mut cursor)?;
    cursor += 32; // Skewed fraction, overbar, and underbar constants.
    let radical_vertical_gap = read_math_value(data, &mut cursor)?;
    let radical_display_style_vertical_gap = read_math_value(data, &mut cursor)?;
    let radical_rule_thickness = read_math_value(data, &mut cursor)?;
    let radical_extra_ascender = read_math_value(data, &mut cursor)?;
    let radical_kern_before_degree = read_math_value(data, &mut cursor)?;
    let radical_kern_after_degree = read_math_value(data, &mut cursor)?;
    let radical_degree_bottom_raise_percent = read_u16(data, cursor)?;

    Ok(MathConstants {
        script_percent_scale_down,
        script_script_percent_scale_down,
        axis_height,
        subscript_shift_down,
        subscript_top_max,
        subscript_baseline_drop_min,
        superscript_shift_up,
        superscript_bottom_min,
        superscript_baseline_drop_max,
        sub_superscript_gap_min,
        space_after_script,
        fraction_numerator_shift_up,
        fraction_numerator_display_style_shift_up,
        fraction_denominator_shift_down,
        fraction_denominator_display_style_shift_down,
        fraction_numerator_gap_min,
        fraction_num_display_style_gap_min,
        fraction_rule_thickness,
        fraction_denominator_gap_min,
        fraction_denom_display_style_gap_min,
        radical_vertical_gap,
        radical_display_style_vertical_gap,
        radical_rule_thickness,
        radical_extra_ascender,
        radical_kern_before_degree,
        radical_kern_after_degree,
        radical_degree_bottom_raise_percent,
    })
}

pub(super) fn parse_math_italic_corrections(
    data: &[u8],
    table: TableRef,
) -> Result<Vec<GlyphItalicCorrection>, String> {
    checked_slice(data, table.offset, table.length)?;
    checked_slice(data, table.offset, 10)?;

    let glyph_info_offset = read_u16(data, table.offset + 6)? as usize;
    if glyph_info_offset == 0 {
        return Ok(Vec::new());
    }

    let glyph_info = checked_add(table.offset, glyph_info_offset)?;
    checked_slice(data, glyph_info, 8)?;

    let italics_offset = read_u16(data, glyph_info)? as usize;
    if italics_offset == 0 {
        return Ok(Vec::new());
    }

    let italics_info = checked_add(glyph_info, italics_offset)?;
    checked_slice(data, italics_info, 4)?;

    let coverage_offset = read_u16(data, italics_info)? as usize;
    let correction_count = read_u16(data, italics_info + 2)? as usize;
    if correction_count == 0 {
        return Ok(Vec::new());
    }
    if coverage_offset == 0 {
        return Err("MATH MathItalicsCorrectionInfo has corrections but no coverage".to_owned());
    }

    let correction_records = italics_info + 4;
    checked_slice(data, correction_records, correction_count * 4)?;

    let coverage = parse_coverage(data, checked_add(italics_info, coverage_offset)?)?;
    if coverage.len() != correction_count {
        return Err(format!(
            "MATH italics coverage count {} does not match correction count {}",
            coverage.len(),
            correction_count
        ));
    }

    let mut corrections = Vec::with_capacity(correction_count);
    for (index, glyph_id) in coverage.into_iter().enumerate() {
        corrections.push(GlyphItalicCorrection {
            glyph_id,
            value: read_i16(data, correction_records + index * 4)?,
        });
    }
    corrections.sort_by_key(|correction| correction.glyph_id);

    Ok(corrections)
}

pub(super) fn parse_coverage(data: &[u8], offset: usize) -> Result<Vec<u32>, String> {
    checked_slice(data, offset, 4)?;
    match read_u16(data, offset)? {
        1 => parse_coverage_format1(data, offset),
        2 => parse_coverage_format2(data, offset),
        format => Err(format!("unsupported OpenType coverage format {format}")),
    }
}

pub(super) fn parse_coverage_format1(data: &[u8], offset: usize) -> Result<Vec<u32>, String> {
    let glyph_count = read_u16(data, offset + 2)? as usize;
    checked_slice(data, offset + 4, glyph_count * 2)?;

    let mut glyphs = Vec::with_capacity(glyph_count);
    for index in 0..glyph_count {
        glyphs.push(read_u16(data, offset + 4 + index * 2)? as u32);
    }
    Ok(glyphs)
}

pub(super) fn parse_coverage_format2(data: &[u8], offset: usize) -> Result<Vec<u32>, String> {
    let range_count = read_u16(data, offset + 2)? as usize;
    checked_slice(data, offset + 4, range_count * 6)?;

    let mut indexed_glyphs = Vec::new();
    for index in 0..range_count {
        let record = offset + 4 + index * 6;
        let start_glyph_id = read_u16(data, record)?;
        let end_glyph_id = read_u16(data, record + 2)?;
        let start_coverage_index = read_u16(data, record + 4)? as u32;
        if start_glyph_id > end_glyph_id {
            return Err(
                "OpenType coverage range has start glyph greater than end glyph".to_owned(),
            );
        }

        for glyph_id in start_glyph_id..=end_glyph_id {
            let coverage_index = start_coverage_index + u32::from(glyph_id - start_glyph_id);
            indexed_glyphs.push((coverage_index, u32::from(glyph_id)));
        }
    }

    indexed_glyphs.sort_by_key(|(coverage_index, _)| *coverage_index);
    Ok(indexed_glyphs
        .into_iter()
        .map(|(_, glyph_id)| glyph_id)
        .collect())
}

pub(super) fn read_math_value(data: &[u8], cursor: &mut usize) -> Result<i16, String> {
    let value = read_i16(data, *cursor)?;
    *cursor += 4;
    Ok(value)
}
