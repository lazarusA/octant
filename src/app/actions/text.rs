//! Matching spoken or typed text against names in any script: words are
//! compared with case, spaces and punctuation aside ("T 2 M" is `t2m`,
//! "sea surface temperature" is `sea_surface_temperature`), and names in
//! scripts written without spaces (Chinese, Japanese, Korean) as substrings.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Mutex, OnceLock};

use crate::i18n::number::halfwidth;
use crate::ui::fonts::{Script, script_of};

/// Han, kana and Hangul: written without spaces between words.
pub(super) fn is_cjk(c: char) -> bool {
    script_of(c).is_some_and(|s| s != Script::Other)
}

/// Whether `c` joins the characters around it into one name (`z_1000`,
/// `t2m`): a number touching it is part of a name, not a value.
pub(super) fn joins(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Whether `text` is an address rather than a name (a URL or a path,
/// `C:\Data` too).
pub(super) fn is_address(text: &str) -> bool {
    let text = text.trim();
    let drive = matches!(text.as_bytes(), [d, b':', b'\\' | b'/', ..] if d.is_ascii_alphabetic());
    text.contains("://") || text.starts_with('/') || text.starts_with('~') || drive
}

/// Lowercase, full-width forms (`ｔ２ｍ`, the ideographic space) as ASCII;
/// addresses (`https://x/Data.zarr`) kept as written, whose case matters.
pub(super) fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for piece in text.split_inclusive(char::is_whitespace) {
        if is_address(piece.trim_start_matches(is_cjk)) {
            out.push_str(piece);
            continue;
        }
        out.extend(piece.chars().map(halfwidth).flat_map(char::to_lowercase));
    }
    out
}

/// The first address in normalized `text` (its trailing punctuation and
/// the CJK words touching it, `打开https://x/a.zarr然后播放`, left out) and
/// where it is.
pub(super) fn address_in(text: &str) -> Option<(String, Range<usize>)> {
    let mut start = 0;
    for piece in text.split_inclusive(char::is_whitespace) {
        let word = piece.trim_start();
        let lead = piece.len() - word.trim_start_matches(is_cjk).len();
        let word = word.trim_start_matches(is_cjk).trim_end();
        // CJK after a file name is the next word (`a.zarr然后播放`); after a
        // slash, the name itself (`/数据/温度数据`).
        let cut = word.trim_end_matches(is_cjk);
        let word = if cut.ends_with(|c: char| c.is_ascii_alphanumeric()) {
            cut
        } else {
            word
        };
        let word = word.trim_end_matches(['.', ',', ';', '?', '!']);
        let at = start + lead;
        start += piece.len();
        if !word.is_empty() && is_address(word) {
            return Some((word.to_string(), at..at + word.len()));
        }
    }
    None
}

/// The word spans of normalized `text`: runs of letters and digits, a CJK
/// run apart from what touches it (`t2mを` is `t2m`, `を`).
pub(super) fn word_spans(text: &str) -> Vec<Range<usize>> {
    let mut spans = Vec::new();
    let mut start: Option<(usize, bool)> = None;
    for (i, c) in text.char_indices() {
        let word = c.is_alphanumeric();
        match start {
            Some((s, cjk)) if !word || cjk != is_cjk(c) => {
                spans.push(s..i);
                start = word.then_some((i, is_cjk(c)));
            }
            None if word => start = Some((i, is_cjk(c))),
            _ => {}
        }
    }
    if let Some((s, _)) = start {
        spans.push(s..text.len());
    }
    spans
}

/// A name or phrase prepared for matching.
#[derive(Debug, Clone)]
pub(super) struct Name {
    /// Normalized, for CJK substring search.
    pub norm: String,
    /// Its words joined: what whole-word runs are compared with.
    pub compact: String,
    pub cjk: bool,
}

impl Name {
    pub fn new(text: &str) -> Self {
        let norm = normalize(text.trim());
        let compact = compact(&norm);
        let cjk = norm.chars().any(is_cjk);
        Self { norm, compact, cjk }
    }
}

/// A phrase of a language's table prepared once for the session: the
/// tables are fixed, so each phrase is normalized only the first time.
pub(super) fn phrase(text: &'static str) -> &'static Name {
    static PREPARED: OnceLock<Mutex<HashMap<&'static str, &'static Name>>> = OnceLock::new();
    let prepared = PREPARED.get_or_init(Default::default);
    let mut prepared = prepared.lock().unwrap_or_else(|p| p.into_inner());
    // Leaked once per phrase of the fixed tables: bounded, kept anyway.
    prepared
        .entry(text)
        .or_insert_with(|| Box::leak(Box::new(Name::new(text))))
}

/// The words of normalized `text` joined.
pub(super) fn compact(norm: &str) -> String {
    word_spans(norm).into_iter().map(|r| &norm[r]).collect()
}

/// A normalized utterance (or one command of it) and its words.
#[derive(Clone)]
pub(super) struct Segment {
    pub text: String,
    pub words: Vec<Range<usize>>,
}

impl Segment {
    pub fn new(text: &str) -> Self {
        let text = normalize(text);
        let words = word_spans(&text);
        Self { text, words }
    }

    /// Where `name` first occurs outside `taken`: a substring when it holds
    /// CJK, else a run of whole words that joined equal its words joined.
    pub fn find(&self, name: &Name, taken: &[Range<usize>]) -> Option<Range<usize>> {
        let free = |r: &Range<usize>| !taken.iter().any(|t| overlaps(t, r));
        if name.compact.is_empty() {
            return None;
        }
        if name.cjk {
            return self
                .text
                .match_indices(&name.norm)
                .map(|(at, _)| at..at + name.norm.len())
                .find(free);
        }
        for (i, first) in self.words.iter().enumerate() {
            let mut rest = name.compact.as_str();
            for word in &self.words[i..] {
                let Some(after) = rest.strip_prefix(&self.text[word.clone()]) else {
                    break;
                };
                if after.is_empty() {
                    let span = first.start..word.end;
                    if free(&span) {
                        return Some(span);
                    }
                    break;
                }
                rest = after;
            }
        }
        None
    }

    /// The text of the words outside `taken`, separated by spaces.
    pub fn rest(&self, taken: &[Range<usize>]) -> String {
        let free = |w: &Range<usize>| !taken.iter().any(|t| overlaps(t, w));
        let words: Vec<&str> = self
            .words
            .iter()
            .filter(|w| free(w))
            .map(|w| &self.text[w.clone()])
            .collect();
        words.join(" ")
    }
}

pub(super) fn overlaps(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}

/// Edits between `a` and `b`, by character, in `row` (reused between calls).
fn edit_distance(a: &str, b: &[char], row: &mut Vec<usize>) -> usize {
    row.clear();
    row.extend(0..=b.len());
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let next = (diagonal + usize::from(ca != cb))
                .min(row[j] + 1)
                .min(row[j + 1] + 1);
            diagonal = row[j + 1];
            row[j + 1] = next;
        }
    }
    row[b.len()]
}

/// How unlike `heard` (compact) is to `name` (compact), 0 the same, 1
/// nothing alike; the best of every run of `heard`'s words.
pub(super) fn unlikeness(heard: &Segment, name: &str) -> f32 {
    let chars: Vec<char> = name.chars().collect();
    let (mut joined, mut row) = (String::new(), Vec::new());
    let mut best = 1.0f32;
    for i in 0..heard.words.len() {
        joined.clear();
        for word in &heard.words[i..] {
            joined.push_str(&heard.text[word.clone()]);
            let longest = joined.chars().count().max(chars.len()).max(1);
            best = best.min(edit_distance(&joined, &chars, &mut row) as f32 / longest as f32);
            if joined.len() > name.len() * 2 {
                break;
            }
        }
    }
    best
}
