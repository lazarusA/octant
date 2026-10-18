//! About Octant modal dialog and vector icons gallery.

pub mod credits;
pub mod icon_scale;
pub mod icons;
#[cfg(test)]
mod license_sheet;
mod license_wrap;
pub mod licenses;
mod octant_cube;
pub mod overview;
pub mod types;

pub use types::AboutTab;

use crate::app::OctantApp;
use crate::i18n::{AboutKey, CommonKey, KeymapKey, tr};
use crate::ui::icons::UiIconExt;
use crate::ui::keyboard::windows;
use icons::show_icons_tab;
use overview::show_overview_tab;

/// Window widths shared by every tab.
const DEFAULT_WIDTH: f32 = 450.0;
const MIN_WIDTH: f32 = 360.0;
const MAX_WIDTH: f32 = 720.0;

/// Render the About Octant modal dialog window.
pub fn show_about_window(app: &mut OctantApp, ctx: &egui::Context) {
    if app.layout.show_icon_gallery_window {
        app.layout.show_about_window = true;
    }
    let key = egui::Id::unique("about_window");
    let opened = windows::follow_window(ctx, key, app.layout.show_about_window);
    if !app.layout.show_about_window {
        return;
    }

    let storage_id = egui::Id::unique(("about_window", "active_tab"));
    let mut active_tab: AboutTab = ctx
        .data(|d| d.get_temp(storage_id))
        .unwrap_or(AboutTab::About);
    if app.layout.show_icon_gallery_window {
        active_tab = AboutTab::Icons;
        app.layout.show_icon_gallery_window = false;
    }

    let (default_size, min_size, max_size) = window_sizes(active_tab, ctx.viewport_rect().size());
    let title = match active_tab {
        AboutTab::About => tr(AboutKey::WindowTitle),
        AboutTab::Icons => tr(AboutKey::IconsWindowTitle),
    };

    let mut open = true;
    let mut shortcuts = false;
    let response = egui::Window::new(title)
        .title_bar(false)
        .default_size(default_size)
        .min_size(min_size)
        .max_size(max_size)
        .resizable(true)
        .collapsible(false)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            windows::trap(ui, opened);
            egui::Frame::default()
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    open = window_body(ui, &mut active_tab, &mut shortcuts)
                });
        });

    ctx.data_mut(|d| d.insert_temp(storage_id, active_tab));
    let rect = response.map(|r| r.response.rect);
    let dismissed = rect.is_some_and(|r| windows::dismissed(ctx, r));
    app.layout.show_about_window = open && !shortcuts && !dismissed;
    app.layout.show_shortcuts |= shortcuts;
}

/// Default, minimum and maximum window size. Both tabs share one width so
/// switching tabs never resizes the window sideways; only the default height
/// differs.
fn window_sizes(tab: AboutTab, screen: egui::Vec2) -> (egui::Vec2, egui::Vec2, egui::Vec2) {
    let default_h = match tab {
        AboutTab::About => (screen.y * 0.65).clamp(360.0, 520.0),
        AboutTab::Icons => (screen.y * 0.75).clamp(420.0, 600.0),
    };
    (
        egui::vec2(DEFAULT_WIDTH, default_h),
        egui::vec2(MIN_WIDTH, 260.0),
        egui::vec2(MAX_WIDTH, (screen.y * 0.85).max(320.0)),
    )
}

/// Tab bar, active tab content and footer, whose link sets `shortcuts` to
/// open the keyboard shortcut list. Returns `false` once the close button is
/// clicked.
fn window_body(ui: &mut egui::Ui, active_tab: &mut AboutTab, shortcuts: &mut bool) -> bool {
    let mut open = true;
    ui.horizontal(|ui| {
        let mut label_buf = [0; 64];
        let tabs = [
            (AboutTab::About, tr(CommonKey::About)),
            (AboutTab::Icons, types::icons_tab_label(&mut label_buf)),
        ];
        for (tab, label) in tabs {
            let text = egui::RichText::new(label).strong();
            if ui.selectable_label(*active_tab == tab, text).clicked() {
                *active_tab = tab;
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.close_button(tr(CommonKey::CloseEsc)).clicked() {
                open = false;
            }
        });
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(6.0);
    match *active_tab {
        AboutTab::About => show_overview_tab(ui, active_tab),
        AboutTab::Icons => show_icons_tab(ui),
    }
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        ui.small(tr(AboutKey::Licensed));
        *shortcuts = ui.link(tr(KeymapKey::Title)).clicked();
    });
    open
}

pub fn show_icon_gallery_window(app: &mut OctantApp, ctx: &egui::Context) {
    if app.layout.show_icon_gallery_window {
        app.layout.show_about_window = true;
        show_about_window(app, ctx);
    }
}
