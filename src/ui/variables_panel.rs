use crate::app::OctantApp;
use egui::{DragValue, Sense, Stroke, Ui, Vec2};

/// Positioned to the right of the Settings overlay using the previous frame's settings width.
pub fn show_variable_controls(app: &mut OctantApp, ctx: &egui::Context, canvas_rect: egui::Rect) {
    if !app.show_variable_controls || app.active_dataset_metadata.is_none() {
        return;
    }

    // Position to the right of Settings (or at left edge of canvas if Settings is hidden).
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

                    // — Variable overview header (collapsible, with Plot Data button to the right of variable name) —
                    let header_id =
                        ui.make_persistent_id(format!("var_info_header_{}", var_info.name));
                    let mut should_plot = false;

                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        header_id,
                        false,
                    )
                    .show_header(ui, |ui| {
                        ui.label(egui::RichText::new(format!("📄 {}", var_info.name)).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(egui::RichText::new("📊 Plot Data").strong().small())
                                .clicked()
                            {
                                should_plot = true;
                            }
                        });
                    })
                    .body(|ui| {
                        show_variable_info(ui, &var_info);
                    });

                    if should_plot {
                        app.load_selected_variable_slice();
                    }

                    ui.add_space(4.0);

                    // — Dimension sliders (collapsible) —
                    egui::CollapsingHeader::new("🎛️ Dimension Sliders")
                        .default_open(true)
                        .show(ui, |ui| {
                            show_dimension_sliders(app, ui, &var_info, &dim_coords);
                        });
                });
        });
}

fn show_variable_info(ui: &mut egui::Ui, var_info: &crate::stores::VariableInfo) {
    ui.horizontal(|ui| {
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

            double_slider_with_inputs(ui, &dim_name, &mut start_idx, &mut end_idx, 0, max_idx);

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

/// A compact double-slider: [start input] ——●————●—— [end input]
///
/// - `start`/`end` are drag-editable via the two DragValue boxes on either side
///   (click and type, or click-drag like a normal DragValue).
/// - The two dots on the track between them can also be dragged directly.
/// - `start` is always clamped to `min..=end`, `end` to `start..=max`.
///
/// `id_source` must be unique per call site when this is used in a loop (e.g. the
/// dimension name or index) so repeated calls don't collide on the same widget IDs.
///
/// Returns `true` if either value changed this frame.
pub fn double_slider_with_inputs(
    ui: &mut Ui,
    id_source: impl egui::AsIdSalt,
    start: &mut usize,
    end: &mut usize,
    min: usize,
    max: usize,
) -> bool {
    let mut changed = false;
    let handle_radius: f32 = 6.0;
    let base_id = ui.id().with("double_slider").with(id_source);

    ui.horizontal(|ui| {
        // --- left numeric input ---
        changed |= ui
            .push_id(base_id.with("start_input"), |ui| {
                ui.add(DragValue::new(start).range(min..=*end).speed(1))
            })
            .inner
            .changed();

        // --- track ---
        let track_width = (ui.available_width() - 70.0).max(40.0);
        let (rect, _resp) = ui.allocate_exact_size(
            Vec2::new(track_width, 2.0 * handle_radius + 4.0),
            Sense::hover(),
        );

        let span = max.saturating_sub(min).max(1) as f32;
        let left = rect.left() + handle_radius;
        let right = rect.right() - handle_radius;

        let to_x = |v: usize| left + ((v - min) as f32 / span) * (right - left);
        let from_x = |x: f32| {
            let t = ((x - left) / (right - left)).clamp(0.0, 1.0);
            min + (t * span).round() as usize
        };

        let painter = ui.painter_at(rect);
        let mid_y = rect.center().y;

        // base line
        painter.line_segment(
            [egui::pos2(left, mid_y), egui::pos2(right, mid_y)],
            Stroke::new(2.0, ui.visuals().widgets.inactive.bg_fill),
        );

        let x0 = to_x(*start);
        let x1 = to_x(*end);

        // highlighted selected span
        painter.line_segment(
            [egui::pos2(x0, mid_y), egui::pos2(x1, mid_y)],
            Stroke::new(4.0, ui.visuals().selection.bg_fill),
        );

        // start handle
        let start_rect =
            egui::Rect::from_center_size(egui::pos2(x0, mid_y), Vec2::splat(2.0 * handle_radius));
        let start_resp = ui.interact(start_rect, base_id.with("start_handle"), Sense::drag());
        if let Some(pos) = start_resp
            .dragged()
            .then(|| start_resp.interact_pointer_pos())
            .flatten()
        {
            let v = from_x(pos.x).min(*end);
            if v != *start {
                *start = v;
                changed = true;
            }
        }
        painter.circle(
            egui::pos2(x0, mid_y),
            handle_radius,
            ui.visuals().widgets.inactive.bg_fill,
            ui.style().interact(&start_resp).fg_stroke,
        );

        // end handle
        let end_rect =
            egui::Rect::from_center_size(egui::pos2(x1, mid_y), Vec2::splat(2.0 * handle_radius));
        let end_resp = ui.interact(end_rect, base_id.with("end_handle"), Sense::drag());
        if let Some(pos) = end_resp
            .dragged()
            .then(|| end_resp.interact_pointer_pos())
            .flatten()
        {
            let v = from_x(pos.x).max(*start);
            if v != *end {
                *end = v;
                changed = true;
            }
        }
        painter.circle(
            egui::pos2(x1, mid_y),
            handle_radius,
            ui.visuals().widgets.inactive.bg_fill,
            ui.style().interact(&end_resp).fg_stroke,
        );

        // --- right numeric input ---
        changed |= ui
            .push_id(base_id.with("end_input"), |ui| {
                ui.add(DragValue::new(end).range(*start..=max).speed(1))
            })
            .inner
            .changed();
    });

    *start = (*start).clamp(min, max);
    *end = (*end).clamp(min, max).max(*start);

    changed
}
