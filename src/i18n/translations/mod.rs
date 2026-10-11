//! Translation registry and language dispatch with canonical English fallback.

mod de;
mod en;
mod es;
mod fi;
mod fr;
mod id;
mod it;
mod no;
mod pt;
mod ru;
mod sv;
mod uk;

use crate::i18n::keys::Key;
use crate::i18n::lang::Language;

/// Looks up the translation for `(lang, key)`, falling back to English for
/// untranslated keys and for languages without translations yet (CJK).
#[inline]
pub fn lookup(lang: Language, key: Key) -> &'static str {
    exact(lang, key).unwrap_or_else(|| en::text(key))
}

/// The translation of `key` in `lang` itself, `None` when it has none.
pub fn exact(lang: Language, key: Key) -> Option<&'static str> {
    match lang {
        Language::English => Some(en::text(key)),
        Language::Spanish => es::text(key),
        Language::French => fr::text(key),
        Language::German => de::text(key),
        Language::Portuguese => pt::text(key),
        Language::Italian => it::text(key),
        Language::Ukrainian => uk::text(key),
        Language::Russian => ru::text(key),
        Language::Indonesian => id::text(key),
        Language::Swedish => sv::text(key),
        Language::Norwegian => no::text(key),
        Language::Finnish => fi::text(key),
        Language::ChineseSimp | Language::Japanese | Language::Korean => None,
    }
}
