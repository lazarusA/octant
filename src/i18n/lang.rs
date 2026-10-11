//! Language definitions, ISO codes, and native display names.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum Language {
    #[default]
    English,
    Spanish,
    French,
    German,
    Portuguese,
    Italian,
    Ukrainian,
    Russian,
    Indonesian,
    Swedish,
    Norwegian,
    Finnish,

    ChineseSimp,
    Japanese,
    Korean,
}

impl Language {
    /// Every language, in declaration order, so
    /// `ALL[lang as usize] == lang`.
    pub const ALL: &'static [Self] = &[
        Self::English,
        Self::Spanish,
        Self::French,
        Self::German,
        Self::Portuguese,
        Self::Italian,
        Self::Ukrainian,
        Self::Russian,
        Self::Indonesian,
        Self::Swedish,
        Self::Norwegian,
        Self::Finnish,
        Self::ChineseSimp,
        Self::Japanese,
        Self::Korean,
    ];

    /// The language stored as `index`, English for an unknown index.
    pub fn from_index(index: u8) -> Self {
        Self::ALL
            .get(usize::from(index))
            .copied()
            .unwrap_or_default()
    }

    /// Native localized name displayed in the language selector.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Spanish => "Español",
            Self::French => "Français",
            Self::German => "Deutsch",
            Self::Portuguese => "Português",
            Self::Italian => "Italiano",
            Self::Ukrainian => "Українська",
            Self::Russian => "Русский",
            Self::Indonesian => "Bahasa Indonesia",
            Self::Swedish => "Svenska",
            Self::Norwegian => "Norsk",
            Self::Finnish => "Suomi",
            Self::ChineseSimp => "简体中文",
            Self::Japanese => "日本語",
            Self::Korean => "한국어",
        }
    }

    /// Two-letter ISO 639-1 language code.
    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::German => "de",
            Self::Portuguese => "pt",
            Self::Italian => "it",
            Self::Ukrainian => "uk",
            Self::Russian => "ru",
            Self::Indonesian => "id",
            Self::Swedish => "sv",
            Self::Norwegian => "no",
            Self::Finnish => "fi",
            Self::ChineseSimp => "zh",
            Self::Japanese => "ja",
            Self::Korean => "ko",
        }
    }
}
