//! Variables Panel and Dimension Controller Subsystem.

pub mod dimension_slider;
#[cfg(test)]
mod header_tests;
pub mod info;

pub use dimension_slider::{
    calculate_download_sizes, calculate_max_animated_steps, calculate_selected_2d_elements,
    calculate_selected_volume_elements, double_slider_with_inputs, format_byte_size,
    init_layer_composite_defaults, init_variable_dimension_defaults, is_volume_allowed,
    is_volume_allowed_for_selection, show_dimension_sliders,
};
pub use info::show_variable_info;

use crate::app::OctantApp;
use crate::ui::icons::{Icon, IconSize, IconTone, ToolbarButton, UiIconExt};

/// Width of the Dimensions panel's content; every dimension box fills it.
const PANEL_MAX_W: f32 = 340.0;
/// Narrowest the panel gets when the canvas leaves little room.
const PANEL_MIN_W: f32 = 220.0;

/// Positioned to the right of the Settings overlay using the previous frame's settings width.
pub fn show_variable_controls(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if !app.show_variable_controls || app.selected.metadata.is_none() {
        return;
    }

    let x_offset =
        8.0 + if app.show_variables_overlay && app.variables_overlay_width > 0.0 {
            app.variables_overlay_width + 16.0
        } else {
            0.0
        } + if app.show_settings_panel && app.settings_overlay_width > 0.0 {
            app.settings_overlay_width + 16.0
        } else {
            0.0
        };

    // Fixed width, shrinking only when the canvas would otherwise clip it.
    // The 24 px covers the popup frame margins and the gap to the canvas edge.
    let room = canvas_rect.width() - x_offset - 24.0;
    let panel_w = PANEL_MAX_W.min(room).max(PANEL_MIN_W);

    egui::Area::new(egui::Id::new("octant_variables_panel"))
        .fixed_pos(egui::pos2(
            canvas_rect.left() + x_offset,
            canvas_rect.top() + 8.0,
        ))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .stroke(egui::Stroke::NONE)
                .show(ui, |ui| {
                    ui.set_width(panel_w);

                    let Some(var_info) = app.selected.variable_info().cloned() else {
                        ui.label("No variable selected.");
                        return;
                    };
                    if header_row(ui, &var_info) {
                        app.show_variable_controls = false;
                        if !app.show_variables_overlay {
                            app.revert_selected_state_to_plotted();
                        }
                    }
                    if plot_row(app, ui) {
                        app.plot_from_panel();
                    }

                    ui.add_space(4.0);

                    egui::CollapsingHeader::new("Dimension Sliders")
                        .default_open(true)
                        .show(ui, |ui| {
                            show_dimension_sliders(app, ui, &var_info);
                        });
                });
        });
}

/// The header's first row: the variable's name, expanding to its details,
/// and the close button. Whether the close button was clicked.
fn header_row(ui: &mut egui::Ui, var_info: &crate::data::VariableInfo) -> bool {
    let header_id = ui.make_persistent_id(("var_info_header", &var_info.name));
    let mut close = false;
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), header_id, false)
        .show_header(ui, |ui| {
            ui.icon(Icon::VariableDoc, IconSize::Sm);
            let name = match var_info.group_path() {
                Some(group) => format!("{} ({})", var_info.leaf_name(), group),
                None => var_info.name.clone(),
            };
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                close = ui.close_button("Close Dimension Panel").clicked();
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add(egui::Label::new(egui::RichText::new(name).strong()).truncate());
                });
            });
        })
        .body(|ui| show_variable_info(ui, var_info));
    close
}

/// The header's second row, right-aligned: the overlay toggle, then the Plot
/// button, which reads "Add Overlay" while the toggle is on. Whether it was
/// clicked.
fn plot_row(app: &mut OctantApp, ui: &mut egui::Ui) -> bool {
    let unavailable = app.overlay_unavailable();
    if unavailable.is_some() {
        app.plot_as_overlay = false;
    }
    let (icon, text) = if app.plot_as_overlay {
        (Icon::Layers, "Add Overlay")
    } else {
        let plot_icon = crate::ui::plot_type::plot_type_icon(app.selected.plot_type);
        (plot_icon, "Plot Data")
    };
    let mut plot = false;
    // One row tall: a bare right-to-left layout would take the panel's height.
    let row = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
    let layout = egui::Layout::right_to_left(egui::Align::Center);
    ui.allocate_ui_with_layout(row, layout, |ui| {
        plot = ui
            .outlined_icon_button(icon, text, IconTone::Default)
            .clicked();
        let toggle = ToolbarButton::new(Icon::Layers, "Overlay")
            .compact(true)
            .icon_size(IconSize::Sm)
            .toggled(app.plot_as_overlay)
            .hover("Add this variable as an overlay over the plot");
        let response = ui.add_enabled(unavailable.is_none(), toggle);
        let response = match unavailable {
            Some(reason) => response.on_disabled_hover_text(reason),
            None => response,
        };
        if response.clicked() {
            app.plot_as_overlay = !app.plot_as_overlay;
        }
    });
    plot
}
