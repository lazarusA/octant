//! Variables Panel and Dimension Controller Subsystem.

pub mod dimension_slider;
pub mod info;

pub use dimension_slider::{
    build_slice_request, build_slice_request_for_plotted, calculate_download_sizes,
    calculate_max_animated_steps, calculate_selected_2d_elements,
    calculate_selected_volume_elements, double_slider_with_inputs, format_byte_size,
    init_variable_dimension_defaults, is_volume_allowed_for_selection, show_dimension_sliders,
};
pub use info::show_variable_info;

use crate::app::OctantApp;
use crate::ui::icons::{Icon, UiIconExt};

/// Positioned to the right of the Settings overlay using the previous frame's settings width.
pub fn show_variable_controls(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if !app.show_variable_controls || app.active_dataset_metadata.is_none() {
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
                    ui.set_max_width(320.0);

                    let (var_info, dim_coords) = if let Some(meta) = &app.active_dataset_metadata {
                        if let Some(v) = meta.variables.get(app.selected_variable_idx) {
                            (v.clone(), meta.dimension_coordinates.clone())
                        } else {
                            ui.label("No variable selected.");
                            return;
                        }
                    } else {
                        return;
                    };

                    let header_id = ui.make_persistent_id(("var_info_header", &var_info.name));
                    let mut should_plot = false;

                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        header_id,
                        false,
                    )
                    .show_header(ui, |ui| {
                        ui.icon(Icon::VariableDoc, 13.0);
                        let display_name = if let Some(group) = var_info.group_path() {
                            format!("{} ({})", var_info.leaf_name(), group)
                        } else {
                            var_info.name.clone()
                        };
                        ui.label(egui::RichText::new(display_name).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.icon_button(Icon::Variables, "Plot Data").clicked() {
                                should_plot = true;
                            }
                        });
                    })
                    .body(|ui| {
                        show_variable_info(ui, &var_info);
                    });

                    if should_plot {
                        app.show_hero = false;
                        app.sync_plotted_state_from_selected();
                        app.load_selected_variable_block();
                        app.open_only_settings_panel();
                    }

                    ui.add_space(4.0);

                    egui::CollapsingHeader::new("Dimension Sliders")
                        .default_open(true)
                        .show(ui, |ui| {
                            show_dimension_sliders(app, ui, &var_info, &dim_coords);
                        });
                });
        });
}
