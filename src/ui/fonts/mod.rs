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
//! tried before the subsets so one font draws all CJK text; the web build
//! fetches the slices of the Noto fonts those characters are in
//! (`web_slices.rs`, built by `tools/cjk_slices`).

mod cjk;
mod dataset_text;
mod scan;
pub mod slice_index;
#[cfg(test)]
mod slice_index_tests;
mod store;
#[cfg(not(target_arch = "wasm32"))]
mod system;
#[cfg(test)]
mod tests;
#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
mod web_slices;

pub use cjk::CjkFont;
pub use scan::Scripts;

use dataset_text::DatasetText;

use crate::app::OctantApp;
use crate::i18n::Language;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Set when fonts finish loading outside the UI frame (the desktop font
/// thread, web fetches); the next frame reinstalls them on the UI thread.
static FONTS_ARRIVED: AtomicBool = AtomicBool::new(false);

/// Bumped whenever the font definitions are rebuilt, so text measured with
/// the old fonts (cached widths) is measured again.
static GENERATION: AtomicU64 = AtomicU64::new(0);
/// The pass from which the latest definitions are in use: egui installs new
/// fonts at the start of the next pass.
static ACTIVE_FROM: AtomicU64 = AtomicU64::new(0);

/// The font definitions' generation, changing whenever they are rebuilt;
/// `u64::MAX` while new definitions wait for the next pass, so nothing
/// measured with the old fonts is kept under the new generation.
pub fn generation(ctx: &egui::Context) -> u64 {
    if ctx.cumulative_pass_nr() < ACTIVE_FROM.load(Ordering::Acquire) {
        return u64::MAX;
    }
    GENERATION.load(Ordering::Acquire)
}

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

    /// Called every frame. Reinstalls fonts that arrived since the last
    /// frame, and makes the staged dataset's CJK text drawable when the
    /// dataset, its coordinate labels or the UI language change (a key
    /// comparison otherwise): reads what is new in it (`DatasetText`), puts
    /// the font for its Han characters first, and loads the fonts it needs.
    pub fn watch_dataset(ctx: &egui::Context, app: &OctantApp) {
        if FONTS_ARRIVED.swap(false, Ordering::AcqRel) {
            apply(ctx, crate::i18n::language());
        }
        /// The dataset text read so far, and the key it was last read at.
        #[derive(Default)]
        struct Watched {
            text: DatasetText,
            key: Option<(u64, u64, Language)>,
        }
        static WATCHED: Mutex<Option<Watched>> = Mutex::new(None);
        let Some(meta) = &app.selected.metadata else {
            return;
        };
        let lang = crate::i18n::language();
        let generation = app.selected.metadata_generation;
        let key = (generation, app.coordinates_revision, lang);
        let mut watched = WATCHED.lock().unwrap_or_else(|p| p.into_inner());
        let Watched { text, key: seen } = watched.get_or_insert_with(Watched::default);
        if seen.replace(key) == Some(key) {
            return;
        }
        text.update(meta, generation);
        let scripts = text.scripts;
        if !scripts.any() {
            return;
        }
        let han = scripts.han_font(lang, crate::i18n::locale::system_language());
        if store::set_data_first(han) {
            apply(ctx, lang);
        }
        #[cfg(not(target_arch = "wasm32"))]
        system::request(ctx, scripts.needed(han), han, text.han_chars());
        #[cfg(target_arch = "wasm32")]
        web_slices::request_dataset(ctx, text, generation, han, lang);
    }
}

/// Called by font loaders that finish outside the UI frame: the next frame
/// reinstalls the fonts on the UI thread, so a rebuild never overwrites one
/// made for a language switch in between.
fn fonts_arrived(ctx: &egui::Context) {
    FONTS_ARRIVED.store(true, Ordering::Release);
    ctx.request_repaint();
}

/// Rebuilds egui's font definitions: its defaults, then every loaded CJK
/// font as a fallback in `lang`'s order (each script's system font before
/// its subset). Rebuilding the atlas is costly, so this only runs when the
/// language changes, a font arrives or a dataset changes the order.
fn apply(ctx: &egui::Context, lang: Language) {
    let mut fonts = egui::FontDefinitions::default();
    for font in CjkFont::fallback_order(lang, store::data_first()) {
        // A system font draws all of its script; the subset and the slices
        // fetched for dataset text fill in.
        let system = store::system(font).map(|data| (font.system_key().to_owned(), data));
        let subset = Some((font.key().to_owned(), store::subset(font)));
        for (key, data) in system.into_iter().chain(subset).chain(store::slices(font)) {
            for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                fonts.families.entry(family).or_default().push(key.clone());
            }
            fonts.font_data.insert(key, data);
        }
    }
    ctx.set_fonts(fonts);
    GENERATION.fetch_add(1, Ordering::AcqRel);
    ACTIVE_FROM.store(ctx.cumulative_pass_nr() + 1, Ordering::Release);
}
