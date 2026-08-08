use crate::{
    app::{AnimationRole, OctantApp, SpatialRole},
    utils::zarr::{DimensionSelection, SliceRequest},
};
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

                    // — Block-cache toggle (opt-in OctantBlock redesign path) —
                    // Only RemoteZarr has a working storage backend right now
                    // (see cache::storage::build_storage_for); flag that
                    // inline rather than letting the toggle silently no-op
                    // for the other store kinds.
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.checkbox(
                            &mut app.use_block_cache,
                            "🧊 Use block cache (experimental)",
                        );
                        if app.use_block_cache
                            && app.selected_store_kind != crate::app::StoreKind::RemoteZarr
                        {
                            ui.label(
                                egui::RichText::new("⚠ backend not wired yet for this store kind")
                                    .small()
                                    .weak(),
                            );
                        }
                    });

                    if should_plot {
                        if app.use_block_cache {
                            app.load_selected_variable_block();
                        } else {
                            app.load_selected_variable_slice();
                        }
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
    _dim_coords: &std::collections::HashMap<String, Vec<String>>,
) {
    let rank = var_info.shape.len();

    for i in 0..rank {
        let dim_size = var_info.shape[i] as usize;
        let dim_name = var_info
            .dimension_names
            .get(i)
            .cloned()
            .unwrap_or_else(|| format!("dim_{}", i));

        ui.group(|ui| {
            ui.horizontal(|ui| {
                // --- ACTIVE TOGGLE ---
                ui.checkbox(&mut app.dim_config[i].active, "");

                ui.label(
                    egui::RichText::new(format!("{} (size {})", dim_name, dim_size))
                        .strong()
                        .small(),
                );

                // --- SPATIAL ROLE SELECTOR ---
                let mut spatial = app.dim_config[i].spatial;
                egui::ComboBox::from_id_salt(format!("spatial_role_{}", i))
                    .selected_text(format!("{:?}", spatial))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut spatial, SpatialRole::None, "None");
                        ui.selectable_value(&mut spatial, SpatialRole::X, "X");
                        ui.selectable_value(&mut spatial, SpatialRole::Y, "Y");
                        ui.selectable_value(&mut spatial, SpatialRole::Z, "Z");
                    });

                // --- ANIMATION ROLE SELECTOR ---
                let mut anim = app.dim_config[i].animation;
                egui::ComboBox::from_id_salt(format!("anim_role_{}", i))
                    .selected_text(match anim {
                        AnimationRole::None => "None".to_string(),
                        AnimationRole::Animated => "Animated".to_string(),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut anim, AnimationRole::None, "None");
                        ui.selectable_value(&mut anim, AnimationRole::Animated, "Animated");
                    });

                apply_role_change(i, spatial, anim, app);
            });

            ui.add_space(4.0);

            // --- SLIDER OR INDEX ---
            if app.dim_config[i].active {
                let (mut start, mut end) = app.selected_dim_ranges[i];
                double_slider_with_inputs(ui, &dim_name, &mut start, &mut end, 0, dim_size - 1);
                app.selected_dim_ranges[i] = (start, end);
                app.selected_dim_indices[i] = start;
            } else {
                ui.add(DragValue::new(&mut app.selected_dim_indices[i]).range(0..=dim_size - 1));
                app.selected_dim_ranges[i] =
                    (app.selected_dim_indices[i], app.selected_dim_indices[i]);
            }
        });

        ui.add_space(6.0);
    }
}

fn apply_role_change(dim: usize, spatial: SpatialRole, anim: AnimationRole, app: &mut OctantApp) {
    // --- Update spatial role ---
    app.dim_config[dim].spatial = spatial;

    // Remove dim from spatial list if needed
    app.spatial_dims.retain(|&d| d != dim);

    if spatial != SpatialRole::None {
        app.spatial_dims.push(dim);

        // Sort X,Y,Z order
        app.spatial_dims
            .sort_by_key(|&d| match app.dim_config[d].spatial {
                SpatialRole::X => 0,
                SpatialRole::Y => 1,
                SpatialRole::Z => 2,
                SpatialRole::None => 99,
            });
    }

    // --- Update animation role ---
    app.dim_config[dim].animation = anim;

    if anim == AnimationRole::Animated {
        // Only one animated dimension allowed
        app.animated_dim = Some(dim);

        // Remove from spatial dims (cannot be both)
        app.spatial_dims.retain(|&d| d != dim);
    } else {
        if app.animated_dim == Some(dim) {
            app.animated_dim = None;
        }
    }
}

pub fn build_slice_request(app: &OctantApp, var_name: &str, shape: &[u64]) -> SliceRequest {
    let selections = shape
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let (start, end) = app.selected_dim_ranges[i];
            if start == end {
                DimensionSelection::Index(start)
            } else {
                DimensionSelection::Range(start..end)
            }
        })
        .collect();

    SliceRequest {
        variable: var_name.to_string(),
        selections,
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
