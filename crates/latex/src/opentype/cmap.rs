//! OpenType character-to-glyph mapping formats and delta arithmetic.
use super::*;

impl Cmap {
    pub(super) fn glyph_id(&self, data: &[u8], codepoint: u32) -> Option<u32> {
        match self {
            Self::Format12 { groups } => groups.iter().find_map(|group| {
                if (group.start_char_code..=group.end_char_code).contains(&codepoint) {
                    Some(group.start_glyph_id + codepoint - group.start_char_code)
                } else {
                    None
                }
            }),
            Self::Format4 { segments } => {
                let codepoint = u16::try_from(codepoint).ok()?;
                segments.iter().find_map(|segment| {
                    if !(segment.start_code..=segment.end_code).contains(&codepoint) {
                        return None;
                    }

                    if segment.id_range_offset == 0 {
                        let glyph_id = wrapping_glyph_delta(codepoint, segment.id_delta);
                        return (glyph_id != 0).then_some(glyph_id as u32);
                    }

                    let glyph_offset = segment.id_range_offset_offset
                        + segment.id_range_offset as usize
                        + usize::from(codepoint - segment.start_code) * 2;
                    let raw_glyph_id = read_u16_opt(data, glyph_offset)?;
                    if raw_glyph_id == 0 {
                        return None;
                    }

                    let glyph_id = wrapping_glyph_delta(raw_glyph_id, segment.id_delta);
                    (glyph_id != 0).then_some(glyph_id as u32)
                })
            }
        }
    }
}

pub(super) fn wrapping_glyph_delta(value: u16, delta: i16) -> u16 {
    ((i32::from(value) + i32::from(delta)) & 0xffff) as u16
}

pub(super) fn parse_cmap(data: &[u8], table: TableRef) -> Result<Cmap, String> {
    let num_tables = read_u16(data, table.offset + 2)?;
    let mut best: Option<(u8, usize, u16)> = None;

    for index in 0..num_tables as usize {
        let record = table.offset + 4 + index * 8;
        let platform_id = read_u16(data, record)?;
        let encoding_id = read_u16(data, record + 2)?;
        let subtable_offset = read_u32(data, record + 4)? as usize;
        let offset = checked_add(table.offset, subtable_offset)?;
        let format = read_u16(data, offset)?;
        let score = match (format, platform_id, encoding_id) {
            (12, 3, 10) | (12, 0, 4) => 40,
            (4, 3, 1) | (4, 0, 3) | (4, 0, 4) => 30,
            (12, _, _) => 20,
            (4, _, _) => 10,
            _ => 0,
        };
        if score == 0 {
            continue;
        }
        if best
            .as_ref()
            .map(|(best_score, _, _)| score > *best_score)
            .unwrap_or(true)
        {
            best = Some((score, offset, format));
        }
    }

    let Some((_, offset, format)) = best else {
        return Err("cmap table does not contain a supported format 4 or 12 subtable".to_owned());
    };

    match format {
        4 => parse_cmap_format4(data, offset),
        12 => parse_cmap_format12(data, offset),
        _ => unreachable!("unsupported cmap format was filtered out"),
    }
}

pub(super) fn parse_cmap_format4(data: &[u8], offset: usize) -> Result<Cmap, String> {
    let length = read_u16(data, offset + 2)? as usize;
    let seg_count = usize::from(read_u16(data, offset + 6)? / 2);
    let end_codes = offset + 14;
    let start_codes = end_codes + seg_count * 2 + 2;
    let id_deltas = start_codes + seg_count * 2;
    let id_range_offsets = id_deltas + seg_count * 2;
    checked_slice(data, offset, length)?;
    checked_slice(data, id_range_offsets, seg_count * 2)?;

    let mut segments = Vec::with_capacity(seg_count);
    for index in 0..seg_count {
        let start_code = read_u16(data, start_codes + index * 2)?;
        let end_code = read_u16(data, end_codes + index * 2)?;
        if start_code == 0xffff && end_code == 0xffff {
            continue;
        }
        segments.push(Format4Segment {
            start_code,
            end_code,
            id_delta: read_i16(data, id_deltas + index * 2)?,
            id_range_offset: read_u16(data, id_range_offsets + index * 2)?,
            id_range_offset_offset: id_range_offsets + index * 2,
        });
    }

    Ok(Cmap::Format4 { segments })
}

pub(super) fn parse_cmap_format12(data: &[u8], offset: usize) -> Result<Cmap, String> {
    let length = read_u32(data, offset + 4)? as usize;
    let group_count = read_u32(data, offset + 12)? as usize;
    checked_slice(data, offset, length)?;
    checked_slice(data, offset + 16, group_count * 12)?;

    let mut groups = Vec::with_capacity(group_count);
    for index in 0..group_count {
        let group = offset + 16 + index * 12;
        groups.push(Format12Group {
            start_char_code: read_u32(data, group)?,
            end_char_code: read_u32(data, group + 4)?,
            start_glyph_id: read_u32(data, group + 8)?,
        });
    }

    Ok(Cmap::Format12 { groups })
}
