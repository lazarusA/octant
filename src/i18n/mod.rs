//! Internationalization (i18n) subsystem for Octant.
//!
//! Provides zero-allocation, compile-time verified string translation tables
//! returning `&'static str` for high-performance immediate-mode rendering in egui.
//!
//! The active language is process-wide (`set_language`), so code without the
//! app (plot type names, colormap kinds, settings reasons) translates with
//! [`tr`] too. Tests never change it: they call [`translate`] with a language.

pub mod keys;
pub mod lang;
pub mod locale;
pub mod template;
pub mod translations;

#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod tests;

use std::sync::atomic::{AtomicU8, Ordering};

pub use keys::{
    AboutKey, ColormapKey, CommonKey, DatasetKey, ExportKey, HoverKey, Key, NoticesKey,
    PlaybackKey, PlotKey, PlotOptionsKey, SettingsKey, VariablesKey,
};
pub use lang::Language;
pub use template::{fill, filled};

/// The active language, stored as its index in [`Language::ALL`].
static LANGUAGE: AtomicU8 = AtomicU8::new(Language::English as u8);

/// Makes `lang` the language every [`tr`] call translates into.
pub fn set_language(lang: Language) {
    LANGUAGE.store(lang as u8, Ordering::Relaxed);
}

/// The active language.
pub fn language() -> Language {
    Language::from_index(LANGUAGE.load(Ordering::Relaxed))
}

/// Translates `key` into the active language.
#[inline]
pub fn tr(key: impl Into<Key>) -> &'static str {
    translate(language(), key)
}

/// Translates `key` into the given `lang`.
///
/// Falls back to canonical English when `lang` has no translation of `key`.
#[inline]
pub fn translate(lang: Language, key: impl Into<Key>) -> &'static str {
    translations::lookup(lang, key.into())
}
