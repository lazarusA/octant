//! The More menu: the right-hand items (Cache, Voice, Language, Theme) when
//! the bar is too narrow for them even as icons, as on phones.

use super::items;
use crate::app::OctantApp;
use crate::i18n::{CommonKey, tr};
use crate::ui::icons::{Icon, ToolbarButton};
use crate::ui::toolbar::roving::{BarNav, Stop};
use crate::ui::{cache, language_picker};

/// Width of the menu while its Cache section is closed: the language list's
/// 180 px and the section indent.
const CLOSED_WIDTH: f32 = 200.0;

pub(super) fn show_more_menu(app: &mut OctantApp, ui: &mut egui::Ui, nav: &mut BarNav) {
    let response = ui.add(
        ToolbarButton::new(Icon::More, tr(CommonKey::More))
            .compact(true)
            .owns_popup(),
    );
    // Clicks inside open and close its sections; choosing a language, voice
    // or flushing the cache closes it themselves (`close_kind(Menu)`).
    egui::Popup::menu(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| show_contents(app, ui));
    nav.push(&response, Stop::menu(&response));
}

/// Cache and Language open in place, not as submenus beside the menu: on a
/// phone those would cover it. The menu widens to the cache menu's width
/// while Cache is open and narrows again once it closes (an egui menu keeps
/// its widest size otherwise), and grows in height as the cache menu does,
/// then scrolls.
fn show_contents(app: &mut OctantApp, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let cache_open_id = egui::Id::unique("octant_more_cache_open");
    let cache_open = ctx.data(|d| d.get_temp(cache_open_id)).unwrap_or(false);
    ui.set_width(if cache_open {
        cache::menu_width(&ctx)
    } else {
        CLOSED_WIDTH
    });
    cache::scroll_menu(ui, |ui| {
        // Framed section headers span the whole menu, so a tap anywhere on
        // one opens it.
        ui.visuals_mut().collapsing_header_frame = true;
        if show_sections(app, ui, cache_open) {
            ctx.data_mut(|d| d.insert_temp(cache_open_id, !cache_open));
        }
    });
}

/// The menu's rows; whether the Cache section's header was clicked.
fn show_sections(app: &mut OctantApp, ui: &mut egui::Ui, cache_open: bool) -> bool {
    let cache = egui::CollapsingHeader::new(tr(CommonKey::Cache))
        .id_salt("more_cache")
        .open(Some(cache_open))
        .show(ui, |ui| cache::show_cache_contents(app, ui));
    egui::CollapsingHeader::new(tr(CommonKey::Language))
        .id_salt("more_language")
        .show(ui, language_picker::show_language_list);
    if items::show_voice_toggle(app, ui, false).clicked() {
        ui.close();
    }
    items::show_theme_toggle(app, ui, false);
    cache.header_response.clicked()
}
