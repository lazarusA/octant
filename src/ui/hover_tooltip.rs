use crate::app::OctantApp;
use crate::plots::PlotType;
use egui::{Color32, Pos2, Rect, Stroke};

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
                // Hover position is outside the 3D sphere globe
                return;
            }

            let view_x = ndc_x;
            let view_y = -ndc_y;
            let view_z = (1.0 - r_sq).sqrt();

            // Un-rotate around X (rot_x) and Y (rot_y)
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
        _ => {
            // 2D Heatmap & 2D Slice Direct Mapping
            let nx = ((hover_pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            let ny = ((hover_pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
            (nx, ny, true, None)
        }
    };

    if !is_valid_hit {
        return;
    }

    // 2. Map normalized position to integer matrix pixel coordinates
    let px = ((norm_x * (matrix.width as f32 - 1.0)) + 0.5) as usize;
    let py = ((norm_y * (matrix.height as f32 - 1.0)) + 0.5) as usize;
    let px = px.min(matrix.width - 1);
    let py = py.min(matrix.height - 1);

    // 3. Fast O(1) data value lookup
    let idx = py * matrix.width + px;
    let raw_val = matrix.values.get(idx).copied().unwrap_or(f32::NAN);

    // 4. Extract units & real dimension metadata
    let mut var_name = "Scalar Field".to_string();
    let mut units_str = String::new();
    let mut dim_info_str = String::new();

    if let Some(meta) = &app.active_dataset_metadata {
        if let Some(var) = meta.variables.get(app.selected_variable_idx) {
            var_name = var.name.clone();
            if let Some(unit) = var.attributes.get("units") {
                units_str = format!(" [{}]", unit);
            }

            // Real Dimension Names (e.g., latitude, longitude, time, depth)
            let dim_y_name = var
                .dimension_names
                .iter()
                .rposition(|d| d.contains("lat") || d.contains("y") || d.contains("row"))
                .and_then(|idx| var.dimension_names.get(idx))
                .cloned()
                .unwrap_or_else(|| "lat".to_string());

            let dim_x_name = var
                .dimension_names
                .iter()
                .rposition(|d| d.contains("lon") || d.contains("x") || d.contains("col"))
                .and_then(|idx| var.dimension_names.get(idx))
                .cloned()
                .unwrap_or_else(|| "lon".to_string());

            // Check if actual coordinate vectors exist in metadata
            let lat_coord_str = meta
                .dimension_coordinates
                .get(&dim_y_name)
                .and_then(|coords| coords.get(py))
                .cloned();

            let lon_coord_str = meta
                .dimension_coordinates
                .get(&dim_x_name)
                .and_then(|coords| coords.get(px))
                .cloned();

            let loc_y = if let Some(c) = lat_coord_str {
                format!("{}: {}", dim_y_name, c)
            } else if let Some((lat_deg, _)) = geo_coords {
                format!("{}: {:.2}°", dim_y_name, lat_deg)
            } else {
                format!("{}: {}", dim_y_name, py)
            };

            let loc_x = if let Some(c) = lon_coord_str {
                format!("{}: {}", dim_x_name, c)
            } else if let Some((_, lon_deg)) = geo_coords {
                format!("{}: {:.2}°", dim_x_name, lon_deg)
            } else {
                format!("{}: {}", dim_x_name, px)
            };

            if var.shape.len() >= 3 {
                let step = app.current_timestep + 1;
                let step_dim_name = var
                    .dimension_names
                    .first()
                    .map(|s| s.as_str())
                    .unwrap_or("time");
                dim_info_str = format!(
                    "{}, {} | {}: {}/{}",
                    loc_y, loc_x, step_dim_name, step, var.shape[0]
                );
            } else {
                dim_info_str = format!("{}, {}", loc_y, loc_x);
            }
        }
    }

    if dim_info_str.is_empty() {
        dim_info_str = format!("y: {}, x: {}", py, px);
    }

    // 5. Draw Subtle Reticle Dot & Crosshair on 2D Canvas
    if app.active_plot_type == PlotType::Heatmap {
        let px_center_x = rect.min.x + ((px as f32 + 0.5) / matrix.width as f32) * rect.width();
        let px_center_y = rect.min.y + ((py as f32 + 0.5) / matrix.height as f32) * rect.height();
        let crosshair_pos = Pos2::new(px_center_x, px_center_y);

        let painter = ui.painter();

        painter.circle_stroke(
            crosshair_pos,
            5.0,
            Stroke::new(1.8_f32, Color32::from_black_alpha(180)),
        );
        painter.circle_stroke(
            crosshair_pos,
            5.0,
            Stroke::new(1.0_f32, ctx.style().visuals.strong_text_color()),
        );
        painter.circle_filled(crosshair_pos, 2.0, ctx.style().visuals.strong_text_color());

        let arm_len = 8.0;
        painter.line_segment(
            [
                Pos2::new(crosshair_pos.x - arm_len, crosshair_pos.y),
                Pos2::new(crosshair_pos.x + arm_len, crosshair_pos.y),
            ],
            Stroke::new(1.0_f32, Color32::from_black_alpha(150)),
        );
        painter.line_segment(
            [
                Pos2::new(crosshair_pos.x, crosshair_pos.y - arm_len),
                Pos2::new(crosshair_pos.x, crosshair_pos.y + arm_len),
            ],
            Stroke::new(1.0_f32, Color32::from_black_alpha(150)),
        );
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
    let strong_text = ctx.style().visuals.strong_text_color();
    let text_color = ctx.style().visuals.text_color();

    let screen_rect = ctx.screen_rect();
    let tooltip_w = 230.0;
    let tooltip_h = 72.0;

    let mut tooltip_pos = Pos2::new(hover_pos.x + 16.0, hover_pos.y + 16.0);
    if tooltip_pos.x + tooltip_w > screen_rect.max.x - 10.0 {
        tooltip_pos.x = hover_pos.x - tooltip_w - 10.0;
    }
    if tooltip_pos.y + tooltip_h > screen_rect.max.y - 10.0 {
        tooltip_pos.y = hover_pos.y - tooltip_h - 10.0;
    }

    egui::Area::new(egui::Id::new("octant_hover_pixel_tooltip"))
        .order(egui::Order::Tooltip)
        .fixed_pos(tooltip_pos)
        .show(ctx, |ui| {
            egui::Frame::window(&ui.style())
                .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.set_width(tooltip_w - 20.0);
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

                        // Real Dimension Names & Coordinates
                        ui.label(egui::RichText::new(&dim_info_str).small().color(text_color));
                    });
                });
        });
}
