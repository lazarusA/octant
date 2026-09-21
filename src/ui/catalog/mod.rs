//! Responsive Dataset Catalog modal dialog.

pub mod card;
pub mod filters;
pub mod header;
pub mod list;

pub use card::render_entry_card;
pub use filters::{format_tab, render_search_and_filters};
pub use header::{format_count, render_header};
pub use list::{contains_ignore_ascii_case, render_entries_list};

use crate::app::OctantApp;
use crate::catalog::{GEOTIFF_CATALOG, ICECHUNK_CATALOG, PROCEDURAL_CATALOG, ZARR_CATALOG};

/// Render the responsive, centered Dataset Catalog modal dialog.
pub fn show_catalog_window(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_catalog_window {
        return;
    }

    let screen_rect = ctx.viewport_rect();
    let is_mobile = screen_rect.width() < 520.0;

    let modal_w = (screen_rect.width() - 24.0).clamp(280.0, 780.0);
    let modal_h = (screen_rect.height() - 36.0).clamp(320.0, 680.0);
    let modal_rect =
        egui::Rect::from_center_size(screen_rect.center(), egui::vec2(modal_w, modal_h));

    let mut should_close = ctx.input(|i| i.key_pressed(egui::Key::Escape));

    // 1. Scrim / Backdrop overlay: dims background and dismisses modal on click outside
    let backdrop_id = egui::Id::new("catalog_modal_backdrop");
    egui::Area::new(backdrop_id)
        .fixed_pos(screen_rect.min)
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            let (resp, painter) = ui.allocate_painter(screen_rect.size(), egui::Sense::click());
            let dim_color = if ui.visuals().dark_mode {
                egui::Color32::from_black_alpha(150)
            } else {
                egui::Color32::from_black_alpha(70)
            };
            painter.rect_filled(screen_rect, 0.0, dim_color);
            if resp.clicked() {
                should_close = true;
            }
        });

    // 2. Centered Modal Dialog Card
    let modal_id = egui::Id::new("catalog_modal_dialog");
    egui::Area::new(modal_id)
        .fixed_pos(modal_rect.min)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.set_width(modal_w);
            ui.set_height(modal_h);

            egui::Frame::window(ui.style())
                .inner_margin(egui::Margin::symmetric(14, 12))
                .corner_radius(8.0)
                .show(ui, |ui| {
                    let total_count = ZARR_CATALOG.len()
                        + ICECHUNK_CATALOG.len()
                        + GEOTIFF_CATALOG.len()
                        + PROCEDURAL_CATALOG.len();

                    render_header(ui, total_count, &mut should_close);
                    ui.add_space(8.0);

                    render_search_and_filters(app, ui, is_mobile, total_count);
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    render_entries_list(app, ui, is_mobile, &mut should_close);
                });
        });

    if should_close {
        app.show_catalog_window = false;
    }
}
