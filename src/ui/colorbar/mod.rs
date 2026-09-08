//! Floating Interactive Colorbar Overlay Subsystem.

pub mod handles;
pub mod ticks;

pub use handles::{draw_clip_triangles, draw_end_range_inputs};
pub use ticks::{ColorbarTick, format_scientific_tick, generate_colorbar_ticks};

use crate::app::OctantApp;
use egui::{Color32, Mesh, Pos2, Rect, Shape, Vec2, epaint::Vertex};

/// Renders the floating glassmorphic colorbar overlay panel.
pub fn show_colorbar_overlay(app: &mut OctantApp, ctx: &egui::Context) {
    if !app.show_colorbar {
        return;
    }

    let effective_colormap = app.preview_colormap.unwrap_or(app.active_colormap);
    let (min_val, max_val) = (app.color_range_min, app.color_range_max);

    let default_label = app.default_colorbar_label();
    let mut current_label = app.colorbar_label();

    let style = ctx.style_of(ctx.theme());
    let strong_text_color = style.visuals.strong_text_color();
    let text_color = style.visuals.text_color();
    let border_color = style.visuals.widgets.noninteractive.fg_stroke.color;

    let screen_rect = ctx.input(|i| i.viewport_rect());
    let panel_w = 490.0;
    let panel_h = 88.0;

    let center_x = screen_rect.center().x;
    let bottom_bar_top = screen_rect.max.y - 42.0;
    let panel_min = Pos2::new(center_x - (panel_w / 2.0), bottom_bar_top - panel_h - 8.0);

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

                        let bar_w = 400.0;
                        let total_h = 38.0;

                        let (widget_rect, response) =
                            ui.allocate_exact_size(Vec2::new(bar_w, total_h), egui::Sense::hover());

                        let bar_rect = Rect::from_min_size(widget_rect.min, Vec2::new(bar_w, 13.0));

                        let is_3d = app.active_plot_type == crate::plots::PlotType::Volume
                            || app.active_plot_type == crate::plots::PlotType::PointCloud;
                        let is_categorical_active = !is_3d && app.is_categorical;

                        let unique_vals = if is_categorical_active {
                            app.matrix_data
                                .as_ref()
                                .and_then(|m| m.detect_unique_values())
                        } else {
                            None
                        };

                        if is_categorical_active {
                            draw_categorical_colorbar(
                                app,
                                ui,
                                bar_rect,
                                min_val,
                                max_val,
                                effective_colormap,
                                unique_vals,
                                border_color,
                                strong_text_color,
                            );
                        } else {
                            draw_continuous_colorbar(
                                app,
                                ui,
                                bar_rect,
                                min_val,
                                max_val,
                                effective_colormap,
                                border_color,
                                strong_text_color,
                                text_color,
                            );
                        }

                        draw_clip_triangles(app, ui, bar_rect);
                        draw_end_range_inputs(app, ui, bar_rect, min_val, max_val);

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

#[allow(clippy::too_many_arguments)]
fn draw_categorical_colorbar(
    app: &OctantApp,
    ui: &mut egui::Ui,
    bar_rect: Rect,
    min_val: f32,
    max_val: f32,
    effective_colormap: u32,
    unique_vals: Option<Vec<f32>>,
    border_color: Color32,
    strong_text_color: Color32,
) {
    let cat_vals: Vec<f32> = if let Some(unique) = unique_vals {
        unique
    } else {
        let range = (max_val - min_val).max(1e-30);
        (0..10)
            .map(|i| min_val + (i as f32 + 0.5) / 10.0 * range)
            .collect()
    };

    let num_cats = cat_vals.len();
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
        let color = crate::utils::colormap::sample_colormap_rgb(effective_colormap, norm_scaled);

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

        mesh.indices
            .extend_from_slice(&[idx, idx + 1, idx + 2, idx + 1, idx + 3, idx + 2]);
    }
    ui.painter().add(Shape::mesh(mesh));

    ui.painter().rect_stroke(
        bar_rect,
        0.0,
        egui::Stroke::new(1.0_f32, border_color),
        egui::StrokeKind::Middle,
    );

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
}

#[allow(clippy::too_many_arguments)]
fn draw_continuous_colorbar(
    app: &OctantApp,
    ui: &mut egui::Ui,
    bar_rect: Rect,
    min_val: f32,
    max_val: f32,
    effective_colormap: u32,
    border_color: Color32,
    strong_text_color: Color32,
    text_color: Color32,
) {
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
        let color = crate::utils::colormap::sample_colormap_rgb(effective_colormap, norm_scaled);

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

    ui.painter().rect_stroke(
        bar_rect,
        0.0,
        egui::Stroke::new(1.0_f32, border_color),
        egui::StrokeKind::Middle,
    );

    let ticks = generate_colorbar_ticks(min_val, max_val, app.active_scale_type, app.scale_param);

    for tick in ticks {
        let x = bar_rect.min.x + tick.t_pos * bar_rect.width();

        if tick.is_major {
            ui.painter().line_segment(
                [Pos2::new(x, bar_rect.min.y), Pos2::new(x, bar_rect.max.y)],
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

        app.custom_colorbar_label = Some("Surface Temp (Celsius)".to_string());
        assert_eq!(app.colorbar_label(), "Surface Temp (Celsius)");
        assert_eq!(app.default_colorbar_label(), "Scalar Field");

        app.reset_colorbar_label();
        assert_eq!(app.colorbar_label(), "Scalar Field");
        assert!(app.custom_colorbar_label.is_none());

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

        app.custom_colorbar_label = Some("Custom Temp".to_string());
        assert_eq!(app.colorbar_label(), "Custom Temp");
        assert_eq!(app.default_colorbar_label(), "air_temp (degK)");

        app.reset_colorbar_label();
        assert_eq!(app.colorbar_label(), "air_temp (degK)");
    }

    #[test]
    fn test_colorbar_range_reset() {
        let mut app = OctantApp::default();

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

        app.color_range_min = 20.0;
        app.color_range_max = 50.0;
        app.lock_color_bounds = true;

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

        assert!((major_ticks[0].val - 10.0).abs() < 1e-4);
        assert!((major_ticks[4].val - 50.0).abs() < 1e-4);
        assert!((major_ticks[2].val - 30.0).abs() < 1e-4);
    }
}
