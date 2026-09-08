//! Interactive hover tooltips, glassmorphic card overlays, and 3D raycasting.

pub mod callout;
pub mod camera;
pub mod format;
pub mod raycast_sphere;
pub mod raycast_surface;
pub mod raycast_volume;
pub mod sample_1d;
pub mod sample_2d;

pub use callout::draw_leader_callout;
pub use camera::{Camera3D, Ray3D, intersect_aabb};
pub use format::{
    enrich_entries_with_animated_and_collapsed_dims, format_dimension_coord,
    get_dimension_origin_and_full_len,
};
pub use raycast_sphere::{get_normalized_radial_dr, raycast_sphere, sphere_target_pos};
pub use raycast_surface::{get_normalized_surface_height, raycast_surface, surface_target_pos};
pub use raycast_volume::{VolumeSampler, volume_target_pos};
pub use sample_1d::{draw_line_guidelines_and_reticle, sample_line_series, screen_to_norm_1d};
pub use sample_2d::Transform2D;

use crate::app::OctantApp;
use crate::data::{DatasetMetadata, MatrixData, VariableInfo};
use crate::plots::PlotType;
use egui::{Pos2, Rect};
use std::collections::HashSet;

/// Renders the floating glassmorphic tooltip card and connecting leader lines for the hovered data point.
pub fn show_hover_tooltip(
    app: &OctantApp,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    response: &egui::Response,
    rect: Rect,
) {
    let hover_pos = match response.hover_pos() {
        Some(pos) => pos,
        None => return,
    };

    if !app.show_hover_card {
        return;
    }

    let matrix = match &app.matrix_data {
        Some(m) if m.width > 0 && m.height > 0 && !m.values.is_empty() => m,
        _ => return,
    };

    let camera = Camera3D::from_app(app, rect);
    let sampler = VolumeSampler::from_app(app, matrix);
    let transform_2d = Transform2D::from_app(app, rect, matrix);

    // 1. Resolve normalized coordinates & raycast hit
    let (norm_x, norm_y, is_valid_hit, geo_coords, point_3d_hit) = resolve_hit_coordinates(
        app,
        matrix,
        &camera,
        &sampler,
        &transform_2d,
        rect,
        hover_pos,
    );

    if !is_valid_hit {
        return;
    }

    // 2. Metadata lookup (variable, units, dimension names)
    let meta = app
        .plotted_dataset_metadata
        .as_ref()
        .or(app.active_dataset_metadata.as_ref());

    let var = meta.and_then(|m| {
        m.variables
            .get(app.plotted_variable_idx)
            .or_else(|| m.variables.get(app.selected_variable_idx))
            .or_else(|| m.variables.first())
    });

    let var_name = var.map(|v| v.name.as_str()).unwrap_or("variable");
    let units_str = resolve_variable_units(var);

    // 3. Coordinate String Formatting & Series Detection
    let (raw_val, dim_entries, px, py) = resolve_cell_value_and_dim_entries(
        app,
        matrix,
        meta,
        var,
        &sampler,
        norm_x,
        norm_y,
        geo_coords,
        point_3d_hit,
    );

    // 4. Guidelines and reticle for 1D line charts
    if app.active_plot_type == PlotType::Line {
        draw_line_guidelines_and_reticle(app, ctx, ui, rect, px, raw_val);
    }

    // 5. Forward-Project Exact Center to Screen Space
    let target_pos = resolve_target_screen_pos(
        app,
        matrix,
        &camera,
        &sampler,
        &transform_2d,
        px,
        py,
        raw_val,
        point_3d_hit,
    );

    // 6. Draw Card & Leader Line
    draw_tooltip_card(
        ctx,
        ui,
        rect,
        hover_pos,
        target_pos,
        app.active_plot_type,
        var_name,
        raw_val,
        &units_str,
        &dim_entries,
    );
}

#[allow(clippy::type_complexity)]
fn resolve_hit_coordinates(
    app: &OctantApp,
    matrix: &MatrixData,
    camera: &Camera3D,
    sampler: &VolumeSampler,
    transform_2d: &Transform2D,
    rect: Rect,
    hover_pos: Pos2,
) -> (
    f32,
    f32,
    bool,
    Option<(f32, f32)>,
    Option<(usize, usize, usize, f32)>,
) {
    match app.active_plot_type {
        PlotType::Sphere => {
            if let Some((nx, ny, geo)) = raycast_sphere(app, matrix, camera, hover_pos) {
                (nx, ny, true, geo, None)
            } else {
                (0.0, 0.0, false, None, None)
            }
        }
        PlotType::Surface => {
            if let Some((nx, ny, geo)) = raycast_surface(app, matrix, camera, hover_pos) {
                (nx, ny, true, geo, None)
            } else {
                (0.0, 0.0, false, None, None)
            }
        }
        PlotType::PointCloud | PlotType::Volume => {
            let (_, world_ray) = camera.cast_ray(hover_pos);
            let aspects = app.get_3d_aspect_ratio();
            if let Some((hit_x, hit_y, hit_z, hit_val)) =
                sampler.march_ray(app, &world_ray, aspects, true)
            {
                let nx = (hit_x as f32 + 0.5) / sampler.width as f32;
                let ny = (hit_y as f32 + 0.5) / sampler.height as f32;
                (nx, ny, true, None, Some((hit_x, hit_y, hit_z, hit_val)))
            } else {
                (0.0, 0.0, false, None, None)
            }
        }
        PlotType::Line => {
            let is_inside = rect.contains(hover_pos);
            let (nx, ny) = screen_to_norm_1d(app, rect, hover_pos);
            (nx, ny, is_inside, None, None)
        }
        _ => {
            let (nx, ny) = transform_2d.screen_to_norm(hover_pos);
            let is_inside = rect.contains(hover_pos);
            let (orig_w, orig_h) = if let Some(pyr) = &app.active_pyramid {
                (pyr.original_width, pyr.original_height)
            } else {
                (matrix.width, matrix.height)
            };
            let (px, py) = matrix.grid.find_cell_from_norm(nx, ny, orig_w, orig_h);
            let (cell_lon_rad, cell_lat_rad) =
                matrix.grid.cell_center_lon_lat_rad(px, py, orig_w, orig_h);
            let geo_coords = if matrix.grid.requires_geo_coords() {
                Some((cell_lat_rad.to_degrees(), cell_lon_rad.to_degrees()))
            } else {
                None
            };
            (nx, ny, is_inside, geo_coords, None)
        }
    }
}

fn resolve_variable_units(var: Option<&VariableInfo>) -> String {
    var.and_then(|v| {
        v.units
            .as_deref()
            .or(v.attributes.get("units").map(|s| s.as_str()))
    })
    .map(|u| {
        let clean = u.trim();
        if clean.is_empty() || clean == "1" || clean == "none" || clean == "dimensionless" {
            String::new()
        } else {
            format!("\u{00A0}{}", clean)
        }
    })
    .unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
fn resolve_cell_value_and_dim_entries(
    app: &OctantApp,
    matrix: &MatrixData,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    sampler: &VolumeSampler,
    norm_x: f32,
    norm_y: f32,
    geo_coords: Option<(f32, f32)>,
    point_3d_hit: Option<(usize, usize, usize, f32)>,
) -> (f32, Vec<String>, usize, usize) {
    if app.active_plot_type == PlotType::Line {
        let (profile_values, profile_length, line_count) = app.get_line_profile_payload();
        let prof_len = profile_length as usize;
        let l_count = line_count as usize;

        let (sample_idx, best_line_idx, val) =
            sample_line_series(app, norm_x, norm_y, &profile_values, prof_len, l_count);
        let mut used_dims = HashSet::new();

        let (dim_name, prof_dim_idx) = resolve_line_profile_dim(app, var);
        let (origin_prof, full_prof_len) = if let Some(idx) = prof_dim_idx {
            used_dims.insert(idx);
            get_dimension_origin_and_full_len(app, var, idx)
        } else {
            (0, prof_len)
        };
        let global_sample = (origin_prof + sample_idx).min(full_prof_len.saturating_sub(1));

        let loc_str = format_dimension_coord(
            meta,
            var,
            Some(&app.plotted_store_target_input),
            &dim_name,
            global_sample,
            full_prof_len,
            None,
        );
        let mut entries = vec![loc_str];

        if l_count > 1 {
            enrich_line_series_ortho_dim(
                app,
                meta,
                var,
                &mut entries,
                &mut used_dims,
                best_line_idx,
                l_count,
            );
        }

        enrich_entries_with_animated_and_collapsed_dims(
            app,
            meta,
            var,
            &mut entries,
            &mut used_dims,
        );

        (val, entries, sample_idx, best_line_idx)
    } else if let Some((hit_x, hit_y, hit_z, hit_val)) = point_3d_hit {
        let entries = resolve_3d_dim_entries(app, meta, var, sampler, hit_x, hit_y, hit_z);
        (hit_val, entries, hit_x, hit_y)
    } else {
        let (orig_w, orig_h) = if let Some(pyr) = &app.active_pyramid {
            (pyr.original_width, pyr.original_height)
        } else {
            (matrix.width, matrix.height)
        };
        let (px, py) = matrix
            .grid
            .find_cell_from_norm(norm_x, norm_y, orig_w, orig_h);

        let val = if let Some(pyr) = &app.active_pyramid
            && let Some(base_lvl) = pyr.levels.first()
        {
            let idx = py * orig_w + px;
            base_lvl.values.get(idx).copied().unwrap_or(f32::NAN)
        } else {
            let idx = py * matrix.width + px;
            matrix.values.get(idx).copied().unwrap_or(f32::NAN)
        };

        let mut used_dims = HashSet::new();
        let entries = resolve_2d_dim_entries(
            app,
            meta,
            var,
            px,
            py,
            orig_w,
            orig_h,
            geo_coords,
            &mut used_dims,
        );

        (val, entries, px, py)
    }
}

fn resolve_line_profile_dim(
    app: &OctantApp,
    var: Option<&VariableInfo>,
) -> (String, Option<usize>) {
    if let Some(v) = var {
        let (explicit_x, explicit_y, explicit_z) =
            v.resolve_spatial_dim_indices(if !app.plotted_dim_config.is_empty() {
                &app.plotted_dim_config
            } else {
                &app.dim_config
            });

        let p_idx = match app.line_profile_dim_idx {
            0 => explicit_x.or_else(|| v.dimension_names.len().checked_sub(1)),
            1 => explicit_y.or_else(|| v.dimension_names.len().checked_sub(2)),
            _ => explicit_z.or_else(|| {
                (0..v.dimension_names.len())
                    .find(|&i| Some(i) != explicit_x && Some(i) != explicit_y)
            }),
        };

        let name = p_idx
            .and_then(|i| v.dimension_names.get(i).cloned())
            .or_else(|| app.get_spatial_dim_name(app.line_profile_dim_idx))
            .unwrap_or_else(|| match app.line_profile_dim_idx {
                2 => "z".to_string(),
                1 => "y".to_string(),
                _ => "x".to_string(),
            });

        (name, p_idx)
    } else {
        let name = app
            .get_spatial_dim_name(app.line_profile_dim_idx)
            .unwrap_or_else(|| match app.line_profile_dim_idx {
                2 => "z".to_string(),
                1 => "y".to_string(),
                _ => "x".to_string(),
            });
        (name, None)
    }
}

fn enrich_line_series_ortho_dim(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    entries: &mut Vec<String>,
    used_dims: &mut HashSet<usize>,
    best_line_idx: usize,
    l_count: usize,
) {
    if let Some(v) = var {
        let (explicit_x, explicit_y, _) =
            v.resolve_spatial_dim_indices(if !app.plotted_dim_config.is_empty() {
                &app.plotted_dim_config
            } else {
                &app.dim_config
            });

        let ortho_dim_idx = match app.line_profile_dim_idx {
            0 => explicit_y,
            1 => explicit_x,
            _ => None,
        };

        if let Some(o_idx) = ortho_dim_idx
            && let Some(ortho_name) = v.dimension_names.get(o_idx)
        {
            let (origin_ortho, full_ortho_len) =
                get_dimension_origin_and_full_len(app, Some(v), o_idx);
            let global_ortho = (origin_ortho + best_line_idx).min(full_ortho_len.saturating_sub(1));
            let ortho_str = format_dimension_coord(
                meta,
                Some(v),
                Some(&app.plotted_store_target_input),
                ortho_name,
                global_ortho,
                full_ortho_len,
                None,
            );
            entries.insert(0, ortho_str);
            used_dims.insert(o_idx);
        } else {
            entries.insert(
                0,
                format!("series:\u{00A0}{}/{}", best_line_idx + 1, l_count),
            );
        }
    } else {
        entries.insert(
            0,
            format!("series:\u{00A0}{}/{}", best_line_idx + 1, l_count),
        );
    }
}

fn resolve_3d_dim_entries(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    sampler: &VolumeSampler,
    hit_x: usize,
    hit_y: usize,
    hit_z: usize,
) -> Vec<String> {
    let data_x = (hit_x + sampler.shift_x) % sampler.width;
    let data_y = (hit_y + sampler.shift_y) % sampler.height;
    let data_z = (hit_z + sampler.shift_z) % sampler.depth;

    let mut used_dims = HashSet::new();

    if let Some(v) = var {
        let (explicit_x, explicit_y, explicit_z) =
            v.resolve_spatial_dim_indices(if !app.plotted_dim_config.is_empty() {
                &app.plotted_dim_config
            } else {
                &app.dim_config
            });

        let x_idx = explicit_x.unwrap_or(v.dimension_names.len().saturating_sub(1));
        let y_idx = explicit_y.unwrap_or(v.dimension_names.len().saturating_sub(2));
        let z_idx =
            explicit_z.or_else(|| (0..v.dimension_names.len()).find(|&i| i != x_idx && i != y_idx));

        used_dims.insert(x_idx);
        used_dims.insert(y_idx);

        let dim_y_name = explicit_y
            .and_then(|i| v.dimension_names.get(i).cloned())
            .or_else(|| app.get_spatial_dim_name(1))
            .unwrap_or_else(|| "y".to_string());

        let dim_x_name = explicit_x
            .and_then(|i| v.dimension_names.get(i).cloned())
            .or_else(|| app.get_spatial_dim_name(0))
            .unwrap_or_else(|| "x".to_string());

        let dim_z_name = z_idx
            .and_then(|i| v.dimension_names.get(i).cloned())
            .or_else(|| app.get_spatial_dim_name(2))
            .unwrap_or_else(|| "z".to_string());

        let (origin_x, full_x_len) = get_dimension_origin_and_full_len(app, Some(v), x_idx);
        let (origin_y, full_y_len) = get_dimension_origin_and_full_len(app, Some(v), y_idx);

        let global_x = (origin_x + data_x).min(full_x_len.saturating_sub(1));
        let global_y = (origin_y + data_y).min(full_y_len.saturating_sub(1));

        let loc_y = format_dimension_coord(
            meta,
            Some(v),
            Some(&app.plotted_store_target_input),
            &dim_y_name,
            global_y,
            full_y_len,
            None,
        );
        let loc_x = format_dimension_coord(
            meta,
            Some(v),
            Some(&app.plotted_store_target_input),
            &dim_x_name,
            global_x,
            full_x_len,
            None,
        );

        let mut list = vec![loc_y, loc_x];

        if sampler.depth > 1 {
            let z_dim = z_idx.unwrap_or(0);
            if let Some(zi) = z_idx {
                used_dims.insert(zi);
            }
            let (origin_z, full_z_len) = get_dimension_origin_and_full_len(app, Some(v), z_dim);
            let global_z = (origin_z + data_z).min(full_z_len.saturating_sub(1));

            let loc_z = format_dimension_coord(
                meta,
                Some(v),
                Some(&app.plotted_store_target_input),
                &dim_z_name,
                global_z,
                full_z_len,
                None,
            );
            list.insert(0, loc_z);
        }

        enrich_entries_with_animated_and_collapsed_dims(
            app,
            meta,
            Some(v),
            &mut list,
            &mut used_dims,
        );

        list
    } else if sampler.depth > 1 {
        let dim_z_name = app
            .get_spatial_dim_name(2)
            .unwrap_or_else(|| "z".to_string());
        let dim_y_name = app
            .get_spatial_dim_name(1)
            .unwrap_or_else(|| "y".to_string());
        let dim_x_name = app
            .get_spatial_dim_name(0)
            .unwrap_or_else(|| "x".to_string());
        vec![
            format!("{}:\u{00A0}{}/{}", dim_z_name, data_z + 1, sampler.depth),
            format!("{}:\u{00A0}{}/{}", dim_y_name, data_y + 1, sampler.height),
            format!("{}:\u{00A0}{}/{}", dim_x_name, data_x + 1, sampler.width),
        ]
    } else {
        let dim_y_name = app
            .get_spatial_dim_name(1)
            .unwrap_or_else(|| "y".to_string());
        let dim_x_name = app
            .get_spatial_dim_name(0)
            .unwrap_or_else(|| "x".to_string());
        vec![
            format!("{}:\u{00A0}{}/{}", dim_y_name, data_y + 1, sampler.height),
            format!("{}:\u{00A0}{}/{}", dim_x_name, data_x + 1, sampler.width),
        ]
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_2d_dim_entries(
    app: &OctantApp,
    meta: Option<&DatasetMetadata>,
    var: Option<&VariableInfo>,
    px: usize,
    py: usize,
    orig_w: usize,
    orig_h: usize,
    geo_coords: Option<(f32, f32)>,
    used_dims: &mut HashSet<usize>,
) -> Vec<String> {
    if let Some(v) = var {
        let (explicit_x, explicit_y, _) =
            v.resolve_spatial_dim_indices(if !app.plotted_dim_config.is_empty() {
                &app.plotted_dim_config
            } else {
                &app.dim_config
            });

        let x_idx = explicit_x.unwrap_or(v.dimension_names.len().saturating_sub(1));
        let y_idx = explicit_y.unwrap_or(v.dimension_names.len().saturating_sub(2));

        used_dims.insert(x_idx);
        used_dims.insert(y_idx);

        let dim_y_name = explicit_y
            .and_then(|i| v.dimension_names.get(i))
            .cloned()
            .unwrap_or_else(|| "y".to_string());

        let dim_x_name = explicit_x
            .and_then(|i| v.dimension_names.get(i))
            .cloned()
            .unwrap_or_else(|| "x".to_string());

        let geo_y = geo_coords.map(|(lat, _)| lat);
        let geo_x = geo_coords.map(|(_, lon)| lon);

        let (origin_x, full_x_len) = get_dimension_origin_and_full_len(app, Some(v), x_idx);
        let (origin_y, full_y_len) = get_dimension_origin_and_full_len(app, Some(v), y_idx);

        let global_x = (origin_x + px).min(full_x_len.saturating_sub(1));
        let global_y = (origin_y + py).min(full_y_len.saturating_sub(1));

        let loc_y = format_dimension_coord(
            meta,
            Some(v),
            Some(&app.plotted_store_target_input),
            &dim_y_name,
            global_y,
            full_y_len,
            geo_y,
        );
        let loc_x = format_dimension_coord(
            meta,
            Some(v),
            Some(&app.plotted_store_target_input),
            &dim_x_name,
            global_x,
            full_x_len,
            geo_x,
        );

        let mut list = vec![loc_y, loc_x];
        enrich_entries_with_animated_and_collapsed_dims(app, meta, Some(v), &mut list, used_dims);

        list
    } else {
        vec![
            format!("y:\u{00A0}{}/{}", py + 1, orig_h),
            format!("x:\u{00A0}{}/{}", px + 1, orig_w),
        ]
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_target_screen_pos(
    app: &OctantApp,
    matrix: &MatrixData,
    camera: &Camera3D,
    sampler: &VolumeSampler,
    transform_2d: &Transform2D,
    px: usize,
    py: usize,
    raw_val: f32,
    point_3d_hit: Option<(usize, usize, usize, f32)>,
) -> Option<Pos2> {
    match app.active_plot_type {
        PlotType::Sphere => sphere_target_pos(app, matrix, camera, px, py, raw_val),
        PlotType::Surface => surface_target_pos(app, matrix, camera, px, py, raw_val),
        PlotType::PointCloud | PlotType::Volume => {
            if let Some((hit_x, hit_y, hit_z, _)) = point_3d_hit {
                let aspects = app.get_3d_aspect_ratio();
                volume_target_pos(
                    camera,
                    (hit_x, hit_y, hit_z),
                    (sampler.width, sampler.height, sampler.depth),
                    aspects,
                )
            } else {
                None
            }
        }
        PlotType::Heatmap | PlotType::Block => {
            let (orig_w, orig_h) = if let Some(pyr) = &app.active_pyramid {
                (pyr.original_width, pyr.original_height)
            } else {
                (matrix.width, matrix.height)
            };
            let (u_c, v_c) = matrix.grid.cell_center_norm(px, py, orig_w, orig_h);
            Some(transform_2d.norm_to_screen(u_c, v_c))
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_tooltip_card(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    rect: Rect,
    hover_pos: Pos2,
    target_pos: Option<Pos2>,
    plot_type: PlotType,
    var_name: &str,
    raw_val: f32,
    units_str: &str,
    dim_entries: &[String],
) {
    let val_formatted = if raw_val.is_nan() {
        "NaN".to_string()
    } else if raw_val.abs() >= 1e4 || (raw_val.abs() <= 1e-3 && raw_val != 0.0) {
        format!("{:.4e}", raw_val)
    } else {
        format!("{:.4}", raw_val)
    };

    let style = ctx.style_of(ctx.theme());
    let strong_text = style.visuals.strong_text_color();
    let text_color = style.visuals.text_color();

    let screen_rect = ctx.input(|i| i.viewport_rect());
    let tooltip_w = 210.0;
    let tooltip_est_h = if dim_entries.len() > 2 { 84.0 } else { 68.0 };

    let is_connected_mode = plot_type != PlotType::Line;
    let mut tooltip_pos = if is_connected_mode {
        let offset_x = if hover_pos.x >= rect.center().x {
            36.0
        } else {
            -tooltip_w - 36.0
        };
        let offset_y = if hover_pos.y >= rect.center().y {
            -tooltip_est_h - 18.0
        } else {
            18.0
        };
        Pos2::new(hover_pos.x + offset_x, hover_pos.y + offset_y)
    } else {
        Pos2::new(hover_pos.x + 14.0, hover_pos.y + 14.0)
    };

    if tooltip_pos.x + tooltip_w > screen_rect.max.x - 10.0 {
        tooltip_pos.x = screen_rect.max.x - tooltip_w - 10.0;
    }
    if tooltip_pos.x < screen_rect.min.x + 10.0 {
        tooltip_pos.x = screen_rect.min.x + 10.0;
    }
    if tooltip_pos.y + tooltip_est_h > screen_rect.max.y - 10.0 {
        tooltip_pos.y = screen_rect.max.y - tooltip_est_h - 10.0;
    }
    if tooltip_pos.y < screen_rect.min.y + 10.0 {
        tooltip_pos.y = screen_rect.min.y + 10.0;
    }

    let tooltip_rect = Rect::from_min_size(tooltip_pos, egui::vec2(tooltip_w, tooltip_est_h));

    if let Some(target) = target_pos {
        draw_leader_callout(ui.painter(), ctx, target, tooltip_rect);
    }

    egui::Area::new(egui::Id::new("octant_hover_pixel_tooltip"))
        .order(egui::Order::Tooltip)
        .fixed_pos(tooltip_pos)
        .show(ctx, |ui| {
            egui::Frame::window(ui.style())
                .inner_margin(egui::Margin::symmetric(8, 5))
                .show(ui, |ui| {
                    ui.set_max_width(tooltip_w - 16.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(var_name)
                                .small()
                                .strong()
                                .color(strong_text),
                        );

                        ui.add_space(2.0);

                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Val:").small().color(text_color));
                            ui.label(
                                egui::RichText::new(format!("{}{}", val_formatted, units_str))
                                    .size(15.0)
                                    .strong()
                                    .color(strong_text),
                            );
                        });

                        ui.add_space(1.0);

                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing.x = 4.0;
                            ui.spacing_mut().item_spacing.y = 1.0;
                            for (idx, entry) in dim_entries.iter().enumerate() {
                                if idx > 0 {
                                    ui.label(
                                        egui::RichText::new("•")
                                            .size(8.0)
                                            .color(text_color.linear_multiply(0.4)),
                                    );
                                }
                                ui.label(egui::RichText::new(entry).small().color(text_color));
                            }
                        });
                    });
                });
        });
}
