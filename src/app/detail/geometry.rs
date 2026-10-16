//! Where a detail tile goes: the plotted window and the visible part of it on
//! level 0's index grid, the cells of a finer level covering it, and where
//! those cells sit on the plotted quad (its `tile_bounds`).

use std::ops::Range;

use crate::data::multiscale::Multiscale;

/// Slack for positions read from floating-point factors and offsets.
const EPS: f64 = 1e-9;

/// One plotted axis (x or y) on level 0's index grid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axis {
    /// The dimension in the stack.
    pub dim: usize,
    /// Level-0 positions of the plotted window's lower and upper edges.
    pub lo: f64,
    pub hi: f64,
    /// The plot shows the axis opposite to storage order.
    pub flipped: bool,
}

impl Axis {
    /// The axis `dim` plotted from `level`'s cells `start..=end`.
    pub fn plotted(
        set: &Multiscale,
        level: usize,
        dim: usize,
        (start, end): (usize, usize),
        flipped: bool,
    ) -> Option<Self> {
        let lvl = set.level(level)?;
        let (f, off) = (*lvl.factor.get(dim)?, *lvl.offset.get(dim)?);
        let (start, end) = (start.min(end) as f64, start.max(end) as f64 + 1.0);
        Some(Self {
            dim,
            lo: off + start * f,
            hi: off + end * f,
            flipped,
        })
    }

    /// The level-0 positions shown between fractions `t0..t1` of the plotted
    /// quad (0 at its left or top edge), clamped to the window.
    pub fn visible(&self, (t0, t1): (f64, f64)) -> (f64, f64) {
        let (t0, t1) = (t0.clamp(0.0, 1.0), t1.clamp(0.0, 1.0));
        let (s0, s1) = if self.flipped {
            (1.0 - t1, 1.0 - t0)
        } else {
            (t0, t1)
        };
        let span = self.hi - self.lo;
        (self.lo + s0 * span, self.lo + s1 * span)
    }

    /// The cells of `level` covering `visible` widened by `margin` of its
    /// span on each side, aligned outward to `chunk` cells when that at
    /// most doubles them, and never past the plotted window (a tile draws
    /// only inside the plotted quad).
    pub fn cells(
        &self,
        set: &Multiscale,
        level: usize,
        visible: (f64, f64),
        margin: f64,
        chunk: u64,
    ) -> Option<Range<u64>> {
        let lvl = set.level(level)?;
        let (f, off, len) = (
            *lvl.factor.get(self.dim)?,
            *lvl.offset.get(self.dim)?,
            *lvl.shape.get(self.dim)?,
        );
        let pad = (visible.1 - visible.0) * margin;
        let at = |x: f64| (x - off) / f;
        let inside_start = (at(self.lo) - EPS).ceil().max(0.0) as u64;
        let inside_end = ((at(self.hi) + EPS).floor().max(0.0) as u64).min(len);
        let raw = (
            (at(visible.0 - pad) + EPS).floor().max(0.0) as u64,
            (at(visible.1 + pad) - EPS).ceil().max(0.0) as u64,
        );
        let chunk = chunk.max(1);
        let inside = |(s, e): (u64, u64)| (s.max(inside_start), e.min(inside_end));
        let raw = inside(raw);
        let aligned = inside((raw.0 / chunk * chunk, raw.1.div_ceil(chunk) * chunk));
        // Chunks far larger than the window (a shard holding a whole level)
        // would read far more than the view: then the window as it is.
        let wide = aligned.1.saturating_sub(aligned.0) > 2 * raw.1.saturating_sub(raw.0).max(1);
        let (start, end) = if wide { raw } else { aligned };
        (end > start).then_some(start..end)
    }

    /// Level-0 positions of `level`'s cells `cells`.
    pub fn cover(&self, set: &Multiscale, level: usize, cells: &Range<u64>) -> Option<(f64, f64)> {
        let lvl = set.level(level)?;
        let (f, off) = (*lvl.factor.get(self.dim)?, *lvl.offset.get(self.dim)?);
        Some((off + cells.start as f64 * f, off + cells.end as f64 * f))
    }

    /// Where level-0 positions `cover` sit on the plotted quad, as fractions
    /// from its left or top edge.
    pub fn quad_fraction(&self, (lo, hi): (f64, f64)) -> (f32, f32) {
        let span = (self.hi - self.lo).max(EPS);
        let (t0, t1) = ((lo - self.lo) / span, (hi - self.lo) / span);
        let (t0, t1) = if self.flipped {
            (1.0 - t1, 1.0 - t0)
        } else {
            (t0, t1)
        };
        (t0 as f32, t1 as f32)
    }
}

/// Whether level-0 positions `cover` hold all of `visible`.
pub fn covers(cover: (f64, f64), visible: (f64, f64)) -> bool {
    cover.0 <= visible.0 + EPS && cover.1 >= visible.1 - EPS
}

/// The screen pixels showing `visible` fractions `t0..t1` of the quad along
/// one axis of `px` canvas pixels: the canvas spans `t0..t1` before clamping
/// to the quad.
pub fn pixels_over(px: u32, (t0, t1): (f64, f64)) -> u32 {
    let raw = (t1 - t0).max(EPS);
    let shown = t1.min(1.0) - t0.max(0.0);
    (f64::from(px) * (shown / raw).clamp(0.0, 1.0))
        .round()
        .max(1.0) as u32
}
