//! 3D heightfield surface raycasting and target point projection.

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::ui::hover::camera::Camera3D;
use egui::Pos2;

/// Computes normalized surface height on the 3D surface mesh matching surface.wgsl
pub fn get_normalized_surface_height(app: &OctantApp, val: f32) -> f32 {
    if val.is_nan() || !val.is_finite() {
        return 0.0;
    }
    let cmin = app.color_range_min;
    let cmax = app.color_range_max;
    let range = (cmax - cmin).max(1e-6);
    let disp = app.surface_displacement_strength;

    let mult = match app.surface_mode {
        1 => 0.6, // Flat Steps
        _ => 0.8, // Smooth Terrain (0) and 3D Lego Cubes (2)
    };

    if cmin < 0.0 && cmax > 0.0 {
        let max_abs = cmin.abs().max(cmax.abs());
        (val / max_abs).clamp(-1.0, 1.0) * mult * disp
    } else {
        let norm_val = ((val - cmin) / range).clamp(0.0, 1.0);
        norm_val * mult * disp
    }
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
    let data_aspect = match &matrix.grid {
        crate::data::CoordinateGrid::Healpix { .. } => 2.0,
        _ => (matrix.width as f32 / matrix.height.max(1) as f32).max(0.1),
    };

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

    let (mut px, mut py) = if let crate::data::CoordinateGrid::Healpix { .. } = &matrix.grid {
        let lon_rad = u.clamp(0.0, 1.0) * 2.0 * std::f32::consts::PI;
        let lat_rad = (0.5 - v.clamp(0.0, 1.0)) * std::f32::consts::PI;
        matrix
            .grid
            .find_cell_from_lon_lat_rad(lon_rad, lat_rad, matrix.width, matrix.height)
            .unwrap_or((0, 0))
    } else {
        matrix.grid.find_cell_from_norm(
            u.clamp(0.0, 1.0),
            v.clamp(0.0, 1.0),
            matrix.width,
            matrix.height,
        )
    };
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
            let (ref_px, ref_py) = if let crate::data::CoordinateGrid::Healpix { .. } = &matrix.grid
            {
                let lon_rad = u.clamp(0.0, 1.0) * 2.0 * std::f32::consts::PI;
                let lat_rad = (0.5 - v.clamp(0.0, 1.0)) * std::f32::consts::PI;
                matrix
                    .grid
                    .find_cell_from_lon_lat_rad(lon_rad, lat_rad, matrix.width, matrix.height)
                    .unwrap_or((0, 0))
            } else {
                matrix.grid.find_cell_from_norm(
                    u.clamp(0.0, 1.0),
                    v.clamp(0.0, 1.0),
                    matrix.width,
                    matrix.height,
                )
            };
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
    let data_aspect = match &matrix.grid {
        crate::data::CoordinateGrid::Healpix { .. } => 2.0,
        _ => (matrix.width as f32 / matrix.height.max(1) as f32).max(0.1),
    };
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
