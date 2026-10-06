//! Dimension row controls and metric banners.

use crate::app::{AnimationRole, OctantApp, SpatialRole};
use crate::data::VariableInfo;
use crate::ui::icons::{Icon, IconSize, IconTone, UiIconExt};
use crate::utils::format_byte_size;
use egui::{RichText, Ui};

use super::double_slider::{VALUE_BOX_W, double_slider_with_inputs};
use super::metrics::{
    calculate_download_sizes, calculate_selected_2d_elements, calculate_selected_volume_elements,
};
use super::roles::apply_role_change;

/// Renders the complete dimension sliders section including capacity and bandwidth metrics.
pub fn show_dimension_sliders(
    app: &mut OctantApp,
    ui: &mut Ui,
    var_info: &VariableInfo,
    _dim_coords: &std::collections::HashMap<String, Vec<String>>,
) {
    let rank = var_info.shape.len();

    if app.dim_config.len() != rank {
        super::defaults::init_variable_dimension_defaults(app, var_info);
    }

    let (requested_bytes, total_bytes) =
        calculate_download_sizes(var_info, &app.dim_config, &app.selected_dim_ranges);

    let requested_cells: u64 = if rank > 0 {
        var_info
            .shape
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                if app.dim_config.get(i).is_some_and(|c| c.active) {
                    if let Some(&(start, end)) = app.selected_dim_ranges.get(i) {
                        (end.saturating_sub(start) + 1).min(s as usize) as u64
                    } else {
                        s
                    }
                } else {
                    1
                }
            })
            .try_fold(1u64, |acc, x| acc.checked_mul(x))
            .unwrap_or(u64::MAX)
    } else {
        1
    };

    ui.horizontal(|ui| {
        ui.label(RichText::new("Download").strong());
        ui.label(
            RichText::new(format!(
                "{} ({} cells)",
                format_byte_size(requested_bytes),
                crate::utils::format_count_metric(requested_cells.min(usize::MAX as u64) as usize)
            ))
            .strong(),
        );
        ui.label(RichText::new(format!("/ {}", format_byte_size(total_bytes))).weak());
    });
    ui.add_space(4.0);

    let total_2d_elements = calculate_selected_2d_elements(app);
    if total_2d_elements > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
        let data_mb = (total_2d_elements as f64 * 4.0) / (1024.0 * 1024.0);
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.icon_toned(Icon::Bolt, IconSize::Xs, IconTone::Info);
                ui.label(
                    RichText::new(format!(
                        "Large 2D selection ({} cells, {:.0} MB): Automatic multi-resolution pyramid aggregation is enabled.",
                        crate::utils::format_count_metric(total_2d_elements),
                        data_mb,
                    ))
                    .small()
                    .color(IconTone::Info.color(ui.visuals())),
                );
            });
        });
        ui.add_space(2.0);
    } else if total_2d_elements > crate::plots::common::MAX_2D_SURFACE_ELEMENTS {
        let data_mb = (total_2d_elements as f64 * 4.0) / (1024.0 * 1024.0);
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.icon_toned(Icon::Info, IconSize::Xs, IconTone::Warning);
                ui.label(
                    RichText::new(format!(
                        "3D Globe & 3D Surface meshes are disabled for this large selection ({} cells, {:.0} MB). 2D Plane and 1D Line plots remain fully active.",
                        crate::utils::format_count_metric(total_2d_elements),
                        data_mb,
                    ))
                    .small()
                    .color(IconTone::Warning.color(ui.visuals())),
                );
            });
        });
        ui.add_space(2.0);
    }

    let total_vol_elements = calculate_selected_volume_elements(app);
    if total_vol_elements > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
        let vol_mb = (total_vol_elements as f64 * 4.0) / (1024.0 * 1024.0);
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.icon_toned(Icon::Warning, IconSize::Xs, IconTone::Warning);
                ui.label(
                    RichText::new(format!(
                        "3D Volume & Point Cloud are disabled for this selection: volume size ({:.0} MB) exceeds the 128 MB GPU storage buffer limit. 2D Plane, 1D Line, and 3D Globe remain active.",
                        vol_mb
                    ))
                    .small()
                    .color(IconTone::Warning.color(ui.visuals())),
                );
            });
        });
        ui.add_space(2.0);
    }

    for i in 0..rank {
        let dim_size = var_info.shape[i] as usize;
        let dim_name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("dim");

        let is_animated = app.dim_config[i].animation == AnimationRole::Animated;

        ui.group(|ui| {
            // Fill the panel width so every dimension box lines up.
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.checkbox(&mut app.dim_config[i].active, "");

                ui.label(
                    RichText::new(format!("{} ({})", dim_name, dim_size))
                        .strong()
                        .small(),
                );

                // Role selects pinned to the right edge. Right-to-left order:
                // the animation select is added first so it ends up rightmost.
                let mut spatial = app.dim_config[i].spatial;
                let mut anim = app.dim_config[i].animation;
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt(("anim_role", i))
                        .selected_text(match anim {
                            AnimationRole::None => "None",
                            AnimationRole::Animated => "Animated",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut anim, AnimationRole::None, "None");
                            ui.selectable_value(&mut anim, AnimationRole::Animated, "Animated");
                        });

                    egui::ComboBox::from_id_salt(("spatial_role", i))
                        .selected_text(match spatial {
                            SpatialRole::None => "None",
                            SpatialRole::Grid => "Grid",
                            SpatialRole::X => "X",
                            SpatialRole::Y => "Y",
                            SpatialRole::Z => "Z",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut spatial, SpatialRole::None, "None");
                            ui.selectable_value(&mut spatial, SpatialRole::Grid, "Grid");
                            ui.selectable_value(&mut spatial, SpatialRole::X, "X");
                            ui.selectable_value(&mut spatial, SpatialRole::Y, "Y");
                            ui.selectable_value(&mut spatial, SpatialRole::Z, "Z");
                        });
                });

                apply_role_change(i, spatial, anim, app);
            });

            ui.add_space(4.0);

            if app.dim_config[i].active {
                let (mut start, mut end) = app.selected_dim_ranges[i];
                double_slider_with_inputs(
                    ui,
                    dim_name,
                    &mut start,
                    &mut end,
                    0,
                    dim_size.saturating_sub(1),
                );

                app.selected_dim_ranges[i] = (start, end);
                if is_animated {
                    let max_idx = dim_size.saturating_sub(1);
                    if app.is_playing || app.current_timestep > end || app.current_timestep < start
                    {
                        app.selected_dim_indices[i] = app.current_timestep.min(max_idx);
                    } else {
                        app.selected_dim_indices[i] = app.current_timestep.clamp(start, end);
                    }
                } else {
                    app.selected_dim_indices[i] = start;
                }
                app.dim_config[i].range = (start, end);
                app.dim_config[i].index = app.selected_dim_indices[i];
            } else {
                let max_index = dim_size.saturating_sub(1);
                if is_animated
                    && (app.is_playing || app.current_timestep != app.selected_dim_indices[i])
                {
                    app.selected_dim_indices[i] = app.current_timestep.min(max_index);
                }
                let mut changed = false;
                ui.horizontal(|ui| {
                    ui.label("Index:");
                    // Slider fills the row up to a fixed-width value box,
                    // matching the range rows. The slider runs first so the box
                    // shows the dragged value on the same frame.
                    let index = &mut app.selected_dim_indices[i];
                    let spacing = ui.spacing().item_spacing.x;
                    ui.spacing_mut().slider_width =
                        (ui.available_width() - VALUE_BOX_W - spacing).max(40.0);
                    let s_resp = ui.add(egui::Slider::new(index, 0..=max_index).show_value(false));
                    let height = ui.spacing().interact_size.y;
                    let v_resp = ui.add_sized(
                        [VALUE_BOX_W, height],
                        egui::DragValue::new(index).range(0..=max_index).speed(1),
                    );
                    changed = s_resp.dragged() || v_resp.dragged() || s_resp.clicked();
                });
                if changed && is_animated {
                    app.current_timestep = app.selected_dim_indices[i];
                }
                app.selected_dim_ranges[i] =
                    (app.selected_dim_indices[i], app.selected_dim_indices[i]);
                app.dim_config[i].range = app.selected_dim_ranges[i];
                app.dim_config[i].index = app.selected_dim_indices[i];
            }
        });

        ui.add_space(6.0);
    }
}
