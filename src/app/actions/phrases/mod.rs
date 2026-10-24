//! Spoken or typed commands to actions, offline, in every UI language. Each
//! language's table gives the verbs' phrasings; what they act on is found
//! among the dataset's own names (`resolve::Vocabulary`), whatever the
//! language, as data names are never translated. Whisper writes each
//! utterance in the language spoken, which may not be the UI's: the UI
//! language's table is tried first, then the others.

mod de;
mod en;
mod es;
mod fi;
mod fr;
mod id;
mod it;
mod ja;
mod ko;
mod no;
mod pt;
mod ru;
mod segment;
mod sv;
#[cfg(test)]
mod tests;
mod uk;
mod zh;

use std::ops::Range;

use super::AppAction;
use super::resolve::Vocabulary;
use super::text::{Name, Segment};
use crate::i18n::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Verb {
    Select,
    Plot,
    Play,
    Pause,
    Next,
    Previous,
    Colormap,
    ResetView,
}

/// A language's phrasings of each verb, and the words that end one command
/// and start the next ("then").
pub(super) struct Phrases {
    pub verbs: &'static [(Verb, &'static [&'static str])],
    pub then: &'static [&'static str],
}

pub(super) fn table(lang: Language) -> &'static Phrases {
    match lang {
        Language::English => &en::PHRASES,
        Language::Spanish => &es::PHRASES,
        Language::French => &fr::PHRASES,
        Language::German => &de::PHRASES,
        Language::Portuguese => &pt::PHRASES,
        Language::Italian => &it::PHRASES,
        Language::Indonesian => &id::PHRASES,
        Language::Swedish => &sv::PHRASES,
        Language::Norwegian => &no::PHRASES,
        Language::Finnish => &fi::PHRASES,
        Language::Ukrainian => &uk::PHRASES,
        Language::Russian => &ru::PHRASES,
        Language::ChineseSimp => &zh::PHRASES,
        Language::Japanese => &ja::PHRASES,
        Language::Korean => &ko::PHRASES,
    }
}

/// One command of an utterance: an action, or text no table understood.
#[derive(Debug, Clone, PartialEq)]
pub enum Parsed {
    Action(AppAction),
    NotUnderstood(String),
}

/// The commands in `utterance`, read with the first language whose verbs
/// it holds (`first`, then the others); without any verb, by the names it
/// holds alone.
pub fn parse(utterance: &str, vocab: &Vocabulary, first: Language) -> Vec<Parsed> {
    let others = Language::ALL.iter().copied().filter(|&l| l != first);
    for lang in std::iter::once(first).chain(others) {
        let phrases = table(lang);
        let segments = split(utterance, phrases);
        if segments.iter().any(|s| !verbs_in(phrases, s).is_empty()) {
            return parse_segments(&segments, phrases, vocab);
        }
    }
    let phrases = table(first);
    parse_segments(&split(utterance, phrases), phrases, vocab)
}

fn parse_segments(segments: &[Segment], phrases: &Phrases, vocab: &Vocabulary) -> Vec<Parsed> {
    segments
        .iter()
        .flat_map(|s| segment::parse_segment(s, phrases, vocab))
        .collect()
}

/// Sentence punctuation that ends a command (a full stop only before a
/// space or the end, so `45.5` stays whole).
fn ends_command(c: char, next: Option<char>) -> bool {
    matches!(
        c,
        ',' | ';'
            | '!'
            | '?'
            | '\u{3001}'
            | '\u{3002}'
            | '\u{FF0C}'
            | '\u{FF01}'
            | '\u{FF1F}'
            | '\u{FF1B}'
    ) || (c == '.' && next.is_none_or(char::is_whitespace))
}

/// `utterance` cut at sentence punctuation and the language's "then" words
/// into commands holding words.
fn split(utterance: &str, phrases: &Phrases) -> Vec<Segment> {
    let then: Vec<Name> = phrases.then.iter().map(|p| Name::new(p)).collect();
    let mut out = Vec::new();
    let mut chars = utterance.char_indices().peekable();
    let mut start = 0;
    let mut pieces = Vec::new();
    while let Some((i, c)) = chars.next() {
        if ends_command(c, chars.peek().map(|&(_, n)| n)) {
            pieces.push(&utterance[start..i]);
            start = i + c.len_utf8();
        }
    }
    pieces.push(&utterance[start..]);
    for piece in pieces {
        let mut segment = Segment::new(piece);
        while let Some(span) = earliest(&segment, &then) {
            out.push(Segment::new(&segment.text[..span.start]));
            segment = Segment::new(&segment.text[span.end..]);
        }
        out.push(segment);
    }
    out.retain(|s| !s.words.is_empty());
    out
}

/// The first of `names` in `segment`, the longest where several start alike.
fn earliest(segment: &Segment, names: &[Name]) -> Option<Range<usize>> {
    names
        .iter()
        .filter_map(|n| segment.find(n))
        .min_by_key(|r| (r.start, std::cmp::Reverse(r.end)))
}

/// The verbs in `segment` in order, overlapping phrasings ("next", "next
/// step") read once as the longest.
pub(super) fn verbs_in(phrases: &Phrases, segment: &Segment) -> Vec<(Range<usize>, Verb)> {
    let mut found: Vec<(Range<usize>, Verb)> = phrases
        .verbs
        .iter()
        .flat_map(|(verb, says)| says.iter().map(move |p| (verb, p)))
        .filter_map(|(verb, p)| segment.find(&Name::new(p)).map(|r| (r, *verb)))
        .collect();
    found.sort_by_key(|(r, _)| (r.start, std::cmp::Reverse(r.end)));
    let mut kept: Vec<(Range<usize>, Verb)> = Vec::new();
    for (r, verb) in found {
        if !kept.iter().any(|(k, _)| super::text::overlaps(k, &r)) {
            kept.push((r, verb));
        }
    }
    kept
}
