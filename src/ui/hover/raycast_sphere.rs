//! 3D globe perspective raycasting and target point projection.

use std::f32::consts::PI;

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::ui::hover::camera::{Camera3D, Ray3D};
use crate::ui::hover::raycast_surface::height_scale;
use crate::ui::hover::smooth::{bracket, bracket_wrapped, index_sample, march};
use crate::utils::height::HeightScale;
use egui::Pos2;

/// The globe as sphere.wgsl draws it, by longitude and latitude (radians).
pub struct SphereField<'a> {
    matrix: &'a MatrixData,
    scale: HeightScale,
    mode: u32,
    disp: f32,
}

impl<'a> SphereField<'a> {
    pub fn new(app: &OctantApp, matrix: &'a MatrixData) -> Self {
        Self {
            matrix,
            scale: height_scale(app, true),
            mode: app.plot_configs.mesh.sphere_mode,
            disp: app.plot_configs.mesh.sphere_displacement,
        }
    }

    fn cell(&self, lon: f32, lat: f32) -> Option<(usize, usize)> {
        let (w, h) = (self.matrix.width, self.matrix.height);
        self.matrix.grid.find_cell_from_lon_lat_rad(lon, lat, w, h)
    }

    /// Radial displacement of cell `(px, py)` (`get_normalized_radial_dr`).
    fn dr_at(&self, px: usize, py: usize) -> f32 {
        if self.mode == 0 {
            return 0.0;
        }
        let val = self
            .matrix
            .values
            .get(py * self.matrix.width + px)
            .copied()
            .unwrap_or(f32::NAN);
        if !val.is_finite() {
            return 0.0;
        }
        self.scale.normalized(val) * 0.4 * self.disp
    }

    /// Continuous cell coordinates of `(lon, lat)` (cell `k` spans
    /// `[k, k + 1]`) on grids whose smooth mode places vertices through
    /// `get_lon_lat` linearly (global and regional regular), and whether
    /// longitudes wrap around.
    fn cell_coords(&self, lon: f32, lat: f32) -> Option<(f32, f32, bool)> {
        let grid = &self.matrix.grid;
        let (w, h) = (self.matrix.width as f32, self.matrix.height as f32);
        match grid.render_coord_mode() {
            0 => Some(((lon + PI) / (2.0 * PI) * w, (0.5 - lat / PI) * h, true)),
            1 => {
                let [lon_min, lon_max] = grid.lon_bounds_rad();
                let [lat_min, lat_max] = grid.lat_bounds_rad();
                let lon = if lon_max > PI && lon < 0.0 {
                    lon + 2.0 * PI
                } else {
                    lon
                };
                let u = (lon - lon_min) / (lon_max - lon_min).abs().max(1e-6);
                let v = (lat_max - lat) / (lat_max - lat_min).abs().max(1e-6);
                Some((u * (w - 1.0) + 0.5, v * (h - 1.0) + 0.5, false))
            }
            _ => None,
        }
    }

    /// Smooth-mode displacement toward `(lon, lat)`, between the vertices
    /// around it (`smooth_corner`).
    fn smooth_dr(&self, lon: f32, lat: f32) -> Option<f32> {
        let (fx, fy, wrap) = self.cell_coords(lon, lat)?;
        let (w, h) = (self.matrix.width, self.matrix.height);
        let (x0, x1, tx) = if wrap {
            bracket_wrapped(fx, w)
        } else {
            bracket(fx, fx.max(0.0) as usize, w, |j| index_sample(j, w))
        };
        let (y0, y1, ty) = bracket(fy, fy.max(0.0) as usize, h, |j| index_sample(j, h));
        let row = |y| lerp(self.dr_at(x0, y), self.dr_at(x1, y), tx);
        Some(lerp(row(y0), row(y1), ty))
    }

    /// Outer radius of the drawn globe toward `(lon, lat)`, if data is there.
    fn radius(&self, lon: f32, lat: f32) -> Option<f32> {
        let (px, py) = self.cell(lon, lat)?;
        let dr = self.dr_at(px, py);
        Some(
            1.0 + match self.mode {
                1 => self.smooth_dr(lon, lat).unwrap_or(dr),
                3 => dr.max(0.0),
                _ => dr,
            },
        )
    }

    /// Angular size of the smallest cell side, the march's step scale.
    fn cell_angle(&self) -> f32 {
        let grid = &self.matrix.grid;
        let (w, h) = (
            self.matrix.width.max(1) as f32,
            self.matrix.height.max(1) as f32,
        );
        if grid.render_coord_mode() == 1 {
            let [lon_min, lon_max] = grid.lon_bounds_rad();
            let [lat_min, lat_max] = grid.lat_bounds_rad();
            let dx = (lon_max - lon_min).abs() / (w - 1.0).max(1.0);
            let dy = (lat_max - lat_min).abs() / (h - 1.0).max(1.0);
            dx.min(dy).max(1e-5)
        } else {
            (2.0 * PI / w).min(PI / h)
        }
    }

    /// The first `(lon, lat)` where `ray` crosses the drawn globe, or the
    /// unit sphere when it crosses nothing.
    pub fn hit(&self, ray: &Ray3D) -> Option<(f32, f32)> {
        let reach = if self.mode == 0 {
            0.0
        } else {
            0.4 * self.disp.abs() * if self.scale.flip { 2.0 } else { 1.0 }
        };
        let at = |t: f32| {
            let p = [0, 1, 2].map(|i| ray.origin[i] + t * ray.dir[i]);
            let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt().max(1e-6);
            (p[0].atan2(p[2]), (p[1] / r).clamp(-1.0, 1.0).asin(), r)
        };
        // Voxel columns are solid down to the unit sphere, so their walls
        // count as below their top.
        let below = |t: f32| {
            let (lon, lat, r) = at(t);
            self.radius(lon, lat).map(|top| r <= top)
        };
        let marched = sphere_span(ray, 1.0 + reach + 1e-3).and_then(|(t_in, t_out)| {
            let start = below(t_in).unwrap_or(false);
            march(t_in, t_out, 0.5 * self.cell_angle(), |t| {
                below(t).is_some_and(|b| b != start)
            })
        });
        let t = marched.or_else(|| sphere_span(ray, 1.0).map(|(t_in, _)| t_in))?;
        let (lon, lat, _) = at(t);
        Some((lon, lat))
    }
}

/// Where `ray` enters and leaves the sphere of radius `r` about the origin,
/// from its origin onward.
fn sphere_span(ray: &Ray3D, r: f32) -> Option<(f32, f32)> {
    let b = (0..3).map(|i| ray.origin[i] * ray.dir[i]).sum::<f32>();
    let c = (0..3).map(|i| ray.origin[i] * ray.origin[i]).sum::<f32>() - r * r;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let root = disc.sqrt();
    let (t_in, t_out) = (-b - root, -b + root);
    (t_out > 0.0).then_some((t_in.max(0.0), t_out))
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
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
    let (_, world_ray) = camera.cast_ray(hover_pos);
    let (lon_rad, lat_rad) = SphereField::new(app, matrix).hit(&world_ray)?;
    let (w, h) = (matrix.width, matrix.height);
    let (px, py) = matrix
        .grid
        .find_cell_from_lon_lat_rad(lon_rad, lat_rad, w, h)?;
    let (nx, ny) = matrix.grid.cell_center_norm(px, py, w, h);
    let (cell_lon_rad, cell_lat_rad) = matrix.grid.cell_center_lon_lat_rad(px, py, w, h);

    Some((
        nx,
        ny,
        Some((cell_lat_rad.to_degrees(), cell_lon_rad.to_degrees())),
    ))
}

/// Forward projects the hovered cell's center, on the drawn globe, to screen
/// space.
pub fn sphere_target_pos(
    app: &OctantApp,
    matrix: &MatrixData,
    camera: &Camera3D,
    px: usize,
    py: usize,
) -> Option<Pos2> {
    let (lon_c, lat_c) = matrix
        .grid
        .cell_center_lon_lat_rad(px, py, matrix.width, matrix.height);
    let r = SphereField::new(app, matrix)
        .radius(lon_c, lat_c)
        .unwrap_or(1.0);

    let world_x = r * lat_c.cos() * lon_c.sin();
    let world_y = r * lat_c.sin();
    let world_z = r * lat_c.cos() * lon_c.cos();

    camera.project_point([world_x, world_y, world_z])
}
