//! 3D heightfield surface raycasting and target point projection.

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::ui::hover::camera::Camera3D;
use egui::Pos2;

/// Computes normalized surface height on the 3D surface mesh matching surface.wgsl
#[inline]
pub fn get_normalized_surface_height(app: &OctantApp, val: f32) -> f32 {
    crate::utils::math::compute_normalized_surface_height(
        val,
        app.layers.base.color.range_min,
        app.layers.base.color.range_max,
        app.surface_mode,
        app.surface_displacement_strength,
    )
}

/// Raycasts against the 3D elevation surface heightfield.
/// Returns `(norm_x, norm_y, geo_coords)`.
#[allow(clippy::type_complexity)]
pub fn raycast_surface(
    app: &OctantApp,
    matrix: &MatrixData,
    camera: &Camera3D,
    hover_pos: Pos2,
) -> Option<(f32, f32, Option<(f32, f32)>)> {
    let (_, world_ray) = camera.cast_ray(hover_pos);
    let data_aspect = matrix.grid.data_aspect_ratio(matrix.width, matrix.height);

    if world_ray.dir[1].abs() < 1e-5 {
        return None;
    }

    let t0 = -world_ray.origin[1] / world_ray.dir[1];
    if t0 <= 0.0 {
        return None;
    }

    let hit_x = world_ray.origin[0] + t0 * world_ray.dir[0];
    let hit_z = world_ray.origin[2] + t0 * world_ray.dir[2];

    let mut u = ((hit_x / data_aspect) + 1.0) * 0.5;
    let mut v = (hit_z + 1.0) * 0.5;

    if !(-0.1..=1.1).contains(&u) || !(-0.1..=1.1).contains(&v) {
        return None;
    }

    let (mut px, mut py) = matrix.grid.find_cell_from_surface_uv(
        u.clamp(0.0, 1.0),
        v.clamp(0.0, 1.0),
        matrix.width,
        matrix.height,
    );
    let cell_val = matrix
        .values
        .get(py * matrix.width + px)
        .copied()
        .unwrap_or(f32::NAN);
    let h = get_normalized_surface_height(app, cell_val);
    let target_h = if app.surface_mode == 2 { h.max(0.0) } else { h };

    let t_ref = (target_h - world_ray.origin[1]) / world_ray.dir[1];
    if t_ref > 0.0 {
        let ref_x = world_ray.origin[0] + t_ref * world_ray.dir[0];
        let ref_z = world_ray.origin[2] + t_ref * world_ray.dir[2];
        let u_ref = ((ref_x / data_aspect) + 1.0) * 0.5;
        let v_ref = (ref_z + 1.0) * 0.5;
        if (-0.05..=1.05).contains(&u_ref) && (-0.05..=1.05).contains(&v_ref) {
            u = u_ref;
            v = v_ref;
            let (ref_px, ref_py) = matrix.grid.find_cell_from_surface_uv(
                u.clamp(0.0, 1.0),
                v.clamp(0.0, 1.0),
                matrix.width,
                matrix.height,
            );
            px = ref_px;
            py = ref_py;
        }
    }

    let (nx, ny) = matrix
        .grid
        .cell_center_norm(px, py, matrix.width, matrix.height);
    let (cell_lon_rad, cell_lat_rad) =
        matrix
            .grid
            .cell_center_lon_lat_rad(px, py, matrix.width, matrix.height);
    let geo_coords = if matrix.grid.requires_geo_coords() {
        Some((cell_lat_rad.to_degrees(), cell_lon_rad.to_degrees()))
    } else {
        None
    };

    Some((nx, ny, geo_coords))
}

/// Forward projects the 3D position of the hovered surface cell to screen space.
pub fn surface_target_pos(
    app: &OctantApp,
    matrix: &MatrixData,
    camera: &Camera3D,
    px: usize,
    py: usize,
    raw_val: f32,
) -> Option<Pos2> {
    let data_aspect = matrix.grid.data_aspect_ratio(matrix.width, matrix.height);
    let height = get_normalized_surface_height(app, raw_val);
    let world_y = if app.surface_mode == 2 {
        height.max(0.0) // Lego cube top face
    } else {
        height
    };

    let (world_x, world_z) =
        matrix
            .grid
            .cell_center_surface_xz(px, py, matrix.width, matrix.height, data_aspect);

    camera.project_point([world_x, world_y, world_z])
}
