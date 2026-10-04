//! Colormap picker: toolbar popup with search, kind/family filters, gradient
//! swatches for every registered colormap, a reverse toggle and the custom
//! colormap editor.

pub mod editor;
pub mod filters;
pub mod label;
pub mod list;
pub mod search;
pub mod swatch;

#[cfg(test)]
mod tests;

use crate::app::OctantApp;
use crate::ui::icons::{Icon, ToolbarButton, UiIconExt};
use crate::utils::colormap::registry;

/// Persistent (per-session) picker UI state.
#[derive(Default)]
pub struct PickerState {
    pub filter: search::FilterKey,
    pub cache: search::FilterCache,
    pub swatches: swatch::SwatchAtlas,
    pub editor: editor::EditorState,
}

const POPUP_WIDTH: f32 = 300.0;
const ACTIVE_SWATCH_WIDTH: f32 = 120.0;

pub fn show_colormap_menu(app: &mut OctantApp, ui: &mut egui::Ui, compact: bool) {
    let button_response = ui.add(
        ToolbarButton::new(Icon::Colormap, "Colormap")
            .compact(compact)
            .owns_popup(),
    );
    egui::Popup::from_toggle_button_response(&button_response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| render_colormap_contents(app, ui));
}

fn render_colormap_contents(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.set_width(POPUP_WIDTH);
    app.colormaps.picker.swatches.ensure(ui.ctx());

    show_active_row(app, ui);
    ui.separator();

    ui.search_field(
        &mut app.colormaps.picker.filter.query,
        "Search colormaps...",
        None,
    );
    filters::show(ui, &mut app.colormaps.picker.filter);
    ui.add_space(2.0);
    list::show(app, ui);

    ui.separator();
    editor::show(app, ui);
    ui.separator();
    show_colorbar_options(app, ui);
}

/// Active colormap swatch, its (elided) name and the reverse toggle.
fn show_active_row(app: &mut OctantApp, ui: &mut egui::Ui) {
    let id = app.active_colormap;
    ui.horizontal(|ui| {
        let (rect, swatch_response) =
            ui.allocate_exact_size(egui::vec2(ACTIVE_SWATCH_WIDTH, 14.0), egui::Sense::hover());
        app.colormaps
            .picker
            .swatches
            .paint(ui, rect, id, app.colormaps.reversed);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(&mut app.colormaps.reversed, "Reversed");
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let color = ui.visuals().strong_text_color();
                let font = egui::TextStyle::Body.resolve(ui.style());
                registry::with_entry(id, |e| {
                    let galley = label::elided(ui, &e.name, font, color, ui.available_width());
                    ui.label(galley)
                        .union(swatch_response)
                        .on_hover_ui(|ui| label::hover_details(ui, e));
                });
            });
        });
    });
}

fn show_colorbar_options(app: &mut OctantApp, ui: &mut egui::Ui) {
    ui.checkbox(&mut app.show_colorbar, "Show Colorbar");
    if app.show_colorbar {
        ui.add(
            egui::Slider::new(&mut app.colorbar_transparency, 0.0..=1.0)
                .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
        );
        ui.label(egui::RichText::new("Box transparency"));
    }
}

/// Makes `id` the active colormap, leaving RGB composite mode if needed.
pub fn select_colormap(app: &mut OctantApp, id: u32) {
    if app.rgb_composite_mode {
        app.rgb_composite_mode = false;
        app.load_selected_variable_block();
    }
    app.active_colormap = id;
    app.preview_colormap = None;
}
