pub mod active;
pub mod cache_stats;
pub mod intake;
pub mod manager;
#[cfg(test)]
mod tests;

pub use active::show_active_dataset;
pub use cache_stats::show_cache_stats;
pub use intake::show_source_intake;
pub use manager::show_dataset_manager;

use crate::app::OctantApp;
use crate::i18n::{CommonKey, DatasetKey, tr};
use crate::ui::icons::Icon;
use crate::ui::panel_header::{self, PanelHeader};
use crate::ui::panel_layout::{self, Panel};

/// Shortest the scrolled dataset body gets on a very short canvas.
const MIN_BODY_HEIGHT: f32 = 120.0;

/// Anchored to the canvas area, docked or positioned according to [`panel_layout`].
pub fn show_dataset_window(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if !app.layout.show_dataset_panel {
        app.layout.dataset_overlay_width = 0.0;
        return;
    }

    let origin = panel_layout::origin(app, Panel::Dataset, canvas_rect);
    let mut header = None;
    let area_resp = egui::Area::new(egui::Id::unique("octant_dataset_area"))
        .fixed_pos(origin)
        .constrain_to(canvas_rect)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .stroke(egui::Stroke::NONE)
                .show(ui, |ui| header = show_panel(app, ui, canvas_rect));
        });

    let rect = area_resp.response.rect;
    app.layout.dataset_overlay_width = rect.width();
    if let Some(header) = header {
        panel_layout::apply_grip(app, Panel::Dataset, header.grip, rect, canvas_rect);
        if header.close {
            app.layout.show_dataset_panel = false;
        }
    }
}

/// The collapsible "Dataset" header over the scrolled body.
fn show_panel(app: &mut OctantApp, ui: &mut egui::Ui, canvas: egui::Rect) -> Option<PanelHeader> {
    ui.set_width(panel_layout::PANEL_W);
    let header_id = ui.make_persistent_id("dataset_panel_header");
    let mut header = None;
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), header_id, true)
        .show_header(ui, |ui| {
            let title = panel_header::show(
                ui,
                Icon::Dataset,
                tr(CommonKey::Dataset),
                tr(DatasetKey::CloseDatasetHover),
            );
            header = Some(title);
        })
        .body(|ui| {
            let max_height = panel_layout::room_below(ui, canvas).max(MIN_BODY_HEIGHT);
            ui.set_max_height(max_height);
            egui::ScrollArea::vertical()
                .id_salt("dataset_panel_scroll")
                .max_height(max_height)
                .show(ui, |ui| show_dataset_body(app, ui));
        });
    header
}

/// Body content organized into Active Dataset, Open Source, Dataset Manager, and Cache sections.
fn show_dataset_body(app: &mut OctantApp, ui: &mut egui::Ui) {
    section(ui, tr(DatasetKey::ActiveDataset));
    active::show_active_dataset(app, ui);

    if app.plate.is_some() {
        section(ui, tr(DatasetKey::Plate));
        crate::ui::plate::show_plate_browser(app, ui);
    }

    section(ui, tr(DatasetKey::OpenSource));
    intake::show_source_intake(app, ui);

    section(ui, tr(DatasetKey::DatasetManager));
    manager::show_dataset_manager(app, ui);

    ui.add_space(6.0);
    ui.separator();
    egui::CollapsingHeader::new(tr(DatasetKey::MemoryCache))
        .id_salt("dataset_memory_cache")
        .show(ui, |ui| {
            cache_stats::show_cache_stats(app, ui);
        });
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(4.0);
    ui.separator();
    ui.label(egui::RichText::new(title).weak());
}
