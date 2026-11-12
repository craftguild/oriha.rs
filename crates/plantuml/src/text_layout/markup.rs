//! Inline style-tag parsing and text-span construction.
use super::*;

pub(crate) fn parse_inline_text_spans(line: &str) -> Vec<InlineTextSpan> {
    let mut spans = Vec::new();
    let mut style = InlineTextStyle::default();
    let mut buffer = String::new();
    let mut chars = line.char_indices().peekable();

    while let Some((_, character)) = chars.next() {
        if character == '*' && chars.peek().is_some_and(|(_, next)| *next == '*') {
            chars.next();
            push_inline_text_span(&mut spans, &mut buffer, &style);
            style.bold = !style.bold;
            continue;
        }
        if character == '"' && chars.peek().is_some_and(|(_, next)| *next == '"') {
            chars.next();
            push_inline_text_span(&mut spans, &mut buffer, &style);
            style.monospace = !style.monospace;
            continue;
        }
        if character == '/' && chars.peek().is_some_and(|(_, next)| *next == '/') {
            chars.next();
            push_inline_text_span(&mut spans, &mut buffer, &style);
            style.italic = !style.italic;
            continue;
        }
        if character == '-' && chars.peek().is_some_and(|(_, next)| *next == '-') {
            chars.next();
            push_inline_text_span(&mut spans, &mut buffer, &style);
            style.strike = !style.strike;
            continue;
        }
        if character == '_' && chars.peek().is_some_and(|(_, next)| *next == '_') {
            chars.next();
            push_inline_text_span(&mut spans, &mut buffer, &style);
            style.underline = !style.underline;
            continue;
        }
        if character == '~' && chars.peek().is_some_and(|(_, next)| *next == '~') {
            chars.next();
            push_inline_text_span(&mut spans, &mut buffer, &style);
            style.wavy = !style.wavy;
            continue;
        }
        if character != '<' {
            buffer.push(character);
            continue;
        }

        let mut tag = String::new();
        let mut closed = false;
        for (_, tag_character) in chars.by_ref() {
            if tag_character == '>' {
                closed = true;
                break;
            }
            tag.push(tag_character);
        }

        let Some(inline_tag) = closed.then(|| parse_inline_tag(&tag)).flatten() else {
            buffer.push('<');
            buffer.push_str(&tag);
            if closed {
                buffer.push('>');
            }
            continue;
        };

        push_inline_text_span(&mut spans, &mut buffer, &style);
        apply_inline_tag(inline_tag, &mut style, &mut spans);
    }

    push_inline_text_span(&mut spans, &mut buffer, &style);
    if spans.is_empty() {
        spans.push(InlineTextSpan {
            text: String::new(),
            style: InlineTextStyle::default(),
        });
    }
    spans
}

pub(super) fn apply_inline_tag(
    inline_tag: InlineTag,
    style: &mut InlineTextStyle,
    spans: &mut Vec<InlineTextSpan>,
) {
    match inline_tag {
        InlineTag::BoldStart => style.bold = true,
        InlineTag::BoldEnd => style.bold = false,
        InlineTag::ItalicStart => style.italic = true,
        InlineTag::ItalicEnd => style.italic = false,
        InlineTag::UnderlineStart { color } => {
            style.underline = true;
            if color.is_some() {
                style.decoration_color = color;
            }
        }
        InlineTag::UnderlineEnd => {
            style.underline = false;
            style.decoration_color = None;
        }
        InlineTag::StrikeStart { color } => {
            style.strike = true;
            if color.is_some() {
                style.decoration_color = color;
            }
        }
        InlineTag::StrikeEnd => {
            style.strike = false;
            style.decoration_color = None;
        }
        InlineTag::WavyStart { color } => {
            style.wavy = true;
            if color.is_some() {
                style.decoration_color = color;
            }
        }
        InlineTag::WavyEnd => {
            style.wavy = false;
            style.decoration_color = None;
        }
        InlineTag::FontStart { color } => style.color = color,
        InlineTag::FontEnd => style.color = None,
        InlineTag::ColorStart { color } => style.color = color,
        InlineTag::ColorEnd => style.color = None,
        InlineTag::BackgroundStart { color } => style.background_color = color,
        InlineTag::BackgroundEnd => style.background_color = None,
        InlineTag::SizeStart { size } => style.font_size = size,
        InlineTag::SizeEnd => style.font_size = None,
        InlineTag::ImageFallback => {
            let mut fallback_style = style.clone();
            fallback_style.monospace = true;
            spans.push(InlineTextSpan {
                text: "(Cannot decode)".to_owned(),
                style: fallback_style,
            });
        }
    }
}

pub(super) fn parse_inline_tag(tag: &str) -> Option<InlineTag> {
    let tag = tag.trim();
    match tag.to_ascii_lowercase().as_str() {
        "b" => Some(InlineTag::BoldStart),
        "/b" => Some(InlineTag::BoldEnd),
        "i" => Some(InlineTag::ItalicStart),
        "/i" => Some(InlineTag::ItalicEnd),
        "u" => Some(InlineTag::UnderlineStart { color: None }),
        "/u" => Some(InlineTag::UnderlineEnd),
        "/font" => Some(InlineTag::FontEnd),
        "/color" => Some(InlineTag::ColorEnd),
        "/back" => Some(InlineTag::BackgroundEnd),
        "/size" => Some(InlineTag::SizeEnd),
        "/s" | "/strike" => Some(InlineTag::StrikeEnd),
        "/w" => Some(InlineTag::WavyEnd),
        _ => {
            let lower = tag.to_ascii_lowercase();
            if lower.starts_with("font ") {
                return Some(InlineTag::FontStart {
                    color: inline_font_color(tag),
                });
            }
            if lower.starts_with("color ") {
                return Some(InlineTag::ColorStart {
                    color: inline_color_argument(tag.strip_prefix("color").unwrap_or_default()),
                });
            }
            if lower.starts_with("u:") {
                return Some(InlineTag::UnderlineStart {
                    color: inline_colon_color(tag),
                });
            }
            if lower.starts_with("s:") {
                return Some(InlineTag::StrikeStart {
                    color: inline_colon_color(tag),
                });
            }
            if lower.starts_with("w:") {
                return Some(InlineTag::WavyStart {
                    color: inline_colon_color(tag),
                });
            }
            if lower.starts_with("back:") {
                return Some(InlineTag::BackgroundStart {
                    color: inline_colon_color(tag),
                });
            }
            if lower.starts_with("size:") {
                return Some(InlineTag::SizeStart {
                    size: inline_colon_number(tag),
                });
            }
            if lower.starts_with("img ") {
                return Some(InlineTag::ImageFallback);
            }
            None
        }
    }
}

pub(super) fn inline_font_color(tag: &str) -> Option<String> {
    tag.split_whitespace().find_map(|part| {
        let (key, value) = part.split_once('=')?;
        key.eq_ignore_ascii_case("color")
            .then(|| normalize_inline_color(value.trim_matches(['"', '\'']).trim()).to_owned())
    })
}

pub(super) fn inline_color_argument(argument: &str) -> Option<String> {
    let color = argument.trim();
    (!color.is_empty()).then(|| normalize_inline_color(color).to_owned())
}

pub(super) fn inline_colon_color(tag: &str) -> Option<String> {
    let (_, color) = tag.split_once(':')?;
    inline_color_argument(color)
}

pub(super) fn inline_colon_number(tag: &str) -> Option<u16> {
    let (_, number) = tag.split_once(':')?;
    number.trim().parse().ok()
}

pub(super) fn normalize_inline_color(color: &str) -> &str {
    match color.to_ascii_lowercase().as_str() {
        "red" => "#FF0000",
        "blue" => "#0000FF",
        "green" => "#008000",
        "black" => "#000000",
        "white" => "#FFFFFF",
        "purple" => "#800080",
        _ => color,
    }
}

pub(super) fn push_inline_text_span(
    spans: &mut Vec<InlineTextSpan>,
    buffer: &mut String,
    style: &InlineTextStyle,
) {
    if buffer.is_empty() {
        return;
    }
    spans.push(InlineTextSpan {
        text: std::mem::take(buffer),
        style: style.clone(),
    });
}
