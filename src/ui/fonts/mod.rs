//! Font configuration and fallback management for Octant.
//!
//! Provides font definitions for egui and hooks for on-demand CJK font fallback.

use crate::i18n::Language;

/// Procedural font manager for Octant.
pub struct FontEngine;

impl FontEngine {
    /// Configures font definitions on application startup.
    pub fn init(_ctx: &egui::Context) {
        // Phase 1: Uses default egui Ubuntu-Light and Hack fonts (Latin + Cyrillic).
        // TODO(Phase 2): Register CJK fallback font from OS (Desktop) or HTTP (WASM).
    }

    /// Reactive hook invoked when the active language changes.
    pub fn on_language_changed(_ctx: &egui::Context, _lang: Language) {
        // TODO(Phase 2): Trigger async font loading if `!lang.is_zero_cost()`.
    }
}
