//! Index mapping between level 0 and any level of a stack, the same for
//! every dimension (x, y, z, time, channel, ...).

use std::ops::Range;

use super::types::Multiscale;

/// Slack for positions read from floating-point factors and offsets.
const EPS: f64 = 1e-9;

impl Multiscale {
    /// The cells of `level` along `dim` covering the level-0 index range
    /// `base` (half-open), clamped to the level. A non-empty range always
    /// maps to at least one cell. `None` for an unknown level or dimension.
    pub fn to_level(&self, level: usize, dim: usize, base: Range<u64>) -> Option<Range<u64>> {
        let lvl = self.level(level)?;
        let (f, off, len) = (*lvl.factor.get(dim)?, lvl.offset[dim], lvl.shape[dim]);
        let start = floor((base.start as f64 - off) / f).min(len.saturating_sub(1));
        let end = ceil((base.end as f64 - off) / f).min(len);
        Some(non_empty(start, end, base.is_empty(), len))
    }

    /// The level-0 indices covered by the cells `range` of `level` along
    /// `dim` (half-open), clamped to level 0.
    pub fn to_base(&self, level: usize, dim: usize, range: Range<u64>) -> Option<Range<u64>> {
        let lvl = self.level(level)?;
        let (f, off) = (*lvl.factor.get(dim)?, lvl.offset[dim]);
        let len = self.base().shape[dim];
        let start = floor(off + range.start as f64 * f).min(len.saturating_sub(1));
        let end = ceil(off + range.end as f64 * f).min(len);
        Some(non_empty(start, end, range.is_empty(), len))
    }

    /// The cell of `level` holding the center of level-0 cell `index` along
    /// `dim`, clamped to the level.
    pub fn level_index(&self, level: usize, dim: usize, index: u64) -> Option<u64> {
        let lvl = self.level(level)?;
        let (f, off, len) = (*lvl.factor.get(dim)?, lvl.offset[dim], lvl.shape[dim]);
        Some(floor((index as f64 + 0.5 - off) / f).min(len.saturating_sub(1)))
    }

    /// The level-0 cell under the center of `level`'s cell `index` along
    /// `dim`, clamped to level 0.
    pub fn base_index(&self, level: usize, dim: usize, index: u64) -> Option<u64> {
        let lvl = self.level(level)?;
        let (f, off) = (*lvl.factor.get(dim)?, lvl.offset[dim]);
        let len = self.base().shape[dim];
        Some(floor(off + (index as f64 + 0.5) * f).min(len.saturating_sub(1)))
    }
}

/// `x` rounded down, at least 0, within `EPS` of the next integer up.
fn floor(x: f64) -> u64 {
    (x + EPS).floor().max(0.0) as u64
}

/// `x` rounded up, at least 0, within `EPS` of the next integer down.
fn ceil(x: f64) -> u64 {
    (x - EPS).ceil().max(0.0) as u64
}

/// `start..end`, widened to one cell when a non-empty input mapped to none.
fn non_empty(start: u64, end: u64, input_empty: bool, len: u64) -> Range<u64> {
    if input_empty || end > start {
        start..end.max(start)
    } else {
        start..(start + 1).min(len.max(1))
    }
}
