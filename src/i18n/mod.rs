//! Internationalization (i18n) subsystem for Octant.
//!
//! Provides zero-allocation, compile-time verified string translation tables
//! returning `&'static str` for high-performance immediate-mode rendering in egui.
//!
//! The active language is process-wide (`set_language`), so code without the
//! app (plot type names, colormap kinds, settings reasons) translates with
//! [`tr`] too. Tests never change it: they call [`translate`] with a language,
//! or draw UI in one language on their own thread with [`with_language`].

pub mod date;
pub mod keys;
pub mod lang;
pub mod locale;
pub mod number;
pub mod template;
pub mod translations;

#[cfg(test)]
mod guard_tests;
#[cfg(test)]
mod number_tests;
#[cfg(test)]
mod tests;

use std::sync::atomic::{AtomicU8, Ordering};

pub use keys::{
    AboutKey, ColormapKey, CommonKey, DatasetKey, ExportKey, HoverKey, Key, KeymapKey, NoticesKey,
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
    #[cfg(test)]
    if let Some(lang) = TEST_LANGUAGE.get() {
        return lang;
    }
    Language::from_index(LANGUAGE.load(Ordering::Relaxed))
}

#[cfg(test)]
thread_local! {
    /// This test thread's language, set only within [`with_language`].
    static TEST_LANGUAGE: std::cell::Cell<Option<Language>> = const { std::cell::Cell::new(None) };
}

/// Runs `f` with [`tr`] translating into `lang` on this thread only, leaving
/// the process-wide language (and tests on other threads) alone.
#[cfg(test)]
pub fn with_language<R>(lang: Language, f: impl FnOnce() -> R) -> R {
    /// Restores the thread's previous language, also when `f` panics.
    struct Restore(Option<Language>);
    impl Drop for Restore {
        fn drop(&mut self) {
            TEST_LANGUAGE.set(self.0);
        }
    }
    let _restore = Restore(TEST_LANGUAGE.replace(Some(lang)));
    f()
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
