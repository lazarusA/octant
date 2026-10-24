//! Spoken or typed commands to actions, offline, in every UI language. Each
//! language's table gives the verbs' phrasings; what they act on is found
//! among the dataset's own names (`resolve::Vocabulary`), whatever the
//! language, as data names are never translated. Whisper writes each
//! utterance in the language spoken, which may not be the UI's: the UI
//! language's table is tried first, then the others.

#[cfg(test)]
mod ask_tests;
mod de;
mod dims;
#[cfg(test)]
mod dims_tests;
mod en;
mod es;
mod fi;
mod fr;
mod id;
mod it;
mod ja;
mod ko;
mod layer_words;
#[cfg(test)]
mod layers_tests;
mod named;
mod no;
#[cfg(test)]
mod open_tests;
mod pt;
mod ru;
mod segment;
mod sv;
mod table;
#[cfg(test)]
mod tests;
mod uk;
mod values;
mod zh;

use std::ops::Range;

use super::AppAction;
use super::resolve::Vocabulary;
use super::text::{Name, Segment, overlaps};
use crate::i18n::Language;
use table::{AXES, Phrases, table};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verb {
    Open,
    /// Moves a dimension to a value said with it.
    Goto,
    /// Asks what a variable, the plot or the dataset is.
    Describe,
    /// Searches the variables.
    Search,
    /// Lists what the dataset (or a group) holds.
    List,
    /// Hides a layer.
    Hide,
    /// Shows a hidden layer again.
    Reveal,
    /// Removes an overlay.
    Remove,
    /// A layer's color range, with two numbers.
    Range,
    /// Opens the export dialog.
    Export,
    Select,
    Plot,
    Play,
    Pause,
    Next,
    Previous,
    Colormap,
    ResetView,
}

/// One command of an utterance: an action, or text no table understood.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum Parsed {
    Action(AppAction),
    NotUnderstood(String),
}

/// The commands in `utterance`, each with the language it is read in: the
/// first language whose verbs it holds (`first`, then the others); without
/// any verb, `first`, its names read alone.
pub(super) fn commands(utterance: &str, first: Language) -> Vec<(Language, Segment)> {
    let others = Language::ALL.iter().copied().filter(|&l| l != first);
    let lang = std::iter::once(first)
        .chain(others)
        .find(|&lang| {
            let phrases = table(lang);
            split(utterance, phrases)
                .iter()
                .any(|s| !verbs_in(phrases, s).is_empty())
        })
        .unwrap_or(first);
    split(utterance, table(lang))
        .into_iter()
        .map(|s| (lang, s))
        .collect()
}

/// One command's actions, its names looked up in `vocab`.
pub(super) fn parse_command(lang: Language, segment: &Segment, vocab: &Vocabulary) -> Vec<Parsed> {
    segment::parse_segment(segment, table(lang), vocab)
}

/// Every command of `utterance` read against one `vocab`.
#[cfg(test)]
pub(super) fn parse(utterance: &str, vocab: &Vocabulary, first: Language) -> Vec<Parsed> {
    commands(utterance, first)
        .iter()
        .flat_map(|(lang, s)| parse_command(*lang, s, vocab))
        .collect()
}

/// Sentence punctuation that ends a command, `before` being the text before
/// it: a full stop or comma only before a space or the end (`45.5` and
/// `45,5` stay whole), and no full stop after a day ("15. Januar").
fn ends_command(c: char, before: &str, next: Option<char>) -> bool {
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
    ) || (matches!(c, '.' | ',') && next.is_none_or(char::is_whitespace));
    ends && !after_day
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
        if ends_command(c, &utterance[start..i], chars.peek().map(|&(_, n)| n)) {
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
        cut_after_open(phrases, segment, &mut out);
    }
    out.retain(|s| !s.words.is_empty());
    out
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
fn earliest(segment: &Segment, names: &[Name]) -> Option<Range<usize>> {
    names
        .iter()
        .filter_map(|n| segment.find(n))
        .min_by_key(|r| (r.start, std::cmp::Reverse(r.end)))
}

/// The verbs in `segment` in order, overlapping phrasings ("next", "next
/// step") read once as the longest.
fn verbs_in(phrases: &Phrases, segment: &Segment) -> Vec<(Range<usize>, Verb)> {
    let mut found: Vec<(Range<usize>, Verb)> = phrases
        .verbs
        .iter()
        .flat_map(|(verb, says)| says.iter().map(move |p| (verb, p)))
        .filter_map(|(verb, p)| segment.find(&Name::new(p)).map(|r| (r, *verb)))
        .collect();
    found.sort_by_key(|(r, _)| (r.start, std::cmp::Reverse(r.end)));
    let mut kept: Vec<(Range<usize>, Verb)> = Vec::new();
    for (r, verb) in found {
        if !kept.iter().any(|(k, _)| overlaps(k, &r)) {
            kept.push((r, verb));
        }
    }
    kept
}

/// Where one of `words` is said outside `taken`.
fn word_of(segment: &Segment, words: &[&str], taken: &[Range<usize>]) -> Option<Range<usize>> {
    words
        .iter()
        .filter_map(|w| segment.find(&Name::new(w)))
        .find(|s| !taken.iter().any(|t| overlaps(t, s)))
}
