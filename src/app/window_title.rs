//! The window title (desktop) or page title (web), "Octant: " followed by the
//! translated subtitle, kept in step with the UI language.

use crate::i18n::{AboutKey, Language, tr};

/// The title in the current language.
pub fn title() -> String {
    format!("Octant: {}", tr(AboutKey::Subtitle))
}

/// Sets the title again when the language differs from the one it was last
/// set in; call every frame (the title is only rebuilt on a change).
pub fn sync(ctx: &egui::Context) {
    let lang = crate::i18n::language();
    let id = egui::Id::new("window_title_language");
    if ctx.data(|d| d.get_temp::<Language>(id)) == Some(lang) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(id, lang));
    set(ctx, title());
}

#[cfg(not(target_arch = "wasm32"))]
fn set(ctx: &egui::Context, title: String) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
}

#[cfg(target_arch = "wasm32")]
fn set(_ctx: &egui::Context, title: String) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title(&title);
    }
}
