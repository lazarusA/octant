//! Which CJK fonts a dataset's own text needs: its variable names, long
//! names, units, attributes, dimension names and coordinate labels, read
//! once per dataset (and again when coordinate labels arrive).

use super::CjkFont;
use crate::data::{CoordValues, DatasetMetadata};
use crate::i18n::Language;

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
        for c in text.chars().filter(|c| !c.is_ascii()) {
            match c {
                '\u{3040}'..='\u{30FF}' | '\u{31F0}'..='\u{31FF}' | '\u{FF66}'..='\u{FF9F}' => {
                    self.kana = true
                }
                '\u{1100}'..='\u{11FF}' | '\u{3130}'..='\u{318F}' | '\u{AC00}'..='\u{D7AF}' => {
                    self.hangul = true
                }
                '\u{3400}'..='\u{4DBF}'
                | '\u{4E00}'..='\u{9FFF}'
                | '\u{F900}'..='\u{FAFF}'
                | '\u{20000}'..='\u{3134F}' => self.han = true,
                _ => {}
            }
        }
    }

    /// The scripts of every text in `meta`.
    pub fn of(meta: &DatasetMetadata) -> Self {
        let mut scripts = Scripts::default();
        scripts.add(&meta.name);
        for var in &meta.variables {
            scripts.add(&var.name);
            for text in [&var.long_name, &var.units].into_iter().flatten() {
                scripts.add(text);
            }
            for name in &var.dimension_names {
                scripts.add(name);
            }
            for (key, value) in &var.attributes {
                scripts.add(key);
                scripts.add(value);
            }
        }
        for (dim, coords) in &meta.dimension_coordinates {
            scripts.add(dim);
            if let CoordValues::Labels(labels) = coords {
                for label in labels.iter() {
                    scripts.add(label);
                }
            }
        }
        scripts
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
    /// kana, Korean for Hangul, and `han_font` for Han characters.
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
