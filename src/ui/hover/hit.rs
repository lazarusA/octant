//! Hover hit coordinate and 3D raycast target screen position resolution.

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::data::render::decimate::{drawn_index, sample_index};
use crate::plots::PlotType;
use crate::ui::hover::camera::Camera3D;
use crate::ui::hover::raycast_sphere::{raycast_sphere, sphere_target_pos};
use crate::ui::hover::raycast_surface::{raycast_surface, surface_target_pos};
use crate::ui::hover::raycast_volume::{VolumeSampler, volume_target_pos};
use crate::ui::hover::sample_1d::screen_to_norm_1d;
use crate::ui::hover::sample_2d::Transform2D;
use egui::{Pos2, Rect};

/// Result of resolving hover hit coordinates over active 1D, 2D, or 3D plots.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HitResult {
    pub norm_x: f32,
    pub norm_y: f32,
    pub is_valid: bool,
    pub geo_coords: Option<(f32, f32)>,
    pub point_3d: Option<(usize, usize, usize, f32)>,
    /// The full-resolution cell a thinned mesh draws where it was hit, which
    /// the fractions alone can miss by up to a stride (`drawn_cell`).
    pub cell: Option<(usize, usize)>,
}

impl HitResult {
    pub const fn invalid() -> Self {
        Self {
            norm_x: 0.0,
            norm_y: 0.0,
            is_valid: false,
            geo_coords: None,
            point_3d: None,
            cell: None,
        }
    }

    pub const fn norm_2d(
        norm_x: f32,
        norm_y: f32,
        is_valid: bool,
        geo: Option<(f32, f32)>,
    ) -> Self {
        Self {
            norm_x,
            norm_y,
            is_valid,
            geo_coords: geo,
            point_3d: None,
            cell: None,
        }
    }

    pub const fn point_3d(norm_x: f32, norm_y: f32, point: (usize, usize, usize, f32)) -> Self {
        Self {
            norm_x,
            norm_y,
            is_valid: true,
            geo_coords: None,
            point_3d: Some(point),
            cell: None,
        }
    }
}

/// The full-resolution cell the base layer's thinned mesh draws at
/// `(nx, ny)` on its own grid (`sample_index`); `None` when not thinned.
pub(crate) fn drawn_cell(app: &OctantApp, nx: f32, ny: f32) -> Option<(usize, usize)> {
    let base = &app.layers.base;
    let (mesh, full) = (base.data.mesh.as_ref()?, base.data.matrix.as_ref()?);
    let m = &mesh.matrix;
    let (tx, ty) = m.grid.find_cell_from_norm(nx, ny, m.width, m.height);
    let k = mesh.stride;
    Some((
        sample_index(tx, k, full.width),
        sample_index(ty, k, full.height),
    ))
}

fn hit_sphere(
    app: &OctantApp,
    matrix: Option<&MatrixData>,
    camera: &Camera3D,
    hover_pos: Pos2,
) -> HitResult {
    // Marched over the grid the sphere draws (`Layer::mesh_view`).
    if let Some(matrix) = matrix.map(|m| app.layers.base.mesh_view(m).0)
        && let Some((nx, ny, geo)) = raycast_sphere(app, matrix, camera, hover_pos)
    {
        HitResult {
            cell: drawn_cell(app, nx, ny),
            ..HitResult::norm_2d(nx, ny, true, geo)
        }
    } else {
        HitResult::invalid()
    }
}

fn hit_surface(
    app: &OctantApp,
    matrix: Option<&MatrixData>,
    camera: &Camera3D,
    hover_pos: Pos2,
) -> HitResult {
    if let Some(matrix) = matrix.map(|m| app.layers.base.mesh_view(m).0)
        && let Some((nx, ny, geo)) = raycast_surface(app, matrix, camera, hover_pos)
    {
        HitResult {
            cell: drawn_cell(app, nx, ny),
            ..HitResult::norm_2d(nx, ny, true, geo)
        }
    } else {
        HitResult::invalid()
    }
}

fn hit_3d_volume(
    app: &OctantApp,
    camera: &Camera3D,
    sampler: Option<&VolumeSampler>,
    hover_pos: Pos2,
) -> HitResult {
    let Some(sampler) = sampler else {
        return HitResult::invalid();
    };
    let world_ray = camera.cast_ray(hover_pos);
    let aspects = app.get_3d_aspect_ratio();
    if let Some((hit_x, hit_y, hit_z, hit_val)) = sampler.march_ray(app, &world_ray, aspects, true)
    {
        let nx = (hit_x as f32 + 0.5) / sampler.width as f32;
        let ny = (hit_y as f32 + 0.5) / sampler.height as f32;
        HitResult::point_3d(nx, ny, (hit_x, hit_y, hit_z, hit_val))
    } else {
        HitResult::invalid()
    }
}

fn hit_2d_grid(
    app: &OctantApp,
    matrix: Option<&MatrixData>,
    transform_2d: &Transform2D,
    rect: Rect,
    hover_pos: Pos2,
) -> HitResult {
    let Some(matrix) = matrix else {
        return HitResult::invalid();
    };
    let (nx, ny) = transform_2d.screen_to_norm(hover_pos);
    let is_inside = rect.contains(hover_pos);
    let (orig_w, orig_h) = if let Some(pyr) = &app.layers.base.data.pyramid {
        (pyr.original_width, pyr.original_height)
    } else {
        (matrix.width, matrix.height)
    };
    let (px, py) = matrix.grid.find_cell_from_norm(nx, ny, orig_w, orig_h);
    let (cell_lon_rad, cell_lat_rad) = matrix.grid.cell_center_lon_lat_rad(px, py, orig_w, orig_h);
    let geo_coords = if matrix.grid.requires_geo_coords() {
        Some((cell_lat_rad.to_degrees(), cell_lon_rad.to_degrees()))
    } else {
        None
    };
    HitResult::norm_2d(nx, ny, is_inside, geo_coords)
}

pub fn resolve_hit_coordinates(
    app: &OctantApp,
    matrix: Option<&MatrixData>,
    camera: &Camera3D,
    sampler: Option<&VolumeSampler>,
    transform_2d: &Transform2D,
    rect: Rect,
    hover_pos: Pos2,
) -> HitResult {
    match app.effective_canvas_plot_type() {
        PlotType::Sphere => hit_sphere(app, matrix, camera, hover_pos),
        PlotType::Surface => hit_surface(app, matrix, camera, hover_pos),
        PlotType::PointCloud | PlotType::Volume => hit_3d_volume(app, camera, sampler, hover_pos),
        PlotType::Line => {
            let is_inside = rect.contains(hover_pos);
            let (nx, ny) = screen_to_norm_1d(app, rect, hover_pos);
            HitResult::norm_2d(nx, ny, is_inside, None)
        }
        _ => hit_2d_grid(app, matrix, transform_2d, rect, hover_pos),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_target_screen_pos(
    app: &OctantApp,
    matrix: Option<&MatrixData>,
    camera: &Camera3D,
    sampler: Option<&VolumeSampler>,
    transform_2d: &Transform2D,
    px: usize,
    py: usize,
    point_3d_hit: Option<(usize, usize, usize, f32)>,
) -> Option<Pos2> {
    match app.effective_canvas_plot_type() {
        // On the drawn grid's cell holding `(px, py)`.
        PlotType::Sphere => {
            let full = matrix?;
            let (mesh, k) = app.layers.base.mesh_view(full);
            let (x, y) = (
                drawn_index(px, k, full.width),
                drawn_index(py, k, full.height),
            );
            sphere_target_pos(app, mesh, camera, x, y)
        }
        PlotType::Surface => {
            let full = matrix?;
            let (mesh, k) = app.layers.base.mesh_view(full);
            let (x, y) = (
                drawn_index(px, k, full.width),
                drawn_index(py, k, full.height),
            );
            surface_target_pos(app, mesh, camera, x, y)
        }
        PlotType::PointCloud | PlotType::Volume => {
            let sampler = sampler?;
            if let Some((hit_x, hit_y, hit_z, _)) = point_3d_hit {
                let aspects = app.get_3d_aspect_ratio();
                // On the drawn point (a thinned cloud's grid).
                let ((x, y), (w, h)) = sampler.drawn_cell(hit_x, hit_y);
                volume_target_pos(camera, (x, y, hit_z), (w, h, sampler.depth), aspects)
            } else {
                None
            }
        }
        PlotType::Heatmap => {
            let matrix = matrix?;
            let (orig_w, orig_h) = if let Some(pyr) = &app.layers.base.data.pyramid {
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
