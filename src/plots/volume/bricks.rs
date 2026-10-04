//! Empty-space skipping grid: for every 8³ brick of the volume, the min and
//! max of the valid values the shader can read while sampling inside it, and
//! whether it holds missing voxels. Rays cross bricks that cannot contribute
//! to the current mode in one jump.
//!
//! The footprint reaches `REACH` voxels past the brick on every side and wraps
//! around each axis: trilinear filtering reads one voxel further, missing-voxel
//! fills average one more, and wrapping covers ring-buffer seams.

use super::encode::{self, Dims};
use std::ops::Range;

/// Brick edge in voxels; matches `BRICK` in `shaders/volume/skip.wgsl`.
pub const BRICK: usize = 8;
const REACH: usize = 2;

/// Stats of a brick without valid voxels: min above max.
const EMPTY: [f32; 4] = [f32::MAX, f32::MIN, 0.0, 0.0];

/// Brick grid extent for a volume of `dims`.
pub fn brick_dims(dims: Dims) -> Dims {
    Dims {
        w: dims.w.div_ceil(BRICK),
        h: dims.h.div_ceil(BRICK),
        d: dims.d.div_ceil(BRICK),
    }
}

/// Scalar a brick's range is taken over: the value, or a composite's
/// brightness (brightest channel, as its alpha), in [0, 1].
pub fn decode(v: f32, composite: bool) -> Option<f32> {
    if !encode::is_valid(v) {
        return None;
    }
    if !composite {
        return Some(v);
    }
    let packed = v as u32;
    let brightest = (packed & 0xFF)
        .max((packed >> 8) & 0xFF)
        .max((packed >> 16) & 0xFF);
    Some(brightest as f32 / 255.0)
}

/// Voxel indices of brick `b`'s footprint along an axis of `n` voxels.
fn footprint(b: usize, n: usize) -> impl Iterator<Item = usize> {
    let start = b * BRICK + n * REACH - REACH;
    let len = (BRICK + 2 * REACH).min(n);
    (0..len).map(move |i| (start + i) % n)
}

/// `[min, max, has_missing, 0]` of every brick in layers `bz` (x fastest).
pub fn brick_stats(values: &[f32], dims: Dims, composite: bool, bz: Range<usize>) -> Vec<[f32; 4]> {
    let grid = brick_dims(dims);
    let bz = bz.start.min(grid.d)..bz.end.min(grid.d);
    let mut out = Vec::with_capacity(bz.len() * grid.w * grid.h);
    if values.len() < dims.w * dims.h * dims.d {
        return out;
    }
    for bzi in bz {
        for byi in 0..grid.h {
            for bxi in 0..grid.w {
                let mut stats = EMPTY;
                for z in footprint(bzi, dims.d) {
                    for y in footprint(byi, dims.h) {
                        let row = (z * dims.h + y) * dims.w;
                        for x in footprint(bxi, dims.w) {
                            match decode(values[row + x], composite) {
                                Some(v) => {
                                    stats[0] = stats[0].min(v);
                                    stats[1] = stats[1].max(v);
                                }
                                None => stats[2] = 1.0,
                            }
                        }
                    }
                }
                out.push(stats);
            }
        }
    }
    out
}

/// Brick layers whose footprint covers changed planes `z` (wrapping).
pub fn affected_layers(z: Range<usize>, depth: usize) -> Vec<usize> {
    let layers = depth.div_ceil(BRICK);
    if z.is_empty() || layers == 0 {
        return Vec::new();
    }
    (0..layers)
        .filter(|&b| footprint(b, depth).any(|plane| z.contains(&plane)))
        .collect()
}
