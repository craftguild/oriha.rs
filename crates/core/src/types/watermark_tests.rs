use super::*;

#[test]
fn default_watermark_uses_markup_text_and_qr_svg_data_url() {
    let watermark = default_watermark_options();

    assert_eq!(watermark.text, DEFAULT_WATERMARK_TEXT);
    let image_src = watermark.image_src.expect("default watermark QR image");
    assert!(image_src.starts_with("data:image/svg+xml,"));
    assert!(image_src.contains("%3Csvg"));
    assert_eq!(watermark.opacity, None);
}

#[test]
fn qr_svg_data_url_encodes_payload_as_svg_data_url() {
    let image_src = qr_svg_data_url("https://example.com/custom");

    assert!(image_src.starts_with("data:image/svg+xml,"));
    assert!(image_src.contains("%3Cpath"));
    assert!(!image_src.contains('<'));
    assert!(!image_src.contains('>'));
}
