//! About Octant modal dialog and vector icons gallery.

pub mod icons;
pub mod overview;
pub mod types;

pub use types::{AboutTab, ICON_CATEGORIES, ICONS_TAB_LABEL};

use crate::app::OctantApp;
use icons::show_icons_tab;
use overview::show_overview_tab;

/// Render the About Octant modal dialog window.
pub fn show_about_window(app: &mut OctantApp, ctx: &egui::Context) {
    if app.show_icon_gallery_window {
        app.show_about_window = true;
    }

    if !app.show_about_window {
        return;
    }

    let storage_id = egui::Id::new(("about_window", "active_tab"));
    let mut active_tab: AboutTab = ctx
        .data(|d| d.get_temp(storage_id))
        .unwrap_or(AboutTab::Overview);

    if app.show_icon_gallery_window {
        active_tab = AboutTab::Icons;
        app.show_icon_gallery_window = false;
    }

    let mut open = app.show_about_window;

    let screen_size = ctx.viewport_rect().size();
    let max_screen_h = (screen_size.y * 0.85).max(320.0);

    let (default_size, min_size, max_size) = match active_tab {
        AboutTab::Overview => (
            egui::vec2(450.0, (screen_size.y * 0.65).clamp(360.0, 520.0)),
            egui::vec2(360.0, 260.0),
            egui::vec2(720.0, max_screen_h),
        ),
        AboutTab::Icons => (
            egui::vec2(560.0, (screen_size.y * 0.75).clamp(420.0, 600.0)),
            egui::vec2(420.0, 280.0),
            egui::vec2(840.0, max_screen_h),
        ),
    };

    let title = match active_tab {
        AboutTab::Overview => "About Octant",
        AboutTab::Icons => "Native Vector Icons",
    };

    let response = egui::Window::new(title)
        .open(&mut open)
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
                .show(ui, |ui| {
                    // Header navigation bar with Tab Switcher
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                active_tab == AboutTab::Overview,
                                egui::RichText::new("Overview").strong(),
                            )
                            .clicked()
                        {
                            active_tab = AboutTab::Overview;
                        }

                        if ui
                            .selectable_label(
                                active_tab == AboutTab::Icons,
                                egui::RichText::new(ICONS_TAB_LABEL).strong(),
                            )
                            .clicked()
                        {
                            active_tab = AboutTab::Icons;
                        }
                    });

                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(6.0);

                    match active_tab {
                        AboutTab::Overview => {
                            show_overview_tab(app, ui, &mut active_tab);
                        }
                        AboutTab::Icons => {
                            show_icons_tab(ui);
                        }
                    }

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Footer
                    ui.horizontal(|ui| {
                        ui.small("Licensed under MIT or Apache-2.0");
                    });
                });
        });

    ctx.data_mut(|d| d.insert_temp(storage_id, active_tab));

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        open = false;
    }

    if let Some(r) = response
        && let Some(pos) = ctx.input(|i| i.pointer.interact_pos())
        && ctx.input(|i| i.pointer.primary_pressed())
        && !r.response.rect.contains(pos)
    {
        open = false;
    }

    app.show_about_window = open;
}

pub fn show_icon_gallery_window(app: &mut OctantApp, ctx: &egui::Context) {
    if app.show_icon_gallery_window {
        app.show_about_window = true;
        show_about_window(app, ctx);
    }
}
