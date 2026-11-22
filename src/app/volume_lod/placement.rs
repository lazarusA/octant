//! Where the plotted window sits on level 0's grid and in the box, and the
//! uniforms placing the window and the levels for the shader.

use super::Lod;
use super::select::Placement;
use crate::app::OctantApp;
use crate::data::multiscale::bricks::BRICK;
use crate::plots::volume::bricks::{BrickLevelGpu, BrickUniforms, MAX_LEVELS};

/// Table cells inside `place`'s window.
pub(super) fn table_window(place: &Placement, grid: [u32; 3]) -> [std::ops::Range<u32>; 3] {
    std::array::from_fn(|i| {
        let b = BRICK as f64;
        let start = (place.lo[i] / b).floor().max(0.0) as u32;
        let end = (place.hi[i] / b).ceil().max(0.0) as u32;
        start.min(grid[i])..end.min(grid[i])
    })
}

impl OctantApp {
    /// Where the plotted window of dimensions `dims` (x, y, z) sits in the
    /// box, and which of them are drawn opposite to storage.
    pub(super) fn lod_placement(&self, dims: [usize; 3]) -> Option<(Placement, [bool; 3])> {
        let axes = [
            self.plotted_axis(dims[0])?,
            self.plotted_axis(dims[1])?,
            self.plotted_axis(dims[2])?,
        ];
        let flip = axes.map(|a| a.flipped);
        let (ax, ay, az) = self.get_3d_aspect_ratio();
        let place = Placement {
            lo: axes.map(|a| a.lo),
            hi: axes.map(|a| a.hi),
            // The unit box runs y up and z toward the viewer.
            mirror: [flip[0], !flip[1], !flip[2]],
            aspect: [ax, ay, az],
        };
        Some((place, flip))
    }
}

impl Lod {
    /// The uniforms placing the window and the levels for the shader.
    pub(super) fn uniforms(&self) -> BrickUniforms {
        let bricks = self.bricks();
        let mut levels = [BrickLevelGpu::default(); MAX_LEVELS];
        for (l, slot) in levels.iter_mut().enumerate().take(self.set.len()) {
            let Some(level) = self.set.level(l) else {
                continue;
            };
            let at = |v: &[f64], d: usize| v.get(d).copied().unwrap_or(0.0) as f32;
            let counts = bricks.counts(l);
            *slot = BrickLevelGpu {
                factor: [
                    at(&level.factor, self.dims[0]),
                    at(&level.factor, self.dims[1]),
                    at(&level.factor, self.dims[2]),
                    0.0,
                ],
                offset: [
                    at(&level.offset, self.dims[0]),
                    at(&level.offset, self.dims[1]),
                    at(&level.offset, self.dims[2]),
                    0.0,
                ],
                count: [counts[0], counts[1], counts[2], 0],
            };
        }
        let grid = bricks.grid();
        let p = &self.place;
        BrickUniforms {
            lo: [p.lo[0] as f32, p.lo[1] as f32, p.lo[2] as f32, 0.0],
            span: std::array::from_fn(|i| {
                if i < 3 {
                    (p.hi[i] - p.lo[i]) as f32
                } else {
                    0.0
                }
            }),
            flip: [
                u32::from(self.flip[0]),
                u32::from(self.flip[1]),
                u32::from(self.flip[2]),
                0,
            ],
            grid: [grid[0], grid[1], grid[2], 0],
            value: [self.offset, self.scale, 0.0, 0.0],
            slots: [self.slots[0], self.slots[1], self.slots[2], 0],
            levels,
        }
    }

    /// Voxels across the window at `level`: the ray's step when it is the
    /// finest drawn.
    pub(super) fn level_dims(&self, level: usize) -> [u32; 3] {
        let Some(level) = self.set.level(level) else {
            return [1; 3];
        };
        std::array::from_fn(|i| {
            let f = level
                .factor
                .get(self.dims[i])
                .copied()
                .unwrap_or(1.0)
                .max(1e-9);
            ((self.place.hi[i] - self.place.lo[i]) / f).ceil().max(1.0) as u32
        })
    }
}

/// `(offset, scale)` of atlas values over the range `lo..hi`: a unit scale
/// for an empty or single-value range.
pub(super) fn value_encoding(lo: f32, hi: f32) -> (f32, f32) {
    if !(lo.is_finite() && hi.is_finite()) {
        return (0.0, 1.0);
    }
    let span = hi - lo;
    let tiny = span <= 1e-6 * lo.abs().max(1.0);
    (lo, if tiny { lo.abs().max(1.0) } else { span })
}

/// The offset that makes 8-bit integer data of `data_type` start at 0 in a
/// byte atlas (`uint8` 0, `int8` -128), `None` for other types.
pub(super) fn byte_offset(data_type: &str) -> Option<f32> {
    let t = data_type.trim().trim_start_matches(['<', '>', '|']);
    match t.to_ascii_lowercase().as_str() {
        "uint8" | "u8" | "u1" => Some(0.0),
        "int8" | "i8" | "i1" => Some(-128.0),
        _ => None,
    }
}

/// Whether every value is a whole number in `offset..=offset + 255`: data a
/// byte atlas holds exactly. `values` is the plotted level; a finer level
/// of the same store holds values of the same kind.
// shortcut: a fill value masked only at finer levels shows as `offset` in a
// byte atlas; read the store's calibration here if such a store turns up.
pub(super) fn bytes_hold(values: &[f32], offset: f32) -> bool {
    all_values(values, |v| {
        v.fract() == 0.0 && v >= offset && v <= offset + 255.0
    })
}

/// Widest span of whole values f16 holds one apart over `(v - lo) / span`
/// (11 significant bits).
pub(super) const F16_WHOLE_SPAN: f32 = 2048.0;

/// Whether `data_type` is an integer type wider than a byte.
fn wide_integer_type(data_type: &str) -> bool {
    let t = data_type.trim().trim_start_matches(['<', '>', '|']);
    let t = t.to_ascii_lowercase();
    let bits = t
        .strip_prefix("uint")
        .or_else(|| t.strip_prefix("int"))
        .or_else(|| t.strip_prefix('u'))
        .or_else(|| t.strip_prefix('i'));
    // Bits ("uint16", "u16") or numpy's bytes ("u2"); "u8" is a byte here
    // (`byte_offset`).
    matches!(bits, Some("16" | "32" | "64" | "2" | "4"))
}

/// Whether `values` (missing ones aside) are whole numbers of an integer
/// `data_type` spanning more than f16 holds apart (`F16_WHOLE_SPAN`): ids
/// of a segmentation, which an f16 atlas would merge.
pub(super) fn wide_integers(data_type: &str, values: &[f32], (lo, hi): (f32, f32)) -> bool {
    wide_integer_type(data_type)
        && hi - lo > F16_WHOLE_SPAN
        && all_values(values, |v| !v.is_finite() || v.fract() == 0.0)
}

/// Whether `holds` holds for every value (in parallel: a plotted level can
/// hold hundreds of millions).
fn all_values(values: &[f32], holds: impl Fn(f32) -> bool + Sync) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use rayon::prelude::*;
        values
            .par_chunks(1 << 16)
            .all(|c| c.iter().all(|&v| holds(v)))
    }
    #[cfg(target_arch = "wasm32")]
    values.iter().all(|&v| holds(v))
}
