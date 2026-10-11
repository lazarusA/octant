//! The language to start in when none was chosen yet: the first of the
//! system's preferred locales that Octant translates (the OS settings on the
//! desktop, `navigator.languages` on the web).

use super::Language;

/// The system's preferred language, `None` when Octant has none of them.
pub fn system_language() -> Option<Language> {
    sys_locale::get_locales().find_map(|tag| from_locale(&tag))
}

/// The language of a locale tag, BCP 47 (`pt-BR`, `zh-Hans-CN`) or POSIX
/// (`de_DE.UTF-8`, `sr_RS@latin`). Traditional Chinese (`zh-Hant`, `zh-TW`,
/// `zh-HK`, `zh-MO`) has no translation, so it gives `None`.
pub fn from_locale(tag: &str) -> Option<Language> {
    let tag = tag.split(['.', '@']).next()?;
    let mut subtags = tag.split(['-', '_']);
    let primary = subtags.next()?;
    let is = |code: &str| primary.eq_ignore_ascii_case(code);
    if is("zh") {
        let traditional = subtags.any(|s| {
            ["hant", "tw", "hk", "mo"]
                .iter()
                .any(|t| s.eq_ignore_ascii_case(t))
        });
        return (!traditional).then_some(Language::ChineseSimp);
    }
    // Norwegian Bokmål and Nynorsk, and Indonesian's legacy `in` code.
    if is("nb") || is("nn") {
        return Some(Language::Norwegian);
    }
    if is("in") {
        return Some(Language::Indonesian);
    }
    Language::ALL.iter().copied().find(|lang| is(lang.code()))
}
