//! Fonts: egui's own (Latin, Greek, Cyrillic) plus the Chinese, Japanese and
//! Korean fallbacks, ordered so the UI language's own font is tried first.
//!
//! The desktop build bundles the three translation subsets. The web build
//! bundles only the tiny name subsets (for the language picker) and fetches
//! a language's translation subset when it becomes the UI language, keeping
//! the fonts out of the `.wasm`.
//!
//! Datasets can hold CJK text the subsets don't cover (variable names,
//! attributes, labels). `watch_dataset` reads each dataset's text once; the
//! desktop then loads the system's own CJK fonts it needs (`system.rs`),
//! tried before the subsets so one font draws all CJK text.

mod cjk;
mod scan;
mod store;
#[cfg(not(target_arch = "wasm32"))]
mod system;
#[cfg(test)]
mod tests;
#[cfg(target_arch = "wasm32")]
mod web;

pub use cjk::CjkFont;
pub use scan::Scripts;

use crate::app::OctantApp;
use crate::i18n::Language;
use std::sync::Mutex;

/// Installs Octant's fonts into egui and keeps them in step with the language.
pub struct FontEngine;

impl FontEngine {
    /// Installs the fonts for the current language; call once at startup,
    /// after the saved language is restored.
    pub fn init(ctx: &egui::Context) {
        Self::on_language_changed(ctx, crate::i18n::language());
    }

    /// Reorders the CJK fallbacks for `lang` and, on the web, starts fetching
    /// its font when it isn't loaded yet.
    pub fn on_language_changed(ctx: &egui::Context, lang: Language) {
        apply(ctx, lang);
        #[cfg(target_arch = "wasm32")]
        if let Some(font) = CjkFont::for_language(lang) {
            web::fetch_ui_font(ctx.clone(), font);
        }
    }

    /// Reads the staged dataset's text when it or its coordinate labels
    /// change (cheap otherwise) and makes its CJK text drawable: the font
    /// for its Han characters goes first, and the desktop loads the system
    /// fonts it needs.
    pub fn watch_dataset(ctx: &egui::Context, app: &OctantApp) {
        static SCANNED: Mutex<Option<(u64, u64)>> = Mutex::new(None);
        let key = (app.selected.metadata_generation, app.coordinates_revision);
        {
            let mut scanned = SCANNED.lock().unwrap_or_else(|p| p.into_inner());
            if scanned.replace(key) == Some(key) {
                return;
            }
        }
        let Some(meta) = &app.selected.metadata else {
            return;
        };
        let scripts = Scripts::of(meta);
        if !scripts.any() {
            return;
        }
        let lang = crate::i18n::language();
        let han = scripts.han_font(lang, crate::i18n::locale::system_language());
        if store::set_data_first(han) {
            apply(ctx, lang);
        }
        #[cfg(not(target_arch = "wasm32"))]
        system::request(ctx, scripts.needed(han));
    }
}

/// Rebuilds egui's font definitions: its defaults, then every loaded CJK
/// font as a fallback in `lang`'s order (each script's system font before
/// its subset). Rebuilding the atlas is costly, so this only runs when the
/// language changes, a font arrives or a dataset changes the order.
fn apply(ctx: &egui::Context, lang: Language) {
    let mut fonts = egui::FontDefinitions::default();
    for font in CjkFont::fallback_order(lang, store::data_first()) {
        // A system font draws all of its script; the subset fills in.
        let system = store::system(font).map(|data| (font.system_key(), data));
        let subset = Some((font.key(), store::subset(font)));
        for (key, data) in system.into_iter().chain(subset) {
            fonts.font_data.insert(key.to_owned(), data);
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts
                    .families
                    .entry(family)
                    .or_default()
                    .push(key.to_owned());
            }
        }
    }
    ctx.set_fonts(fonts);
}
