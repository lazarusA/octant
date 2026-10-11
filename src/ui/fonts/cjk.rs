//! The Chinese, Japanese and Korean fonts: Noto Sans SC, JP and KR, subset by
//! `cargo run --features font-tools --bin subset_fonts` to the characters
//! Octant's translations use (`*-ui.otf`) and to each language's own name
//! (`*-names.otf`, so the language picker can show every name).

use crate::i18n::Language;

/// One of the three CJK fonts. Each language draws Han characters its own
/// way, so each has its own font, tried first while it is the UI language.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CjkFont {
    /// Noto Sans SC, Simplified Chinese.
    Sc,
    /// Noto Sans JP, Japanese.
    Jp,
    /// Noto Sans KR, Korean.
    Kr,
}

impl CjkFont {
    pub const ALL: [CjkFont; 3] = [CjkFont::Sc, CjkFont::Jp, CjkFont::Kr];

    /// The font that draws `lang`, `None` for languages egui's own fonts cover.
    pub fn for_language(lang: Language) -> Option<CjkFont> {
        match lang {
            Language::ChineseSimp => Some(CjkFont::Sc),
            Language::Japanese => Some(CjkFont::Jp),
            Language::Korean => Some(CjkFont::Kr),
            _ => None,
        }
    }

    /// The language whose text this font is subset to.
    pub fn language(self) -> Language {
        match self {
            CjkFont::Sc => Language::ChineseSimp,
            CjkFont::Jp => Language::Japanese,
            CjkFont::Kr => Language::Korean,
        }
    }

    /// The font's name in egui's font definitions.
    pub fn key(self) -> &'static str {
        match self {
            CjkFont::Sc => "noto-sans-sc",
            CjkFont::Jp => "noto-sans-jp",
            CjkFont::Kr => "noto-sans-kr",
        }
    }

    /// File name of the subset covering the translations, under `assets/fonts/`
    /// (copied next to the web build as `fonts/`).
    pub fn ui_file(self) -> &'static str {
        match self {
            CjkFont::Sc => "noto-sans-sc-ui.otf",
            CjkFont::Jp => "noto-sans-jp-ui.otf",
            CjkFont::Kr => "noto-sans-kr-ui.otf",
        }
    }

    /// File name of the subset covering only the language's own name.
    pub fn names_file(self) -> &'static str {
        match self {
            CjkFont::Sc => "noto-sans-sc-names.otf",
            CjkFont::Jp => "noto-sans-jp-names.otf",
            CjkFont::Kr => "noto-sans-kr-names.otf",
        }
    }

    /// The fonts in the order egui tries them: the UI language's own first.
    pub fn fallback_order(lang: Language) -> [CjkFont; 3] {
        let mut order = Self::ALL;
        if let Some(first) = Self::for_language(lang)
            && let Some(at) = order.iter().position(|&f| f == first)
        {
            order[..=at].rotate_right(1);
        }
        order
    }

    /// The characters a font subset must cover for `lang`'s text: every
    /// non-ASCII character of its translations, plus its own name.
    pub fn required_chars(self) -> std::collections::BTreeSet<char> {
        let lang = self.language();
        crate::i18n::Key::all()
            .flat_map(|key| crate::i18n::translate(lang, key).chars())
            .chain(lang.display_name().chars())
            .filter(|c| !c.is_ascii())
            .collect()
    }
}
