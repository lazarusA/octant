//! An utterance cut into commands: at sentence punctuation (never inside
//! an address), at the language's "then" words, and where a verb follows
//! "open", as what comes after waits for the dataset.

use std::ops::Range;

use super::{Phrases, Verb, verbs_in};
use crate::app::actions::text::{Name, Segment, address_in, is_address, is_cjk, phrase};

/// Sentence punctuation that ends a command, `before` and `after` being the
/// text around it: a full stop or comma only before a space or the end
/// (`45.5` and `45,5` stay whole), no comma before a number ("January 15,
/// 2020") and no full stop after a day ("15. Januar").
fn ends_command(c: char, before: &str, after: &str) -> bool {
    let digits = before.bytes().rev().take_while(u8::is_ascii_digit).count();
    let after_day = c == '.' && (1..=2).contains(&digits) && {
        let rest = &before[..before.len() - digits];
        !rest.chars().next_back().is_some_and(char::is_alphanumeric)
    };
    let ends = matches!(
        c,
        ';' | '!'
            | '?'
            | '\u{3001}'
            | '\u{3002}'
            | '\u{FF0C}'
            | '\u{FF01}'
            | '\u{FF1F}'
            | '\u{FF1B}'
    ) || (matches!(c, '.' | ',')
        && after.chars().next().is_none_or(char::is_whitespace));
    let before_number = c == ',' && after.trim_start().starts_with(|d: char| d.is_ascii_digit());
    ends && !after_day && !before_number
}

/// `utterance` cut at sentence punctuation and the language's "then" words
/// into commands holding words, with the bytes those words took.
pub(super) fn split(utterance: &str, phrases: &Phrases) -> (Vec<Segment>, usize) {
    let then: Vec<&Name> = phrases.then.iter().map(|p| phrase(p)).collect();
    let mut out = Vec::new();
    let mut then_bytes = 0;
    let mut start = 0;
    let mut pieces = Vec::new();
    for (i, c) in utterance.char_indices() {
        if ends_command(c, &utterance[start..i], &utterance[i + c.len_utf8()..])
            // CJK punctuation always ends one: addresses are ASCII.
            && !(c.is_ascii() && in_address(utterance, i))
        {
            pieces.push(&utterance[start..i]);
            start = i + c.len_utf8();
        }
    }
    pieces.push(&utterance[start..]);
    for piece in pieces {
        let mut segment = Segment::new(piece);
        while let Some(span) = earliest(&segment, &then) {
            then_bytes += span.len();
            out.push(Segment::new(&segment.text[..span.start]));
            segment = Segment::new(&segment.text[span.end..]);
        }
        cut_after_open(phrases, segment, &mut out);
    }
    out.retain(|s| !s.words.is_empty());
    (out, then_bytes)
}

/// Whether byte `i` of `text` is inside an address (the `?` of a query
/// string), which punctuation never cuts.
fn in_address(text: &str, i: usize) -> bool {
    // The word's own length: the whitespace before it may be wider than a
    // byte (a no-break or ideographic space).
    let start = i - text[..i]
        .split(char::is_whitespace)
        .next_back()
        .map_or(0, str::len);
    let end = text[i..]
        .find(char::is_whitespace)
        .map_or(text.len(), |w| i + w);
    let word = text[start..end].trim_start_matches(is_cjk);
    is_address(word) && i + 1 < end
}

/// `segment` cut where a verb follows "open": what comes after waits for
/// the dataset, its names unknown until it loads.
fn cut_after_open(phrases: &Phrases, segment: Segment, out: &mut Vec<Segment>) {
    let verbs = verbs_in(phrases, &segment);
    let after_open = verbs
        .windows(2)
        .find(|pair| pair[0].1 == Verb::Open)
        .map(|pair| pair[1].0.start);
    match after_open {
        Some(cut) => {
            out.push(Segment::new(&segment.text[..cut]));
            cut_after_open(phrases, Segment::new(&segment.text[cut..]), out);
        }
        None => out.push(segment),
    }
}

/// The first of `names` in `segment`, the longest where several start alike.
fn earliest(segment: &Segment, names: &[&Name]) -> Option<Range<usize>> {
    let address: Vec<Range<usize>> = address_in(&segment.text)
        .map(|(_, span)| span)
        .into_iter()
        .collect();
    names
        .iter()
        .filter_map(|n| segment.find(n, &address))
        .min_by_key(|r| (r.start, std::cmp::Reverse(r.end)))
}
