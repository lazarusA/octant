//! Hover hit coordinate and 3D raycast target screen position resolution.

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::plots::PlotType;
use crate::ui::hover::camera::Camera3D;
use crate::ui::hover::raycast_sphere::{raycast_sphere, sphere_target_pos};
use crate::ui::hover::raycast_surface::{raycast_surface, surface_target_pos};
use crate::ui::hover::raycast_volume::{VolumeSampler, volume_target_pos};
use crate::ui::hover::sample_1d::screen_to_norm_1d;
use crate::ui::hover::sample_2d::Transform2D;
use egui::{Pos2, Rect};

#[allow(clippy::type_complexity)]
pub fn resolve_hit_coordinates(
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
    let canvas_plot_type = app.effective_canvas_plot_type();
    match canvas_plot_type {
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
pub fn resolve_target_screen_pos(
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
    match app.effective_canvas_plot_type() {
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
        PlotType::Heatmap => {
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
