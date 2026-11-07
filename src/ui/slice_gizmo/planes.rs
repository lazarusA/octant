//! The planes mode's guides: each shown plane drawn as its outline across
//! the box, dragged anywhere on it along its own axis.

use super::drag::solve;
use super::guide::Guide;
use super::space::Space;
use crate::app::Slicer;
use egui::Vec2;

/// One of a plane's four outline edges: the plane across axis `plane`, the
/// edge on the box face of axis `side` (its `hi` face or not), running along
/// the remaining axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaneEdge {
    pub plane: usize,
    pub side: usize,
    pub hi: bool,
}

impl Guide for PlaneEdge {
    fn point(&self, slice: &Slicer, t: f32) -> [f32; 3] {
        let mut p = [0.0; 3];
        p[self.plane] = slice.planes[self.plane].unwrap_or(0.5);
        p[self.side] = if self.hi { 1.0 } else { 0.0 };
        p[3 - self.plane - self.side] = t;
        p
    }

    /// The box face it lies on facing the camera.
    fn visible(&self, space: &Space, slice: &Slicer, t: f32) -> bool {
        space.faces_camera(self.point(slice, t), self.side, self.hi)
    }
}

/// The outline edges of every shown plane.
pub fn edges(planes: &[Option<f32>; 3]) -> ([PlaneEdge; 12], usize) {
    let mut out = [PlaneEdge {
        plane: 0,
        side: 1,
        hi: false,
    }; 12];
    let mut n = 0;
    for plane in (0..3).filter(|&k| planes[k].is_some()) {
        for side in (0..3).filter(|&k| k != plane) {
            for hi in [false, true] {
                out[n] = PlaneEdge { plane, side, hi };
                n += 1;
            }
        }
    }
    (out, n)
}

/// Moves the plane of `edge` along its axis so its point at `t` follows
/// `motion` (screen points).
pub fn apply(space: &Space, slice: &mut Slicer, edge: PlaneEdge, t: f32, motion: Vec2) {
    let k = edge.plane;
    let Some(at) = slice.planes[k] else {
        return;
    };
    let mut dirs = [None; 3];
    dirs[k] = space.screen_dir(edge.point(slice, t), k);
    slice.planes[k] = Some((at + solve(dirs, motion)[k]).clamp(0.0, 1.0));
}
