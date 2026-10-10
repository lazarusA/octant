//! About Octant modal dialog and vector icons gallery.

pub mod credits;
pub mod icon_scale;
pub mod icons;
#[cfg(test)]
mod license_sheet;
mod license_wrap;
pub mod licenses;
pub mod overview;
pub mod types;

pub use types::AboutTab;

use crate::app::OctantApp;
use crate::ui::icons::UiIconExt;
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
    if !app.layout.show_about_window {
        return;
    }

    let storage_id = egui::Id::new(("about_window", "active_tab"));
    let mut active_tab: AboutTab = ctx
        .data(|d| d.get_temp(storage_id))
        .unwrap_or(AboutTab::About);
    if app.layout.show_icon_gallery_window {
        active_tab = AboutTab::Icons;
        app.layout.show_icon_gallery_window = false;
    }

    let (default_size, min_size, max_size) = window_sizes(active_tab, ctx.viewport_rect().size());
    let title = match active_tab {
        AboutTab::About => "About Octant",
        AboutTab::Icons => "Native Vector Icons",
    };

    let mut open = true;
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
            egui::Frame::default()
                .inner_margin(egui::Margin::symmetric(14, 8))
                .show(ui, |ui| open = window_body(app, ui, &mut active_tab));
        });

    ctx.data_mut(|d| d.insert_temp(storage_id, active_tab));
    let rect = response.map(|r| r.response.rect);
    app.layout.show_about_window = open && !dismissed(ctx, rect);
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

/// Tab bar, active tab content and footer. Returns `false` once the close
/// button is clicked.
fn window_body(app: &mut OctantApp, ui: &mut egui::Ui, active_tab: &mut AboutTab) -> bool {
    let mut open = true;
    ui.horizontal(|ui| {
        let mut label_buf = [0; 32];
        let tabs = [
            (AboutTab::About, "About"),
            (AboutTab::Icons, types::icons_tab_label(&mut label_buf)),
        ];
        for (tab, label) in tabs {
            let text = egui::RichText::new(label).strong();
            if ui.selectable_label(*active_tab == tab, text).clicked() {
                *active_tab = tab;
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.close_button("Close (Esc)").clicked() {
                open = false;
            }
        });
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(6.0);
    match *active_tab {
        AboutTab::About => show_overview_tab(app, ui, active_tab),
        AboutTab::Icons => show_icons_tab(ui),
    }
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);
    ui.small("Licensed under MIT or Apache-2.0");
    open
}

/// True when Escape is pressed or the primary button goes down outside the
/// window `rect`.
fn dismissed(ctx: &egui::Context, rect: Option<egui::Rect>) -> bool {
    ctx.input(|i| {
        let clicked_outside = i.pointer.primary_pressed()
            && rect
                .zip(i.pointer.interact_pos())
                .is_some_and(|(r, pos)| !r.contains(pos));
        i.key_pressed(egui::Key::Escape) || clicked_outside
    })
}

pub fn show_icon_gallery_window(app: &mut OctantApp, ctx: &egui::Context) {
    if app.layout.show_icon_gallery_window {
        app.layout.show_about_window = true;
        show_about_window(app, ctx);
    }
}
