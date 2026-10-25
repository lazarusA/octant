//! Values said in a command: the parts of dates in any language's form
//! (years, month names, days; `dates.rs` joins them into ISO dates) and
//! numbers (`45.5`, `45,5`, `-30`), each with where it was said.

use std::ops::Range;

use crate::app::actions::text::{Segment, is_cjk, joins, overlaps};

/// Markers following a day in Chinese, Japanese and Korean dates.
const DAY_MARKERS: [&str; 2] = ["日", "일"];
/// Markers of the year-first order of Chinese, Japanese and Korean dates.
pub(super) const YEAR_MARKERS: [char; 2] = ['年', '년'];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Part {
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

/// Whether a number is said at most two words from month name `m` ("15 de
/// enero de 2020"), so the "may" of "may I see sst in 2020" is no month.
fn by_number(segment: &Segment, m: &Range<usize>) -> bool {
    let (words, text) = (&segment.words, &segment.text);
    let month = |k: usize| words.get(k).is_some_and(|w| overlaps(m, w));
    (0..words.len()).filter(|&j| !month(j)).any(|j| {
        number_word(&text[words[j].clone()]).is_some() && (j.saturating_sub(2)..=j + 2).any(month)
    })
}

/// The month names beside a number, or (through them) at most two words
/// from another kept month ("January to March 2020").
fn kept_months(segment: &Segment, months: &mut Vec<(Range<usize>, Part)>) {
    let word_at = |r: &Range<usize>| segment.words.iter().position(|w| overlaps(r, w));
    let mut kept: Vec<bool> = months.iter().map(|(m, _)| by_number(segment, m)).collect();
    while let Some(i) = (0..months.len()).find(|&i| {
        !kept[i]
            && (0..months.len()).any(|j| {
                let (a, b) = (word_at(&months[i].0), word_at(&months[j].0));
                kept[j] && a.zip(b).is_some_and(|(a, b)| a.abs_diff(b) <= 2)
            })
    }) {
        kept[i] = true;
    }
    let mut keep = kept.into_iter();
    months.retain(|_| keep.next().unwrap_or(false));
}

/// Whether the word at `r` stands alone: no `_` joins it to another, so
/// the 1000 of `z_1000` is part of a name (words already split `t2m` from
/// `2020年`).
fn alone(text: &str, r: &Range<usize>) -> bool {
    !text[..r.start].ends_with('_') && !text[r.end..].starts_with('_')
}

/// The date parts said outside `taken`, in order: month names beside a
/// number, years (1000-2999) and days (1-31 at most a word from a month
/// name, before a day marker, or after one of `until` after such a day).
pub(super) fn parts(
    segment: &Segment,
    months: &[&[&str]; 12],
    until: &[&str],
    taken: &[Range<usize>],
) -> Vec<(Range<usize>, Part)> {
    let mut found = month_spans(segment, months, taken);
    kept_months(segment, &mut found);
    let words = &segment.words;
    let month_word = |k: usize| {
        words
            .get(k)
            .is_some_and(|w| found.iter().any(|(m, _)| overlaps(m, w)))
    };
    let (mut numbers, mut loose) = (Vec::new(), Vec::new());
    for (k, word) in words.iter().enumerate() {
        let Some(n) = number_word(&segment.text[word.clone()]) else {
            continue;
        };
        if taken.iter().any(|t| overlaps(t, word)) || month_word(k) || !alone(&segment.text, word) {
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
        } else if (1..=31).contains(&n) {
            loose.push((k, n));
        }
    }
    days_across(segment, until, &loose, &mut numbers);
    found.extend(numbers);
    found.sort_by_key(|(r, _)| r.start);
    found
}

/// The `loose` numbers (word index, value) said after one of `until`
/// that follows a day in `found` ("January 15 to 20"), which are days too
/// though no month name stands beside them. Never before a day: the 10 of
/// "level 10 to 15 January" is the level's.
fn days_across(
    segment: &Segment,
    until: &[&str],
    loose: &[(usize, u32)],
    found: &mut Vec<(Range<usize>, Part)>,
) {
    let words = &segment.words;
    let between = |k: usize| {
        words
            .get(k)
            .is_some_and(|w| until.contains(&&segment.text[w.clone()]))
    };
    for &(k, n) in loose {
        let day = |d: usize| {
            let at = words.get(d);
            found
                .iter()
                .any(|(r, p)| Some(r) == at && matches!(p, Part::Day(_)))
        };
        if k.checked_sub(2).is_some_and(|d| between(k - 1) && day(d)) {
            found.push((words[k].clone(), Part::Day(n)));
        }
    }
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
        // `i` is on a character boundary: it only ever moves by characters
        // or over ASCII digits.
        let alone_before = !text[..i].chars().next_back().is_some_and(joins);
        if !(starts && alone_before) {
            i += text[i..].chars().next().map_or(1, char::len_utf8);
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
        let alone_after = !text[i..].chars().next().is_some_and(joins);
        let span = start..i;
        if alone_after && !taken.iter().any(|t| overlaps(t, &span)) {
            out.push((text[span.clone()].replace(',', "."), span));
        }
    }
    out
}
