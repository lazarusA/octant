//! 3D globe perspective raycasting and target point projection.

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::ui::hover::camera::Camera3D;
use egui::Pos2;

/// Computes normalized radial displacement on the 3D sphere matching sphere.wgsl
pub fn get_normalized_radial_dr(app: &OctantApp, val: f32) -> f32 {
    if val.is_nan() || !val.is_finite() || app.sphere_mode == 0 {
        return 0.0;
    }
    let cmin = app.color_range_min;
    let cmax = app.color_range_max;
    let range = (cmax - cmin).max(1e-6);
    let disp = app.sphere_displacement_strength;

    if cmin < 0.0 && cmax > 0.0 {
        let max_abs = cmin.abs().max(cmax.abs());
        (val / max_abs).clamp(-1.0, 1.0) * 0.4 * disp
    } else {
        let norm_val = ((val - cmin) / range).clamp(0.0, 1.0);
        norm_val * 0.4 * disp
    }
}

/// Raycasts against the 3D unit sphere/terrain globe.
/// Returns `(norm_x, norm_y, geo_coords)`.
#[allow(clippy::type_complexity)]
pub fn raycast_sphere(
    app: &OctantApp,
    matrix: &MatrixData,
    camera: &Camera3D,
    hover_pos: Pos2,
) -> Option<(f32, f32, Option<(f32, f32)>)> {
    let (view_ray, _) = camera.cast_ray(hover_pos);
    let dx = view_ray.dir[0];
    let dy = view_ray.dir[1];
    let dz = view_ray.dir[2];

    let max_r = if app.sphere_mode > 0 {
        1.0 + 0.4 * app.sphere_displacement_strength
    } else {
        1.0
    };

    let b = camera.cam_dist * dz;
    let c_max = camera.cam_dist * camera.cam_dist - max_r * max_r;
    let discr_max = b * b - c_max;

    if discr_max < 0.0 {
        return None;
    }

    let mut r = 1.0_f32;
    let c = camera.cam_dist * camera.cam_dist - r * r;
    let discr = b * b - c;
    let t = if discr >= 0.0 {
        -b - discr.sqrt()
    } else {
        -b - discr_max.sqrt()
    };

    let pos_rot_x = t * dx;
    let pos_rot_y = t * dy;
    let pos_rot_z = camera.cam_dist + t * dz;

    // Inverse rotate around X by -rx
    let pos_y_rot_x = pos_rot_x;
    let pos_y_rot_y = camera.cx * pos_rot_y + camera.sx * pos_rot_z;
    let pos_y_rot_z = -camera.sx * pos_rot_y + camera.cx * pos_rot_z;

    // Inverse rotate around Y by -ry
    let pos_3d_x = camera.cy * pos_y_rot_x - camera.sy * pos_y_rot_z;
    let pos_3d_y = pos_y_rot_y;
    let pos_3d_z = camera.sy * pos_y_rot_x + camera.cy * pos_y_rot_z;

    let mut lat_rad = (pos_3d_y / r).clamp(-1.0, 1.0).asin();
    let mut lon_rad = pos_3d_x.atan2(pos_3d_z);

    if app.sphere_mode > 0
        && let Some((init_px, init_py)) =
            matrix
                .grid
                .find_cell_from_lon_lat_rad(lon_rad, lat_rad, matrix.width, matrix.height)
    {
        let cell_val = matrix
            .values
            .get(init_py * matrix.width + init_px)
            .copied()
            .unwrap_or(f32::NAN);
        let dr = get_normalized_radial_dr(app, cell_val);
        r = 1.0 + dr;

        let c_ref = camera.cam_dist * camera.cam_dist - r * r;
        let discr_ref = b * b - c_ref;
        if discr_ref >= 0.0 {
            let t_ref = -b - discr_ref.sqrt();
            let pr_x = t_ref * dx;
            let pr_y = t_ref * dy;
            let pr_z = camera.cam_dist + t_ref * dz;

            let py_x = pr_x;
            let py_y = camera.cx * pr_y + camera.sx * pr_z;
            let py_z = -camera.sx * pr_y + camera.cx * pr_z;

            let p3_x = camera.cy * py_x - camera.sy * py_z;
            let p3_y = py_y;
            let p3_z = camera.sy * py_x + camera.cy * py_z;

            lat_rad = (p3_y / r).clamp(-1.0, 1.0).asin();
            lon_rad = p3_x.atan2(p3_z);
        }
    }

    let (px_found, py_found) =
        matrix
            .grid
            .find_cell_from_lon_lat_rad(lon_rad, lat_rad, matrix.width, matrix.height)?;

    let (nx, ny) = matrix
        .grid
        .cell_center_norm(px_found, py_found, matrix.width, matrix.height);
    let (cell_lon_rad, cell_lat_rad) =
        matrix
            .grid
            .cell_center_lon_lat_rad(px_found, py_found, matrix.width, matrix.height);

    Some((
        nx,
        ny,
        Some((cell_lat_rad.to_degrees(), cell_lon_rad.to_degrees())),
    ))
}

/// Forward projects the 3D position of the hovered sphere cell to screen space.
pub fn sphere_target_pos(
    app: &OctantApp,
    matrix: &MatrixData,
    camera: &Camera3D,
    px: usize,
    py: usize,
    raw_val: f32,
) -> Option<Pos2> {
    let (lon_c, lat_c) = matrix
        .grid
        .cell_center_lon_lat_rad(px, py, matrix.width, matrix.height);

    let dr = get_normalized_radial_dr(app, raw_val);
    let r = 1.0 + dr;

    let world_x = r * lat_c.cos() * lon_c.sin();
    let world_y = r * lat_c.sin();
    let world_z = r * lat_c.cos() * lon_c.cos();

    camera.project_point([world_x, world_y, world_z])
}
