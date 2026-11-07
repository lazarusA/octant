//! The gizmo's strokes in the crop tool's metrics: edge lines (dotted where
//! hidden), the crop tool's edge pill, L-bracket corners; each over a halo
//! when lit.

use super::guide::{Guide, Polyline, SEGMENTS};
use super::handles::Handle;
use super::paint::Colors;
use super::space::Space;
use crate::app::Slicer;
use crate::ui::crop_overlay::handles::{PILL_LEN, PILL_W, draw_edge_pill};
use egui::{Color32, Painter, Pos2, Stroke};

/// The crop tool's metrics: border width, corner arm length and width.
const BORDER: f32 = 1.5;
const ARM_LEN: f32 = 16.0;
const ARM_W: f32 = 3.0;
/// Hidden edges' dots: their spacing along the edge and radius (points).
const DOT_GAP: f32 = 4.0;
const DOT_R: f32 = 0.9;
/// How far a lit guide's halo reaches past it on each side (points).
const HALO: f32 = 1.0;

/// An edge's line, solid where `shown` at its fraction and dotted elsewhere
/// (dots `DOT_GAP` apart along the screen, whatever the segments' lengths),
/// over its halo when `lit`.
pub fn line(p: &Painter, line: &Polyline, shown: impl Fn(f32) -> bool, lit: bool, c: Colors) {
    let (width, dot) = if lit {
        (2.0 * BORDER, 2.0 * DOT_R)
    } else {
        (BORDER, DOT_R)
    };
    if lit {
        trace(p, line, &shown, width + 2.0 * HALO, dot + HALO, |_| c.halo);
    }
    trace(p, line, &shown, width, dot, |on| c.of_guide(on, lit).accent);
}

/// Strokes `line` `width` wide where shown and dots of radius `dot`
/// elsewhere, in `color` of whether shown.
fn trace(
    p: &Painter,
    line: &Polyline,
    shown: &impl Fn(f32) -> bool,
    width: f32,
    dot: f32,
    color: impl Fn(bool) -> Color32,
) {
    // Distance left to the next dot.
    let mut next = 0.0;
    for i in 0..line.segments {
        let Some((a, b)) = line.segment(i) else {
            continue;
        };
        if shown(line.t(i, 0.5)) {
            p.line_segment([a, b], (width, color(true)));
            next = 0.0;
            continue;
        }
        let len = a.distance(b);
        let mut d = next;
        while d < len {
            p.circle_filled(a + (b - a) * (d / len), dot, color(false));
            d += DOT_GAP;
        }
        next = d - len;
    }
}

/// The crop tool's pill at the middle of an edge (`at` its screen point at
/// a fraction along it), turned along the edge, over its halo when `lit`.
pub fn pill(p: &Painter, at: impl Fn(f32) -> Option<Pos2>, c: Colors, lit: bool) {
    let (Some(mid), Some(a), Some(b)) = (at(0.5), at(0.45), at(0.55)) else {
        return;
    };
    let dir = b - a;
    if dir.length_sq() < 1e-6 {
        return;
    }
    if lit {
        let half = dir.normalized() * (0.5 * PILL_LEN + HALO);
        // The pill's 1 point outline, then the halo.
        let width = PILL_W + 2.0 * (1.0 + HALO);
        p.line_segment([mid - half, mid + half], (width, c.halo));
    }
    draw_edge_pill(p, mid, dir.y.atan2(dir.x), c.accent, c.outline);
}

/// The crop tool's L bracket: an arm following each edge meeting at the
/// corner (curved on the sphere), at most half the edge, so brackets stay
/// apart; over its halo when `lit`.
pub fn corner(p: &Painter, space: &Space, slice: &Slicer, h: &Handle, lit: bool, c: Colors) {
    let point = h.point(slice, 0.0);
    let Some(start) = space.screen(point) else {
        return;
    };
    let accent = c.of_guide(h.visible(space, slice, 0.0), lit).accent;
    let width = if lit { ARM_W + 1.0 } else { ARM_W };
    let arms = |stroke: Stroke| {
        for k in 0..space.dims {
            // Along axis k, toward its opposite face.
            let Some(hi) = h.sides[k] else { continue };
            let other = if hi { slice.lo[k] } else { slice.hi[k] };
            arm(p, space, (point, start), (k, other), stroke);
        }
    };
    if lit {
        arms(Stroke::new(width + 2.0 * HALO, c.halo));
    }
    arms(Stroke::new(width, accent));
}

/// One bracket arm from the corner `point` (at `start` on screen) toward
/// `other` along axis `k`, `ARM_LEN` long along the edge, never past its
/// middle.
fn arm(
    p: &Painter,
    space: &Space,
    (point, start): ([f32; 3], Pos2),
    (k, other): (usize, f32),
    stroke: Stroke,
) {
    let steps = if space.curved() { SEGMENTS / 2 } else { 1 };
    let (mut prev, mut left) = (start, ARM_LEN);
    for i in 1..=steps {
        let mut q = point;
        q[k] += (other - point[k]) * 0.5 * i as f32 / steps as f32;
        let Some(next) = space.screen(q) else { break };
        let len = prev.distance(next);
        if len >= left {
            p.line_segment([prev, prev + (next - prev) * (left / len)], stroke);
            return;
        }
        p.line_segment([prev, next], stroke);
        (prev, left) = (next, left - len);
    }
}
