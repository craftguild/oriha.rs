use super::*;

trait ChargeWork {
    fn charge(&mut self, bytes: usize);
}

mod work_defaults {
    #[derive(Default)]
    pub(crate) struct WorkCounter {}

    impl super::ChargeWork for WorkCounter {
        #[inline]
        fn charge(&mut self, _bytes: usize) {}
    }
}

#[allow(unused_imports)]
use work_defaults::*;
#[cfg(test)]
use work_tests::WorkCounter;

/// One lexical window is indexed at a time. The inline cursor only moves forward.
/// Failed candidates at successive `{`/`(` boundaries reuse suffix validation;
/// no-delimiter input and invalid long tokens are scanned a bounded number of times.
pub(super) struct ExtendedAutolinks<'a> {
    pub(super) source: &'a str,
    pub(super) end: usize,
    pub(super) trimmed_end: usize,
    pub(super) trim_ends: Vec<usize>,
    pub(super) paren_offset: usize,
    pub(super) paren_excess: isize,
    pub(super) mailto_end: usize,
    pub(super) last_at: Option<usize>,
    pub(super) email: Option<std::ops::Range<usize>>,
    pub(super) mailto: Option<std::ops::Range<usize>>,
    pub(super) authority_end: usize,
    pub(super) authority: Option<std::ops::Range<usize>>,
    pub(super) work: WorkCounter,
}

impl<'a> ExtendedAutolinks<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self {
            source,
            end: 0,
            trimmed_end: 0,
            trim_ends: Vec::new(),
            paren_offset: 0,
            paren_excess: 0,
            mailto_end: 0,
            last_at: None,
            email: None,
            mailto: None,
            authority_end: 0,
            authority: None,
            work: WorkCounter::default(),
        }
    }

    pub(super) fn charge(&mut self, bytes: usize) {
        self.work.charge(bytes);
    }

    pub(super) fn window(&mut self, start: usize) {
        if start < self.end {
            let removed = &self.source[self.paren_offset..start];
            self.paren_excess -= paren_excess(removed);
            self.charge(removed.len());
            self.paren_offset = start;
            self.trimmed_end =
                self.trim_ends[(self.paren_excess.max(0) as usize).min(self.trim_ends.len() - 1)];
            return;
        }
        let input = &self.source[start..];
        let len = input
            .char_indices()
            .find_map(|(i, ch)| (!is_extended_autolink_char(ch)).then_some(i))
            .unwrap_or(input.len());
        self.end = start + len;
        let entity_end = start + trim_trailing_entity_reference(&input[..len]).len();
        self.trim_ends.clear();
        let mut end = entity_end;
        loop {
            end = start
                + self.source[start..end]
                    .trim_end_matches(['.', ',', ':', ';', '?', '!', ']'])
                    .len();
            self.trim_ends.push(end);
            if !self.source[start..end].ends_with(')') {
                break;
            }
            end -= 1;
        }
        self.paren_offset = start;
        self.paren_excess = paren_excess(&self.source[start..entity_end]);
        self.trimmed_end =
            self.trim_ends[(self.paren_excess.max(0) as usize).min(self.trim_ends.len() - 1)];
        self.mailto_end = start + trim_mailto_email_end(&input[..len]).len();
        self.last_at = input[..len].rfind('@').map(|i| start + i);
        self.email = email_suffix(self.source, start, *self.trim_ends.last().unwrap());
        self.mailto = if input[..len].ends_with(['-', '_']) {
            None
        } else {
            email_suffix(self.source, start, self.mailto_end)
        };
        // Conservatively account all passes, including email validation and trimming.
        self.charge(12 * (len + 1));
        self.authority_end = start;
        self.authority = None;
    }

    pub(super) fn authority(&mut self, start: usize) -> Option<&std::ops::Range<usize>> {
        if start >= self.authority_end {
            // Successive authority windows do not overlap: skip past the preceding
            // slash before scanning again. The cached suffix also handles many
            // scheme-looking starts before the same slash without rescanning.
            let limit = (*self.trim_ends.last().unwrap()).max(start);
            let end = self.source[start..limit]
                .find('/')
                .map_or(limit, |i| start + i);
            self.authority_end = end;
            self.authority = email_suffix(self.source, start, end);
            self.charge(4 * (end - start + 1));
        }
        self.authority.as_ref()
    }

    pub(super) fn parse_extended_autolink(&mut self, start: usize) -> Option<(usize, InlineKind)> {
        if !self.source[start..]
            .chars()
            .next()
            .is_some_and(is_extended_autolink_char)
        {
            return None;
        }
        self.window(start);
        let input = &self.source[start..self.end];
        let (end, prefix) = if input.starts_with("www.") {
            if !self.source[start..self.trimmed_end].contains('.') {
                return None;
            }
            (self.trimmed_end, "http://")
        } else {
            // A scheme has at most 32 ASCII bytes. Never search the rest of a
            // document (or even an entire long token) to disprove a scheme.
            let colon = input.bytes().take(33).position(|b| b == b':');
            self.charge(2 * input.len().min(33));
            if let Some(colon) = colon.filter(|&i| is_autolink_scheme(&input[..i])) {
                let scheme = &input[..colon];
                let rest = start + colon + 1;
                if rest >= self.end {
                    return None;
                }
                if scheme.eq_ignore_ascii_case("mailto") {
                    let email = self.mailto.as_ref()?;
                    if rest < email.start || rest >= self.last_at? {
                        return None;
                    }
                    (self.mailto_end, "")
                } else if scheme.eq_ignore_ascii_case("xmpp") {
                    if self.source[rest..self.end].starts_with("//") {
                        return None;
                    }
                    let email = self.authority(rest)?.clone();
                    if rest < email.start
                        || rest >= self.source[email.clone()].rfind('@')? + email.start
                    {
                        return None;
                    }
                    // Preserve the existing email-scheme path rule (one segment).
                    let slash = self.authority_end;
                    let end = if slash < self.trimmed_end {
                        let tail = &self.source[slash + 1..self.trimmed_end];
                        let end = tail.find('/').map_or(self.trimmed_end, |i| slash + 1 + i);
                        // A successful candidate consumes the examined tail. Failed
                        // candidates return before this scan.
                        self.charge(end - slash);
                        end
                    } else {
                        self.trimmed_end
                    };
                    (end, "")
                } else if (scheme.eq_ignore_ascii_case("http")
                    || scheme.eq_ignore_ascii_case("https"))
                    && self.source[rest..self.end].starts_with("//")
                    && rest + 2 < self.trimmed_end
                {
                    (self.trimmed_end, "")
                } else {
                    // Scheme-shaped identifiers such as Std::Lang are ordinary
                    // text here. Explicit <scheme:...> autolinks remain separate.
                    return None;
                }
            } else {
                let email = self.email.as_ref()?;
                if start < email.start || start >= self.last_at? {
                    return None;
                }
                (email.end, "mailto:")
            }
        };
        if end <= start {
            return None;
        }
        let text = &self.source[start..end];
        Some((
            end - start,
            InlineKind::Link {
                destination: format!("{prefix}{text}"),
                title: None,
                children: vec![InlineKind::Text(text.to_string()).into()],
            },
        ))
    }
}

/// All starts within the final valid local-part suffix have the same valid domain.
/// Index that suffix once, including the invalid-domain/no-at cases.
pub(super) fn email_suffix(
    source: &str,
    start: usize,
    end: usize,
) -> Option<std::ops::Range<usize>> {
    let text = &source[start..end];
    let at = text.rfind('@')?;
    let local_start = text[..at]
        .char_indices()
        .rev()
        .find_map(|(i, ch)| (!is_email_local_char(ch)).then_some(i + ch.len_utf8()))
        .unwrap_or(0);
    is_autolink_email(&text[local_start..]).then_some(start + local_start..end)
}

pub(super) fn is_email_local_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(ch)
}

pub(super) fn paren_excess(value: &str) -> isize {
    value.bytes().fold(0isize, |balance, b| {
        balance + isize::from(b == b')') - isize::from(b == b'(')
    })
}

pub(super) fn trim_trailing_entity_reference(value: &str) -> &str {
    if !value.ends_with(';') {
        return value;
    }
    let Some(amp) = value.rfind('&') else {
        return value;
    };
    let entity = &value[amp + 1..value.len() - 1];
    if !entity.is_empty() && entity.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        &value[..amp]
    } else {
        value
    }
}

pub(super) fn trim_mailto_email_end(value: &str) -> &str {
    let mut end = value.len();
    while let Some(ch) = value[..end].chars().next_back() {
        if matches!(ch, '.' | '/') {
            end -= ch.len_utf8();
        } else {
            break;
        }
    }
    &value[..end]
}

pub(super) fn is_extended_autolink_char(ch: char) -> bool {
    !ch.is_whitespace() && !matches!(ch, '<' | '>' | '"' | '\'')
}

pub(super) fn is_extended_autolink_boundary(ch: Option<char>) -> bool {
    ch.is_none_or(|ch| ch.is_whitespace() || matches!(ch, '(' | '[' | '{' | '<'))
}

#[cfg(test)]
mod work_tests {
    pub(crate) type WorkCounter = usize;

    impl super::ChargeWork for WorkCounter {
        fn charge(&mut self, bytes: usize) {
            super::ChargeWork::charge(&mut super::work_defaults::WorkCounter::default(), bytes);
            *self += bytes;
        }
    }
}
