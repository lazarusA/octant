use crate::app::OctantApp;
use egui::{Color32, Mesh, Pos2, Rect, Shape, Vec2, epaint::Vertex};

struct ColorbarTick {
    t_pos: f32,
    _val: f32,
    is_major: bool,
    label: Option<String>,
}

pub fn show_colorbar_overlay(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_colorbar {
        return;
    }

    let effective_colormap = app.preview_colormap.unwrap_or(app.active_colormap);

    // Read active min_val and max_val bounds (locked or dynamic)
    let (min_val, max_val) = (app.color_range_min, app.color_range_max);

    let default_label = app.default_colorbar_label();
    let mut current_label = app.colorbar_label();

    // Read current theme visual colors for high-contrast dark/light mode rendering
    let style = ctx.style_of(ctx.theme());
    let strong_text_color = style.visuals.strong_text_color();
    let text_color = style.visuals.text_color();
    let border_color = style.visuals.widgets.noninteractive.fg_stroke.color;

    // Position floating panel centered horizontally fixed right above the bottom toolbar
    let screen_rect = ctx.input(|i| i.viewport_rect());
    let panel_w = 490.0;
    let panel_h = 88.0;

    let center_x = screen_rect.center().x;
    let bottom_bar_top = screen_rect.max.y - 42.0; // Bottom bar height + margin
    let panel_min = Pos2::new(center_x - (panel_w / 2.0), bottom_bar_top - panel_h - 8.0);

    // Render system-themed glassmorphic floating area overlay
    egui::Area::new(egui::Id::new("octant_colorbar_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(panel_min)
        .show(ctx, |ui| {
            egui::Frame::window(ui.style())
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.set_width(panel_w - 24.0);

                    ui.vertical_centered(|ui| {
                        // Editable Colorbar Title
                        ui.horizontal(|ui| {
                            let avail = ui.available_width();
                            let text_w = (avail - 40.0).clamp(100.0, 320.0);
                            let pad = ((avail - text_w) / 2.0).max(0.0);
                            ui.add_space(pad);

                            let text_edit = egui::TextEdit::singleline(&mut current_label)
                                .hint_text(&default_label)
                                .font(egui::TextStyle::Body)
                                .horizontal_align(egui::Align::Center)
                                .desired_width(text_w)
                                .frame(egui::Frame::NONE);

                            let resp = ui
                                .add(text_edit)
                                .on_hover_text("Colorbar title. Click to edit.");
                            if resp.changed() {
                                if current_label.trim().is_empty() || current_label == default_label
                                {
                                    app.custom_colorbar_label = None;
                                } else {
                                    app.custom_colorbar_label = Some(current_label.clone());
                                }
                            }
                        });

                        ui.add_space(3.0);

                        // Reserve exact 400x38 rect for horizontal gradient bar + inward/outward ticks + input fields
                        let bar_w = 400.0;
                        let total_h = 38.0;

                        let (widget_rect, response) =
                            ui.allocate_exact_size(Vec2::new(bar_w, total_h), egui::Sense::hover());

                        let bar_rect = Rect::from_min_size(widget_rect.min, Vec2::new(bar_w, 13.0));

                        // Check if categorical mode is active or unique entries exist
                        let unique_vals = app
                            .matrix_data
                            .as_ref()
                            .and_then(|m| m.detect_unique_values());
                        let is_categorical_active = app.is_categorical || unique_vals.is_some();

                        if is_categorical_active {
                            let cat_vals: Vec<f32> = if let Some(unique) = unique_vals {
                                unique
                            } else {
                                // Default 10 equal bins across [min_val, max_val]
                                let range = (max_val - min_val).max(1e-30);
                                (0..10)
                                    .map(|i| min_val + (i as f32 + 0.5) / 10.0 * range)
                                    .collect()
                            };

                            let num_cats = cat_vals.len();

                            // 1. Build Discrete Color Band Mesh
                            let mut mesh = Mesh::default();
                            for (i, &val) in cat_vals.iter().enumerate() {
                                let t_start = i as f32 / num_cats as f32;
                                let t_end = (i + 1) as f32 / num_cats as f32;

                                let norm_scaled = crate::utils::colormap::apply_color_scale_cpu(
                                    val,
                                    min_val,
                                    max_val,
                                    app.active_scale_type,
                                    app.scale_param,
                                );
                                let color = crate::utils::colormap::sample_colormap_rgb(
                                    effective_colormap,
                                    norm_scaled,
                                );

                                let x_start = bar_rect.min.x + t_start * bar_rect.width();
                                let x_end = bar_rect.min.x + t_end * bar_rect.width();

                                let idx = mesh.vertices.len() as u32;
                                mesh.vertices.push(Vertex {
                                    pos: Pos2::new(x_start, bar_rect.min.y),
                                    uv: Pos2::ZERO,
                                    color,
                                });
                                mesh.vertices.push(Vertex {
                                    pos: Pos2::new(x_start, bar_rect.max.y),
                                    uv: Pos2::ZERO,
                                    color,
                                });
                                mesh.vertices.push(Vertex {
                                    pos: Pos2::new(x_end, bar_rect.min.y),
                                    uv: Pos2::ZERO,
                                    color,
                                });
                                mesh.vertices.push(Vertex {
                                    pos: Pos2::new(x_end, bar_rect.max.y),
                                    uv: Pos2::ZERO,
                                    color,
                                });

                                mesh.indices.extend_from_slice(&[
                                    idx,
                                    idx + 1,
                                    idx + 2,
                                    idx + 1,
                                    idx + 3,
                                    idx + 2,
                                ]);
                            }
                            ui.painter().add(Shape::mesh(mesh));

                            // Draw crisp outline around colorbar rect
                            ui.painter().rect_stroke(
                                bar_rect,
                                0.0,
                                egui::Stroke::new(1.0_f32, border_color),
                                egui::StrokeKind::Middle,
                            );

                            // 2. Draw Band Boundary Dividers
                            for i in 1..num_cats {
                                let t_div = i as f32 / num_cats as f32;
                                let x_div = bar_rect.min.x + t_div * bar_rect.width();
                                ui.painter().line_segment(
                                    [
                                        Pos2::new(x_div, bar_rect.min.y),
                                        Pos2::new(x_div, bar_rect.max.y),
                                    ],
                                    egui::Stroke::new(1.0_f32, Color32::from_black_alpha(120)),
                                );
                            }

                            // 3. Draw Centered Ticks and Centered Labels inside each discrete band
                            for (i, &val) in cat_vals.iter().enumerate() {
                                let t_center = (i as f32 + 0.5) / num_cats as f32;
                                let x = bar_rect.min.x + t_center * bar_rect.width();

                                let y_in = bar_rect.max.y - 4.5;
                                let y_out = bar_rect.max.y + 5.5;

                                ui.painter().line_segment(
                                    [Pos2::new(x, y_in), Pos2::new(x, y_out)],
                                    egui::Stroke::new(2.2_f32, Color32::from_black_alpha(180)),
                                );
                                ui.painter().line_segment(
                                    [Pos2::new(x, y_in), Pos2::new(x, y_out)],
                                    egui::Stroke::new(1.2_f32, strong_text_color),
                                );

                                // Avoid painting label if it overlaps the min/max input fields at ends
                                if t_center > 0.12 && t_center < 0.88 {
                                    let label_text = format_scientific_tick(val);
                                    ui.painter().text(
                                        Pos2::new(x, y_out + 2.0),
                                        egui::Align2::CENTER_TOP,
                                        label_text,
                                        egui::FontId::proportional(11.0),
                                        strong_text_color,
                                    );
                                }
                            }
                        } else {
                            // Standard Continuous Gradient Mesh
                            let num_segments = 128;
                            let mut mesh = Mesh::default();

                            for i in 0..=num_segments {
                                let t = i as f32 / num_segments as f32;
                                let raw_val = crate::utils::colormap::unscale_norm_to_value(
                                    t,
                                    min_val,
                                    max_val,
                                    app.active_scale_type,
                                    app.scale_param,
                                );
                                let norm_scaled = crate::utils::colormap::apply_color_scale_cpu(
                                    raw_val,
                                    min_val,
                                    max_val,
                                    app.active_scale_type,
                                    app.scale_param,
                                );
                                let color = crate::utils::colormap::sample_colormap_rgb(
                                    effective_colormap,
                                    norm_scaled,
                                );

                                let x = bar_rect.min.x + t * bar_rect.width();

                                let idx_top = mesh.vertices.len() as u32;
                                mesh.vertices.push(Vertex {
                                    pos: Pos2::new(x, bar_rect.min.y),
                                    uv: Pos2::ZERO,
                                    color,
                                });
                                mesh.vertices.push(Vertex {
                                    pos: Pos2::new(x, bar_rect.max.y),
                                    uv: Pos2::ZERO,
                                    color,
                                });

                                if i > 0 {
                                    let prev_top = idx_top - 2;
                                    let prev_bottom = idx_top - 1;

                                    mesh.indices.push(prev_top);
                                    mesh.indices.push(prev_bottom);
                                    mesh.indices.push(idx_top + 1);

                                    mesh.indices.push(prev_top);
                                    mesh.indices.push(idx_top + 1);
                                    mesh.indices.push(idx_top);
                                }
                            }

                            ui.painter().add(Shape::mesh(mesh));

                            // Draw crisp outline around colorbar rect
                            ui.painter().rect_stroke(
                                bar_rect,
                                0.0,
                                egui::Stroke::new(1.0_f32, border_color),
                                egui::StrokeKind::Middle,
                            );

                            // Continuous Major & Minor Ticks
                            let ticks = generate_colorbar_ticks(
                                min_val,
                                max_val,
                                app.active_scale_type,
                                app.scale_param,
                            );

                            for tick in ticks {
                                let x = bar_rect.min.x + tick.t_pos * bar_rect.width();

                                if tick.is_major {
                                    ui.painter().line_segment(
                                        [
                                            Pos2::new(x, bar_rect.min.y),
                                            Pos2::new(x, bar_rect.max.y),
                                        ],
                                        egui::Stroke::new(1.0_f32, Color32::from_black_alpha(80)),
                                    );

                                    let y_in = bar_rect.max.y - 4.5;
                                    let y_out = bar_rect.max.y + 5.5;

                                    ui.painter().line_segment(
                                        [Pos2::new(x, y_in), Pos2::new(x, y_out)],
                                        egui::Stroke::new(2.2_f32, Color32::from_black_alpha(180)),
                                    );
                                    ui.painter().line_segment(
                                        [Pos2::new(x, y_in), Pos2::new(x, y_out)],
                                        egui::Stroke::new(1.2_f32, strong_text_color),
                                    );

                                    // Render intermediate major tick labels between the lower/upper input fields
                                    if let Some(label_text) = &tick.label
                                        && tick.t_pos > 0.12
                                        && tick.t_pos < 0.88
                                    {
                                        ui.painter().text(
                                            Pos2::new(x, y_out + 2.0),
                                            egui::Align2::CENTER_TOP,
                                            label_text,
                                            egui::FontId::proportional(11.0),
                                            strong_text_color,
                                        );
                                    }
                                } else {
                                    let y_in = bar_rect.max.y - 3.0;
                                    let y_out = bar_rect.max.y + 3.5;

                                    ui.painter().line_segment(
                                        [Pos2::new(x, y_in), Pos2::new(x, y_out)],
                                        egui::Stroke::new(1.8_f32, Color32::from_black_alpha(180)),
                                    );
                                    ui.painter().line_segment(
                                        [Pos2::new(x, y_in), Pos2::new(x, y_out)],
                                        egui::Stroke::new(1.0_f32, text_color),
                                    );
                                }
                            }
                        }

                        // Low & High Clip Triangles on Colorbar Ends (when enabled in settings)
                        let tri_w = 12.0_f32;

                        if app.use_lowclip {
                            let low_tri_rect = Rect::from_min_max(
                                Pos2::new(bar_rect.min.x - tri_w, bar_rect.min.y),
                                Pos2::new(bar_rect.min.x, bar_rect.max.y),
                            );

                            crate::ui::color_picker::ShapeColorPicker::new(
                                "colorbar_lowclip_picker",
                                &mut app.lowclip_color,
                                crate::ui::color_picker::ColorShape::LeftTriangle,
                            )
                            .title("Low Clip Color (< Min)")
                            .tooltip("Low Clip color (< Min). Click to select color.")
                            .show_at(ui, low_tri_rect);
                        }

                        if app.use_highclip {
                            let high_tri_rect = Rect::from_min_max(
                                Pos2::new(bar_rect.max.x, bar_rect.min.y),
                                Pos2::new(bar_rect.max.x + tri_w, bar_rect.max.y),
                            );

                            crate::ui::color_picker::ShapeColorPicker::new(
                                "colorbar_highclip_picker",
                                &mut app.highclip_color,
                                crate::ui::color_picker::ColorShape::RightTriangle,
                            )
                            .title("High Clip Color (> Max)")
                            .tooltip("High Clip color (> Max). Click to select color.")
                            .anchor_offset(egui::Vec2::new(-170.0, -250.0))
                            .show_at(ui, high_tri_rect);
                        }

                        // Lower and Upper End Ticks Input Fields centered directly on end ticks
                        let input_w = 60.0;
                        let input_h = 18.0;
                        let drag_speed = ((max_val - min_val).abs() / 100.0).max(1e-4);

                        let mut new_min = app.color_range_min;
                        let mut new_max = app.color_range_max;

                        let min_rect = Rect::from_center_size(
                            Pos2::new(bar_rect.min.x, bar_rect.max.y + 6.0 + input_h / 2.0),
                            Vec2::new(input_w, input_h),
                        );
                        let max_rect = Rect::from_center_size(
                            Pos2::new(bar_rect.max.x, bar_rect.max.y + 6.0 + input_h / 2.0),
                            Vec2::new(input_w, input_h),
                        );

                        let min_resp = ui
                            .put(
                                min_rect,
                                egui::DragValue::new(&mut new_min)
                                    .speed(drag_speed)
                                    .custom_formatter(|val, _| format_scientific_tick(val as f32))
                                    .custom_parser(|s| s.trim().parse::<f64>().ok()),
                            )
                            .on_hover_text(
                                "Lower end range (Min). Drag to adjust or click to type.",
                            );

                        let max_resp = ui
                            .put(
                                max_rect,
                                egui::DragValue::new(&mut new_max)
                                    .speed(drag_speed)
                                    .custom_formatter(|val, _| format_scientific_tick(val as f32))
                                    .custom_parser(|s| s.trim().parse::<f64>().ok()),
                            )
                            .on_hover_text(
                                "Upper end range (Max). Drag to adjust or click to type.",
                            );

                        if min_resp.changed() || new_min != app.color_range_min {
                            app.color_range_min = new_min;
                            app.volume_cmin = new_min;
                            app.lock_color_bounds = true;
                        }
                        if max_resp.changed() || new_max != app.color_range_max {
                            app.color_range_max = new_max;
                            app.volume_cmax = new_max;
                            app.lock_color_bounds = true;
                        }

                        // Interactive Hover Tooltip
                        if let Some(hover_pos) = response.hover_pos() {
                            let norm_x =
                                ((hover_pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
                            let hover_val = crate::utils::colormap::unscale_norm_to_value(
                                norm_x,
                                min_val,
                                max_val,
                                app.active_scale_type,
                                app.scale_param,
                            );
                            response.on_hover_text(format!(
                                "Val: {}",
                                format_scientific_tick(hover_val)
                            ));
                        }
                    });
                });
        });
}

fn generate_colorbar_ticks(
    min_val: f32,
    max_val: f32,
    scale_type: u32,
    scale_param: f32,
) -> Vec<ColorbarTick> {
    let mut ticks = Vec::new();

    if scale_type == 1 {
        // Logarithmic scale major (powers of 10) & minor (2..9 subdivisions per decade) tick generation
        let safe_min = if min_val <= 1e-15 {
            1e-12_f32.min(max_val * 1e-6)
        } else {
            min_val
        };
        let safe_max = max_val.max(safe_min * 1.0001);

        let log_min = safe_min.log10();
        let log_max = safe_max.log10();
        let log_range = (log_max - log_min).max(1e-6);
        let gamma = if scale_param > 0.0 && scale_param != 1.0 {
            scale_param
        } else {
            1.0
        };

        if log_range >= 0.8 {
            let dec_start = log_min.floor() as i32;
            let dec_end = log_max.ceil() as i32;

            for dec in (dec_start - 1)..=dec_end {
                let base = 10.0_f32.powi(dec);

                // Major tick at 1 * 10^dec
                if base >= safe_min * 0.999 && base <= safe_max * 1.001 {
                    let norm_linear = ((base.log10() - log_min) / log_range).clamp(0.0, 1.0);
                    let t_pos = norm_linear.powf(gamma);
                    ticks.push(ColorbarTick {
                        t_pos,
                        _val: base,
                        is_major: true,
                        label: Some(format_scientific_tick(base)),
                    });
                }

                // Minor ticks at m * 10^dec for m in 2..9
                for m in 2..10 {
                    let m_val = m as f32 * base;
                    if m_val > safe_min && m_val < safe_max {
                        let norm_linear = ((m_val.log10() - log_min) / log_range).clamp(0.0, 1.0);
                        let t_pos = norm_linear.powf(gamma);
                        ticks.push(ColorbarTick {
                            t_pos,
                            _val: m_val,
                            is_major: false,
                            label: None,
                        });
                    }
                }
            }

            // Ensure min and max bounds are present as major ticks if not already added
            if !ticks.iter().any(|t| (t.t_pos - 0.0).abs() < 0.02) {
                ticks.push(ColorbarTick {
                    t_pos: 0.0,
                    _val: safe_min,
                    is_major: true,
                    label: Some(format_scientific_tick(safe_min)),
                });
            }
            if !ticks.iter().any(|t| (t.t_pos - 1.0).abs() < 0.02) {
                ticks.push(ColorbarTick {
                    t_pos: 1.0,
                    _val: safe_max,
                    is_major: true,
                    label: Some(format_scientific_tick(safe_max)),
                });
            }

            ticks.sort_by(|a, b| a.t_pos.partial_cmp(&b.t_pos).unwrap());
            return ticks;
        }
    }

    // Default Linear & Standard Scale Ticks (5 major ticks, 4 minor subdivisions per interval)
    let major_positions = [0.00, 0.25, 0.50, 0.75, 1.00];
    for &t_maj in &major_positions {
        let val = crate::utils::colormap::unscale_norm_to_value(
            t_maj,
            min_val,
            max_val,
            scale_type,
            scale_param,
        );
        ticks.push(ColorbarTick {
            t_pos: t_maj,
            _val: val,
            is_major: true,
            label: Some(format_scientific_tick(val)),
        });
    }

    // Add 4 minor subdivisions between each pair of major ticks
    for i in 0..4 {
        let t_start = major_positions[i];
        let t_end = major_positions[i + 1];
        for step in 1..5 {
            let t_min = t_start + (t_end - t_start) * (step as f32 / 5.0);
            let val = crate::utils::colormap::unscale_norm_to_value(
                t_min,
                min_val,
                max_val,
                scale_type,
                scale_param,
            );
            ticks.push(ColorbarTick {
                t_pos: t_min,
                _val: val,
                is_major: false,
                label: None,
            });
        }
    }

    ticks.sort_by(|a, b| a.t_pos.partial_cmp(&b.t_pos).unwrap());

    // De-cluttering collision pass: suppress labels if adjacent labeled ticks are closer than 0.08 (~33px)
    let min_label_spacing = 0.08;
    let mut last_labeled_t: Option<f32> = None;

    for tick in ticks.iter_mut() {
        if tick.is_major && tick.label.is_some() {
            if let Some(last_t) = last_labeled_t {
                if (tick.t_pos - last_t).abs() < min_label_spacing
                    && (1.0 - tick.t_pos).abs() > 0.01
                {
                    tick.label = None;
                } else {
                    last_labeled_t = Some(tick.t_pos);
                }
            } else {
                last_labeled_t = Some(tick.t_pos);
            }
        }
    }

    ticks
}

/// Formats tick values cleanly using integer/decimal or concise scientific notation.
pub fn format_scientific_tick(val: f32) -> String {
    let abs_val = val.abs();
    if abs_val == 0.0 {
        "0".to_string()
    } else if !(0.001..10000.0).contains(&abs_val) {
        let s = format!("{:.2e}", val);
        if let Some((mantissa, exponent)) = s.split_once('e') {
            let clean_mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
            format!("{}e{}", clean_mantissa, exponent)
        } else {
            s
        }
    } else if (val.fract()).abs() < 1e-5 {
        format!("{:.0}", val)
    } else if (val * 10.0).fract().abs() < 1e-5 {
        format!("{:.1}", val)
    } else {
        format!("{:.2}", val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DatasetMetadata, VariableInfo, matrix_data::MatrixData};
    use std::collections::HashMap;

    #[test]
    fn test_format_scientific_tick() {
        assert_eq!(format_scientific_tick(0.0), "0");
        assert_eq!(format_scientific_tick(42.0), "42");
        assert_eq!(format_scientific_tick(15.5), "15.5");
        assert_eq!(format_scientific_tick(100.25), "100.25");
        assert_eq!(format_scientific_tick(0.0000123), "1.23e-5");
        assert_eq!(format_scientific_tick(100000.0), "1e5");
        assert_eq!(format_scientific_tick(-50.0), "-50");
    }

    #[test]
    fn test_colorbar_default_and_custom_label() {
        let mut app = OctantApp::default();
        assert_eq!(app.colorbar_label(), "Scalar Field");
        assert_eq!(app.default_colorbar_label(), "Scalar Field");

        // Custom label override
        app.custom_colorbar_label = Some("Surface Temp (Celsius)".to_string());
        assert_eq!(app.colorbar_label(), "Surface Temp (Celsius)");
        assert_eq!(app.default_colorbar_label(), "Scalar Field");

        // Reset label
        app.reset_colorbar_label();
        assert_eq!(app.colorbar_label(), "Scalar Field");
        assert!(app.custom_colorbar_label.is_none());

        // With metadata and units
        let mut attrs = HashMap::new();
        attrs.insert("units".to_string(), "degK".to_string());
        let var = VariableInfo {
            name: "air_temp".to_string(),
            data_type: "float32".to_string(),
            shape: vec![10, 10],
            dimension_names: vec!["y".to_string(), "x".to_string()],
            chunk_shape: vec![10, 10],
            file_size: 400,
            units: Some("degK".to_string()),
            long_name: None,
            time_coverage_start: None,
            time_coverage_end: None,
            temporal_resolution: None,
            attributes: attrs,
        };
        let meta = DatasetMetadata {
            name: "test_dataset".to_string(),
            store_type: "zarr".to_string(),
            variables: vec![var],
            dimension_coordinates: HashMap::new(),
        };
        app.plotted_dataset_metadata = Some(meta);
        app.plotted_variable_idx = 0;

        assert_eq!(app.default_colorbar_label(), "air_temp (degK)");
        assert_eq!(app.colorbar_label(), "air_temp (degK)");

        // Set custom override over metadata default
        app.custom_colorbar_label = Some("Custom Temp".to_string());
        assert_eq!(app.colorbar_label(), "Custom Temp");
        assert_eq!(app.default_colorbar_label(), "air_temp (degK)");

        // Reset
        app.reset_colorbar_label();
        assert_eq!(app.colorbar_label(), "air_temp (degK)");
    }

    #[test]
    fn test_colorbar_range_reset() {
        let mut app = OctantApp::default();

        // Simulate matrix data with min 12.0 and max 88.0
        let mdata = MatrixData::new(
            10,
            10,
            vec![12.0; 100],
            12.0,
            88.0,
            "test_ds".to_string(),
            1,
        );
        app.matrix_data = Some(mdata);

        // User manually customized range
        app.color_range_min = 20.0;
        app.color_range_max = 50.0;
        app.lock_color_bounds = true;

        // Reset color range
        app.reset_color_range();

        assert_eq!(app.color_range_min, 12.0);
        assert_eq!(app.color_range_max, 88.0);
        assert_eq!(app.volume_cmin, 12.0);
        assert_eq!(app.volume_cmax, 88.0);
        assert!(!app.lock_color_bounds);
    }

    #[test]
    fn test_colorbar_ticks_generation_custom_bounds() {
        let ticks = generate_colorbar_ticks(10.0, 50.0, 0, 1.0);
        assert!(!ticks.is_empty());
        let major_ticks: Vec<_> = ticks.iter().filter(|t| t.is_major).collect();
        assert_eq!(major_ticks.len(), 5);

        // First major tick at 0.0 -> value 10.0
        assert!((major_ticks[0]._val - 10.0).abs() < 1e-4);
        // Last major tick at 1.0 -> value 50.0
        assert!((major_ticks[4]._val - 50.0).abs() < 1e-4);
        // Mid major tick at 0.5 -> value 30.0
        assert!((major_ticks[2]._val - 30.0).abs() < 1e-4);
    }
}
