//! Translation registry and language dispatch with canonical English fallback.

mod cjk_stubs;
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

/// Looks up the translation for `(lang, key)`, falling back to English.
#[inline]
pub fn lookup(lang: Language, key: Key) -> &'static str {
    let text = match lang {
        Language::English => None,
        Language::Spanish => es::translate(key),
        Language::French => fr::translate(key),
        Language::German => de::translate(key),
        Language::Portuguese => pt::translate(key),
        Language::Italian => it::translate(key),
        Language::Ukrainian => uk::translate(key),
        Language::Russian => ru::translate(key),
        Language::Indonesian => id::translate(key),
        Language::Swedish => sv::translate(key),
        Language::Norwegian => no::translate(key),
        Language::Finnish => fi::translate(key),
        Language::ChineseSimp => cjk_stubs::translate_zh(key),
        Language::Japanese => cjk_stubs::translate_ja(key),
        Language::Korean => cjk_stubs::translate_ko(key),
    };

    text.unwrap_or_else(|| en::translate(key))
}
