//! The 3D plots' shapes the slicer's fractions map onto: the volume's box,
//! the surface's cells and heights, the globe's wedge; placed as the
//! renderers and the hover place cells.

use std::f32::consts::PI;

use crate::app::OctantApp;
use crate::data::MatrixData;
use crate::ui::hover::raycast_sphere::SphereField;
use crate::ui::hover::raycast_surface::SurfaceField;

/// Thickness given to a flat w axis so faces keep a normal.
pub(super) const THIN: f32 = 1e-3;

/// A 3D plot's shape.
#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    /// Volume and point cloud: a box centred on the origin, `half` its
    /// half sizes (`get_3d_aspect_ratio` halves).
    Box { half: [f32; 3] },
    /// Surface: u along x, v along z (`SurfaceField` cell placement), w
    /// along the drawn heights.
    Surface {
        field: SurfaceField<'a>,
        y: [f32; 2],
    },
    /// Sphere: longitude and latitude at fractions 0 and 1 (cell edges),
    /// the radii.
    Sphere {
        lon: [f32; 2],
        lat: [f32; 2],
        r: [f32; 2],
    },
}

impl Kind<'_> {
    pub(super) fn world(&self, [u, v, w]: [f32; 3]) -> [f32; 3] {
        let lerp = |[a, b]: [f32; 2], t: f32| a + (b - a) * t;
        match self {
            Kind::Box { half } => [
                (2.0 * u - 1.0) * half[0],
                (1.0 - 2.0 * v) * half[1],
                (1.0 - 2.0 * w) * half[2],
            ],
            Kind::Surface { field, y } => {
                let su = field.uv_of_fraction(u, true);
                let sv = field.uv_of_fraction(v, false);
                let (x, z) = field.uv_to_world(su, sv);
                [x, lerp(*y, w), z]
            }
            Kind::Sphere { lon, lat, r } => {
                let (lon, lat, r) = (lerp(*lon, u), lerp(*lat, v), lerp(*r, w));
                [
                    r * lat.cos() * lon.sin(),
                    r * lat.sin(),
                    r * lat.cos() * lon.cos(),
                ]
            }
        }
    }
}

/// The surface's shape and sizes: its cells along x and z, the drawn
/// heights.
pub(super) fn surface(app: &OctantApp) -> Option<(Kind<'_>, usize, [usize; 3])> {
    let base = &app.layers.base;
    let full = base.data.matrix.as_ref()?;
    let field = SurfaceField::new(app, base.mesh_view(full).0);
    let (y, dims) = thick(field.height_span());
    let sizes = [full.width, full.height, 0];
    Some((Kind::Surface { field, y }, dims, sizes))
}

/// The globe's shape: longitudes and latitudes at the grid's cell edges
/// (north first, as `SphereField` places them), the drawn radii.
pub(super) fn sphere(app: &OctantApp) -> Option<(Kind<'_>, usize, [usize; 3])> {
    let base = &app.layers.base;
    let full = base.data.matrix.as_ref()?;
    let (r, dims) = thick(SphereField::new(app, base.mesh_view(full).0).radius_span());
    let (lon, lat) = sphere_edges(full);
    let sizes = [full.width, full.height, 0];
    Some((Kind::Sphere { lon, lat, r }, dims, sizes))
}

/// Longitude and latitude at fractions 0 and 1: the globe for global grids;
/// for regional ones (`sphere_coord_mode` 1), whose bounds are the first and
/// last cell centers, half a cell past them.
fn sphere_edges(m: &MatrixData) -> ([f32; 2], [f32; 2]) {
    let grid = &m.grid;
    if grid.sphere_coord_mode() != 1 {
        return ([-PI, PI], [0.5 * PI, -0.5 * PI]);
    }
    let edges = |[a, b]: [f32; 2], n: usize| {
        let step = (b - a) / (n.max(2) - 1) as f32;
        [a - 0.5 * step, b + 0.5 * step]
    };
    let [lat_min, lat_max] = grid.lat_bounds_rad();
    (
        edges(grid.lon_bounds_rad(), m.width),
        edges([lat_max, lat_min], m.height),
    )
}

/// A span given `THIN` thickness when flat, and the axes it leaves.
fn thick((a, b): (f32, f32)) -> ([f32; 2], usize) {
    if b - a < THIN {
        ([a, a + THIN], 2)
    } else {
        ([a, b], 3)
    }
}

pub(super) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(super) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
