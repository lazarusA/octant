//! What box handles and plane edges share: a point at a fraction `t` along
//! them, whether the face holding it faces the camera, their screen
//! polyline, and one hit test over any of them.

use super::space::Space;
use crate::app::Slicer;
use egui::Pos2;

/// Segments a curved edge (the sphere's) is drawn and hit tested with;
/// straight ones take one.
pub const SEGMENTS: usize = 24;

/// A draggable part of the gizmo.
pub trait Guide: Copy + PartialEq {
    /// Its point at `t` (0..1) along it, in fractions.
    fn point(&self, slice: &Slicer, t: f32) -> [f32; 3];
    /// Whether a face holding its point at `t` faces the camera.
    fn visible(&self, space: &Space, slice: &Slicer, t: f32) -> bool;
    /// Whether it is a corner: a point, not an edge.
    fn is_corner(&self, _dims: usize) -> bool {
        false
    }

    /// Its screen polyline: one point for a corner, one segment for a
    /// straight edge, `SEGMENTS` for a curved one.
    fn polyline(&self, space: &Space, slice: &Slicer) -> Polyline {
        let segments = if self.is_corner(space.dims) {
            0
        } else if space.curved() {
            SEGMENTS
        } else {
            1
        };
        let points = std::array::from_fn(|i| {
            let t = i as f32 / segments.max(1) as f32;
            (i <= segments).then(|| space.screen(self.point(slice, t)))?
        });
        Polyline { points, segments }
    }
}

/// Screen points along a guide, `None` behind the camera.
pub struct Polyline {
    pub points: [Option<Pos2>; SEGMENTS + 1],
    /// Segments in use (0 for a corner).
    pub segments: usize,
}

impl Polyline {
    /// The fraction along the guide at segment `i` plus `s` of it.
    pub fn t(&self, i: usize, s: f32) -> f32 {
        (i as f32 + s) / self.segments.max(1) as f32
    }

    /// Segment `i`'s ends, when both are on screen.
    pub fn segment(&self, i: usize) -> Option<(Pos2, Pos2)> {
        Some((self.points[i]?, self.points[i + 1]?))
    }

    /// Distance from `pos` and the nearest fraction along it.
    fn distance(&self, pos: Pos2) -> Option<(f32, f32)> {
        if self.segments == 0 {
            return self.points[0].map(|p| (p.distance(pos), 0.0));
        }
        let mut best: Option<(f32, f32)> = None;
        for i in 0..self.segments {
            let Some((a, b)) = self.segment(i) else {
                continue;
            };
            let ab = b - a;
            let s = ((pos - a).dot(ab) / ab.length_sq().max(1e-6)).clamp(0.0, 1.0);
            let d = (a + ab * s).distance(pos);
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, self.t(i, s)));
            }
        }
        best
    }
}

/// The guide under `pos` and where along it: within `radius` points of an
/// edge, twice that of a corner (whose bracket arms reach that far);
/// corners before edges, visible guides before hidden ones.
pub fn hit<G: Guide>(
    space: &Space,
    slice: &Slicer,
    guides: &[G],
    pos: Pos2,
    radius: f32,
) -> Option<(G, f32)> {
    let mut best: Option<(f32, G, f32)> = None;
    for g in guides {
        let corner = g.is_corner(space.dims);
        let Some((d, t)) = g.polyline(space, slice).distance(pos) else {
            continue;
        };
        let (reach, bonus) = if corner {
            (2.0 * radius, radius)
        } else {
            (radius, 0.0)
        };
        let hidden = if g.visible(space, slice, t) {
            0.0
        } else {
            0.5 * radius
        };
        let score = d - bonus + hidden;
        if d <= reach && best.is_none_or(|(s, ..)| score < s) {
            best = Some((score, *g, t));
        }
    }
    best.map(|(_, g, t)| (g, t))
}
