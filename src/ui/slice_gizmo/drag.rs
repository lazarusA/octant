//! A handle's drag: screen motion turned into moves of the faces it borders.

use super::guide::Guide;
use super::handles::Handle;
use super::space::Space;
use crate::app::Slicer;
use egui::Vec2;

/// A face moving less than this share of the fastest one is frozen: it is
/// seen edge on.
const FREEZE: f32 = 0.15;
/// Below this `det / (trace / 2)²` the faces move nearly alike on screen:
/// only the best aligned one moves.
const PARALLEL: f32 = 0.05;

/// Moves the faces `handle` borders so its point at `t` follows `motion`
/// (screen points), each kept at least `Space::min_gap` from its opposite.
pub fn apply(space: &Space, slice: &mut Slicer, handle: Handle, t: f32, motion: Vec2) {
    let p = handle.point(slice, t);
    // Screen motion per unit fraction of each face the handle moves.
    let mut dirs = [None; 3];
    for (k, dir) in dirs.iter_mut().enumerate().take(space.dims) {
        if handle.sides[k].is_some() {
            *dir = space.screen_dir(p, k);
        }
    }
    // Faces held at the plot's edge or against their opposite pass their
    // share of the motion to the others.
    let mut moves = solve(dirs, motion);
    for _ in 0..2 {
        let mut blocked = false;
        for k in 0..3 {
            let gap = space.min_gap(k);
            if dirs[k].is_some() && stuck(slice, handle.sides[k], k, moves[k], gap) {
                dirs[k] = None;
                blocked = true;
            }
        }
        if !blocked {
            break;
        }
        moves = solve(dirs, motion);
    }
    for (k, d) in moves.iter().enumerate() {
        let gap = space.min_gap(k);
        match handle.sides[k] {
            // Bounds kept ordered: a cut thinner than a cell (fewer cells
            // than when it was made) must not make `clamp` panic.
            Some(false) => slice.lo[k] = (slice.lo[k] + d).clamp(0.0, (slice.hi[k] - gap).max(0.0)),
            Some(true) => slice.hi[k] = (slice.hi[k] + d).clamp((slice.lo[k] + gap).min(1.0), 1.0),
            None => {}
        }
    }
}

/// Moves the faces `handle` borders to the nearest cell edges, at least
/// one cell apart (on release, so cuts line up with cells).
pub fn snap(space: &Space, slice: &mut Slicer, handle: Handle) {
    for k in 0..space.dims {
        let gap = space.min_gap(k);
        match handle.sides[k] {
            Some(false) => {
                slice.lo[k] = space
                    .snap_edge(k, slice.lo[k])
                    .min(slice.hi[k] - gap)
                    .max(0.0);
            }
            Some(true) => {
                slice.hi[k] = space
                    .snap_edge(k, slice.hi[k])
                    .max(slice.lo[k] + gap)
                    .min(1.0);
            }
            None => {}
        }
    }
}

/// Whether moving the face of `axis` on `side` by `d` pushes it past its
/// limit while it already sits there, `gap` being the thinnest cut.
fn stuck(slice: &Slicer, side: Option<bool>, axis: usize, d: f32, gap: f32) -> bool {
    let closed = slice.hi[axis] - slice.lo[axis] <= gap + 1e-6;
    match side {
        Some(false) => (slice.lo[axis] <= 0.0 && d < 0.0) || (closed && d > 0.0),
        Some(true) => (slice.hi[axis] >= 1.0 && d > 0.0) || (closed && d < 0.0),
        None => false,
    }
}

/// The face moves whose screen directions `dirs` best add up to `motion`:
/// the least squares solution of least norm, over the faces not frozen.
pub fn solve(mut dirs: [Option<Vec2>; 3], motion: Vec2) -> [f32; 3] {
    let fastest = dirs
        .iter()
        .flatten()
        .map(|d| d.length())
        .fold(0.0, f32::max);
    for d in &mut dirs {
        if d.is_some_and(|d| d.length() < FREEZE * fastest) {
            *d = None;
        }
    }
    // G = Σ d dᵀ, a 2x2 matrix [[a, b], [b, c]].
    let (mut a, mut b, mut c) = (0.0, 0.0, 0.0);
    for d in dirs.iter().flatten() {
        a += d.x * d.x;
        b += d.x * d.y;
        c += d.y * d.y;
    }
    let det = a * c - b * b;
    let half_trace = 0.5 * (a + c);
    let mut out = [0.0; 3];
    if half_trace <= 0.0 {
        return out;
    }
    if det > PARALLEL * half_trace * half_trace {
        // Δ_k = d_kᵀ G⁻¹ m.
        let y = Vec2::new(c * motion.x - b * motion.y, a * motion.y - b * motion.x) / det;
        for (o, d) in out.iter_mut().zip(dirs) {
            *o = d.map_or(0.0, |d| d.dot(y));
        }
        return out;
    }
    // Nearly parallel on screen: the face moving most along the motion.
    let best = (0..3)
        .filter_map(|k| dirs[k].map(|d| (k, d.dot(motion).abs())))
        .max_by(|x, y| x.1.total_cmp(&y.1));
    if let Some((k, _)) = best
        && let Some(d) = dirs[k]
    {
        out[k] = d.dot(motion) / d.length_sq();
    }
    out
}
