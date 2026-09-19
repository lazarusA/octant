use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::plots::PlotType;
use crate::ui::hover::callout::draw_leader_callout;
use crate::ui::hover::camera::Camera3D;
use crate::ui::hover::raycast_sphere::{raycast_sphere, sphere_target_pos};
use crate::ui::hover::raycast_surface::{raycast_surface, surface_target_pos};
use crate::ui::hover::raycast_volume::{VolumeSampler, volume_target_pos};
use crate::ui::hover::sample_1d::screen_to_norm_1d;
use crate::ui::hover::sample_2d::Transform2D;
use egui::{Pos2, Rect};

#[allow(clippy::type_complexity)]
pub(crate) fn resolve_hit_coordinates(
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

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_target_screen_pos(
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
pub(crate) fn draw_tooltip_card(
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
    is_rgb: bool,
) {
    let tooltip_w = 210.0;
    let tooltip_est_h = if dim_entries.len() > 2 { 84.0 } else { 68.0 };
    let screen_rect = ctx.input(|i| i.viewport_rect());

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

    tooltip_pos.x = tooltip_pos.x.clamp(
        screen_rect.min.x + 10.0,
        screen_rect.max.x - tooltip_w - 10.0,
    );
    tooltip_pos.y = tooltip_pos.y.clamp(
        screen_rect.min.y + 10.0,
        screen_rect.max.y - tooltip_est_h - 10.0,
    );

    let tooltip_rect = Rect::from_min_size(tooltip_pos, egui::vec2(tooltip_w, tooltip_est_h));

    if let Some(target) = target_pos {
        draw_leader_callout(ui.painter(), ctx, target, tooltip_rect);
    }

    render_tooltip_popup(
        ctx,
        tooltip_pos,
        tooltip_w,
        var_name,
        raw_val,
        units_str,
        dim_entries,
        is_rgb,
    );
}

#[allow(clippy::too_many_arguments)]
fn render_tooltip_popup(
    ctx: &egui::Context,
    tooltip_pos: Pos2,
    tooltip_w: f32,
    var_name: &str,
    raw_val: f32,
    units_str: &str,
    dim_entries: &[String],
    is_rgb: bool,
) {
    let (label_prefix, val_formatted) = if raw_val.is_nan() {
        ("Val:", "NaN".to_string())
    } else if is_rgb {
        let packed = (raw_val.max(0.0) + 0.5) as u32;
        let r = packed & 0xFF;
        let g = (packed >> 8) & 0xFF;
        let b = (packed >> 16) & 0xFF;
        ("RGB:", format!("({}, {}, {})", r, g, b))
    } else if raw_val.abs() >= 1e4 || (raw_val.abs() <= 1e-3 && raw_val != 0.0) {
        ("Val:", format!("{:.4e}", raw_val))
    } else {
        ("Val:", format!("{:.4}", raw_val))
    };

    let style = ctx.style_of(ctx.theme());
    let strong_text = style.visuals.strong_text_color();
    let text_color = style.visuals.text_color();

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
                            ui.label(egui::RichText::new(label_prefix).small().color(text_color));
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
