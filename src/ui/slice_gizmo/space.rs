//! Where the slicer's fractions (u, v, w; `app::state::slicer`) land on
//! screen for each plot type, by the same cell placement the renderers and
//! the hover use: cell `i` of `n` spans `[i/n, (i+1)/n)`.

use crate::app::{OctantApp, SliceAxis};
use crate::plots::PlotType;
use crate::ui::hover::camera::Camera3D;
use crate::ui::hover::sample_2d::Transform2D;
use egui::{Pos2, Rect, Vec2};

use super::shape::{Kind, THIN, dot, sphere, sub, surface};
/// Step of the numerical screen derivative, as a fraction.
const EPS: f32 = 1e-3;
/// Smallest cut along a continuous axis (heights), as a fraction.
const MIN_CONTINUOUS_GAP: f32 = 0.01;

/// How fractions reach the screen.
#[derive(Clone, Copy)]
enum View<'a> {
    /// The heatmap's 2D view.
    Flat(Transform2D),
    /// A 3D plot through the orbit camera.
    Orbit { camera: Camera3D, kind: Kind<'a> },
}

/// The plot's space for the gizmo, built per frame.
#[derive(Clone, Copy)]
pub struct Space<'a> {
    view: View<'a>,
    /// Axes the slicer cuts: 2 (heatmap, a flat surface or globe) or 3.
    pub dims: usize,
    /// Cells along each axis, 0 for a continuous one (heights).
    cells: [usize; 3],
}

impl<'a> Space<'a> {
    /// The canvas plot's space, `None` when nothing is drawn to slice.
    pub fn of(app: &'a OctantApp, rect: Rect) -> Option<Self> {
        let base = &app.layers.base;
        let plot_type = app.effective_canvas_plot_type();
        // Cells along data dimensions; heights are continuous (0).
        let axes = SliceAxis::of(plot_type);
        let along = |sizes: [usize; 3]| {
            std::array::from_fn(|k| match axes[k] {
                SliceAxis::Dim(_) => sizes[k],
                SliceAxis::Height | SliceAxis::None => 0,
            })
        };
        let (kind, dims, sizes) = match plot_type {
            PlotType::Volume | PlotType::PointCloud => {
                let v = base.data.volume.as_ref()?;
                let (x, y, z) = app.get_3d_aspect_ratio();
                let half = [0.5 * x, 0.5 * y, 0.5 * z];
                (Kind::Box { half }, 3, [v.width, v.height, v.depth])
            }
            PlotType::Surface => surface(app)?,
            PlotType::Sphere => sphere(app)?,
            PlotType::Heatmap => {
                let (w, h) = base.data.shown_dimensions_2d()?;
                let view = View::Flat(Transform2D::from_app(app, rect));
                return Some(Self {
                    view,
                    dims: 2,
                    cells: along([w, h, 0]),
                });
            }
            _ => return None,
        };
        let camera = Camera3D::from_app(app, rect);
        let view = View::Orbit { camera, kind };
        let cells = along(sizes);
        Some(Self { view, dims, cells })
    }

    /// A unit box of 10 cells a side seen by `camera`, for tests.
    #[cfg(test)]
    pub fn test_box(camera: Camera3D) -> Self {
        let kind = Kind::Box { half: [0.5; 3] };
        let view = View::Orbit { camera, kind };
        Self {
            view,
            dims: 3,
            cells: [10; 3],
        }
    }

    /// Whether it is a volume's or point cloud's box (where planes slice).
    pub fn is_box(&self) -> bool {
        matches!(
            self.view,
            View::Orbit {
                kind: Kind::Box { .. },
                ..
            }
        )
    }

    /// Whether its edges curve on screen (the sphere's); others are straight
    /// lines, along which a face's facing never changes.
    pub fn curved(&self) -> bool {
        matches!(
            self.view,
            View::Orbit {
                kind: Kind::Sphere { .. },
                ..
            }
        )
    }

    /// Screen position of `uvw`, `None` behind the camera.
    pub fn screen(&self, uvw: [f32; 3]) -> Option<Pos2> {
        match &self.view {
            View::Flat(t) => Some(t.norm_to_screen(uvw[0], uvw[1])),
            View::Orbit { camera, kind } => camera.project_point(kind.world(uvw)),
        }
    }

    /// Screen motion per unit fraction along `axis` at `uvw`.
    pub fn screen_dir(&self, uvw: [f32; 3], axis: usize) -> Option<Vec2> {
        let mut q = uvw;
        q[axis] += EPS;
        Some((self.screen(q)? - self.screen(uvw)?) / EPS)
    }

    /// Whether the face of `axis` at `uvw` (on its `hi` side or not) faces
    /// the camera; every face does on the heatmap.
    pub fn faces_camera(&self, uvw: [f32; 3], axis: usize, hi: bool) -> bool {
        let View::Orbit { camera, kind } = &self.view else {
            return true;
        };
        let p = kind.world(uvw);
        let mut q = uvw;
        q[axis] += if hi { THIN } else { -THIN };
        let n = sub(kind.world(q), p);
        dot(n, sub(camera.orbit.eye(), p)) > 0.0
    }

    /// The thinnest cut along `axis`: one cell, or a sliver of a continuous
    /// axis.
    pub fn min_gap(&self, axis: usize) -> f32 {
        match self.cells[axis] {
            0 => MIN_CONTINUOUS_GAP,
            n => 1.0 / n as f32,
        }
    }

    /// A face at `f` along `axis` moved to the nearest cell edge.
    pub fn snap_edge(&self, axis: usize, f: f32) -> f32 {
        match self.cells[axis] {
            0 => f,
            n => (f * n as f32).round() / n as f32,
        }
    }

    /// A plane at `f` along `axis` moved to the center of its cell.
    pub fn snap_center(&self, axis: usize, f: f32) -> f32 {
        match self.cells[axis] {
            0 => f,
            n => (((f * n as f32) as usize).min(n - 1) as f32 + 0.5) / n as f32,
        }
    }
}
