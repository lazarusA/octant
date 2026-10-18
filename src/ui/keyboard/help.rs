//! The keyboard shortcut list, opened by `?` or the About window's link:
//! every shortcut of the keymap with what it does, grouped by where it
//! works. A modal window; Escape or its close button closes it.

use super::keymap::TABLE;
use super::labels::{self, KeysBuf, SCOPES};
use super::windows;
use crate::app::OctantApp;
use crate::i18n::{KeymapKey as K, tr};
use crate::ui::icons::UiIconExt;

/// Width of the window's content; descriptions wrap within it.
pub(super) const WIDTH: f32 = 440.0;
/// Width of the list's key column.
const KEYS_W: f32 = 130.0;

pub fn show(app: &mut OctantApp, ctx: &egui::Context) {
    let id = egui::Id::unique("keyboard_shortcuts_window");
    let opened = windows::follow_window(ctx, id, app.layout.show_shortcuts);
    if !app.layout.show_shortcuts {
        return;
    }
    let mac = labels::is_mac(ctx);
    let max_height = ctx.viewport_rect().height() * 0.7;
    let mut open = true;
    egui::Window::new(tr(K::Title))
        .id(id)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            windows::trap(ui, opened);
            ui.set_width(WIDTH);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tr(K::Title)).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    open = !ui.close_button(tr(K::CloseHover)).clicked();
                });
            });
            ui.separator();
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .show(ui, |ui| shortcut_list(ui, mac));
        });
    if !open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        app.layout.show_shortcuts = false;
    }
}

/// One section per scope: its title, then a row per shortcut, its keys in
/// a fixed column and what it does wrapping beside them.
fn shortcut_list(ui: &mut egui::Ui, mac: bool) {
    for scope in SCOPES {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(tr(labels::scope_title(scope))).weak());
        for shortcut in TABLE.iter().filter(|s| s.scope == scope) {
            ui.horizontal_top(|ui| {
                let mut buf: KeysBuf = [0; 48];
                let keys = labels::write_keys(&mut buf, shortcut, mac);
                let keys = egui::Label::new(egui::RichText::new(keys).monospace().strong());
                ui.add_sized([KEYS_W, ui.spacing().interact_size.y], keys);
                ui.add(egui::Label::new(tr(labels::describe(shortcut.action))).wrap());
            });
        }
    }
}
