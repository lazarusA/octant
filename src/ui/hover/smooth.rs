//! Heightfield sampling and ray marching shared by the surface and sphere
//! hovers, mirroring the smooth-mode vertices of `smooth_corner`
//! (`shaders/common/height.wgsl`): values sit at cell centers, the first and
//! last stretched to the axis edges, unless the axis wraps around.

/// Most samples a hover ray takes before refining its hit.
const MAX_STEPS: usize = 8192;
/// Bisection steps refining a hit between two samples.
const BISECTIONS: usize = 12;

/// Smooth vertex `j` of an axis of `n` cells, in cell units (cell `k` spans
/// `[k, k + 1]`): its center, or the outer edge for the first and last.
pub fn index_sample(j: usize, n: usize) -> f32 {
    if j == 0 {
        0.0
    } else if j + 1 >= n {
        n as f32
    } else {
        j as f32 + 0.5
    }
}

/// The two smooth vertices around `x` on an axis of `n` cells and the weight
/// of the second: `sample(j)` places vertex `j` (increasing in `j`) and
/// `guess` is the cell holding `x`.
pub fn bracket(
    x: f32,
    guess: usize,
    n: usize,
    sample: impl Fn(usize) -> f32,
) -> (usize, usize, f32) {
    if n < 2 {
        return (0, 0, 0.0);
    }
    let cell = guess.min(n - 1);
    let first = if x < sample(cell) {
        cell.saturating_sub(1)
    } else {
        cell
    }
    .min(n - 2);
    let (a, b) = (sample(first), sample(first + 1));
    let t = if b > a {
        ((x - a) / (b - a)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (first, first + 1, t)
}

/// The two smooth vertices around `x` (in cell units) on an axis of `n`
/// cells that wraps around, and the weight of the second.
pub fn bracket_wrapped(x: f32, n: usize) -> (usize, usize, f32) {
    let n = n.max(1);
    let g = x - 0.5;
    let floor = g.floor();
    let first = (floor as i64).rem_euclid(n as i64) as usize;
    (first, (first + 1) % n, g - floor)
}

/// First ray parameter in `[t0, t1]` where `hit` holds, sampled at most
/// `step` apart and refined by bisection.
pub fn march(t0: f32, t1: f32, step: f32, hit: impl Fn(f32) -> bool) -> Option<f32> {
    if t1 <= t0 {
        return None;
    }
    if hit(t0) {
        return Some(t0);
    }
    let steps = ((t1 - t0) / step.max(1e-6))
        .ceil()
        .clamp(1.0, MAX_STEPS as f32) as usize;
    let dt = (t1 - t0) / steps as f32;
    let mut prev = t0;
    for i in 1..=steps {
        let t = t0 + dt * i as f32;
        if hit(t) {
            let (mut lo, mut hi) = (prev, t);
            for _ in 0..BISECTIONS {
                let mid = 0.5 * (lo + hi);
                if hit(mid) {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            return Some(hi);
        }
        prev = t;
    }
    None
}
