//! Which CJK fonts a dataset's own text needs: its variable names, long
//! names, units, attributes, dimension names and coordinate labels, read
//! once per dataset (and again when coordinate labels arrive).

use super::CjkFont;
use crate::data::DatasetMetadata;
use crate::i18n::Language;

/// A character's CJK script, `Other` for CJK punctuation and symbols.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Script {
    Han,
    Kana,
    Hangul,
    Other,
}

/// The CJK script of `c`, `None` outside the ranges the CJK fonts are
/// sliced over (`tools/cjk_slices` `coverage.rs`).
pub fn script_of(c: char) -> Option<Script> {
    match c {
        '\u{3040}'..='\u{30FF}' | '\u{31F0}'..='\u{31FF}' | '\u{FF66}'..='\u{FF9F}' => {
            Some(Script::Kana)
        }
        '\u{1100}'..='\u{11FF}' | '\u{3130}'..='\u{318F}' | '\u{AC00}'..='\u{D7AF}' => {
            Some(Script::Hangul)
        }
        '\u{3400}'..='\u{4DBF}'
        | '\u{4E00}'..='\u{9FFF}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{20000}'..='\u{3134F}' => Some(Script::Han),
        '\u{2E80}'..='\u{D7FF}' | '\u{FE30}'..='\u{FE4F}' | '\u{FF00}'..='\u{FFEF}' => {
            Some(Script::Other)
        }
        _ => None,
    }
}

/// The CJK scripts found in a dataset's text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Scripts {
    /// Han characters (shared by Chinese, Japanese and Korean).
    pub han: bool,
    /// Hiragana or katakana: Japanese.
    pub kana: bool,
    /// Hangul: Korean.
    pub hangul: bool,
}

impl Scripts {
    pub fn any(self) -> bool {
        self.han || self.kana || self.hangul
    }

    /// Adds the scripts of `text`.
    pub fn add(&mut self, text: &str) {
        for script in text.chars().filter(|c| !c.is_ascii()).filter_map(script_of) {
            self.mark(script);
        }
    }

    /// Records one character's script.
    pub fn mark(&mut self, script: Script) {
        match script {
            Script::Kana => self.kana = true,
            Script::Hangul => self.hangul = true,
            Script::Han => self.han = true,
            Script::Other => {}
        }
    }

    /// The scripts of every text in `meta`.
    pub fn of(meta: &DatasetMetadata) -> Self {
        super::dataset_text::DatasetText::of(meta).scripts
    }

    /// The font to try first for Han characters: the UI language's when it
    /// is Chinese, Japanese or Korean, else the system language's, else the
    /// dataset's own (kana means Japanese, Hangul Korean), else Chinese.
    pub fn han_font(self, ui: Language, system: Option<Language>) -> CjkFont {
        CjkFont::for_language(ui)
            .or_else(|| system.and_then(CjkFont::for_language))
            .unwrap_or(if self.kana {
                CjkFont::Jp
            } else if self.hangul {
                CjkFont::Kr
            } else {
                CjkFont::Sc
            })
    }

    /// The fonts this text needs, by `CjkFont::ALL` index: Japanese for
    /// kana, Korean for Hangul, and `han_font` for Han characters (the
    /// desktop adds the Chinese font when `han_font` lacks some of them,
    /// `system::request`).
    pub fn needed(self, han_font: CjkFont) -> [bool; 3] {
        let mut needed = [false; 3];
        let mut need = |font: CjkFont| {
            if let Some(slot) = CjkFont::ALL.iter().position(|&f| f == font) {
                needed[slot] = true;
            }
        };
        if self.kana {
            need(CjkFont::Jp);
        }
        if self.hangul {
            need(CjkFont::Kr);
        }
        if self.han {
            need(han_font);
        }
        needed
    }
}
