//! Heightfield sampling shared by the surface and sphere hovers, mirroring
//! the smooth-mode vertices of `smooth_corner`
//! (`shaders/common/height.wgsl`): values sit at cell centers, the first and
//! last stretched to the axis edges, unless the axis wraps around.

use crate::data::MatrixData;

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

/// Value of cell `(px, py)`, NaN past the data.
pub fn value_at(matrix: &MatrixData, px: usize, py: usize) -> f32 {
    matrix
        .values
        .get(py * matrix.width + px)
        .copied()
        .unwrap_or(f32::NAN)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
