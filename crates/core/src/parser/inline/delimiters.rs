use super::*;

pub(super) fn collect_strikethrough_runs(
    source: &str,
) -> BTreeMap<usize, (InlineDelimiterToken, usize)> {
    let mut open: Option<(usize, usize)> = None;
    let mut runs = BTreeMap::new();
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next().expect("valid UTF-8");
        if ch == '\\' {
            if let Some(next) = source[index + ch.len_utf8()..].chars().next() {
                if is_ascii_punctuation(next) {
                    index += ch.len_utf8() + next.len_utf8();
                    continue;
                }
            }
        }
        if ch == '`' && (index == 0 || source.as_bytes()[index - 1] != b'`') {
            if let Some((consumed, _)) = parse_code_span(&source[index..]) {
                index += consumed;
                continue;
            }
        }
        if ch == '<' {
            if let Some((consumed, _)) = parse_autolink_or_inline_html(&source[index..], true) {
                index += consumed;
                continue;
            }
        }
        if source[index..].starts_with("~~") {
            let len = delimiter_run_len(source, index, b'~');
            if len > 2 {
                open = None;
                index += len;
                continue;
            }
            if let Some((open_index, open_len)) = open.take() {
                if open_len == len {
                    runs.insert(
                        open_index,
                        (InlineDelimiterToken::StrikethroughOpen, open_len),
                    );
                    runs.insert(index, (InlineDelimiterToken::StrikethroughClose, len));
                } else {
                    open = Some((index, len));
                }
            } else {
                open = Some((index, len));
            }
            index += len;
            continue;
        }
        if ch == '~' {
            if let Some((open_index, open_len)) = open.take() {
                if open_len == 1 {
                    runs.insert(
                        open_index,
                        (InlineDelimiterToken::StrikethroughOpen, open_len),
                    );
                    runs.insert(index, (InlineDelimiterToken::StrikethroughClose, 1));
                } else {
                    open = Some((index, 1));
                }
            } else {
                open = Some((index, 1));
            }
            index += 1;
            continue;
        }
        index += ch.len_utf8();
    }
    runs
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EmphasisTag {
    Em,
    Strong,
}

#[derive(Clone, Copy)]
pub(super) enum EmphasisRunEvent {
    Open(EmphasisTag),
    Close(EmphasisTag),
}

#[derive(Default)]
pub(super) struct EmphasisRunRender {
    pub(super) len: usize,
    pub(super) events: Vec<(usize, usize, EmphasisRunEvent)>,
}

#[derive(Clone)]
pub(super) struct EmphasisRun {
    pub(super) index: usize,
    pub(super) len: usize,
    pub(super) remaining_len: usize,
    pub(super) marker: u8,
    pub(super) can_open: bool,
    pub(super) can_close: bool,
}

pub(super) fn collect_emphasis_runs(
    source: &str,
    link_references: &BTreeMap<String, LinkReference>,
) -> BTreeMap<usize, EmphasisRunRender> {
    let mut runs = scan_emphasis_runs(source, link_references);
    let mut render_runs: BTreeMap<usize, EmphasisRunRender> = BTreeMap::new();

    for closer_index in 0..runs.len() {
        while runs[closer_index].remaining_len > 0 && runs[closer_index].can_close {
            let Some(opener_index) = matching_emphasis_run(&runs, closer_index) else {
                break;
            };
            let used_len =
                if runs[opener_index].remaining_len >= 2 && runs[closer_index].remaining_len >= 2 {
                    2
                } else {
                    1
                };
            let tag = if used_len == 2 {
                EmphasisTag::Strong
            } else {
                EmphasisTag::Em
            };
            let opener_offset = runs[opener_index].remaining_len - used_len;
            let closer_offset = runs[closer_index].len - runs[closer_index].remaining_len;

            render_runs
                .entry(runs[opener_index].index)
                .or_insert_with(|| EmphasisRunRender {
                    len: runs[opener_index].len,
                    events: Vec::new(),
                })
                .events
                .push((opener_offset, used_len, EmphasisRunEvent::Open(tag)));
            render_runs
                .entry(runs[closer_index].index)
                .or_insert_with(|| EmphasisRunRender {
                    len: runs[closer_index].len,
                    events: Vec::new(),
                })
                .events
                .push((closer_offset, used_len, EmphasisRunEvent::Close(tag)));

            runs[opener_index].remaining_len -= used_len;
            runs[closer_index].remaining_len -= used_len;
            for run in &mut runs[opener_index + 1..closer_index] {
                run.can_open = false;
            }
        }
    }

    for run in render_runs.values_mut() {
        run.events.sort_by_key(|(offset, _, event)| {
            let order = match event {
                EmphasisRunEvent::Close(_) => 0usize,
                EmphasisRunEvent::Open(_) => 1,
            };
            (*offset, order)
        });
    }

    render_runs
}

pub(super) fn scan_emphasis_runs(
    source: &str,
    link_references: &BTreeMap<String, LinkReference>,
) -> Vec<EmphasisRun> {
    let bytes = source.as_bytes();
    let mut runs = Vec::new();
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next().expect("valid UTF-8");
        if ch == '\\' {
            if let Some(next) = source[index + ch.len_utf8()..].chars().next() {
                if is_ascii_punctuation(next) {
                    index += ch.len_utf8() + next.len_utf8();
                    continue;
                }
            }
        }
        if ch == '`' && (index == 0 || bytes[index - 1] != b'`') {
            if let Some((consumed, _)) = parse_code_span(&source[index..]) {
                index += consumed;
                continue;
            }
        }
        if ch == '[' {
            if let Some((consumed, _, _, _)) =
                parse_link_or_image_with_options(&source[index..], true, &BTreeMap::new())
            {
                index += consumed;
                continue;
            }
            if let Some((consumed, _, _)) =
                parse_reference_link_or_image(&source[index..], link_references, true)
            {
                index += consumed;
                continue;
            }
        }
        if ch == '<' {
            if let Some((consumed, _)) = parse_autolink_or_inline_html(&source[index..], false) {
                index += consumed;
                continue;
            }
        }
        if !matches!(bytes[index], b'*' | b'_') {
            index += ch.len_utf8();
            continue;
        }

        let marker = bytes[index];
        let len = delimiter_run_len(source, index, marker);
        let delimiter = classify_emphasis_delimiter(source, index, len, marker, None);
        runs.push(EmphasisRun {
            index,
            len,
            remaining_len: len,
            marker,
            can_open: delimiter.can_open,
            can_close: delimiter.can_close,
        });
        index += len;
    }
    runs
}

pub(super) fn matching_emphasis_run(runs: &[EmphasisRun], closer_index: usize) -> Option<usize> {
    let closer = &runs[closer_index];
    (0..closer_index).rev().find(|opener_index| {
        let opener = &runs[*opener_index];
        opener.marker == closer.marker
            && opener.can_open
            && opener.remaining_len > 0
            && emphasis_runs_can_match(opener, closer)
    })
}

pub(super) fn emphasis_runs_can_match(opener: &EmphasisRun, closer: &EmphasisRun) -> bool {
    if opener.can_close || closer.can_open {
        let sum = opener.len + closer.len;
        if sum.is_multiple_of(3) && !opener.len.is_multiple_of(3) && !closer.len.is_multiple_of(3) {
            return false;
        }
    }
    true
}

pub(super) fn push_emphasis_run(
    out: &mut InlineBuilder<'_>,
    run: &EmphasisRunRender,
    mapped: &MappedText,
    start: usize,
) {
    let mut event_index = 0usize;
    let mut consumed_until = 0usize;
    for offset in 0..run.len {
        while event_index < run.events.len() && run.events[event_index].0 == offset {
            consumed_until = consumed_until.max(offset + run.events[event_index].1);
            let delimiter = match run.events[event_index].2 {
                EmphasisRunEvent::Open(EmphasisTag::Em) => InlineDelimiterToken::EmphasisOpen,
                EmphasisRunEvent::Open(EmphasisTag::Strong) => InlineDelimiterToken::StrongOpen,
                EmphasisRunEvent::Close(EmphasisTag::Em) => InlineDelimiterToken::EmphasisClose,
                EmphasisRunEvent::Close(EmphasisTag::Strong) => InlineDelimiterToken::StrongClose,
            };
            out.push(InlineToken::Delimiter(
                delimiter,
                start + offset..start + offset + run.events[event_index].1,
            ));
            event_index += 1;
        }
        if offset >= consumed_until {
            push_text_token(out, mapped.slice(start + offset..start + offset + 1));
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct EmphasisDelimiter {
    pub(super) can_open: bool,
    pub(super) can_close: bool,
}

pub(super) fn classify_emphasis_delimiter(
    input: &str,
    index: usize,
    len: usize,
    marker: u8,
    initial_before: Option<char>,
) -> EmphasisDelimiter {
    let before = if index == 0 {
        initial_before
    } else {
        input[..index].chars().next_back()
    };
    let after = input[index + len..].chars().next();
    let left = is_left_flanking_delimiter(before, after);
    let right = is_right_flanking_delimiter(before, after);
    let (can_open, can_close) = if marker == b'_' {
        (
            left && (!right || before.is_some_and(is_unicode_punctuation)),
            right && (!left || after.is_some_and(is_unicode_punctuation)),
        )
    } else {
        (left, right)
    };

    EmphasisDelimiter {
        can_open,
        can_close,
    }
}

pub(super) fn delimiter_run_len(input: &str, index: usize, marker: u8) -> usize {
    input.as_bytes()[index..]
        .iter()
        .take_while(|byte| **byte == marker)
        .count()
}

pub(super) fn is_left_flanking_delimiter(before: Option<char>, after: Option<char>) -> bool {
    let Some(after) = after else {
        return false;
    };
    !after.is_whitespace()
        && (!is_unicode_punctuation(after)
            || before.is_none_or(|ch| ch.is_whitespace() || is_unicode_punctuation(ch)))
}

pub(super) fn is_right_flanking_delimiter(before: Option<char>, after: Option<char>) -> bool {
    let Some(before) = before else {
        return false;
    };
    !before.is_whitespace()
        && (!is_unicode_punctuation(before)
            || after.is_none_or(|ch| ch.is_whitespace() || is_unicode_punctuation(ch)))
}

pub(super) fn is_unicode_punctuation(ch: char) -> bool {
    is_ascii_punctuation(ch) || (!ch.is_alphanumeric() && !ch.is_whitespace())
}
