use crate::app::OctantApp;

/// Fractal-Clock-style overlay: no window chrome, no close button.
/// Stacked below the Settings overlay using the previous frame's height.
pub fn show_variable_controls(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if !app.show_variable_controls || app.active_dataset_metadata.is_none() {
        return;
    }

    // Position just below Settings (or at top if Settings is hidden).
    // Use the previous frame's settings height — 1-frame lag is imperceptible.
    let y_offset = if app.show_settings_panel && app.settings_overlay_height > 0.0 {
        app.settings_overlay_height + 16.0
    } else {
        8.0
    };

    egui::Area::new(egui::Id::new("octant_variables_area"))
        .fixed_pos(egui::pos2(
            canvas_rect.left() + 8.0,
            canvas_rect.top() + y_offset,
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

                    // — Variable overview header (collapsible) —
                    egui::CollapsingHeader::new(
                        egui::RichText::new(format!("📄 {}", var_info.name)).strong(),
                    )
                    .default_open(false)
                    .show(ui, |ui| {
                        show_variable_info(ui, &var_info);
                    });

                    // — Dimension sliders (collapsible) —
                    egui::CollapsingHeader::new("🎛️ Dimension Sliders")
                        .default_open(true)
                        .show(ui, |ui| {
                            show_dimension_sliders(app, ui, &var_info, &dim_coords);
                        });

                    ui.add_space(6.0);

                    // — Plot button (always visible at bottom) —
                    let plot_btn = egui::Button::new(egui::RichText::new("📊 Plot Data").strong());
                    if ui
                        .add_sized([ui.available_width(), 32.0], plot_btn)
                        .clicked()
                    {
                        app.load_selected_variable_slice();
                    }
                });
        });
}

fn show_variable_info(ui: &mut egui::Ui, var_info: &crate::stores::VariableInfo) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&var_info.name).strong());
        ui.label(
            egui::RichText::new(format!("[{}]", var_info.data_type))
                .small()
                .weak(),
        );
    });

    if let Some(units) = &var_info.units {
        ui.small(format!("Units: {}", units));
    }
    if let Some(long_name) = &var_info.long_name {
        ui.small(format!("Description: {}", long_name));
    }

    ui.separator();
    ui.small(format!("Shape: {:?}", var_info.shape));
    ui.small(format!("Dimensions: {:?}", var_info.dimension_names));

    if let (Some(start), Some(end)) = (&var_info.time_coverage_start, &var_info.time_coverage_end) {
        let start_clean = start.split('T').next().unwrap_or(start);
        let end_clean = end.split('T').next().unwrap_or(end);
        ui.small(format!("Time: {} → {}", start_clean, end_clean));
    }
    if let Some(res) = &var_info.temporal_resolution {
        ui.small(format!("Resolution: {}", res));
    }

    let size_mb = var_info.file_size as f64 / (1024.0 * 1024.0);
    ui.small(format!("Size: {:.2} MB", size_mb));

    if !var_info.attributes.is_empty() {
        ui.collapsing("Attributes (.zattrs)", |ui| {
            for (k, v) in &var_info.attributes {
                ui.small(format!("{}: {}", k, v));
            }
        });
    }
}

fn show_dimension_sliders(
    app: &mut OctantApp,
    ui: &mut egui::Ui,
    var_info: &crate::stores::VariableInfo,
    dim_coords: &std::collections::HashMap<String, Vec<String>>,
) {
    let dim_count = var_info.shape.len();
    if app.selected_dim_indices.len() != dim_count {
        app.selected_dim_indices = vec![0; dim_count];
    }
    if app.selected_dim_ranges.len() != dim_count {
        app.selected_dim_ranges = var_info
            .shape
            .iter()
            .map(|&s| (0, (s as usize).saturating_sub(1)))
            .collect();
    }

    for (i, shape_dim) in var_info.shape.iter().enumerate() {
        let dim_size = *shape_dim as usize;
        let max_idx = dim_size.saturating_sub(1);
        let dim_name = var_info
            .dimension_names
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("dim_{}", i));

        let (mut start_idx, mut end_idx) = app
            .selected_dim_ranges
            .get(i)
            .copied()
            .unwrap_or((0, max_idx));

        start_idx = start_idx.min(max_idx);
        end_idx = end_idx.clamp(start_idx, max_idx);

        let start_coord = dim_coords
            .get(&dim_name.to_lowercase())
            .and_then(|c| c.get(start_idx).cloned())
            .unwrap_or_else(|| {
                crate::utils::units::format_axis_value(
                    start_idx,
                    dim_size,
                    Some(&dim_name),
                    var_info.units.as_deref(),
                    var_info.time_coverage_start.as_deref(),
                    var_info.temporal_resolution.as_deref(),
                    app.active_dataset_metadata
                        .as_ref()
                        .map(|m| m.name.as_str()),
                )
            });

        let end_coord = dim_coords
            .get(&dim_name.to_lowercase())
            .and_then(|c| c.get(end_idx).cloned())
            .unwrap_or_else(|| {
                crate::utils::units::format_axis_value(
                    end_idx,
                    dim_size,
                    Some(&dim_name),
                    var_info.units.as_deref(),
                    var_info.time_coverage_start.as_deref(),
                    var_info.temporal_resolution.as_deref(),
                    app.active_dataset_metadata
                        .as_ref()
                        .map(|m| m.name.as_str()),
                )
            });

        let selected_count = end_idx.saturating_sub(start_idx) + 1;

        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{}:", dim_name))
                        .strong()
                        .small(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{}/{}", selected_count, dim_size))
                            .small()
                            .weak(),
                    );
                });
            });
            ui.small(format!("{} → {}", start_coord, end_coord));
            ui.add_space(2.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Start:").small());
                let s = ui.add(
                    egui::Slider::new(&mut start_idx, 0..=max_idx)
                        .show_value(true)
                        .trailing_fill(true),
                );
                if s.changed() && start_idx > end_idx {
                    end_idx = start_idx;
                }
            });

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("End:  ").small());
                let e = ui.add(
                    egui::Slider::new(&mut end_idx, 0..=max_idx)
                        .show_value(true)
                        .trailing_fill(true),
                );
                if e.changed() && end_idx < start_idx {
                    start_idx = end_idx;
                }
            });

            if let Some(r) = app.selected_dim_ranges.get_mut(i) {
                *r = (start_idx, end_idx);
            }
            if let Some(idx_ref) = app.selected_dim_indices.get_mut(i) {
                *idx_ref = start_idx;
            }
            if i == 0 {
                app.current_timestep = start_idx;
            }
        });

        ui.add_space(4.0);
    }
}
