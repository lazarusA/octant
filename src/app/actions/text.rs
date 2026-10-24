//! Matching spoken or typed text against names in any script: words are
//! compared with case, spaces and punctuation aside ("T 2 M" is `t2m`,
//! "sea surface temperature" is `sea_surface_temperature`), and names in
//! scripts written without spaces (Chinese, Japanese, Korean) as substrings.

use std::ops::Range;

/// Han, kana and Hangul: written without spaces between words.
pub(super) fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{1100}'..='\u{11FF}'
        | '\u{3040}'..='\u{30FF}'
        | '\u{3130}'..='\u{318F}'
        | '\u{3400}'..='\u{4DBF}'
        | '\u{4E00}'..='\u{9FFF}'
        | '\u{AC00}'..='\u{D7AF}'
        | '\u{F900}'..='\u{FAFF}')
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
        let chars = piece.chars().map(|c| match c {
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
            '\u{3000}' => ' ',
            _ => c,
        });
        out.extend(chars.flat_map(char::to_lowercase));
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
        let word = word.trim_end_matches(['.', ',', ';']);
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

/// The words of normalized `text` joined.
pub(super) fn compact(norm: &str) -> String {
    word_spans(norm).into_iter().map(|r| &norm[r]).collect()
}

/// A normalized utterance (or one command of it) and its words.
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

    /// Where `name` occurs: a substring when it holds CJK, else a run of
    /// whole words that joined equal its words joined.
    pub fn find(&self, name: &Name) -> Option<Range<usize>> {
        if name.compact.is_empty() {
            return None;
        }
        if name.cjk {
            let at = self.text.find(&name.norm)?;
            return Some(at..at + name.norm.len());
        }
        for (i, first) in self.words.iter().enumerate() {
            let mut joined = String::new();
            for word in &self.words[i..] {
                joined.push_str(&self.text[word.clone()]);
                if joined == name.compact {
                    return Some(first.start..word.end);
                }
                if !name.compact.starts_with(&joined) {
                    break;
                }
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

/// Edits between `a` and `b`, by character.
pub(super) fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
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
    let mut best = 1.0f32;
    for i in 0..heard.words.len() {
        let mut joined = String::new();
        for word in &heard.words[i..] {
            joined.push_str(&heard.text[word.clone()]);
            let longest = joined.chars().count().max(name.chars().count()).max(1);
            best = best.min(edit_distance(&joined, name) as f32 / longest as f32);
            if joined.len() > name.len() * 2 {
                break;
            }
        }
    }
    best
}
