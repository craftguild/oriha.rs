use std::sync::OnceLock;

pub const MATH_FONT_FAMILY: &str = "ORIHA STIX Two Math";
pub const MATH_FONT_ASSET_ID: &str = "oriha-stix-two-math";
pub const MATH_FONT_MIME_TYPE: &str = "font/otf";
pub const MATH_FONT_FORMAT: &str = "opentype";

const MATH_FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/STIXTwoMath-Regular.otf");
const BASE64_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

static MATH_FONT_BASE64: OnceLock<String> = OnceLock::new();
static MATH_FONT_FACE_CSS: OnceLock<String> = OnceLock::new();

pub fn math_font_bytes() -> &'static [u8] {
    MATH_FONT_BYTES
}

pub fn math_font_base64() -> &'static str {
    MATH_FONT_BASE64
        .get_or_init(|| encode_base64(math_font_bytes()))
        .as_str()
}

pub fn math_font_face_css() -> &'static str {
    MATH_FONT_FACE_CSS
        .get_or_init(|| {
            format!(
                "@font-face {{ font-family: \"{}\"; src: url(\"data:{};base64,{}\") format(\"{}\"); font-weight: 400; font-style: normal; font-display: block; }}",
                MATH_FONT_FAMILY,
                MATH_FONT_MIME_TYPE,
                math_font_base64(),
                MATH_FONT_FORMAT
            )
        })
        .as_str()
}

fn encode_base64(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().saturating_add(2) / 3 * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);

        encoded.push(BASE64_CHARS[(first >> 2) as usize] as char);
        encoded.push(BASE64_CHARS[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);

        if chunk.len() > 1 {
            encoded.push(
                BASE64_CHARS[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char,
            );
        } else {
            encoded.push('=');
        }

        if chunk.len() > 2 {
            encoded.push(BASE64_CHARS[(third & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }

    encoded
}
