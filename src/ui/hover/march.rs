//! Ray marching against the surface and globe as drawn: samples along the
//! ray report the cell under them and their side of its geometry, and the
//! first sample pair that crosses drawn geometry is refined by bisection.

/// Where a ray sample lies against the geometry of the cell under it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Above,
    Inside,
    Below,
}

impl Side {
    /// Side of height (or radius) `x` against geometry spanning `[lo, hi]`.
    pub fn of(x: f32, lo: f32, hi: f32) -> Self {
        if x > hi {
            Self::Above
        } else if x < lo {
            Self::Below
        } else {
            Self::Inside
        }
    }
}

/// A ray sample: the cell under it (`None` off the drawn grid) and its side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Probe {
    pub cell: Option<(usize, usize)>,
    pub side: Side,
}

impl Probe {
    pub const OFF_GRID: Self = Self {
        cell: None,
        side: Side::Above,
    };
}

/// How the drawn geometry of neighboring cells joins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// One continuous surface (smooth modes): any change of side crosses it.
    Smooth,
    /// Flat patches with open gaps between them (steps): only a change of
    /// side within one cell crosses a patch.
    Patches,
    /// Solid columns between the base and the value (blocks, voxels):
    /// entering one through any face hits it.
    Columns,
}

/// Bisection steps refining a hit between two samples.
const BISECTIONS: usize = 12;

/// Whether the ray crosses drawn geometry between samples `a` and `b`.
fn crosses(shape: Shape, a: Probe, b: Probe) -> bool {
    let (Some(cell_a), Some(cell_b)) = (a.cell, b.cell) else {
        // Off the grid nothing is drawn; a column's outer wall is.
        return shape == Shape::Columns && b.cell.is_some() && b.side == Side::Inside;
    };
    match shape {
        Shape::Smooth => a.side != b.side,
        Shape::Patches => cell_a == cell_b && a.side != b.side,
        Shape::Columns => b.side == Side::Inside || (cell_a == cell_b && a.side != b.side),
    }
}

/// Narrows `[lo, hi]` around where `f` turns true (`f(hi)` holds).
fn bisect(mut lo: f32, mut hi: f32, f: impl Fn(f32) -> bool) -> (f32, f32) {
    for _ in 0..BISECTIONS {
        let mid = 0.5 * (lo + hi);
        if f(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    (lo, hi)
}

/// The first crossing between samples `a` and `b`, which may lie on either
/// side of a cell boundary between them.
fn crossing(
    shape: Shape,
    a: (f32, Probe),
    b: (f32, Probe),
    probe: &impl Fn(f32) -> Probe,
) -> Option<f32> {
    let refine =
        |from: (f32, Probe), to: f32| bisect(from.0, to, |t| crosses(shape, from.1, probe(t))).1;
    if crosses(shape, a.1, b.1) {
        return Some(refine(a, b.0));
    }
    if shape == Shape::Smooth || a.1.cell == b.1.cell {
        return None;
    }
    let (lo, hi) = bisect(a.0, b.0, |t| probe(t).cell != a.1.cell);
    let entry = (hi, probe(hi));
    if crosses(shape, a.1, probe(lo)) {
        Some(refine(a, lo))
    } else if crosses(shape, a.1, entry.1) {
        Some(refine(a, hi))
    } else if crosses(shape, entry.1, b.1) {
        Some(refine(entry, b.0))
    } else {
        None
    }
}

/// First ray parameter in `[t0, t1]` where the ray meets drawn geometry,
/// sampled `step` apart (at most `max_steps` samples).
pub fn march(
    (t0, t1): (f32, f32),
    step: f32,
    max_steps: usize,
    shape: Shape,
    probe: impl Fn(f32) -> Probe,
) -> Option<f32> {
    if t1 <= t0 {
        return None;
    }
    let mut a = (t0, probe(t0));
    if shape == Shape::Columns && a.1.cell.is_some() && a.1.side == Side::Inside {
        return Some(t0);
    }
    let steps = ((t1 - t0) / step.max(1e-6))
        .ceil()
        .clamp(1.0, max_steps.max(1) as f32) as usize;
    let dt = (t1 - t0) / steps as f32;
    for i in 1..=steps {
        let t = t0 + dt * i as f32;
        let b = (t, probe(t));
        if let Some(hit) = crossing(shape, a, b, &probe) {
            return Some(hit);
        }
        a = b;
    }
    None
}

/// Samples a march may take on a `width` x `height` grid: twice the cells a
/// ray can cross, so half-cell steps are never stretched.
pub fn max_steps(width: usize, height: usize) -> usize {
    4 * (width + height) + 64
}
