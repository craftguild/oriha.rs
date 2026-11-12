#[derive(Debug)]
pub(crate) struct ParsedTag<'a> {
    pub(crate) source: &'a str,
    pub(crate) raw_name: String,
    pub(crate) name: String,
    pub(crate) closing: bool,
    pub(crate) self_closing: bool,
    pub(crate) attrs: Vec<ParsedAttr>,
}

#[derive(Debug)]
pub(crate) struct ParsedAttr {
    pub(crate) name: String,
    pub(crate) value: Option<String>,
}

impl<'a> ParsedTag<'a> {
    pub(crate) fn attr_value(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|attr| attr.name.eq_ignore_ascii_case(name))
            .and_then(|attr| attr.value.as_deref())
    }
}

pub(crate) fn parse_tag_at(html: &str, start: usize) -> Option<(usize, Option<ParsedTag<'_>>)> {
    let tail = &html[start..];
    if tail.starts_with("<!--") {
        let end = tail
            .find("-->")
            .map_or(html.len(), |offset| start + offset + 3);
        return Some((end, None));
    }
    if tail.starts_with("<!") || tail.starts_with("<?") {
        let end = find_tag_end(html, start)?;
        return Some((end, None));
    }

    let end = find_tag_end(html, start)?;
    let source = &html[start..end];
    let inner = source.strip_prefix('<')?.strip_suffix('>')?.trim();
    if inner.is_empty() {
        return Some((end, None));
    }

    let closing = inner.starts_with('/');
    let inner = if closing {
        inner[1..].trim_start()
    } else {
        inner
    };
    let self_closing = inner.ends_with('/');
    let inner = if self_closing {
        inner[..inner.len() - 1].trim_end()
    } else {
        inner
    };
    let name_end = inner
        .find(|ch: char| ch.is_ascii_whitespace() || ch == '/')
        .unwrap_or(inner.len());
    if name_end == 0 {
        return Some((end, None));
    }
    let raw_name = inner[..name_end].to_string();
    let name = raw_name.to_ascii_lowercase();
    let attrs = if closing {
        Vec::new()
    } else {
        parse_attrs(&inner[name_end..])
    };

    Some((
        end,
        Some(ParsedTag {
            source,
            raw_name,
            name,
            closing,
            self_closing,
            attrs,
        }),
    ))
}

fn find_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut cursor = start + 1;
    let mut quote = None::<u8>;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        if let Some(quoted) = quote {
            if byte == quoted {
                quote = None;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = Some(byte);
        } else if byte == b'>' {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

fn parse_attrs(input: &str) -> Vec<ParsedAttr> {
    let bytes = input.as_bytes();
    let mut attrs = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }
        let name_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'='
            && bytes[cursor] != b'/'
        {
            cursor += 1;
        }
        if cursor == name_start {
            cursor += 1;
            continue;
        }
        let name = input[name_start..cursor].to_string();
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let value = if cursor < bytes.len() && bytes[cursor] == b'=' {
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor < bytes.len() && (bytes[cursor] == b'"' || bytes[cursor] == b'\'') {
                let quote = bytes[cursor];
                cursor += 1;
                let value_start = cursor;
                while cursor < bytes.len() && bytes[cursor] != quote {
                    cursor += 1;
                }
                let value = input[value_start..cursor].to_string();
                if cursor < bytes.len() {
                    cursor += 1;
                }
                Some(value)
            } else {
                let value_start = cursor;
                while cursor < bytes.len()
                    && !bytes[cursor].is_ascii_whitespace()
                    && bytes[cursor] != b'/'
                {
                    cursor += 1;
                }
                Some(input[value_start..cursor].to_string())
            }
        } else {
            None
        };
        attrs.push(ParsedAttr { name, value });
    }
    attrs
}
