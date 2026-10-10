//! 3D heightfield surface raycasting and target point projection.

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::ui::hover::camera::{Camera3D, Ray3D, intersect_aabb};
use crate::ui::hover::march::{Probe, Shape, Side, march, max_steps};
use crate::ui::hover::smooth::{bracket, lerp, value_at};
use crate::utils::height::{HeightScale, surface_height};
use egui::Pos2;

/// How the base layer's values map to heights on the sphere or the surface.
pub fn height_scale(app: &OctantApp, sphere: bool) -> HeightScale {
    let base = &app.layers.base;
    HeightScale {
        cmin: base.color.range_min,
        cmax: base.color.range_max,
        composite: base.composite.enabled,
        flip: app.plot_configs.mesh.heights(sphere).2,
    }
}

/// The surface as surface.wgsl draws it, in normalized surface coordinates
/// `(u, v)` (`get_cell_normalized_bounds_*`; irregular grids reach past
/// `[0, 1]` by half a cell).
pub struct SurfaceField<'a> {
    matrix: &'a MatrixData,
    scale: HeightScale,
    mode: u32,
    disp: f32,
    /// Half the world width, as surface.wgsl computes it.
    aspect: f32,
}

impl<'a> SurfaceField<'a> {
    pub fn new(app: &OctantApp, matrix: &'a MatrixData) -> Self {
        let (w, h) = (matrix.width.max(1), matrix.height.max(1));
        let aspect = if matrix.grid.is_healpix() {
            2.0
        } else {
            (w as f32 / h as f32).max(0.1)
        };
        Self {
            matrix,
            scale: height_scale(app, false),
            mode: app.plot_configs.mesh.surface_mode,
            disp: app.plot_configs.mesh.surface_displacement,
            aspect,
        }
    }

    fn height_at(&self, px: usize, py: usize) -> f32 {
        let val = value_at(self.matrix, px, py);
        surface_height(val, &self.scale, self.mode, self.disp)
    }

    fn center(&self, k: usize, along_x: bool) -> f32 {
        let (w, h) = (self.matrix.width, self.matrix.height);
        if along_x {
            self.matrix.grid.cell_center_norm(k, 0, w, h).0
        } else {
            self.matrix.grid.cell_center_norm(0, k, w, h).1
        }
    }

    /// Smooth vertex `j` along x or y: its cell's center, or the outer edge
    /// for the first and last (`smooth_corner`).
    fn sample(&self, j: usize, along_x: bool) -> f32 {
        let n = if along_x {
            self.matrix.width
        } else {
            self.matrix.height
        };
        if n < 2 {
            return if j == 0 { 0.0 } else { 1.0 };
        }
        let (a, b) = if j == 0 {
            (self.center(0, along_x), self.center(1, along_x))
        } else if j + 1 >= n {
            (self.center(n - 1, along_x), self.center(n - 2, along_x))
        } else {
            return self.center(j, along_x);
        };
        a - 0.5 * (b - a)
    }

    /// Outer edges of the drawn surface along x or y.
    fn edges(&self, along_x: bool) -> [f32; 2] {
        if self.matrix.grid.is_healpix() {
            return [0.0, 1.0];
        }
        let n = if along_x {
            self.matrix.width
        } else {
            self.matrix.height
        };
        let first = self.sample(0, along_x);
        let last = if n < 2 {
            1.0
        } else {
            self.sample(n - 1, along_x)
        };
        [first.min(last), first.max(last)]
    }

    fn cell(&self, u: f32, v: f32) -> (usize, usize) {
        self.matrix.grid.find_cell_from_surface_uv(
            u.clamp(0.0, 1.0),
            v.clamp(0.0, 1.0),
            self.matrix.width,
            self.matrix.height,
        )
    }

    /// Heights the drawn geometry spans at `(u, v)` in cell `(px, py)`: a
    /// block's column from the zero plane, else the surface's single height.
    fn span(&self, px: usize, py: usize, u: f32, v: f32) -> (f32, f32) {
        let h = self.height_at(px, py);
        match self.mode {
            0 if !self.matrix.grid.is_healpix() => {
                let (x0, x1, tx) = bracket(u, px, self.matrix.width, |j| self.sample(j, true));
                let (y0, y1, ty) = bracket(v, py, self.matrix.height, |j| self.sample(j, false));
                let row = |y| lerp(self.height_at(x0, y), self.height_at(x1, y), tx);
                let s = lerp(row(y0), row(y1), ty);
                (s, s)
            }
            2 => (h.min(0.0), h.max(0.0)),
            _ => (h, h),
        }
    }

    fn shape(&self) -> Shape {
        match self.mode {
            0 => Shape::Smooth,
            1 => Shape::Patches,
            _ => Shape::Columns,
        }
    }

    pub fn uv_to_world(&self, u: f32, v: f32) -> (f32, f32) {
        ((2.0 * u - 1.0) * self.aspect, 2.0 * v - 1.0)
    }

    fn point(&self, ray: &Ray3D, t: f32) -> (f32, f32, f32) {
        let x = ray.origin[0] + t * ray.dir[0];
        let z = ray.origin[2] + t * ray.dir[2];
        let y = ray.origin[1] + t * ray.dir[1];
        ((x / self.aspect + 1.0) * 0.5, (z + 1.0) * 0.5, y)
    }

    fn probe(&self, ray: &Ray3D, t: f32, [u0, u1]: [f32; 2], [v0, v1]: [f32; 2]) -> Probe {
        let (u, v, y) = self.point(ray, t);
        if !(u0..=u1).contains(&u) || !(v0..=v1).contains(&v) {
            return Probe::OFF_GRID;
        }
        let (px, py) = self.cell(u, v);
        let (lo, hi) = self.span(px, py, u, v);
        Probe {
            cell: Some((px, py)),
            side: Side::of(y, lo, hi),
        }
    }

    /// The cell where `ray` first meets the drawn surface, or crosses the
    /// zero plane when it meets nothing.
    pub fn hit(&self, ray: &Ray3D) -> Option<(usize, usize)> {
        let margin = 1e-3;
        let ([u0, u1], [v0, v1]) = (self.edges(true), self.edges(false));
        let (us, vs) = ([u0 - margin, u1 + margin], [v0 - margin, v1 + margin]);
        let (low, high) = self.scale.bounds();
        let k = if self.mode == 1 { 0.6 } else { 0.8 } * self.disp;
        let (y0, y1) = (k * low.min(0.0) - margin, k * high.max(0.0) + margin);
        let (x0, z0) = self.uv_to_world(us[0], vs[0]);
        let (x1, z1) = self.uv_to_world(us[1], vs[1]);
        let (w, h) = (self.matrix.width.max(1), self.matrix.height.max(1));
        let marched = intersect_aabb(ray, [x0, y0, z0], [x1, y1, z1]).and_then(|(t_in, t_out)| {
            let cell = ((x1 - x0) / w as f32).min((z1 - z0) / h as f32);
            let step = 0.5 * cell / ray.dir[0].hypot(ray.dir[2]).max(1e-3);
            let probe = |t| self.probe(ray, t, us, vs);
            march(
                (t_in.max(0.0), t_out),
                step,
                max_steps(w, h),
                self.shape(),
                probe,
            )
        });
        let plane = || {
            (ray.dir[1].abs() > 1e-5)
                .then(|| -ray.origin[1] / ray.dir[1])
                .filter(|t| *t > 0.0)
        };
        let (u, v, _) = self.point(ray, marched.or_else(plane)?);
        ((us[0]..=us[1]).contains(&u) && (vs[0]..=vs[1]).contains(&v)).then(|| self.cell(u, v))
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
    let (px, py) = SurfaceField::new(app, matrix).hit(&world_ray)?;

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

/// Forward projects the hovered surface cell's center, on the drawn surface,
/// to screen space.
pub fn surface_target_pos(
    app: &OctantApp,
    matrix: &MatrixData,
    camera: &Camera3D,
    px: usize,
    py: usize,
) -> Option<Pos2> {
    let field = SurfaceField::new(app, matrix);
    let (w, h) = (matrix.width, matrix.height);
    let (u, v) = matrix.grid.cell_center_norm(px, py, w, h);
    let (world_x, world_z) = matrix
        .grid
        .cell_center_surface_xz(px, py, w, h, field.aspect);
    camera.project_point([world_x, field.span(px, py, u, v).1, world_z])
}
