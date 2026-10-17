//! Desktop only: the system's own fonts for characters no loaded font draws,
//! found by eframe's `egui_system_font_provider` through the operating
//! system's font fallback. Han characters look different in each region, so
//! the provider asks for the locale of the Han font Octant picks (the UI
//! language's, else the dataset's, `Scripts::han_font`), not the system's.

use super::{CjkFont, Scripts, store};
use crate::i18n::Language;
use egui_system_font_provider::SystemFontProvider;
use std::sync::{Arc, Mutex};

/// One provider per locale, shared by every context: each enumerates the
/// system fonts once, on a background thread.
static PROVIDERS: Mutex<[Option<Arc<SystemFontProvider>>; 3]> = Mutex::new([None, None, None]);

/// Where each context keeps the Han font its provider follows.
const FOLLOWED: &str = "octant_fonts_system_locale";

/// The BCP 47 locale the provider looks Han fonts up for.
pub fn locale(font: CjkFont) -> &'static str {
    match font {
        CjkFont::Sc => "zh-Hans",
        CjkFont::Jp => "ja",
        CjkFont::Kr => "ko",
    }
}

/// The Han font system fonts follow under `lang`: the UI language's, else
/// the one chosen for the dataset's Han text, else the system language's.
pub fn han_font(lang: Language) -> CjkFont {
    CjkFont::for_language(lang)
        .or_else(store::data_first)
        .unwrap_or_else(|| {
            Scripts::default().han_font(lang, crate::i18n::locale::system_language())
        })
}

/// Points `ctx`'s system font lookups at `lang`'s Han font; `true` when that
/// changed the providers (egui then resolves fallbacks again).
pub fn follow(ctx: &egui::Context, lang: Language) -> bool {
    let font = han_font(lang);
    let id = egui::Id::unique(FOLLOWED);
    if ctx.data(|d| d.get_temp::<CjkFont>(id)) == Some(font) {
        return false;
    }
    ctx.data_mut(|d| d.insert_temp(id, font));
    ctx.set_font_providers(vec![provider(font)]);
    true
}

/// The shared provider for `font`'s locale, started on first use.
fn provider(font: CjkFont) -> Arc<SystemFontProvider> {
    let mut providers = PROVIDERS.lock().unwrap_or_else(|p| p.into_inner());
    let slot = &mut providers[store::index(font)];
    Arc::clone(
        slot.get_or_insert_with(|| {
            Arc::new(SystemFontProvider::new().with_locale(Some(locale(font))))
        }),
    )
}
