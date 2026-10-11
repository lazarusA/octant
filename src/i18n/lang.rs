//! Language definitions, ISO codes, and native display names.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
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

    // Phase 2 Placeholders (CJK)
    ChineseSimp,
    Japanese,
    Korean,
}

impl Language {
    /// All Phase 1 active zero-cost languages available in the UI selector.
    pub const ALL_PHASE1: &'static [Self] = &[
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
    ];

    /// All languages including Phase 2 placeholders.
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

    /// Returns true if this language uses default fonts without extra font assets.
    pub const fn is_zero_cost(self) -> bool {
        !matches!(self, Self::ChineseSimp | Self::Japanese | Self::Korean)
    }
}
