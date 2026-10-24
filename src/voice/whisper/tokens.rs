//! Whisper's vocabulary from its `tokenizer.json`: tokens back into text,
//! text into tokens (`bpe.rs`, for the prompt), and the special tokens:
//! GPT-2 byte-level BPE, where each character of a token stands for one
//! byte.

use std::collections::HashMap;

use super::bpe::{Bpe, Merge};

#[derive(serde::Deserialize)]
struct File {
    model: Model,
    added_tokens: Vec<Added>,
}

#[derive(serde::Deserialize)]
struct Model {
    vocab: HashMap<String, u32>,
    #[serde(default)]
    merges: Vec<Merge>,
}

#[derive(serde::Deserialize)]
struct Added {
    id: u32,
    content: String,
}

pub(super) struct Tokens {
    /// Each text token's bytes, by id.
    bytes: Vec<Vec<u8>>,
    pub(super) sot: u32,
    pub(super) eot: u32,
    pub(super) transcribe: u32,
    pub(super) no_timestamps: u32,
    pub(super) no_speech: u32,
    /// Language tokens: every id between `sot` and `translate`.
    pub(super) languages: std::ops::Range<u32>,
    /// Opens the text before what is transcribed (a prompt).
    pub(super) start_of_prev: u32,
    bpe: Bpe,
}

impl Tokens {
    pub(super) fn parse(json: &[u8]) -> Result<Self, String> {
        let file: File = serde_json::from_slice(json).map_err(|e| e.to_string())?;
        let byte_of = byte_decoder();
        let mut chars = ['\0'; 256];
        for (&c, &b) in &byte_of {
            chars[b as usize] = c;
        }
        let len = file
            .model
            .vocab
            .values()
            .max()
            .map_or(0, |&m| m as usize + 1);
        let mut bytes = vec![Vec::new(); len];
        for (token, &id) in &file.model.vocab {
            bytes[id as usize] = token
                .chars()
                .filter_map(|c| byte_of.get(&c).copied())
                .collect();
        }
        let special: HashMap<String, u32> = file
            .added_tokens
            .into_iter()
            .map(|a| (a.content, a.id))
            .collect();
        let id = |name: &str| {
            special
                .get(name)
                .copied()
                .ok_or_else(|| format!("no token {name}"))
        };
        let sot = id("<|startoftranscript|>")?;
        Ok(Self {
            sot,
            eot: id("<|endoftext|>")?,
            transcribe: id("<|transcribe|>")?,
            no_timestamps: id("<|notimestamps|>")?,
            no_speech: id("<|nospeech|>").or_else(|_| id("<|nocaptions|>"))?,
            languages: sot + 1..id("<|translate|>")?,
            start_of_prev: id("<|startofprev|>")?,
            bpe: Bpe::new(file.model.merges, file.model.vocab, chars),
            bytes,
        })
    }

    /// The tokens of `text`.
    pub(super) fn encode(&self, text: &str) -> Vec<u32> {
        self.bpe.encode(text)
    }

    /// The text of `ids`, skipping special tokens.
    pub(super) fn decode(&self, ids: &[u32]) -> String {
        let bytes: Vec<u8> = ids
            .iter()
            .filter(|&&id| id < self.eot)
            .filter_map(|&id| self.bytes.get(id as usize))
            .flatten()
            .copied()
            .collect();
        String::from_utf8_lossy(&bytes).trim().to_string()
    }
}

/// GPT-2's map from the printable characters tokens are written in back to
/// bytes: printable Latin-1 stands for itself, the rest for 256 and up.
fn byte_decoder() -> HashMap<char, u8> {
    let printable = |b: u8| matches!(b, b'!'..=b'~' | 0xA1..=0xAC | 0xAE..=0xFF);
    let mut shifted = 0u32;
    (0..=255u8)
        .map(|b| {
            let c = if printable(b) {
                char::from(b)
            } else {
                shifted += 1;
                char::from_u32(255 + shifted).unwrap_or(char::REPLACEMENT_CHARACTER)
            };
            (c, b)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny() -> Tokens {
        let json = r#"{
            "model": {
                "vocab": {"Hello": 0, "Ġworld": 1, "Ã©": 2, "Ċ": 3, "H": 13, "e": 14, "l": 15, "o": 16,
                    "He": 17, "ll": 18, "Hell": 19, "Ġ": 20, "w": 21, "r": 22, "d": 23, "Ġw": 24, "or": 25,
                    "Ġwor": 26, "ld": 27},
                "merges": ["H e", "l l", "He ll", "Hell o", "Ġ w", "o r", "Ġw or", "l d", ["Ġwor", "ld"]]
            },
            "added_tokens": [
                {"id": 4, "content": "<|endoftext|>"},
                {"id": 5, "content": "<|startoftranscript|>"},
                {"id": 6, "content": "<|en|>"},
                {"id": 7, "content": "<|de|>"},
                {"id": 8, "content": "<|translate|>"},
                {"id": 9, "content": "<|transcribe|>"},
                {"id": 10, "content": "<|nospeech|>"},
                {"id": 11, "content": "<|notimestamps|>"},
                {"id": 12, "content": "<|startofprev|>"}
            ]
        }"#;
        Tokens::parse(json.as_bytes()).expect("vocabulary")
    }

    #[test]
    fn bytes_come_back_as_utf8_and_specials_are_skipped() {
        let t = tiny();
        assert_eq!(t.decode(&[5, 6, 0, 1, 2, 4]), "Hello worldé");
        assert_eq!(t.decode(&[3, 0]), "Hello");
    }

    #[test]
    fn special_tokens_and_languages_are_found() {
        let t = tiny();
        assert_eq!(
            (t.sot, t.eot, t.transcribe, t.no_timestamps, t.no_speech),
            (5, 4, 9, 11, 10)
        );
        assert_eq!(t.languages, 6..8);
    }

    #[test]
    fn text_is_encoded_by_ranked_merges() {
        let t = tiny();
        assert_eq!(t.encode("Hello world"), [0, 1]);
        assert_eq!(t.decode(&t.encode("Hello world")), "Hello world");
        assert_eq!(t.start_of_prev, 12);
    }
}
