//! Responsive Dataset Catalog modal dialog.

pub mod card;
pub mod filters;
pub mod header;
pub mod list;
#[cfg(test)]
mod tests;

pub use card::render_entry_card;
pub use filters::render_search_and_filters;
pub use header::render_header;
pub use list::{contains_ignore_ascii_case, render_entries_list};

use crate::app::OctantApp;
use crate::ui::keyboard::windows;

/// Render the responsive, centered Dataset Catalog modal dialog.
pub fn show_catalog_window(app: &mut OctantApp, ctx: &egui::Context) {
    let modal_id = egui::Id::unique("catalog_modal_dialog");
    let opened = windows::follow_window(ctx, modal_id, app.layout.show_catalog_window);
    if !app.layout.show_catalog_window {
        return;
    }

    let screen_rect = ctx.viewport_rect();
    let is_mobile = screen_rect.width() < 520.0;

    let modal_w = (screen_rect.width() - 24.0).clamp(280.0, 780.0);
    let modal_h = (screen_rect.height() - 36.0).clamp(320.0, 680.0);
    let modal_rect =
        egui::Rect::from_center_size(screen_rect.center(), egui::vec2(modal_w, modal_h));

    // Escape, or a click outside the dialog (read from the pointer: the
    // dialog is modal, so nothing under it takes clicks).
    let mut should_close = ctx.input(|i| {
        let outside = i.pointer.primary_clicked()
            && i.pointer
                .interact_pos()
                .is_some_and(|p| !modal_rect.contains(p));
        i.key_pressed(egui::Key::Escape) || outside
    });

    // 1. Scrim / Backdrop overlay: dims the background
    let backdrop_id = egui::Id::unique("catalog_modal_backdrop");
    egui::Area::new(backdrop_id)
        .fixed_pos(screen_rect.min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (_, painter) = ui.allocate_painter(screen_rect.size(), egui::Sense::hover());
            let dim_color = if ui.visuals().dark_mode {
                egui::Color32::from_black_alpha(150)
            } else {
                egui::Color32::from_black_alpha(70)
            };
            painter.rect_filled(screen_rect, 0.0, dim_color);
        });

    // 2. Centered Modal Dialog Card
    egui::Area::new(modal_id)
        .fixed_pos(modal_rect.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            windows::trap(ui, opened);
            ui.set_width(modal_w);
            ui.set_height(modal_h);

            egui::Frame::window(ui.style())
                .inner_margin(egui::Margin::symmetric(14, 12))
                .corner_radius(8.0)
                .show(ui, |ui| {
                    let total_count = crate::catalog::total_entries();

                    render_header(ui, total_count, &mut should_close);
                    ui.add_space(8.0);

                    render_search_and_filters(app, ui, is_mobile, total_count, opened);
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    render_entries_list(app, ui, is_mobile, &mut should_close);
                });
        });

    if should_close {
        app.layout.show_catalog_window = false;
    }
}
