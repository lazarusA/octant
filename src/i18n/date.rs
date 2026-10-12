//! Full dates in the UI language. Chinese and Japanese write them with
//! 年月日 and Korean with 년월일 (`2020年1月15日`, `2020년 1월 15일`), the
//! order ISO 8601 shares; every other language keeps ISO (`2020-01-15`),
//! unambiguous in science. Compact labels (axis ticks) stay ISO everywhere.

use super::{Language, language};
use std::fmt::{self, Display};

/// A calendar date, optionally with an hour, written for the active language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Date {
    pub year: usize,
    pub month: usize,
    pub day: usize,
    pub hour: Option<usize>,
}

impl Date {
    /// A calendar day.
    pub fn day(year: usize, month: usize, day: usize) -> Self {
        Self {
            year,
            month,
            day,
            hour: None,
        }
    }

    /// A day at an hour.
    pub fn at_hour(year: usize, month: usize, day: usize, hour: usize) -> Self {
        Self {
            hour: Some(hour),
            ..Self::day(year, month, day)
        }
    }

    /// The date in `lang`'s style.
    pub fn in_language(self, lang: Language) -> InLanguage {
        InLanguage { date: self, lang }
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.in_language(language()).fmt(f)
    }
}

/// A [`Date`] written in a given language's style.
pub struct InLanguage {
    date: Date,
    lang: Language,
}

impl Display for InLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Date {
            year,
            month,
            day,
            hour,
        } = self.date;
        match self.lang {
            Language::ChineseSimp | Language::Japanese => {
                write!(f, "{year}\u{5E74}{month}\u{6708}{day}\u{65E5}")?;
            }
            Language::Korean => write!(f, "{year}\u{B144} {month}\u{C6D4} {day}\u{C77C}")?,
            _ => write!(f, "{year:04}-{month:02}-{day:02}")?,
        }
        match hour {
            Some(hour) => write!(f, " {hour:02}:00"),
            None => Ok(()),
        }
    }
}

/// The characters `lang`'s dates add to its text (for the font subsets).
pub fn date_chars(lang: Language) -> &'static str {
    match lang {
        Language::ChineseSimp | Language::Japanese => "\u{5E74}\u{6708}\u{65E5}",
        Language::Korean => "\u{B144}\u{C6D4}\u{C77C}",
        _ => "",
    }
}
