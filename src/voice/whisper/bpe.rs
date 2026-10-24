//! Text to Whisper's tokens: GPT-2 byte-level BPE. The text is cut much as
//! GPT-2's pre-tokenizer cuts it (letters, digits or other symbols, each
//! run with the space before it; without its English contractions, and a
//! run of spaces kept whole, which names never hold), each piece's bytes
//! read as the tokens of the printable characters standing for them, then
//! merged pair by pair in the order `tokenizer.json` ranks the merges.

use std::collections::HashMap;

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub(super) enum Merge {
    /// `"Ġ t"`, as older files write it.
    Joined(String),
    /// `["Ġ", "t"]`.
    Pair(String, String),
}

/// A byte with no token of its own (none in Whisper's vocabularies).
const NO_TOKEN: u32 = u32::MAX;

pub(super) struct Bpe {
    /// Each ranked pair of tokens: its rank and the token it merges into.
    ranks: HashMap<(u32, u32), (usize, u32)>,
    /// The token of the printable character standing for each byte.
    byte_tokens: [u32; 256],
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
            .filter_map(|(rank, (a, b))| {
                let merged = ids.get(&format!("{a}{b}"))?;
                Some(((*ids.get(&a)?, *ids.get(&b)?), (rank, *merged)))
            })
            .collect();
        let byte_tokens = chars.map(|c| ids.get(&c.to_string()).copied().unwrap_or(NO_TOKEN));
        Self { ranks, byte_tokens }
    }

    /// The tokens of `text`; bytes with no token are left out.
    pub(super) fn encode(&self, text: &str) -> Vec<u32> {
        pieces(text)
            .flat_map(|piece| self.merge(piece))
            .filter(|&t| t != NO_TOKEN)
            .collect()
    }

    /// `piece`'s tokens once every ranked pair has merged, lowest rank first.
    fn merge(&self, piece: &str) -> Vec<u32> {
        let mut symbols: Vec<u32> = piece
            .bytes()
            .map(|b| self.byte_tokens[b as usize])
            .collect();
        loop {
            let best = symbols
                .windows(2)
                .enumerate()
                .filter_map(|(i, w)| self.ranks.get(&(w[0], w[1])).map(|&(r, t)| (r, i, t)))
                .min();
            let Some((_, i, merged)) = best else {
                return symbols;
            };
            symbols[i] = merged;
            symbols.remove(i + 1);
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
