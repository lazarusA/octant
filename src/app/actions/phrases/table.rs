//! A language's phrase table and which table each language reads with.

use super::Verb;
use crate::i18n::Language;

/// A language's phrasings of each verb, the catalog's name, and the words
/// that end one command and start the next ("then").
pub(super) struct Phrases {
    pub verbs: &'static [(Verb, &'static [&'static str])],
    /// Each month's names as said (with a day, alone, abbreviated),
    /// January first.
    pub months: [&'static [&'static str]; 12],
    /// The words for the time, latitude, longitude and level axes, read as
    /// `AXES`.
    pub axes: [&'static [&'static str]; 4],
    /// The words for the dataset itself ("describe the dataset").
    pub dataset: &'static [&'static str],
    /// The words that make a plot an overlay ("as an overlay", "on top").
    pub overlay: &'static [&'static str],
    /// A plate's well and a well's field.
    pub well: &'static [&'static str],
    pub field: &'static [&'static str],
    /// The dataset catalog's name.
    pub catalog: &'static [&'static str],
    pub then: &'static [&'static str],
}

/// The axes as dimension actions name them (`dims::by_role` reads them).
pub(super) const AXES: [&str; 4] = ["time", "latitude", "longitude", "level"];

pub(super) fn table(lang: Language) -> &'static Phrases {
    match lang {
        Language::English => &super::en::PHRASES,
        Language::Spanish => &super::es::PHRASES,
        Language::French => &super::fr::PHRASES,
        Language::German => &super::de::PHRASES,
        Language::Portuguese => &super::pt::PHRASES,
        Language::Italian => &super::it::PHRASES,
        Language::Indonesian => &super::id::PHRASES,
        Language::Swedish => &super::sv::PHRASES,
        Language::Norwegian => &super::no::PHRASES,
        Language::Finnish => &super::fi::PHRASES,
        Language::Ukrainian => &super::uk::PHRASES,
        Language::Russian => &super::ru::PHRASES,
        Language::ChineseSimp => &super::zh::PHRASES,
        Language::Japanese => &super::ja::PHRASES,
        Language::Korean => &super::ko::PHRASES,
    }
}
