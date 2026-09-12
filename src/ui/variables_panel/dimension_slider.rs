//! Dimension slider controls, range configuration, and slice calculations.

use crate::app::{AnimationRole, DimConfig, OctantApp, SpatialRole};
use crate::data::VariableInfo;
use crate::data::slice_request::{DimensionSelection, SliceRequest};
use crate::ui::icons::{Icon, UiIconExt};
pub use crate::utils::format_byte_size;
use egui::{DragValue, RichText, Sense, Stroke, Ui, Vec2};

/// Initializes default dimension roles (spatial X, Y, Z and animation) for a selected variable.
pub fn init_variable_dimension_defaults(app: &mut OctantApp, var_info: &VariableInfo) {
    let rank = var_info.shape.len();

    app.dim_config = vec![DimConfig::default(); rank];
    app.selected_dim_indices = vec![0; rank];
    app.selected_dim_ranges.clear();
    app.spatial_dims.clear();
    app.animated_dim = None;

    for i in 0..rank {
        let dim_size = var_info.shape[i] as usize;
        let chunk_size = var_info.chunk_shape.get(i).copied().unwrap_or(0) as usize;
        let range_end = if chunk_size > 0 {
            chunk_size.min(dim_size).saturating_sub(1)
        } else {
            dim_size.saturating_sub(1)
        };
        app.selected_dim_ranges.push((0, range_end));
    }

    if rank == 1 {
        let dim_name = var_info
            .dimension_names
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");
        let is_grid = crate::utils::coordinates::is_healpix_dim_name(dim_name);
        app.dim_config[0].spatial = if is_grid {
            SpatialRole::Grid
        } else {
            SpatialRole::X
        };
        app.dim_config[0].active = true;
        app.dim_config[0].range = app.selected_dim_ranges[0];
        app.spatial_dims.push(0);
        if is_grid {
            app.active_plot_type = crate::plots::PlotType::Heatmap;
        }
        return;
    }

    // Check if this variable has a discrete global grid / HEALPix dimension
    let healpix_dim_idx = (0..rank).find(|&i| {
        let name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("");
        crate::utils::coordinates::is_healpix_dim_name(name)
    });

    if let Some(grid_i) = healpix_dim_idx {
        app.dim_config[grid_i].spatial = SpatialRole::Grid;
        app.dim_config[grid_i].active = true;
        app.spatial_dims.push(grid_i);
        app.active_plot_type = crate::plots::PlotType::Heatmap;

        let mut z_assigned = false;
        let mut anim_assigned = false;

        // Check for Z (layer/level/depth/elevation) and Anim (time)
        for i in 0..rank {
            if i == grid_i {
                continue;
            }
            let dim_name = var_info
                .dimension_names
                .get(i)
                .map(|s| s.as_str())
                .unwrap_or("");
            if !z_assigned && crate::utils::coordinates::is_spatial_z_name(dim_name) {
                app.dim_config[i].spatial = SpatialRole::Z;
                app.spatial_dims.push(i);
                z_assigned = true;
            }
            if !anim_assigned && crate::utils::coordinates::is_animated_time_name(dim_name) {
                app.dim_config[i].animation = AnimationRole::Animated;
                app.animated_dim = Some(i);
                anim_assigned = true;
            }
        }

        // Fallback animation assignment if time name was not standard
        if !anim_assigned {
            for i in 0..rank {
                if i != grid_i && app.dim_config[i].spatial == SpatialRole::None {
                    app.dim_config[i].animation = AnimationRole::Animated;
                    app.animated_dim = Some(i);
                    break;
                }
            }
        }

        // Synchronize active flags
        for i in 0..rank {
            let spatial = app.dim_config[i].spatial;
            let anim = app.dim_config[i].animation;
            if spatial != SpatialRole::None || anim == AnimationRole::Animated {
                app.dim_config[i].active = true;
            }
            app.dim_config[i].index = app.selected_dim_indices.get(i).copied().unwrap_or(0);
            if let Some(&r) = app.selected_dim_ranges.get(i) {
                app.dim_config[i].range = r;
            }
        }
        return;
    }

    let mut x_assigned = false;
    let mut y_assigned = false;
    let mut z_assigned = false;
    let mut anim_assigned = false;

    // 1. Match explicit named coordinate patterns
    for i in 0..rank {
        let dim_name = var_info
            .dimension_names
            .get(i)
            .map(|s| s.as_str())
            .unwrap_or("");

        if !x_assigned && crate::utils::coordinates::is_spatial_x_name(dim_name) {
            app.dim_config[i].spatial = SpatialRole::X;
            x_assigned = true;
        } else if !y_assigned && crate::utils::coordinates::is_spatial_y_name(dim_name) {
            app.dim_config[i].spatial = SpatialRole::Y;
            y_assigned = true;
        } else if !z_assigned && crate::utils::coordinates::is_spatial_z_name(dim_name) {
            app.dim_config[i].spatial = SpatialRole::Z;
            z_assigned = true;
        }

        if rank >= 3 && !anim_assigned && crate::utils::coordinates::is_animated_time_name(dim_name)
        {
            app.dim_config[i].animation = AnimationRole::Animated;
            app.animated_dim = Some(i);
            anim_assigned = true;
        }
    }

    // 2. Fallback spatial assignment for unassigned dimensions
    for i in 0..rank {
        if app.dim_config[i].spatial == SpatialRole::None
            && app.dim_config[i].animation == AnimationRole::None
        {
            if !y_assigned {
                app.dim_config[i].spatial = SpatialRole::Y;
                y_assigned = true;
            } else if !x_assigned {
                app.dim_config[i].spatial = SpatialRole::X;
                x_assigned = true;
            } else if !z_assigned && rank >= 3 {
                app.dim_config[i].spatial = SpatialRole::Z;
                z_assigned = true;
            }
        }
    }

    // 3. For 3D datasets, assign Z if still unassigned
    if rank >= 3 && !z_assigned {
        for i in 0..rank {
            if app.dim_config[i].spatial == SpatialRole::None {
                app.dim_config[i].spatial = SpatialRole::Z;
                break;
            }
        }
    }

    // 4. For 3D+ datasets, default animation dimension
    if rank >= 3 && !anim_assigned {
        let default_anim = (0..rank)
            .find(|&i| app.dim_config[i].spatial == SpatialRole::Z)
            .unwrap_or(0);
        app.dim_config[default_anim].animation = AnimationRole::Animated;
        app.animated_dim = Some(default_anim);
    }

    // Synchronize active flags and spatial_dims list
    for i in 0..rank {
        let spatial = app.dim_config[i].spatial;
        let anim = app.dim_config[i].animation;
        if spatial != SpatialRole::None {
            app.spatial_dims.push(i);
        }
        if spatial != SpatialRole::None || anim == AnimationRole::Animated {
            app.dim_config[i].active = true;
        }
        app.dim_config[i].index = app.selected_dim_indices.get(i).copied().unwrap_or(0);
        if let Some(&r) = app.selected_dim_ranges.get(i) {
            app.dim_config[i].range = r;
        }
    }

    // Clamp initial 2D selection to GPU limits if needed
    if let (Some(x_idx), Some(y_idx)) = (
        app.dim_config
            .iter()
            .position(|c| c.spatial == SpatialRole::X),
        app.dim_config
            .iter()
            .position(|c| c.spatial == SpatialRole::Y),
    ) {
        let x_span = (app.selected_dim_ranges[x_idx]
            .1
            .saturating_sub(app.selected_dim_ranges[x_idx].0)
            + 1)
        .max(1);
        let y_span = (app.selected_dim_ranges[y_idx]
            .1
            .saturating_sub(app.selected_dim_ranges[y_idx].0)
            + 1)
        .max(1);
        let total_2d = x_span.saturating_mul(y_span);
        if total_2d > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
            let scale = ((crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS as f64)
                / (total_2d as f64))
                .sqrt()
                * 0.95;
            let new_x = ((x_span as f64) * scale).max(1.0) as usize;
            let new_y = ((y_span as f64) * scale).max(1.0) as usize;
            let x_start = app.selected_dim_ranges[x_idx].0;
            let y_start = app.selected_dim_ranges[y_idx].0;
            let max_x = var_info.shape[x_idx].saturating_sub(1) as usize;
            let max_y = var_info.shape[y_idx].saturating_sub(1) as usize;
            app.selected_dim_ranges[x_idx] =
                (x_start, (x_start + new_x.saturating_sub(1)).min(max_x));
            app.selected_dim_ranges[y_idx] =
                (y_start, (y_start + new_y.saturating_sub(1)).min(max_y));
            app.dim_config[x_idx].range = app.selected_dim_ranges[x_idx];
            app.dim_config[y_idx].range = app.selected_dim_ranges[y_idx];
        }
    }

    app.spatial_dims
        .sort_by_key(|&d| match app.dim_config[d].spatial {
            SpatialRole::Grid => 0,
            SpatialRole::X => 1,
            SpatialRole::Y => 2,
            SpatialRole::Z => 3,
            SpatialRole::None => 99,
        });
}

/// Computes the maximum steps along the animated dimension that fit within GPU limits.
pub fn calculate_max_animated_steps(
    var_info: &VariableInfo,
    dim_config: &[DimConfig],
    selected_ranges: &[(usize, usize)],
    anim_dim: usize,
) -> (usize, usize, usize) {
    let rank = var_info.shape.len();
    if anim_dim >= rank {
        return (1, 1, 1);
    }

    let mut spatial_elements_per_step: usize = 1;
    for d in 0..rank {
        if d == anim_dim {
            continue;
        }
        if let Some(cfg) = dim_config.get(d)
            && cfg.active
        {
            let span = if let Some(&(start, end)) = selected_ranges.get(d) {
                end.saturating_sub(start) + 1
            } else {
                var_info.shape[d] as usize
            };
            spatial_elements_per_step = spatial_elements_per_step.saturating_mul(span.max(1));
        }
    }
    if spatial_elements_per_step == 0 {
        spatial_elements_per_step = 1;
    }

    let full_anim_size = var_info.shape[anim_dim] as usize;
    let max_allowed = (crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS
        / spatial_elements_per_step)
        .clamp(1, full_anim_size.max(1));

    let requested = if let Some(&(start, end)) = selected_ranges.get(anim_dim) {
        end.saturating_sub(start) + 1
    } else {
        full_anim_size
    };

    (max_allowed, requested, spatial_elements_per_step)
}

/// Calculates requested download bytes and total file size for a variable.
pub fn calculate_download_sizes(
    var_info: &VariableInfo,
    dim_config: &[DimConfig],
    selected_ranges: &[(usize, usize)],
) -> (u64, u64) {
    let dtype_bytes = crate::utils::data_type_bytes(&var_info.data_type);

    let total_elements: u64 = var_info.shape.iter().copied().product::<u64>().max(1);
    let total_bytes = if var_info.file_size > 0 {
        var_info.file_size
    } else {
        total_elements.saturating_mul(dtype_bytes)
    };

    let rank = var_info.shape.len();
    let mut requested_elements: u64 = 1;
    for i in 0..rank {
        let dim_size = var_info.shape[i] as usize;
        if dim_config.get(i).is_some_and(|c| c.active) {
            let span = if let Some(&(start, end)) = selected_ranges.get(i) {
                (end.saturating_sub(start) + 1).min(dim_size)
            } else {
                dim_size
            };
            requested_elements = requested_elements.saturating_mul(span.max(1) as u64);
        }
    }
    let requested_bytes = requested_elements.saturating_mul(dtype_bytes);

    (requested_bytes, total_bytes)
}

/// Calculates the total 3D volume elements from active dimensions.
pub fn calculate_selected_volume_elements(app: &OctantApp) -> usize {
    let Some(metadata) = &app.active_dataset_metadata else {
        return 0;
    };
    let Some(var_info) = metadata.variables.get(app.selected_variable_idx) else {
        return 0;
    };

    let mut total_elements = 1usize;
    let mut counted = 0;
    for (i, &size) in var_info.shape.iter().enumerate() {
        let is_active = app.dim_config.get(i).map(|c| c.active).unwrap_or(false)
            || app.spatial_dims.contains(&i)
            || app.animated_dim == Some(i);
        if is_active {
            let (start, end) = app
                .selected_dim_ranges
                .get(i)
                .copied()
                .unwrap_or((0, (size as usize).saturating_sub(1)));
            let span = (end.saturating_sub(start) + 1).min(size as usize);
            total_elements = total_elements.saturating_mul(span.max(1));
            counted += 1;
        }
    }
    if counted == 0 {
        var_info.shape.iter().copied().product::<u64>() as usize
    } else {
        total_elements
    }
}

/// Calculates the total 2D plane elements for spatial X and Y dimensions.
pub fn calculate_selected_2d_elements(app: &OctantApp) -> usize {
    let Some(metadata) = &app.active_dataset_metadata else {
        return 0;
    };
    let Some(var_info) = metadata.variables.get(app.selected_variable_idx) else {
        return 0;
    };

    let rank = var_info.shape.len();
    let (x_dim, y_dim, _) = OctantApp::resolve_spatial_axes(
        rank,
        &var_info.dimension_names,
        &var_info.dimension_names,
        &app.dim_config,
    );

    let get_span = |d: usize| -> usize {
        if d >= rank {
            return 1;
        }
        let size = var_info.shape[d] as usize;
        if let Some(&(start, end)) = app.selected_dim_ranges.get(d) {
            (end.saturating_sub(start) + 1).min(size)
        } else {
            size
        }
    };

    let nx = get_span(x_dim);
    let ny = if rank <= 1 || x_dim == y_dim {
        1
    } else {
        get_span(y_dim)
    };
    nx.saturating_mul(ny)
}

/// Checks if 3D Volume / Point Cloud rendering is allowed under GPU storage limits.
pub fn is_volume_allowed_for_selection(app: &OctantApp) -> bool {
    let elements = calculate_selected_volume_elements(app);
    if elements == 0 && app.active_dataset_metadata.is_some() {
        return false;
    }
    if elements > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
        return false;
    }
    if let Some(vdata) = &app.volume_data
        && vdata.values.len() > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS
    {
        return false;
    }
    true
}

/// Builds a SliceRequest for currently plotted dimensions.
pub fn build_slice_request_for_plotted(
    app: &OctantApp,
    var_name: &str,
    shape: &[u64],
) -> SliceRequest {
    let selections = shape
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let dim_size = s as usize;
            let (start, end) = app
                .plotted_selected_dim_ranges
                .get(i)
                .copied()
                .unwrap_or((0, dim_size.saturating_sub(1)));
            if start == end {
                DimensionSelection::Index(start)
            } else {
                DimensionSelection::Range {
                    start,
                    end: (end + 1).min(dim_size),
                }
            }
        })
        .collect();

    SliceRequest {
        variable: var_name.to_string(),
        selections,
    }
}

/// Builds a SliceRequest for currently selected dimensions.
pub fn build_slice_request(app: &OctantApp, var_name: &str, shape: &[u64]) -> SliceRequest {
    let selections = shape
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            let dim_size = s as usize;
            let (start, end) = app
                .selected_dim_ranges
                .get(i)
                .copied()
                .unwrap_or((0, dim_size.saturating_sub(1)));
            if start == end {
                DimensionSelection::Index(start)
            } else {
                DimensionSelection::Range {
                    start,
                    end: (end + 1).min(dim_size),
                }
            }
        })
        .collect();

    SliceRequest {
        variable: var_name.to_string(),
        selections,
    }
}

/// Double slider with numeric input fields on both sides.
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
        changed |= ui
            .push_id(base_id.with("start_input"), |ui| {
                ui.add(DragValue::new(start).range(min..=*end).speed(1))
            })
            .inner
            .changed();

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

        painter.line_segment(
            [egui::pos2(left, mid_y), egui::pos2(right, mid_y)],
            Stroke::new(2.0, ui.visuals().widgets.inactive.bg_fill),
        );

        let x0 = to_x(*start);
        let x1 = to_x(*end);

        painter.line_segment(
            [egui::pos2(x0, mid_y), egui::pos2(x1, mid_y)],
            Stroke::new(4.0, ui.visuals().selection.bg_fill),
        );

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

/// Renders the complete dimension sliders section including capacity and bandwidth metrics.
pub fn show_dimension_sliders(
    app: &mut OctantApp,
    ui: &mut Ui,
    var_info: &VariableInfo,
    _dim_coords: &std::collections::HashMap<String, Vec<String>>,
) {
    let rank = var_info.shape.len();

    if app.dim_config.len() != rank {
        init_variable_dimension_defaults(app, var_info);
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
            .product()
    } else {
        1
    };

    ui.horizontal(|ui| {
        ui.label(RichText::new("Download").strong());
        ui.label(
            RichText::new(format!(
                "{} ({} cells)",
                format_byte_size(requested_bytes),
                crate::utils::format_count_metric(requested_cells as usize)
            ))
            .strong(),
        );
        ui.label(RichText::new(format!("/ {}", format_byte_size(total_bytes))).weak());
    });
    ui.add_space(4.0);

    let total_2d_elements = calculate_selected_2d_elements(app);
    if total_2d_elements > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
        let data_mb = (total_2d_elements * 4) as f64 / (1024.0 * 1024.0);
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.icon_colored(Icon::Bolt, 13.0, egui::Color32::from_rgb(100, 200, 255));
                ui.label(
                    RichText::new(format!(
                        "Large 2D selection ({} cells, {:.0} MB): Automatic multi-resolution pyramid aggregation is enabled.",
                        crate::utils::format_count_metric(total_2d_elements),
                        data_mb,
                    ))
                    .small()
                    .color(egui::Color32::from_rgb(100, 200, 255)),
                );
            });
        });
        ui.add_space(2.0);
    } else if total_2d_elements > crate::plots::common::MAX_2D_SURFACE_ELEMENTS {
        let data_mb = (total_2d_elements * 4) as f64 / (1024.0 * 1024.0);
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.icon_colored(Icon::Info, 13.0, egui::Color32::from_rgb(255, 180, 80));
                ui.label(
                    RichText::new(format!(
                        "3D Globe & 3D Surface meshes are disabled for this large selection ({} cells, {:.0} MB). 2D Plane and 1D Line plots remain fully active.",
                        crate::utils::format_count_metric(total_2d_elements),
                        data_mb,
                    ))
                    .small()
                    .color(egui::Color32::from_rgb(255, 180, 80)),
                );
            });
        });
        ui.add_space(2.0);
    }

    let total_vol_elements = calculate_selected_volume_elements(app);
    if total_vol_elements > crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS {
        let vol_mb = (total_vol_elements * 4) as f64 / (1024.0 * 1024.0);
        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.icon_colored(Icon::Warning, 13.0, egui::Color32::from_rgb(255, 180, 80));
                ui.label(
                    RichText::new(format!(
                        "3D Volume & Point Cloud are disabled for this selection: volume size ({:.0} MB) exceeds the 128 MB GPU storage buffer limit. 2D Plane, 1D Line, and 3D Globe remain active.",
                        vol_mb
                    ))
                    .small()
                    .color(egui::Color32::from_rgb(255, 180, 80)),
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
            ui.horizontal(|ui| {
                ui.checkbox(&mut app.dim_config[i].active, "");

                ui.label(
                    RichText::new(format!("{} (size {})", dim_name, dim_size))
                        .strong()
                        .small(),
                );

                let mut spatial = app.dim_config[i].spatial;
                egui::ComboBox::from_id_salt(("spatial_role", i))
                    .selected_text(match spatial {
                        SpatialRole::None => "None",
                        SpatialRole::Grid => "Grid (2D/Globe)",
                        SpatialRole::X => "X",
                        SpatialRole::Y => "Y",
                        SpatialRole::Z => "Z",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut spatial, SpatialRole::None, "None");
                        ui.selectable_value(&mut spatial, SpatialRole::Grid, "Grid (2D/Globe)");
                        ui.selectable_value(&mut spatial, SpatialRole::X, "X");
                        ui.selectable_value(&mut spatial, SpatialRole::Y, "Y");
                        ui.selectable_value(&mut spatial, SpatialRole::Z, "Z");
                    });

                let mut anim = app.dim_config[i].animation;
                egui::ComboBox::from_id_salt(("anim_role", i))
                    .selected_text(match anim {
                        AnimationRole::None => "None",
                        AnimationRole::Animated => "Animated",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut anim, AnimationRole::None, "None");
                        ui.selectable_value(&mut anim, AnimationRole::Animated, "Animated");
                    });

                apply_role_change(i, spatial, anim, app);
            });

            ui.add_space(4.0);

            if app.dim_config[i].active {
                let (mut start, mut end) = app.selected_dim_ranges[i];
                double_slider_with_inputs(ui, dim_name, &mut start, &mut end, 0, dim_size - 1);

                app.selected_dim_ranges[i] = (start, end);
                if is_animated {
                    app.selected_dim_indices[i] = app.current_timestep.clamp(start, end);
                } else {
                    app.selected_dim_indices[i] = start;
                }
                app.dim_config[i].range = (start, end);
                app.dim_config[i].index = app.selected_dim_indices[i];
            } else {
                ui.horizontal(|ui| {
                    ui.label("Index:");
                    ui.add(egui::Slider::new(
                        &mut app.selected_dim_indices[i],
                        0..=dim_size.saturating_sub(1),
                    ));
                });
                app.selected_dim_ranges[i] =
                    (app.selected_dim_indices[i], app.selected_dim_indices[i]);
                app.dim_config[i].range = app.selected_dim_ranges[i];
                app.dim_config[i].index = app.selected_dim_indices[i];
            }
        });

        ui.add_space(6.0);
    }
}

fn apply_role_change(dim: usize, spatial: SpatialRole, anim: AnimationRole, app: &mut OctantApp) {
    let old_spatial = app.dim_config[dim].spatial;
    let old_anim = app.dim_config[dim].animation;

    if spatial != old_spatial && spatial != SpatialRole::None {
        for j in 0..app.dim_config.len() {
            if j != dim {
                let should_clear = (spatial == SpatialRole::Grid
                    && (app.dim_config[j].spatial == SpatialRole::Grid
                        || app.dim_config[j].spatial == SpatialRole::X
                        || app.dim_config[j].spatial == SpatialRole::Y))
                    || app.dim_config[j].spatial == spatial
                    || (app.dim_config[j].spatial == SpatialRole::Grid
                        && (spatial == SpatialRole::X || spatial == SpatialRole::Y));

                if should_clear {
                    app.dim_config[j].spatial = SpatialRole::None;
                    if app.dim_config[j].animation == AnimationRole::None {
                        app.dim_config[j].active = false;
                    }
                }
            }
        }
    }

    if anim != old_anim && anim == AnimationRole::Animated {
        for j in 0..app.dim_config.len() {
            if j != dim && app.dim_config[j].animation == AnimationRole::Animated {
                app.dim_config[j].animation = AnimationRole::None;
                if app.dim_config[j].spatial == SpatialRole::None {
                    app.dim_config[j].active = false;
                }
            }
        }
    }

    app.dim_config[dim].spatial = spatial;
    app.dim_config[dim].animation = anim;

    if spatial != SpatialRole::None || anim == AnimationRole::Animated {
        app.dim_config[dim].active = true;
    }

    if spatial != SpatialRole::None
        && let Some(dim_size) = app
            .active_dataset_metadata
            .as_ref()
            .and_then(|meta| meta.variables.get(app.selected_variable_idx))
            .and_then(|v_info| v_info.shape.get(dim).copied())
    {
        let dim_sz = dim_size as usize;
        if dim < app.selected_dim_ranges.len() {
            let (st, en) = app.selected_dim_ranges[dim];
            if st == en {
                app.selected_dim_ranges[dim] = (0, dim_sz.saturating_sub(1));
            }
        }
    }

    app.spatial_dims.clear();
    for j in 0..app.dim_config.len() {
        if app.dim_config[j].spatial != SpatialRole::None {
            app.spatial_dims.push(j);
        }
    }
    app.spatial_dims
        .sort_by_key(|&d| match app.dim_config[d].spatial {
            SpatialRole::Grid => 0,
            SpatialRole::X => 1,
            SpatialRole::Y => 2,
            SpatialRole::Z => 3,
            SpatialRole::None => 99,
        });

    app.animated_dim = app
        .dim_config
        .iter()
        .position(|c| c.animation == AnimationRole::Animated);
}
