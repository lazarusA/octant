//! Internationalization (i18n) subsystem for Octant.
//!
//! Provides zero-allocation, compile-time verified string translation tables
//! returning `&'static str` for high-performance immediate-mode rendering in egui.

pub mod keys;
pub mod lang;
pub mod translations;

#[cfg(test)]
mod tests;

pub use keys::Key;
pub use lang::Language;

/// Translates `key` into the given `lang`.
///
/// Falls back to canonical English if the key is untranslated or in a stubbed language.
#[inline]
pub fn translate(lang: Language, key: Key) -> &'static str {
    translations::lookup(lang, key)
}
