//! CFF indexes, dictionaries, outline ranges and operand decoding.
use super::*;

pub(super) fn parse_cff_outlines(data: &[u8], table: TableRef) -> Result<CffOutlines, String> {
    checked_slice(data, table.offset, table.length)?;
    checked_slice(data, table.offset, 4)?;

    let header_size = data[table.offset + 2] as usize;
    let (_, after_names) = read_cff_index_ranges(data, table.offset, table.offset + header_size)?;
    let (top_dicts, after_top_dicts) = read_cff_index_ranges(data, table.offset, after_names)?;
    let (_, after_strings) = read_cff_index_ranges(data, table.offset, after_top_dicts)?;
    let (global_subrs, _) = read_cff_index_ranges(data, table.offset, after_strings)?;
    let top_dict = top_dicts
        .first()
        .ok_or_else(|| "CFF table has no Top DICT".to_owned())?;
    let top_entries = parse_cff_dict(data, *top_dict)?;
    let charstrings_offset = cff_dict_single(&top_entries, &[17])
        .ok_or_else(|| "CFF Top DICT has no CharStrings offset".to_owned())?
        as usize;
    let (charstrings, _) = read_cff_index_ranges(
        data,
        table.offset,
        checked_add(table.offset, charstrings_offset)?,
    )?;
    let local_subrs = if let Some(private) = cff_dict_operands(&top_entries, &[18]) {
        if private.len() < 2 {
            return Err("CFF Top DICT Private operator is malformed".to_owned());
        }
        let private_size = private[0] as usize;
        let private_offset = checked_add(table.offset, private[1] as usize)?;
        let private_range = ByteRange {
            offset: private_offset,
            length: private_size,
        };
        let private_entries = parse_cff_dict(data, private_range)?;
        if let Some(subrs_offset) = cff_dict_single(&private_entries, &[19]) {
            read_cff_index_ranges(
                data,
                table.offset,
                checked_add(private_offset, subrs_offset as usize)?,
            )?
            .0
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(CffOutlines {
        charstrings,
        global_subrs,
        local_subrs,
    })
}

pub(super) fn read_cff_index_ranges(
    data: &[u8],
    base_offset: usize,
    offset: usize,
) -> Result<(Vec<ByteRange>, usize), String> {
    let count = read_u16(data, offset)? as usize;
    if count == 0 {
        return Ok((Vec::new(), offset + 2));
    }

    let off_size_offset = offset + 2;
    checked_slice(data, off_size_offset, 1)?;
    let off_size = data[off_size_offset] as usize;
    if !(1..=4).contains(&off_size) {
        return Err(format!("unsupported CFF INDEX offSize {off_size}"));
    }

    let offsets_offset = off_size_offset + 1;
    checked_slice(data, offsets_offset, (count + 1) * off_size)?;
    let mut offsets = Vec::with_capacity(count + 1);
    for index in 0..=count {
        offsets.push(read_cff_offset(
            data,
            offsets_offset + index * off_size,
            off_size,
        )?);
    }

    let data_start = offsets_offset + (count + 1) * off_size;
    let mut ranges = Vec::with_capacity(count);
    for index in 0..count {
        let start = offsets[index];
        let end = offsets[index + 1];
        if start == 0 || end < start {
            return Err("malformed CFF INDEX offsets".to_owned());
        }
        let absolute_start = checked_add(data_start, start - 1)?;
        let absolute_end = checked_add(data_start, end - 1)?;
        checked_slice(data, absolute_start, absolute_end - absolute_start)?;
        ranges.push(ByteRange {
            offset: absolute_start,
            length: absolute_end - absolute_start,
        });
    }

    let end = checked_add(data_start, offsets[count] - 1)?;
    checked_slice(data, base_offset, end - base_offset)?;

    Ok((ranges, end))
}

pub(super) fn parse_cff_dict(data: &[u8], range: ByteRange) -> Result<Vec<CffDictEntry>, String> {
    let bytes = checked_slice(data, range.offset, range.length)?;
    let mut entries = Vec::new();
    let mut operands = Vec::new();
    let mut offset = 0;

    while offset < bytes.len() {
        let byte = bytes[offset];
        offset += 1;
        if byte <= 21 {
            let operator = if byte == 12 {
                let escaped = *bytes
                    .get(offset)
                    .ok_or_else(|| "truncated CFF escaped operator".to_owned())?;
                offset += 1;
                vec![12, escaped]
            } else {
                vec![byte]
            };
            entries.push(CffDictEntry {
                operator,
                operands: std::mem::take(&mut operands),
            });
            continue;
        }

        let number = read_cff_dict_number(bytes, &mut offset, byte)?;
        operands.push(number);
    }

    Ok(entries)
}

pub(super) fn cff_dict_operands<'a>(
    entries: &'a [CffDictEntry],
    operator: &[u8],
) -> Option<&'a [i32]> {
    entries
        .iter()
        .find(|entry| entry.operator == operator)
        .map(|entry| entry.operands.as_slice())
}

pub(super) fn cff_dict_single(entries: &[CffDictEntry], operator: &[u8]) -> Option<i32> {
    cff_dict_operands(entries, operator).and_then(|operands| operands.first().copied())
}

pub(super) fn read_cff_dict_number(
    bytes: &[u8],
    offset: &mut usize,
    first: u8,
) -> Result<i32, String> {
    match first {
        28 => {
            let bytes = bytes
                .get(*offset..*offset + 2)
                .ok_or_else(|| "truncated CFF short integer".to_owned())?;
            *offset += 2;
            Ok(i16::from_be_bytes([bytes[0], bytes[1]]) as i32)
        }
        29 => {
            let bytes = bytes
                .get(*offset..*offset + 4)
                .ok_or_else(|| "truncated CFF long integer".to_owned())?;
            *offset += 4;
            Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }
        30 => {
            while let Some(byte) = bytes.get(*offset) {
                *offset += 1;
                if byte >> 4 == 0x0f || byte & 0x0f == 0x0f {
                    break;
                }
            }
            Ok(0)
        }
        32..=246 => Ok(first as i32 - 139),
        247..=250 => {
            let next = *bytes
                .get(*offset)
                .ok_or_else(|| "truncated CFF positive integer".to_owned())?
                as i32;
            *offset += 1;
            Ok((first as i32 - 247) * 256 + next + 108)
        }
        251..=254 => {
            let next = *bytes
                .get(*offset)
                .ok_or_else(|| "truncated CFF negative integer".to_owned())?
                as i32;
            *offset += 1;
            Ok(-((first as i32 - 251) * 256) - next - 108)
        }
        _ => Err(format!("unsupported CFF DICT number byte {first}")),
    }
}

pub(super) fn read_cff_offset(data: &[u8], offset: usize, size: usize) -> Result<usize, String> {
    checked_slice(data, offset, size)?;
    let mut value = 0usize;
    for byte in &data[offset..offset + size] {
        value = (value << 8) | *byte as usize;
    }
    Ok(value)
}
