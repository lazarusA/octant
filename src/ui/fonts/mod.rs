//! Fonts: egui's own (Latin, Greek, Cyrillic) plus the Chinese, Japanese and
//! Korean fallbacks, ordered so the UI language's own font is tried first.
//!
//! The desktop build bundles the three translation subsets. The web build
//! bundles only the tiny name subsets (for the language picker) and fetches
//! a language's translation subset when it becomes the UI language, keeping
//! the fonts out of the `.wasm`.

mod cjk;
mod store;
#[cfg(test)]
mod tests;
#[cfg(target_arch = "wasm32")]
mod web;

pub use cjk::CjkFont;

use crate::i18n::Language;

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
}

/// Rebuilds egui's font definitions: its defaults, then every loaded CJK
/// font as a fallback in `lang`'s order. Rebuilding the atlas is costly, so
/// this only runs when the language changes or a font arrives.
fn apply(ctx: &egui::Context, lang: Language) {
    let mut fonts = egui::FontDefinitions::default();
    for font in CjkFont::fallback_order(lang) {
        let Some(data) = store::get(font) else {
            continue;
        };
        fonts.font_data.insert(font.key().to_owned(), data);
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .push(font.key().to_owned());
        }
    }
    ctx.set_fonts(fonts);
}
