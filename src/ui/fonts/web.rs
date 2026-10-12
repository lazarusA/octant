//! Web only: fetches a language's translation subset from `fonts/` next to
//! the page (Trunk copies `assets/fonts` there) the first time it is needed.

use super::{CjkFont, store};
use crate::i18n::{NoticesKey, filled, tr};
use std::sync::Mutex;

/// Fonts being fetched, by `CjkFont::ALL` index, so each is requested once.
static PENDING: Mutex<[bool; 3]> = Mutex::new([false; 3]);

/// Fetches `font`'s translation subset unless it is loaded or on its way,
/// then reinstalls the fonts and repaints.
pub fn fetch_ui_font(ctx: egui::Context, font: CjkFont) {
    if store::fetched(font).is_some() || !begin(font) {
        return;
    }
    wasm_bindgen_futures::spawn_local(async move {
        let url = format!("fonts/{}", font.ui_file());
        match crate::data::backends::http::fetch_url_bytes(&url).await {
            Ok(bytes) => {
                store::set_fetched(font, bytes);
                super::fonts_arrived(&ctx);
            }
            Err(error) => {
                end(font);
                let language = font.language().display_name();
                crate::ui::toast::report(
                    crate::ui::toast::Severity::Warning,
                    tr(NoticesKey::FontLoadFailed),
                    filled(tr(NoticesKey::FontLoadFailedDetail), &[&language, &error]),
                );
            }
        }
    });
}

/// Marks `font` as being fetched; `false` when it already was.
fn begin(font: CjkFont) -> bool {
    let mut pending = PENDING.lock().unwrap_or_else(|p| p.into_inner());
    let Some(slot) = pending.get_mut(store::index(font)) else {
        return false;
    };
    !std::mem::replace(slot, true)
}

/// Lets a failed fetch be tried again later.
fn end(font: CjkFont) {
    let mut pending = PENDING.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(slot) = pending.get_mut(store::index(font)) {
        *slot = false;
    }
}
