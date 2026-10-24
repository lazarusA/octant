//! Text to Whisper's tokens: GPT-2 byte-level BPE. The text is cut as
//! GPT-2's pre-tokenizer cuts it (letters, digits or other symbols, each
//! run with the space before it), each piece written in the printable
//! characters that stand for its bytes, then its characters merged pair by
//! pair in the order `tokenizer.json` ranks the merges.

use std::collections::HashMap;

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub(super) enum Merge {
    /// `"Ġ t"`, as older files write it.
    Joined(String),
    /// `["Ġ", "t"]`.
    Pair(String, String),
}

pub(super) struct Bpe {
    ranks: HashMap<(String, String), usize>,
    ids: HashMap<String, u32>,
    /// The printable character standing for each byte.
    chars: [char; 256],
}

impl Bpe {
    pub(super) fn new(merges: Vec<Merge>, ids: HashMap<String, u32>, chars: [char; 256]) -> Self {
        let ranks = merges
            .into_iter()
            .filter_map(|m| match m {
                Merge::Joined(s) => s
                    .split_once(' ')
                    .map(|(a, b)| (a.to_string(), b.to_string())),
                Merge::Pair(a, b) => Some((a, b)),
            })
            .enumerate()
            .map(|(rank, pair)| (pair, rank))
            .collect();
        Self { ranks, ids, chars }
    }

    /// The tokens of `text`; pieces with no token are left out.
    pub(super) fn encode(&self, text: &str) -> Vec<u32> {
        pieces(text)
            .flat_map(|piece| self.merge(piece))
            .filter_map(|symbol| self.ids.get(&symbol).copied())
            .collect()
    }

    /// `piece`'s symbols once every ranked pair has merged, lowest rank first.
    fn merge(&self, piece: &str) -> Vec<String> {
        let mut symbols: Vec<String> = piece
            .bytes()
            .map(|b| self.chars[b as usize].to_string())
            .collect();
        loop {
            let best = symbols
                .windows(2)
                .enumerate()
                .filter_map(|(i, w)| {
                    self.ranks
                        .get(&(w[0].clone(), w[1].clone()))
                        .map(|&r| (r, i))
                })
                .min();
            let Some((_, i)) = best else {
                return symbols;
            };
            let next = symbols.remove(i + 1);
            symbols[i].push_str(&next);
        }
    }
}

/// What a character is, as GPT-2's pre-tokenizer groups them.
fn class(c: char) -> u8 {
    if c.is_alphabetic() {
        0
    } else if c.is_numeric() {
        1
    } else if c.is_whitespace() {
        2
    } else {
        3
    }
}

/// `text` cut into runs of one class, a single space joining the run after
/// it (" world"), as GPT-2's pre-tokenizer cuts it.
fn pieces(text: &str) -> impl Iterator<Item = &str> {
    let mut rest = text;
    std::iter::from_fn(move || {
        let mut chars = rest.char_indices().peekable();
        let (_, first) = chars.next()?;
        let mut kind = class(first);
        if first == ' '
            && let Some(&(_, next)) = chars.peek()
            && class(next) != 2
        {
            kind = class(next);
            chars.next();
        }
        let end = chars
            .find(|&(_, c)| class(c) != kind)
            .map_or(rest.len(), |(i, _)| i);
        let (piece, tail) = rest.split_at(end);
        rest = tail;
        Some(piece)
    })
}
