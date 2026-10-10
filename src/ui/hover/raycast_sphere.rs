//! 3D globe perspective raycasting and target point projection.

use std::f32::consts::PI;

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::ui::hover::camera::{Camera3D, Ray3D};
use crate::ui::hover::march::{Probe, Shape, Side, march, max_steps};
use crate::ui::hover::raycast_surface::height_scale;
use crate::ui::hover::smooth::{bracket, bracket_wrapped, index_sample, lerp, value_at};
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

    /// Continuous cell coordinates of `(lon, lat)` (cell `k` spans
    /// `[k, k + 1]`) on grids `get_lon_lat` places linearly (global and
    /// regional regular), and whether longitudes wrap around.
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

    /// The cell drawn toward `(lon, lat)`, including the half-cell border
    /// strips regional grids draw past their bounds.
    fn cell(&self, lon: f32, lat: f32) -> Option<(usize, usize)> {
        let (w, h) = (self.matrix.width, self.matrix.height);
        let Some((fx, fy, wrap)) = self.cell_coords(lon, lat) else {
            return self.matrix.grid.find_cell_from_lon_lat_rad(lon, lat, w, h);
        };
        let on_axis = |f: f32, n: usize| (0.0..=n as f32).contains(&f);
        if !(wrap || on_axis(fx, w)) || !on_axis(fy, h) {
            return None;
        }
        let index = |f: f32, n: usize| (f.max(0.0) as usize).min(n.saturating_sub(1));
        Some((index(fx, w), index(fy, h)))
    }

    /// Radial displacement of cell `(px, py)` (`get_normalized_radial_dr`).
    fn dr_at(&self, px: usize, py: usize) -> f32 {
        let val = value_at(self.matrix, px, py);
        if self.mode == 0 || !val.is_finite() {
            return 0.0;
        }
        self.scale.normalized(val) * 0.4 * self.disp
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

    /// Radii the drawn geometry spans toward `(lon, lat)` in cell `(px, py)`:
    /// a voxel's column from the unit sphere, else the surface's one radius.
    fn span(&self, px: usize, py: usize, lon: f32, lat: f32) -> (f32, f32) {
        let dr = self.dr_at(px, py);
        match self.mode {
            1 => {
                let r = 1.0 + self.smooth_dr(lon, lat).unwrap_or(dr);
                (r, r)
            }
            3 => (1.0 + dr.min(0.0), 1.0 + dr.max(0.0)),
            _ => (1.0 + dr, 1.0 + dr),
        }
    }

    fn shape(&self) -> Shape {
        match self.mode {
            1 => Shape::Smooth,
            2 => Shape::Patches,
            _ => Shape::Columns,
        }
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

    fn probe(&self, ray: &Ray3D, t: f32) -> Probe {
        let (lon, lat, r) = point(ray, t);
        self.cell(lon, lat).map_or(Probe::OFF_GRID, |(px, py)| {
            let (lo, hi) = self.span(px, py, lon, lat);
            Probe {
                cell: Some((px, py)),
                side: Side::of(r, lo, hi),
            }
        })
    }

    /// The cell where `ray` first meets the drawn globe, or crosses the unit
    /// sphere when it meets nothing.
    pub fn hit(&self, ray: &Ray3D) -> Option<(usize, usize)> {
        let marched = (self.mode != 0).then(|| {
            // Between the outermost and innermost drawn radii; below the
            // innermost the ray only sees the inside of the globe.
            let (low, high) = self.scale.bounds();
            let r_in = 1.0 + 0.4 * self.disp * low.min(0.0) - 1e-3;
            let r_out = 1.0 + 0.4 * self.disp * high.max(0.0) + 1e-3;
            let (t_in, t_out) = sphere_span(ray, r_out)?;
            let t_stop = sphere_span(ray, r_in).map_or(t_out, |(t, _)| t);
            let (w, h) = (self.matrix.width, self.matrix.height);
            let step = 0.5 * self.cell_angle();
            let probe = |t| self.probe(ray, t);
            march((t_in, t_stop), step, max_steps(w, h), self.shape(), probe)
        });
        let unit = || sphere_span(ray, 1.0).map(|(t_in, _)| t_in);
        let (lon, lat, _) = point(ray, marched.flatten().or_else(unit)?);
        self.cell(lon, lat)
    }
}

/// Longitude, latitude and radius of `ray` at `t`.
fn point(ray: &Ray3D, t: f32) -> (f32, f32, f32) {
    let p = [0, 1, 2].map(|i| ray.origin[i] + t * ray.dir[i]);
    let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt().max(1e-6);
    (p[0].atan2(p[2]), (p[1] / r).clamp(-1.0, 1.0).asin(), r)
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
    let (px, py) = SphereField::new(app, matrix).hit(&world_ray)?;
    let (w, h) = (matrix.width, matrix.height);
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
    let r = SphereField::new(app, matrix).span(px, py, lon_c, lat_c).1;

    let world_x = r * lat_c.cos() * lon_c.sin();
    let world_y = r * lat_c.sin();
    let world_z = r * lat_c.cos() * lon_c.cos();

    camera.project_point([world_x, world_y, world_z])
}
