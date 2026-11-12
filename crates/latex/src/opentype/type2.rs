//! Type 2 charstring interpretation and glyph bounds accumulation.
use super::*;

impl<'a> Type2BoundsInterpreter<'a> {
    pub(super) fn new(data: &'a [u8], cff: &'a CffOutlines) -> Self {
        Self {
            data,
            cff,
            stack: Vec::new(),
            x: 0.0,
            y: 0.0,
            bounds: None,
            stem_count: 0,
        }
    }

    pub(super) fn run(&mut self, range: ByteRange, depth: usize) -> Option<GlyphBounds> {
        if depth > 24 {
            return None;
        }

        let bytes = checked_slice(self.data, range.offset, range.length).ok()?;
        let mut offset = 0;
        while offset < bytes.len() {
            let byte = bytes[offset];
            if is_type2_number_start(byte) {
                self.stack.push(read_type2_number(bytes, &mut offset)?);
                continue;
            }
            offset += 1;

            match byte {
                1 | 3 | 18 | 23 => self.consume_stems(),
                4 => {
                    self.drop_width_if_extra(1);
                    let dy = self.pop_at(0)?;
                    self.move_by(0.0, dy);
                    self.stack.clear();
                }
                5 => {
                    let args = std::mem::take(&mut self.stack);
                    for pair in args.chunks_exact(2) {
                        self.line_by(pair[0], pair[1]);
                    }
                }
                6 => {
                    let args = std::mem::take(&mut self.stack);
                    let mut horizontal = true;
                    for value in args {
                        if horizontal {
                            self.line_by(value, 0.0);
                        } else {
                            self.line_by(0.0, value);
                        }
                        horizontal = !horizontal;
                    }
                }
                7 => {
                    let args = std::mem::take(&mut self.stack);
                    let mut vertical = true;
                    for value in args {
                        if vertical {
                            self.line_by(0.0, value);
                        } else {
                            self.line_by(value, 0.0);
                        }
                        vertical = !vertical;
                    }
                }
                8 => {
                    let args = std::mem::take(&mut self.stack);
                    for curve in args.chunks_exact(6) {
                        self.curve_by(curve[0], curve[1], curve[2], curve[3], curve[4], curve[5]);
                    }
                }
                10 => {
                    let subr = self.stack.pop()?;
                    let index = (subr + subr_bias(self.cff.local_subrs.len()) as f32) as usize;
                    let range = *self.cff.local_subrs.get(index)?;
                    self.run(range, depth + 1)?;
                }
                11 => return self.final_bounds(),
                14 => {
                    self.stack.clear();
                    return self.final_bounds();
                }
                19 | 20 => {
                    self.consume_stems();
                    let mask_bytes = self.stem_count.div_ceil(8);
                    if offset.checked_add(mask_bytes)? > bytes.len() {
                        return None;
                    }
                    offset += mask_bytes;
                }
                21 => {
                    self.drop_width_if_extra(2);
                    let dx = self.pop_at(0)?;
                    let dy = self.pop_at(1)?;
                    self.move_by(dx, dy);
                    self.stack.clear();
                }
                22 => {
                    self.drop_width_if_extra(1);
                    let dx = self.pop_at(0)?;
                    self.move_by(dx, 0.0);
                    self.stack.clear();
                }
                24 => {
                    let args = std::mem::take(&mut self.stack);
                    let line_start = args.len().checked_sub(2)?;
                    for curve in args[..line_start].chunks_exact(6) {
                        self.curve_by(curve[0], curve[1], curve[2], curve[3], curve[4], curve[5]);
                    }
                    self.line_by(args[line_start], args[line_start + 1]);
                }
                25 => {
                    let args = std::mem::take(&mut self.stack);
                    let curve_start = args.len().checked_sub(6)?;
                    for line in args[..curve_start].chunks_exact(2) {
                        self.line_by(line[0], line[1]);
                    }
                    let curve = &args[curve_start..];
                    self.curve_by(curve[0], curve[1], curve[2], curve[3], curve[4], curve[5]);
                }
                26 => {
                    let args = std::mem::take(&mut self.stack);
                    let mut index = 0;
                    let mut dx1 = 0.0;
                    if args.len() % 4 == 1 {
                        dx1 = args[0];
                        index = 1;
                    }
                    while index + 3 < args.len() {
                        self.curve_by(
                            dx1,
                            args[index],
                            args[index + 1],
                            args[index + 2],
                            0.0,
                            args[index + 3],
                        );
                        dx1 = 0.0;
                        index += 4;
                    }
                }
                27 => {
                    let args = std::mem::take(&mut self.stack);
                    let mut index = 0;
                    let mut dy1 = 0.0;
                    if args.len() % 4 == 1 {
                        dy1 = args[0];
                        index = 1;
                    }
                    while index + 3 < args.len() {
                        self.curve_by(
                            args[index],
                            dy1,
                            args[index + 1],
                            args[index + 2],
                            args[index + 3],
                            0.0,
                        );
                        dy1 = 0.0;
                        index += 4;
                    }
                }
                29 => {
                    let subr = self.stack.pop()?;
                    let index = (subr + subr_bias(self.cff.global_subrs.len()) as f32) as usize;
                    let range = *self.cff.global_subrs.get(index)?;
                    self.run(range, depth + 1)?;
                }
                30 => self.vh_curveto()?,
                31 => self.hv_curveto()?,
                12 => {
                    let escaped = *bytes.get(offset)?;
                    offset += 1;
                    self.escaped(escaped)?;
                }
                _ => return None,
            }
        }

        self.final_bounds()
    }

    pub(super) fn pop_at(&self, index: usize) -> Option<f32> {
        self.stack.get(index).copied()
    }

    pub(super) fn drop_width_if_extra(&mut self, expected_operands: usize) {
        if self.stack.len() > expected_operands {
            self.stack.remove(0);
        }
    }

    pub(super) fn consume_stems(&mut self) {
        if self.stack.len() % 2 == 1 {
            self.stack.remove(0);
        }
        self.stem_count += self.stack.len() / 2;
        self.stack.clear();
    }

    pub(super) fn move_by(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
        self.include_point(self.x, self.y);
    }

    pub(super) fn line_by(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
        self.include_point(self.x, self.y);
    }

    pub(super) fn curve_by(&mut self, dx1: f32, dy1: f32, dx2: f32, dy2: f32, dx3: f32, dy3: f32) {
        let p0 = (self.x, self.y);
        let p1 = (p0.0 + dx1, p0.1 + dy1);
        let p2 = (p1.0 + dx2, p1.1 + dy2);
        let p3 = (p2.0 + dx3, p2.1 + dy3);

        self.include_point(p3.0, p3.1);
        for t in cubic_extrema(p0.0, p1.0, p2.0, p3.0) {
            self.include_point(
                cubic_at(p0.0, p1.0, p2.0, p3.0, t),
                cubic_at(p0.1, p1.1, p2.1, p3.1, t),
            );
        }
        for t in cubic_extrema(p0.1, p1.1, p2.1, p3.1) {
            self.include_point(
                cubic_at(p0.0, p1.0, p2.0, p3.0, t),
                cubic_at(p0.1, p1.1, p2.1, p3.1, t),
            );
        }

        self.x = p3.0;
        self.y = p3.1;
    }

    pub(super) fn vh_curveto(&mut self) -> Option<()> {
        let args = std::mem::take(&mut self.stack);
        let mut index = 0;
        while index + 3 < args.len() {
            let dy1 = args[index];
            let dx2 = args[index + 1];
            let dy2 = args[index + 2];
            let dx3 = args[index + 3];
            index += 4;
            let dy3 = if args.len() - index == 1 {
                let value = args[index];
                index += 1;
                value
            } else {
                0.0
            };
            self.curve_by(0.0, dy1, dx2, dy2, dx3, dy3);
            if index + 3 < args.len() {
                let dx1 = args[index];
                let dx2 = args[index + 1];
                let dy2 = args[index + 2];
                let dy3 = args[index + 3];
                index += 4;
                let dx3 = if args.len() - index == 1 {
                    let value = args[index];
                    index += 1;
                    value
                } else {
                    0.0
                };
                self.curve_by(dx1, 0.0, dx2, dy2, dx3, dy3);
            }
        }
        Some(())
    }

    pub(super) fn hv_curveto(&mut self) -> Option<()> {
        let args = std::mem::take(&mut self.stack);
        let mut index = 0;
        while index + 3 < args.len() {
            let dx1 = args[index];
            let dx2 = args[index + 1];
            let dy2 = args[index + 2];
            let dy3 = args[index + 3];
            index += 4;
            let dx3 = if args.len() - index == 1 {
                let value = args[index];
                index += 1;
                value
            } else {
                0.0
            };
            self.curve_by(dx1, 0.0, dx2, dy2, dx3, dy3);
            if index + 3 < args.len() {
                let dy1 = args[index];
                let dx2 = args[index + 1];
                let dy2 = args[index + 2];
                let dx3 = args[index + 3];
                index += 4;
                let dy3 = if args.len() - index == 1 {
                    let value = args[index];
                    index += 1;
                    value
                } else {
                    0.0
                };
                self.curve_by(0.0, dy1, dx2, dy2, dx3, dy3);
            }
        }
        Some(())
    }

    pub(super) fn escaped(&mut self, operator: u8) -> Option<()> {
        let args = std::mem::take(&mut self.stack);
        match operator {
            34 => {
                self.curve_by(args[0], 0.0, args[1], args[2], args[3], 0.0);
                self.curve_by(args[4], 0.0, args[5], -args[2], args[6], 0.0);
            }
            35 => {
                self.curve_by(args[0], args[1], args[2], args[3], args[4], args[5]);
                self.curve_by(args[6], args[7], args[8], args[9], args[10], args[11]);
            }
            36 => {
                self.curve_by(args[0], args[1], args[2], args[3], args[4], 0.0);
                self.curve_by(args[5], 0.0, args[6], args[7], args[8], 0.0);
            }
            37 => {
                let dx = args[0] + args[2] + args[4] + args[6] + args[8];
                let dy = args[1] + args[3] + args[5] + args[7] + args[9];
                self.curve_by(args[0], args[1], args[2], args[3], args[4], args[5]);
                if dx.abs() > dy.abs() {
                    self.curve_by(args[6], args[7], args[8], args[9], args[10], -dy);
                } else {
                    self.curve_by(args[6], args[7], args[8], args[9], -dx, args[10]);
                }
            }
            _ => return None,
        }
        Some(())
    }

    pub(super) fn include_point(&mut self, x: f32, y: f32) {
        self.bounds = Some(match self.bounds {
            Some(bounds) => GlyphBoundsF32 {
                x_min: bounds.x_min.min(x),
                y_min: bounds.y_min.min(y),
                x_max: bounds.x_max.max(x),
                y_max: bounds.y_max.max(y),
            },
            None => GlyphBoundsF32 {
                x_min: x,
                y_min: y,
                x_max: x,
                y_max: y,
            },
        });
    }

    pub(super) fn final_bounds(&self) -> Option<GlyphBounds> {
        let bounds = self.bounds?;
        Some(GlyphBounds {
            y_max: bounds.y_max.ceil() as i16,
        })
    }
}

pub(super) fn is_type2_number_start(byte: u8) -> bool {
    byte == 28 || byte == 255 || byte >= 32
}

pub(super) fn read_type2_number(bytes: &[u8], offset: &mut usize) -> Option<f32> {
    let byte = *bytes.get(*offset)?;
    *offset += 1;
    match byte {
        28 => {
            let bytes = bytes.get(*offset..*offset + 2)?;
            *offset += 2;
            Some(i16::from_be_bytes([bytes[0], bytes[1]]) as f32)
        }
        32..=246 => Some(byte as f32 - 139.0),
        247..=250 => {
            let next = *bytes.get(*offset)? as f32;
            *offset += 1;
            Some((byte as f32 - 247.0) * 256.0 + next + 108.0)
        }
        251..=254 => {
            let next = *bytes.get(*offset)? as f32;
            *offset += 1;
            Some(-((byte as f32 - 251.0) * 256.0) - next - 108.0)
        }
        255 => {
            let bytes = bytes.get(*offset..*offset + 4)?;
            *offset += 4;
            Some(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f32 / 65536.0)
        }
        _ => None,
    }
}

pub(super) fn subr_bias(count: usize) -> i32 {
    if count < 1240 {
        107
    } else if count < 33_900 {
        1131
    } else {
        32_768
    }
}
