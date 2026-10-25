//! Spoken or typed commands to actions, offline, in every UI language. Each
//! language's table gives the verbs' phrasings; what they act on is found
//! among the dataset's own names (`resolve::Vocabulary`), whatever the
//! language, as data names are never translated. Whisper writes each
//! utterance in the language spoken, which may not be the UI's: the UI
//! language's table is tried first, then the others.

#[cfg(test)]
mod ask_tests;
mod dates;
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
mod split;
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
use super::text::{Segment, address_in, phrase};
use crate::i18n::Language;
use split::split;
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
/// language whose verbs and "then" words cover most of it (`first` on a
/// tie, as languages share words: Spanish and Portuguese `abre`); without
/// any verb, `first`, its names read alone.
pub(super) fn commands(utterance: &str, first: Language) -> Vec<(Language, Segment)> {
    let others = Language::ALL.iter().copied().filter(|&l| l != first);
    let mut best: Option<(usize, Language, Vec<Segment>)> = None;
    let mut first_segments = None;
    for lang in std::iter::once(first).chain(others) {
        let phrases = table(lang);
        let (segments, then) = split(utterance, phrases);
        if lang == first {
            // Kept for an utterance without verbs (its names read alone).
            first_segments = Some(segments.clone());
        }
        let verbs: usize = segments
            .iter()
            .flat_map(|s| verbs_in(phrases, s))
            .map(|(r, _)| r.len())
            .sum();
        if verbs > 0
            && best
                .as_ref()
                .is_none_or(|(score, ..)| verbs + then > *score)
        {
            best = Some((verbs + then, lang, segments));
        }
    }
    let (lang, segments) = match best {
        Some((_, lang, segments)) => (lang, segments),
        None => (first, first_segments.unwrap_or_default()),
    };
    segments.into_iter().map(|s| (lang, s)).collect()
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

/// The verbs in `segment` in order, overlapping phrasings ("next", "next
/// step") read once as the longest; words of an address
/// (`https://x/play/a.zarr`) are no verbs.
fn verbs_in(phrases: &Phrases, segment: &Segment) -> Vec<(Range<usize>, Verb)> {
    let address: Vec<Range<usize>> = address_in(&segment.text)
        .map(|(_, span)| span)
        .into_iter()
        .collect();
    let mut found: Vec<(Range<usize>, Verb)> = phrases
        .verbs
        .iter()
        .flat_map(|(verb, says)| says.iter().map(move |p| (verb, p)))
        .filter_map(|(verb, p)| segment.find(phrase(p), &address).map(|r| (r, *verb)))
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

/// Where one of `words` is said outside `taken`.
fn word_of(
    segment: &Segment,
    words: &[&'static str],
    taken: &[Range<usize>],
) -> Option<Range<usize>> {
    words.iter().find_map(|w| segment.find(phrase(w), taken))
}
