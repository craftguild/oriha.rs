//! Checked SFNT table lookup and bounded binary reads.
use super::*;

pub(super) fn find_table(data: &[u8], tag: &[u8; 4]) -> Result<TableRef, String> {
    let table_count = read_u16(data, 4)? as usize;
    checked_slice(data, 12, table_count * 16)?;

    for index in 0..table_count {
        let record = 12 + index * 16;
        if data.get(record..record + 4) != Some(tag) {
            continue;
        }
        let offset = read_u32(data, record + 8)? as usize;
        let length = read_u32(data, record + 12)? as usize;
        checked_slice(data, offset, length)?;
        return Ok(TableRef { offset, length });
    }

    Err(format!(
        "font does not contain required {} table",
        String::from_utf8_lossy(tag)
    ))
}

pub(super) fn read_u16(data: &[u8], offset: usize) -> Result<u16, String> {
    read_u16_opt(data, offset).ok_or_else(|| format!("unexpected end of font at byte {offset}"))
}

pub(super) fn read_i16(data: &[u8], offset: usize) -> Result<i16, String> {
    let bytes = checked_slice(data, offset, 2)?;
    Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
}

pub(super) fn read_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = checked_slice(data, offset, 4)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

pub(super) fn read_u16_opt(data: &[u8], offset: usize) -> Option<u16> {
    let bytes = data.get(offset..offset + 2)?;
    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
}

pub(super) fn checked_add(left: usize, right: usize) -> Result<usize, String> {
    left.checked_add(right)
        .ok_or_else(|| "font offset overflow".to_owned())
}

pub(super) fn checked_slice(data: &[u8], offset: usize, length: usize) -> Result<&[u8], String> {
    let end = checked_add(offset, length)?;
    data.get(offset..end)
        .ok_or_else(|| format!("font table range {offset}..{end} is out of bounds"))
}
