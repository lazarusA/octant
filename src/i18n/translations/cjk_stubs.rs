//! Phase 2 placeholders for Chinese, Japanese, and Korean.
//!
//! These stubs return `None` so all lookups cleanly fall back to English
//! until Phase 2 implements their complete dictionaries alongside the font engine.

use crate::i18n::keys::Key;

#[inline]
pub fn translate_zh(_key: Key) -> Option<&'static str> {
    None
}

#[inline]
pub fn translate_ja(_key: Key) -> Option<&'static str> {
    None
}

#[inline]
pub fn translate_ko(_key: Key) -> Option<&'static str> {
    None
}
