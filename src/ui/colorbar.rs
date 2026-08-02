use crate::app::OctantApp;
use egui::{epaint::Vertex, Color32, Mesh, Pos2, Rect, Shape, Vec2};

pub fn show_colorbar_overlay(app: &OctantApp, ctx: &egui::Context) {
    if !app.show_colorbar {
        return;
    }

    let effective_colormap = app.preview_colormap.unwrap_or(app.active_colormap);

    // Read actual physical min_val and max_val from MatrixData
    let (min_val, max_val) = if let Some(matrix) = &app.matrix_data {
        (matrix.min_val, matrix.max_val)
    } else {
        (0.0f32, 100.0f32)
    };

    let var_name = if let Some(meta) = &app.active_dataset_metadata {
        meta.variables
            .get(app.selected_variable_idx)
            .map(|v| {
                if let Some(unit) = &v.attributes.get("units") {
                    format!("{} ({})", v.name, unit)
                } else {
                    v.name.clone()
                }
            })
            .unwrap_or_else(|| "Scalar Field".to_string())
    } else {
        "Scalar Field".to_string()
    };

    // Position floating panel centered horizontally fixed right above the bottom toolbar
    let screen_rect = ctx.screen_rect();
    let panel_w = 470.0;
    let panel_h = 62.0;

    let center_x = screen_rect.center().x;
    let bottom_bar_top = screen_rect.max.y - 42.0; // Bottom bar height + margin
    let panel_min = Pos2::new(center_x - (panel_w / 2.0), bottom_bar_top - panel_h - 6.0);

    // Render borderless glassmorphic floating area overlay
    egui::Area::new(egui::Id::new("octant_colorbar_overlay"))
        .order(egui::Order::Foreground)
        .fixed_pos(panel_min)
        .show(ctx, |ui| {
            egui::Frame::window(&ui.style())
                .fill(Color32::from_black_alpha(210))
                .stroke(egui::Stroke::NONE)
                .inner_margin(6.0)
                .show(ui, |ui| {
                    ui.set_width(panel_w - 12.0);

                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new(&var_name)
                                .strong()
                                .color(Color32::from_gray(245)),
                        );

                        ui.add_space(3.0);

                        // Reserve rect for horizontal gradient bar inside panel
                        let bar_w = 410.0;
                        let bar_h = 13.0;

                        let content_rect = ui.available_rect_before_wrap();
                        let bar_min_x = content_rect.center().x - (bar_w / 2.0);
                        let bar_min_y = content_rect.min.y;
                        let bar_rect = Rect::from_min_size(Pos2::new(bar_min_x, bar_min_y), Vec2::new(bar_w, bar_h));

                        // 1. Build Horizontal Multi-stop Gradient Mesh (Left = t=0 min_val, Right = t=1 max_val)
                        let num_segments = 64;
                        let mut mesh = Mesh::default();

                        for i in 0..=num_segments {
                            let t = i as f32 / num_segments as f32; // 0.0 at left (min), 1.0 at right (max)
                            let color = crate::utils::colormap::sample_colormap_rgb(effective_colormap, t);

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

                        // 2. Draw 5 Horizontal Tick Marks & Text Labels (Matching playback font size)
                        let ticks = [
                            (0.00, min_val),
                            (0.25, min_val + (max_val - min_val) * 0.25),
                            (0.50, min_val + (max_val - min_val) * 0.50),
                            (0.75, min_val + (max_val - min_val) * 0.75),
                            (1.00, max_val),
                        ];

                        let tick_y_start = bar_rect.max.y;
                        let tick_y_end = tick_y_start + 4.0;

                        for (t_pos, val) in ticks {
                            let x = bar_rect.min.x + t_pos * bar_rect.width();

                            // Tick line inside panel
                            ui.painter().line_segment(
                                [Pos2::new(x, tick_y_start), Pos2::new(x, tick_y_end)],
                                egui::Stroke::new(1.0f32, Color32::from_gray(190)),
                            );

                            // Alignment (Left for min, Right for max, Center for middle ticks)
                            let align = if t_pos == 0.00 {
                                egui::Align2::LEFT_TOP
                            } else if t_pos == 1.00 {
                                egui::Align2::RIGHT_TOP
                            } else {
                                egui::Align2::CENTER_TOP
                            };

                            let label_text = format_scientific_tick(val);
                            ui.painter().text(
                                Pos2::new(x, tick_y_end + 2.0),
                                align,
                                label_text,
                                egui::FontId::proportional(11.0),
                                Color32::from_gray(230),
                            );
                        }

                        // Interactive Hover Tooltip
                        let response = ui.allocate_rect(bar_rect, egui::Sense::hover());
                        if let Some(hover_pos) = response.hover_pos() {
                            let norm_x = ((hover_pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
                            let hover_val = min_val + norm_x * (max_val - min_val);
                            response.on_hover_text(format!("Val: {}", format_scientific_tick(hover_val)));
                        }
                    });
                });
        });
}

/// Formats tick values using scientific notation for large (>= 10,000) or small (< 0.01) floats.
fn format_scientific_tick(val: f32) -> String {
    let abs_val = val.abs();
    if abs_val >= 10000.0 || (abs_val < 0.01 && abs_val > 0.0) {
        format!("{:.2e}", val)
    } else {
        format!("{:.2}", val)
    }
}
