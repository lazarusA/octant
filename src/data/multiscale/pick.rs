//! Which level of a stack a plot reads: the plot states what it needs per
//! dimension (`ResolutionDemand`), whatever the dimension is.

use std::ops::Range;

use super::types::Multiscale;

/// What a plot needs along one dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimDemand {
    /// Every level-0 index: only levels that keep the dimension whole
    /// (factor 1) qualify. Animated, collapsed and channel dimensions.
    Full,
    /// Any level: as fine as the cell budget allows.
    Any,
    /// At most this many cells over the window (screen pixels): finer levels
    /// draw more cells than the plot shows.
    Pixels(u32),
    /// As few cells as possible: the coarsest level not excluded.
    Coarsest,
}

/// What a plot needs from a stack: one demand per dimension, and at most
/// `max_elements` cells read in all.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionDemand {
    pub dims: Vec<DimDemand>,
    pub max_elements: Option<u64>,
}

impl ResolutionDemand {
    /// Every dimension whole: level 0, whatever its size.
    pub fn finest(rank: usize) -> Self {
        Self {
            dims: vec![DimDemand::Full; rank],
            max_elements: None,
        }
    }
}

/// How `level` of `set` meets `demand` over the level-0 window `window`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fit {
    /// A `Full` dimension is downsampled: never read.
    Excluded,
    /// Readable, but more cells than wanted or allowed.
    TooFine,
    Fits,
}

/// How `level` fits `demand` over `window`; `pixels` false ignores the
/// `Pixels` and `Coarsest` demands (a pinned level keeps only the budget).
fn fit(
    set: &Multiscale,
    level: usize,
    window: &[Range<u64>],
    demand: &ResolutionDemand,
    pixels: bool,
) -> Fit {
    let Some(lvl) = set.level(level) else {
        return Fit::Excluded;
    };
    let mut elements: u64 = 1;
    let mut fits = true;
    for (d, want) in demand.dims.iter().enumerate() {
        let Some(base) = window.get(d) else {
            continue;
        };
        let full = lvl
            .factor
            .get(d)
            .is_some_and(|f| (f - 1.0).abs() < super::types::EPS);
        let cells = match set.to_level(level, d, base.clone()) {
            Some(r) => r.end - r.start,
            None => return Fit::Excluded,
        };
        elements = elements.saturating_mul(cells.max(1));
        match *want {
            DimDemand::Full if !full => return Fit::Excluded,
            DimDemand::Pixels(px) if pixels => fits &= cells <= u64::from(px.max(1)),
            DimDemand::Coarsest if pixels => fits = false,
            DimDemand::Full | DimDemand::Any | DimDemand::Pixels(_) | DimDemand::Coarsest => {}
        }
    }
    if demand.max_elements.is_some_and(|max| elements > max) {
        fits = false;
    }
    if fits { Fit::Fits } else { Fit::TooFine }
}

/// The level of `set` to read over the level-0 `window` (half-open, one
/// range per dimension): the finest level meeting `demand`, else the
/// coarsest one it does not exclude, else level 0.
pub fn fit_level(set: &Multiscale, window: &[Range<u64>], demand: &ResolutionDemand) -> usize {
    let mut coarsest = 0;
    for level in 0..set.len() {
        match fit(set, level, window, demand, true) {
            Fit::Fits => return level,
            Fit::TooFine => coarsest = level,
            Fit::Excluded => {}
        }
    }
    coarsest
}

/// `pinned`, or the finest coarser level within `demand.max_elements` when
/// `pinned` reads more (a pinned level never exceeds the memory budget).
/// Levels downsampling a `Full` dimension are never read: past them, the
/// nearest finer level is.
pub fn pinned_level(
    set: &Multiscale,
    pinned: usize,
    window: &[Range<u64>],
    demand: &ResolutionDemand,
) -> usize {
    let pinned = pinned.min(set.len().saturating_sub(1));
    let mut coarsest = None;
    for level in pinned..set.len() {
        match fit(set, level, window, demand, false) {
            Fit::Fits => return level,
            Fit::TooFine => coarsest = Some(level),
            Fit::Excluded => {}
        }
    }
    // Nothing readable from `pinned` on: the nearest finer readable level.
    coarsest.unwrap_or_else(|| {
        (0..pinned)
            .rev()
            .find(|&l| fit(set, l, window, demand, false) != Fit::Excluded)
            .unwrap_or(0)
    })
}
