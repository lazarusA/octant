//! Values said in a command: dates in any language's form (`2020-01-15`,
//! "15 January 2020", "15 de enero de 2020", `2020年1月15日`) written as
//! ISO (`2020`, `2020-01`, `2020-01-15`), and numbers (`45.5`, `45,5`,
//! `-30`), each with where it was said.

use std::ops::Range;

use crate::app::actions::text::{Segment, is_cjk, overlaps};

/// Markers following a day in Chinese, Japanese and Korean dates.
const DAY_MARKERS: [&str; 2] = ["日", "일"];
/// Markers of the year-first order of Chinese, Japanese and Korean dates.
const YEAR_MARKERS: [char; 2] = ['年', '년'];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Part {
    Year(u32),
    Month(u32),
    Day(u32),
}

/// A number word's value: digits, optionally with an ordinal ending
/// ("15th", "1er").
fn number_word(word: &str) -> Option<u32> {
    let digits = word.bytes().take_while(u8::is_ascii_digit).count();
    let ending = word.len() - digits;
    if digits == 0 || digits > 4 || ending > 2 || !word[digits..].chars().all(char::is_alphabetic) {
        return None;
    }
    word[..digits].parse().ok()
}

/// The month names said outside `taken`, the longest where several
/// overlap (十一月 over 一月).
fn month_spans(
    segment: &Segment,
    months: &[&[&str]; 12],
    taken: &[Range<usize>],
) -> Vec<(Range<usize>, Part)> {
    let mut found: Vec<(Range<usize>, Part)> = Vec::new();
    for (m, names) in (1..).zip(months) {
        for name in *names {
            let mut from = 0;
            while let Some(at) = segment.text[from..].find(name) {
                let span = from + at..from + at + name.len();
                from = span.end;
                let whole = name.chars().any(is_cjk) || segment.words.contains(&span);
                if whole && !taken.iter().any(|t| overlaps(t, &span)) {
                    found.push((span, Part::Month(m)));
                }
            }
        }
    }
    found.sort_by_key(|(r, _)| (r.start, std::cmp::Reverse(r.end)));
    found.dedup_by(|b, a| overlaps(&a.0, &b.0));
    found
}

/// The date parts said outside `taken`, in order: month names, years
/// (1000-2999) and days (1-31 at most a word from a month name, or before
/// a day marker).
fn parts(
    segment: &Segment,
    months: &[&[&str]; 12],
    taken: &[Range<usize>],
) -> Vec<(Range<usize>, Part)> {
    let mut found = month_spans(segment, months, taken);
    let words = &segment.words;
    let month_word = |k: usize| {
        words
            .get(k)
            .is_some_and(|w| found.iter().any(|(m, _)| overlaps(m, w)))
    };
    let mut numbers = Vec::new();
    for (k, word) in words.iter().enumerate() {
        let Some(n) = number_word(&segment.text[word.clone()]) else {
            continue;
        };
        if taken.iter().any(|t| overlaps(t, word)) || month_word(k) {
            continue;
        }
        let marked = words
            .get(k + 1)
            .is_some_and(|w| DAY_MARKERS.contains(&&segment.text[w.clone()]));
        let by_month = (k.saturating_sub(2)..=k + 2).any(|j| j != k && month_word(j));
        if (1000..=2999).contains(&n) {
            numbers.push((word.clone(), Part::Year(n)));
        } else if (1..=31).contains(&n) && (marked || by_month) {
            numbers.push((word.clone(), Part::Day(n)));
        }
    }
    found.extend(numbers);
    found.sort_by_key(|(r, _)| r.start);
    found
}

/// The dates said outside `taken`, as ISO text with their spans. A year
/// takes the month and day said before it ("15 January 2020"), or after it
/// in the year-first order of Chinese, Japanese and Korean.
pub(super) fn dates(
    segment: &Segment,
    months: &[&[&str]; 12],
    taken: &[Range<usize>],
) -> Vec<(String, Range<usize>)> {
    let mut out = iso_dates(segment, taken);
    let mut taken = taken.to_vec();
    taken.extend(out.iter().map(|(_, span)| span.clone()));
    let parts = parts(segment, months, &taken);
    let year_first = segment.text.contains(YEAR_MARKERS);
    let mut group: Vec<(Range<usize>, Part)> = Vec::new();
    let mut flush = |group: &mut Vec<(Range<usize>, Part)>| {
        if let Some(date) = iso(group) {
            out.push(date);
        }
        group.clear();
    };
    for part in parts {
        let is_year = matches!(part.1, Part::Year(_));
        if year_first && is_year {
            flush(&mut group);
        }
        group.push(part);
        if !year_first && is_year {
            flush(&mut group);
        }
    }
    flush(&mut group);
    out.sort_by_key(|(_, span)| span.start);
    out
}

/// Dates written as ISO (`2020-01-15`, `2020-01`, `2020/01/15`) outside
/// `taken`.
fn iso_dates(segment: &Segment, taken: &[Range<usize>]) -> Vec<(String, Range<usize>)> {
    let text = &segment.text;
    let mut out = Vec::new();
    for (start, _) in text.char_indices() {
        let alone = !text[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric);
        if !alone
            || out
                .iter()
                .any(|(_, r): &(String, Range<usize>)| r.contains(&start))
        {
            continue;
        }
        let mut fields = Vec::new();
        let mut end = start;
        for (k, width) in [(0, 4..=4), (1, 1..=2), (2, 1..=2)] {
            let from = if k == 0 {
                end
            } else if matches!(text.as_bytes().get(end), Some(b'-' | b'/')) {
                end + 1
            } else {
                break;
            };
            let digits = text[from..].bytes().take_while(u8::is_ascii_digit).count();
            if !width.contains(&digits) {
                break;
            }
            fields.push(&text[from..from + digits]);
            end = from + digits;
        }
        let alone_after = !text[end..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric);
        let span = start..end;
        if fields.len() >= 2 && alone_after && !taken.iter().any(|t| overlaps(t, &span)) {
            out.push((fields.join("-"), span));
        }
    }
    out
}

/// A group of date parts holding a year as ISO text, with its span.
fn iso(group: &[(Range<usize>, Part)]) -> Option<(String, Range<usize>)> {
    let (mut year, mut month, mut day) = (None, None, None);
    for (_, part) in group {
        match *part {
            Part::Year(y) => year = Some(y),
            Part::Month(m) => month = Some(m),
            Part::Day(d) => day = Some(d),
        }
    }
    let span = group.first()?.0.start..group.last()?.0.end;
    let text = match (year?, month, day) {
        (y, Some(m), Some(d)) => format!("{y:04}-{m:02}-{d:02}"),
        (y, Some(m), None) => format!("{y:04}-{m:02}"),
        (y, None, _) => format!("{y:04}"),
    };
    Some((text, span))
}

/// The numbers said outside `taken`, standing alone (not the 2 of `t2m`),
/// with a point or comma as decimal separator.
pub(super) fn numbers(segment: &Segment, taken: &[Range<usize>]) -> Vec<(String, Range<usize>)> {
    let text = &segment.text;
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let negative = bytes[i] == b'-' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit);
        let starts = bytes[i].is_ascii_digit() || negative;
        let alone_before = i == 0
            || !text[..i]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric);
        if !(starts && alone_before) {
            i += 1;
            continue;
        }
        let start = i;
        i += usize::from(negative);
        let digits = |i: &mut usize| {
            while bytes.get(*i).is_some_and(u8::is_ascii_digit) {
                *i += 1;
            }
        };
        digits(&mut i);
        if matches!(bytes.get(i), Some(b'.' | b','))
            && bytes.get(i + 1).is_some_and(u8::is_ascii_digit)
        {
            i += 1;
            digits(&mut i);
        }
        let alone_after = !text[i..].chars().next().is_some_and(char::is_alphanumeric);
        let span = start..i;
        if alone_after && !taken.iter().any(|t| overlaps(t, &span)) {
            out.push((text[span.clone()].replace(',', "."), span));
        }
    }
    out
}
