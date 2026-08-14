use crate::app::OctantApp;
use crate::plots::PlotType;
use egui::{Pos2, Rect, Stroke};

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

    let matrix = match &app.matrix_data {
        Some(m) if m.width > 0 && m.height > 0 && !m.values.is_empty() => m,
        _ => return,
    };

    // 1. Calculate Normalized Coordinates (norm_x, norm_y) considering 2D vs 3D Plot Mode
    let (norm_x, norm_y, is_valid_hit, geo_coords) = match app.active_plot_type {
        PlotType::Sphere => {
            // 3D Globe Projection Inverse Raycast
            let center = rect.center();
            let half_size = (rect.height() / 2.0).max(1.0);
            let zoom = app.sphere_zoom.max(0.1);

            let ndc_x = (hover_pos.x - center.x) / (half_size * 0.7 * zoom);
            let ndc_y = (hover_pos.y - center.y) / (half_size * 0.7 * zoom);

            let r_sq = ndc_x * ndc_x + ndc_y * ndc_y;
            if r_sq > 1.0 {
                return;
            }

            let view_x = ndc_x;
            let view_y = -ndc_y;
            let view_z = (1.0 - r_sq).sqrt();

            let rx = app.sphere_rotation_x;
            let ry = app.sphere_rotation_y;

            let y1 = view_y * rx.cos() + view_z * rx.sin();
            let z1 = -view_y * rx.sin() + view_z * rx.cos();
            let x1 = view_x;

            let x_model = x1 * ry.cos() - z1 * ry.sin();
            let y_model = y1;
            let z_model = x1 * ry.sin() + z1 * ry.cos();

            let lat_rad = y_model.clamp(-1.0, 1.0).asin();
            let lon_rad = x_model.atan2(z_model);

            let lat_deg = lat_rad.to_degrees();
            let lon_deg = lon_rad.to_degrees();

            let u = (lon_rad + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
            let v = (lat_rad + std::f32::consts::FRAC_PI_2) / std::f32::consts::PI;

            let nx = u.clamp(0.0, 1.0);
            let ny = (1.0 - v).clamp(0.0, 1.0);

            (nx, ny, true, Some((lat_deg, lon_deg)))
        }
        PlotType::Surface | PlotType::Volume | PlotType::PointCloud => {
            // 3D Surface / Box Projection Inverse Raycast
            let center = rect.center();
            let half_size = (rect.height() / 2.0).max(1.0);
            let zoom = app.sphere_zoom.max(0.1);

            let ndc_x = (hover_pos.x - center.x) / (half_size * 0.8 * zoom);
            let ndc_y = (hover_pos.y - center.y) / (half_size * 0.8 * zoom);

            let rx = app.sphere_rotation_x;
            let ry = app.sphere_rotation_y;

            let view_x = ndc_x;
            let view_y = -ndc_y;
            let view_z = 0.0;

            let _y1 = view_y * rx.cos() + view_z * rx.sin();
            let z1 = -view_y * rx.sin() + view_z * rx.cos();
            let x1 = view_x;

            let x_model = x1 * ry.cos() - z1 * ry.sin();
            let z_model = x1 * ry.sin() + z1 * ry.cos();

            let nx = (x_model + 0.5).clamp(0.0, 1.0);
            let ny = (z_model + 0.5).clamp(0.0, 1.0);

            (nx, ny, true, None)
        }
        PlotType::Line => {
            // 1D Line Plot Direct Inverse Mapping matching shader vertex transformation (zero gap)
            let is_inside = rect.contains(hover_pos);
            let zoom = app.line_zoom;
            let gpu_pan_x = app.line_pan.x / (0.5 * rect.width().max(1.0));
            let gpu_pan_y = -app.line_pan.y / (0.5 * rect.height().max(1.0));

            let ndc_x = ((hover_pos.x - rect.min.x) / rect.width().max(1.0)) * 2.0 - 1.0;
            let unpanned_x = (ndc_x - gpu_pan_x) / zoom.max(0.01);
            let nx = ((unpanned_x + 1.0) / 2.0).clamp(0.0, 1.0);

            let ndc_y = 1.0 - ((hover_pos.y - rect.min.y) / rect.height().max(1.0)) * 2.0;
            let unpanned_y = (ndc_y - gpu_pan_y) / zoom.max(0.01);
            let ny = ((unpanned_y + 1.0) / 2.0).clamp(0.0, 1.0);

            (nx, ny, is_inside, None)
        }
        _ => {
            // 2D Heatmap Direct Mapping within canvas_rect
            let (aspect_scale_x, aspect_scale_y) = if app.enforce_data_aspect_ratio
                && let Some(matrix) = &app.matrix_data
            {
                let data_aspect = (matrix.width as f32 / matrix.height as f32).max(0.001);
                let canvas_aspect = rect.width() / rect.height().max(1.0);
                if canvas_aspect > data_aspect {
                    (data_aspect / canvas_aspect, 1.0)
                } else {
                    (1.0, canvas_aspect / data_aspect)
                }
            } else {
                (1.0, 1.0)
            };

            let zoom = app.heatmap_zoom;
            let pan = app.heatmap_pan;
            let gpu_pan_x = pan.x / (0.5 * rect.width().max(1.0));
            let gpu_pan_y = -pan.y / (0.5 * rect.height().max(1.0));

            let ndc_x = ((hover_pos.x - rect.min.x) / rect.width().max(1.0)) * 2.0 - 1.0;
            let unpanned_x = (ndc_x - gpu_pan_x) / zoom.max(0.01);
            let unscaled_x = unpanned_x / aspect_scale_x.max(0.001);
            let nx = ((unscaled_x + 1.0) / 2.0).clamp(0.0, 1.0);

            let ndc_y = 1.0 - ((hover_pos.y - rect.min.y) / rect.height().max(1.0)) * 2.0;
            let unpanned_y = (ndc_y - gpu_pan_y) / zoom.max(0.01);
            let unscaled_y = unpanned_y / aspect_scale_y.max(0.001);
            let ny = ((1.0 - unscaled_y) / 2.0).clamp(0.0, 1.0);

            let is_inside = rect.contains(hover_pos);
            (nx, ny, is_inside, None)
        }
    };

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

    let var_name = var
        .map(|v| v.name.clone())
        .unwrap_or_else(|| "Scalar Field".to_string());

    let units_str = var
        .and_then(|v| {
            v.units
                .as_ref()
                .or_else(|| v.attributes.get("units"))
                .or_else(|| v.attributes.get("unit"))
                .or_else(|| v.attributes.get("UNITS"))
        })
        .map(|u| format!(" [{u}]"))
        .unwrap_or_default();

    // 3. Extract Pixel Value & Location Info based on Active Plot Type
    let (raw_val, dim_entries, px, py) = if app.active_plot_type == PlotType::Line {
        let (profile_values, profile_length, line_count) = app.get_line_profile_payload();
        let prof_len = profile_length as usize;
        let l_count = line_count as usize;

        let sample_idx = if prof_len > 1 {
            ((norm_x * (prof_len - 1) as f32) + 0.5) as usize
        } else {
            0
        }
        .min(prof_len.saturating_sub(1));

        let cmin = app.color_range_min;
        let cmax = app.color_range_max;
        let range = (cmax - cmin).max(1e-6);

        // Find the closest line series to the cursor's Y position
        let mut best_line_idx = 0usize;
        let mut best_dist = f32::INFINITY;
        let mut best_val = f32::NAN;

        if l_count > 0 {
            for line_idx in 0..l_count {
                let idx = line_idx * prof_len + sample_idx;
                if let Some(&v) = profile_values.get(idx) && !v.is_nan() && v.is_finite() {
                    let norm_y_val = (((v - cmin) / range) * 2.0 - 1.0).clamp(-1.0, 1.0);
                    let dist = (norm_y_val - (norm_y * 2.0 - 1.0)).abs();
                    if dist < best_dist {
                        best_dist = dist;
                        best_line_idx = line_idx;
                        best_val = v;
                    }
                }
            }
        }

        let val = if !best_val.is_nan() {
            best_val
        } else {
            profile_values.get(sample_idx).copied().unwrap_or(f32::NAN)
        };

        let dim_name = app
            .get_spatial_dim_name(app.line_profile_dim_idx)
            .unwrap_or_else(|| match app.line_profile_dim_idx {
                2 => "z".to_string(),
                1 => "y".to_string(),
                _ => "x".to_string(),
            });

        let loc_str = format_dimension_coord(meta, &dim_name, sample_idx, prof_len, None);
        let mut entries = vec![loc_str];

        if l_count > 1 {
            // Include orthogonal dimension / series coordinate
            if let Some(v) = var {
                let (explicit_x, explicit_y, _) =
                    v.resolve_spatial_dim_indices(if !app.plotted_dim_config.is_empty() {
                        &app.plotted_dim_config
                    } else {
                        &app.dim_config
                    });

                let ortho_dim_name = match app.line_profile_dim_idx {
                    0 => explicit_y.and_then(|i| v.dimension_names.get(i)),
                    1 => explicit_x.and_then(|i| v.dimension_names.get(i)),
                    _ => None,
                };

                if let Some(ortho_name) = ortho_dim_name {
                    let ortho_str =
                        format_dimension_coord(meta, ortho_name, best_line_idx, l_count, None);
                    entries.insert(0, ortho_str);
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

        (val, entries, sample_idx, best_line_idx)
    } else {
        let px = ((norm_x * (matrix.width as f32 - 1.0)) + 0.5) as usize;
        let py = ((norm_y * (matrix.height as f32 - 1.0)) + 0.5) as usize;
        let px = px.min(matrix.width - 1);
        let py = py.min(matrix.height - 1);

        let idx = py * matrix.width + px;
        let val = matrix.values.get(idx).copied().unwrap_or(f32::NAN);

        let entries = if let Some(v) = var {
            let (explicit_x, explicit_y, _) =
                v.resolve_spatial_dim_indices(if !app.plotted_dim_config.is_empty() {
                    &app.plotted_dim_config
                } else {
                    &app.dim_config
                });

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

            let loc_y = format_dimension_coord(meta, &dim_y_name, py, matrix.height, geo_y);
            let loc_x = format_dimension_coord(meta, &dim_x_name, px, matrix.width, geo_x);

            let mut list = vec![loc_y, loc_x];

            if v.shape.len() >= 3 {
                let total_steps = app
                    .animated_dim_extent()
                    .max(v.shape.first().copied().unwrap_or(1) as usize);

                let step_dim_name = app
                    .animated_dim
                    .and_then(|i| v.dimension_names.get(i))
                    .cloned()
                    .or_else(|| v.dimension_names.first().cloned())
                    .unwrap_or_else(|| "time".to_string());

                let time_coord = meta.and_then(|m| {
                    m.dimension_coordinates
                        .get(&step_dim_name.to_lowercase())
                        .or_else(|| m.dimension_coordinates.get(&step_dim_name))
                        .and_then(|coords| {
                            if coords.len() == total_steps {
                                coords.get(app.current_timestep).cloned()
                            } else if coords.len() >= 2
                                && let (Some(first), Some(last)) = (coords.first(), coords.last())
                                && let (Ok(f_v), Ok(l_v)) =
                                    (first.parse::<f64>(), last.parse::<f64>())
                            {
                                let t = if total_steps > 1 {
                                    app.current_timestep as f64 / (total_steps - 1) as f64
                                } else {
                                    0.0
                                };
                                let val = f_v + t * (l_v - f_v);
                                Some(format!("{:.2}", val))
                            } else {
                                coords.get(app.current_timestep).cloned()
                            }
                        })
                });

                let formatted_val = if let Some(tc) = time_coord {
                    let is_raw_numeric = tc.parse::<f64>().is_ok()
                        && !tc.contains('-')
                        && !tc.contains(':')
                        && !tc.contains('/')
                        && !tc.contains('T');

                    if !is_raw_numeric && !tc.trim().is_empty() {
                        tc
                    } else {
                        crate::utils::units::format_axis_value(
                            app.current_timestep,
                            total_steps,
                            Some(&step_dim_name),
                            v.units
                                .as_deref()
                                .or(v.attributes.get("units").map(|s| s.as_str())),
                            v.time_coverage_start
                                .as_deref()
                                .or(v.attributes.get("time_coverage_start").map(|s| s.as_str())),
                            v.temporal_resolution
                                .as_deref()
                                .or(v.attributes.get("temporal_resolution").map(|s| s.as_str())),
                            Some(&app.plotted_store_target_input),
                        )
                    }
                } else {
                    crate::utils::units::format_axis_value(
                        app.current_timestep,
                        total_steps,
                        Some(&step_dim_name),
                        v.units
                            .as_deref()
                            .or(v.attributes.get("units").map(|s| s.as_str())),
                        v.time_coverage_start
                            .as_deref()
                            .or(v.attributes.get("time_coverage_start").map(|s| s.as_str())),
                        v.temporal_resolution
                            .as_deref()
                            .or(v.attributes.get("temporal_resolution").map(|s| s.as_str())),
                        Some(&app.plotted_store_target_input),
                    )
                };

                let time_display = format!(
                    "{}:\u{00A0}{}",
                    step_dim_name,
                    formatted_val.replace(' ', "\u{00A0}")
                );
                list.push(time_display);
            }

            list
        } else {
            vec![format!("y:\u{00A0}{}", py), format!("x:\u{00A0}{}", px)]
        };

        (val, entries, px, py)
    };

    // 4. Draw Glowing Reticle Marker Dot & Guide Line on 1D Line Plot
    if app.active_plot_type == PlotType::Line {
        let (profile_values, profile_length, line_count) = app.get_line_profile_payload();
        let prof_len = profile_length as usize;
        let l_count = line_count as usize;
        if prof_len > 0 && !profile_values.is_empty() {
            let sample_idx = px.min(prof_len - 1);
            let line_idx = py.min(l_count.saturating_sub(1));
            let data_idx = line_idx * prof_len + sample_idx;
            let val = profile_values.get(data_idx).copied().unwrap_or(raw_val);

            let min_val = app.color_range_min;
            let max_val = app.color_range_max;
            let range = (max_val - min_val).max(1e-6);

            let vertex_norm_x = if prof_len > 1 {
                (sample_idx as f32 / (prof_len - 1) as f32) * 2.0 - 1.0
            } else {
                0.0
            };
            let vertex_norm_y = if val.is_nan() {
                -1.0
            } else {
                (((val - min_val) / range) * 2.0 - 1.0).clamp(-1.0, 1.0)
            };

            let zoom = app.line_zoom;
            let gpu_pan_x = app.line_pan.x / (0.5 * rect.width().max(1.0));
            let gpu_pan_y = -app.line_pan.y / (0.5 * rect.height().max(1.0));

            let transformed_pos_x = vertex_norm_x * zoom + gpu_pan_x;
            let transformed_pos_y = vertex_norm_y * zoom + gpu_pan_y;

            let dot_x = rect.min.x + ((transformed_pos_x + 1.0) / 2.0) * rect.width();
            let dot_y = rect.min.y + ((1.0 - transformed_pos_y) / 2.0) * rect.height();
            let dot_pos = Pos2::new(dot_x, dot_y);

            // Compute data/axis bounds on screen for guidelines (spanning full axis range from start to end)
            let x_start_ndc = -1.0 * zoom + gpu_pan_x;
            let x_end_ndc = 1.0 * zoom + gpu_pan_x;
            let y_top_ndc = 1.0 * zoom + gpu_pan_y;
            let y_bottom_ndc = -1.0 * zoom + gpu_pan_y;

            let x_line_start = rect.min.x + ((x_start_ndc + 1.0) / 2.0) * rect.width();
            let x_line_end = rect.min.x + ((x_end_ndc + 1.0) / 2.0) * rect.width();
            let y_line_top = rect.min.y + ((1.0 - y_top_ndc) / 2.0) * rect.height();
            let y_line_bottom = rect.min.y + ((1.0 - y_bottom_ndc) / 2.0) * rect.height();

            let x_axis_min = x_line_start.min(x_line_end).clamp(rect.min.x, rect.max.x);
            let x_axis_max = x_line_start.max(x_line_end).clamp(rect.min.x, rect.max.x);
            let y_axis_min = y_line_top.min(y_line_bottom).clamp(rect.min.y, rect.max.y);
            let y_axis_max = y_line_top.max(y_line_bottom).clamp(rect.min.y, rect.max.y);

            let visuals = &ctx.style_of(ctx.theme()).visuals;
            let strong_color = visuals.strong_text_color();
            let text_color = visuals.text_color();
            let line_color = visuals.widgets.noninteractive.fg_stroke.color;

            let painter = ui.painter();

            // Full-span Vertical guideline from top axis limit to bottom axis limit
            if dot_x >= rect.min.x && dot_x <= rect.max.x {
                painter.line_segment(
                    [Pos2::new(dot_x, y_axis_min), Pos2::new(dot_x, y_axis_max)],
                    Stroke::new(1.0, line_color.linear_multiply(0.7)),
                );
            }

            // Full-span Horizontal guideline from left axis limit to right axis limit
            if dot_y >= rect.min.y && dot_y <= rect.max.y {
                painter.line_segment(
                    [Pos2::new(x_axis_min, dot_y), Pos2::new(x_axis_max, dot_y)],
                    Stroke::new(1.0, line_color.linear_multiply(0.7)),
                );
            }

            // Only draw reticle dot if inside visible canvas
            if rect.contains(dot_pos) {
                // Subtle system theme aura halo
                painter.circle_filled(dot_pos, 8.0, text_color.linear_multiply(0.12));
                painter.circle_filled(dot_pos, 5.0, text_color.linear_multiply(0.25));

                // Inner high-contrast system ring
                painter.circle_stroke(dot_pos, 4.0, Stroke::new(1.5, strong_color));

                // Solid system center core
                painter.circle_filled(dot_pos, 2.0, strong_color);
            }
        }
    }

    // 5. Draw Subtle Reticle Dot & Crosshair on 2D Canvas
    if app.active_plot_type == PlotType::Heatmap {
        let (aspect_scale_x, aspect_scale_y) = if app.enforce_data_aspect_ratio
            && let Some(matrix) = &app.matrix_data
        {
            let data_aspect = (matrix.width as f32 / matrix.height as f32).max(0.001);
            let canvas_aspect = rect.width() / rect.height().max(1.0);
            if canvas_aspect > data_aspect {
                (data_aspect / canvas_aspect, 1.0)
            } else {
                (1.0, canvas_aspect / data_aspect)
            }
        } else {
            (1.0, 1.0)
        };

        let zoom = app.heatmap_zoom;
        let pan = app.heatmap_pan;
        let gpu_pan_x = pan.x / (0.5 * rect.width().max(1.0));
        let gpu_pan_y = -pan.y / (0.5 * rect.height().max(1.0));

        let norm_pixel_x = ((px as f32 + 0.5) / matrix.width as f32) * 2.0 - 1.0;
        let norm_pixel_y = 1.0 - ((py as f32 + 0.5) / matrix.height as f32) * 2.0;

        let ndc_x = norm_pixel_x * aspect_scale_x * zoom + gpu_pan_x;
        let ndc_y = norm_pixel_y * aspect_scale_y * zoom + gpu_pan_y;

        let px_center_x = rect.min.x + ((ndc_x + 1.0) / 2.0) * rect.width();
        let px_center_y = rect.min.y + ((1.0 - ndc_y) / 2.0) * rect.height();
        let crosshair_pos = Pos2::new(px_center_x, px_center_y);

        if rect.contains(crosshair_pos) {
            let visuals = &ctx.style_of(ctx.theme()).visuals;
            let strong_color = visuals.strong_text_color();
            let text_color = visuals.text_color();

            let painter = ui.painter();

            painter.circle_stroke(
                crosshair_pos,
                5.0,
                Stroke::new(1.8_f32, text_color.linear_multiply(0.2)),
            );
            painter.circle_stroke(
                crosshair_pos,
                5.0,
                Stroke::new(1.0_f32, strong_color),
            );
            painter.circle_filled(
                crosshair_pos,
                2.0,
                strong_color,
            );

            let arm_len = 8.0;
            painter.line_segment(
                [
                    Pos2::new(crosshair_pos.x - arm_len, crosshair_pos.y),
                    Pos2::new(crosshair_pos.x + arm_len, crosshair_pos.y),
                ],
                Stroke::new(1.0_f32, strong_color.linear_multiply(0.7)),
            );
            painter.line_segment(
                [
                    Pos2::new(crosshair_pos.x, crosshair_pos.y - arm_len),
                    Pos2::new(crosshair_pos.x, crosshair_pos.y + arm_len),
                ],
                Stroke::new(1.0_f32, strong_color.linear_multiply(0.7)),
            );
        }
    }

    // 6. Format Value String
    let val_formatted = if raw_val.is_nan() {
        "NaN".to_string()
    } else if raw_val.abs() >= 1e4 || (raw_val.abs() <= 1e-3 && raw_val != 0.0) {
        format!("{:.4e}", raw_val)
    } else {
        format!("{:.4}", raw_val)
    };

    // 7. Render Floating Glassmorphic Tooltip Window near Cursor
    let style = ctx.style_of(ctx.theme());
    let strong_text = style.visuals.strong_text_color();
    let text_color = style.visuals.text_color();

    let screen_rect = ctx.input(|i| i.viewport_rect());
    let tooltip_w = 210.0;
    let tooltip_est_h = if dim_entries.len() > 2 { 84.0 } else { 68.0 };

    let mut tooltip_pos = Pos2::new(hover_pos.x + 14.0, hover_pos.y + 14.0);
    if tooltip_pos.x + tooltip_w > screen_rect.max.x - 10.0 {
        tooltip_pos.x = hover_pos.x - tooltip_w - 10.0;
    }
    if tooltip_pos.y + tooltip_est_h > screen_rect.max.y - 10.0 {
        tooltip_pos.y = hover_pos.y - tooltip_est_h - 10.0;
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
                        // Title / Variable Name
                        ui.label(
                            egui::RichText::new(&var_name)
                                .small()
                                .strong()
                                .color(strong_text),
                        );

                        ui.add_space(2.0);

                        // Value [Units] (Prominent 15.0pt bold font)
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

                        // Real Dimension Names & Coordinates with atomic break-wrap
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

/// Helper to format dimension coordinate values with physical units and proper cardinal degrees.
fn format_dimension_coord(
    meta: Option<&crate::data::DatasetMetadata>,
    dim_name: &str,
    idx: usize,
    total_len: usize,
    geo_fallback: Option<f32>,
) -> String {
    let clean = dim_name.trim().to_lowercase();

    if let Some(geo) = geo_fallback {
        return if clean.contains("lon") {
            let cardinal = if geo >= 0.0 { "°E" } else { "°W" };
            format!("{}:\u{00A0}{:.2}{}", dim_name, geo.abs(), cardinal)
        } else if clean.contains("lat") {
            let cardinal = if geo >= 0.0 { "°N" } else { "°S" };
            format!("{}:\u{00A0}{:.2}{}", dim_name, geo.abs(), cardinal)
        } else {
            format!("{}:\u{00A0}{:.2}°", dim_name, geo)
        };
    }

    if let Some(m) = meta {
        if let Some(coords) = m
            .dimension_coordinates
            .get(&clean)
            .or_else(|| m.dimension_coordinates.get(dim_name))
        {
            if coords.len() == total_len
                && let Some(c) = coords.get(idx)
                && !c.trim().is_empty()
            {
                return format!("{}:\u{00A0}{}", dim_name, c.replace(' ', "\u{00A0}"));
            }
            if coords.len() >= 2
                && let (Some(first), Some(last)) = (coords.first(), coords.last())
                && let (Ok(f_v), Ok(l_v)) = (first.parse::<f64>(), last.parse::<f64>())
            {
                let t = if total_len > 1 {
                    idx as f64 / (total_len - 1) as f64
                } else {
                    0.0
                };
                let val = f_v + t * (l_v - f_v);
                return if clean.contains("lon") {
                    let cardinal = if val >= 0.0 { "°E" } else { "°W" };
                    format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
                } else if clean.contains("lat") {
                    let cardinal = if val >= 0.0 { "°N" } else { "°S" };
                    format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
                } else if clean.contains("depth")
                    || clean.contains("height")
                    || clean.contains("alt")
                {
                    format!("{}:\u{00A0}{:.2}\u{00A0}m", dim_name, val)
                } else {
                    format!("{}:\u{00A0}{:.2}", dim_name, val)
                };
            }
            if let Some(first) = coords.first()
                && !first.trim().is_empty()
            {
                return format!("{}:\u{00A0}{}", dim_name, first.replace(' ', "\u{00A0}"));
            }
        }

        if let Some((min_b, max_b)) = m.get_coord_bounds(dim_name) {
            let t = if total_len > 1 {
                idx as f64 / (total_len - 1) as f64
            } else {
                0.0
            };
            let val = min_b + t * (max_b - min_b);
            return if clean.contains("lon") {
                let cardinal = if val >= 0.0 { "°E" } else { "°W" };
                format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
            } else if clean.contains("lat") {
                let cardinal = if val >= 0.0 { "°N" } else { "°S" };
                format!("{}:\u{00A0}{:.2}{}", dim_name, val.abs(), cardinal)
            } else if clean.contains("depth") || clean.contains("height") || clean.contains("alt") {
                format!("{}:\u{00A0}{:.2}\u{00A0}m", dim_name, val)
            } else {
                format!("{}:\u{00A0}{:.2}", dim_name, val)
            };
        }
    }

    format!("{}:\u{00A0}{}", dim_name, idx)
}
