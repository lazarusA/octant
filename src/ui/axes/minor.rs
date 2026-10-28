//! Minor ticks between the labelled ones: round subdivisions of the major
//! step, or the centers of the data cells, where the hover snaps.

use super::steps::{AxisSpec, AxisTicks, is_degree_axis};
use super::style::MinorPlacement;
use super::ticks::is_time_axis;

/// Most minor ticks an axis holds: a 4K-wide axis at `MIN_GAP_PX` per cell.
pub const MAX_MINOR: usize = 1024;

/// Closest two minor ticks may sit, in screen pixels.
const MIN_GAP_PX: f64 = 6.0;

/// A cell tick this close to a drawn major one (pixels) is left to the major.
const MAJOR_CLEARANCE_PX: f64 = 1.0;

/// `count` evenly spaced centers `first + i * step`, `step > 0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lattice {
    first: f64,
    step: f64,
    count: usize,
}

impl Lattice {
    /// `count` cells laid edge to edge over `a..b` (a heatmap's regular grid).
    pub fn cells(a: f64, b: f64, count: usize) -> Option<Self> {
        let (lo, hi) = (a.min(b), a.max(b));
        let step = (hi - lo) / count as f64;
        (count > 0 && step > 0.0 && step.is_finite()).then_some(Self {
            first: lo + step / 2.0,
            step,
            count,
        })
    }

    /// `count` samples from `a` to `b`, both included (a line plot's points).
    pub fn samples(a: f64, b: f64, count: usize) -> Option<Self> {
        let (lo, hi) = (a.min(b), a.max(b));
        let step = (hi - lo) / count.saturating_sub(1) as f64;
        (count > 1 && step > 0.0 && step.is_finite()).then_some(Self {
            first: lo,
            step,
            count,
        })
    }

    fn center(&self, i: usize) -> f64 {
        self.first + i as f64 * self.step
    }

    /// The indices whose centers lie within `lo..=hi`.
    fn within(&self, lo: f64, hi: f64) -> std::ops::Range<usize> {
        let to = ((hi - self.first) / self.step + 1e-9).floor() + 1.0;
        let to = to.clamp(0.0, self.count as f64);
        let from = ((lo - self.first) / self.step - 1e-9).ceil().clamp(0.0, to);
        from as usize..to as usize
    }

    /// The span its cells cover, edge to edge.
    fn extent(&self) -> (f64, f64) {
        let half = self.step / 2.0;
        (self.first - half, self.center(self.count - 1) + half)
    }
}

/// The data cells along an axis, for cell ticks.
#[derive(Clone, Copy, Debug)]
pub enum CellTicks<'a> {
    /// No cells along the axis (data values, curvilinear and HEALPix grids).
    None,
    /// Evenly spaced cells: the plotted level's, and a detail tile's finer
    /// ones, which replace them over the tile's extent.
    Even {
        plotted: Lattice,
        detail: Option<Lattice>,
    },
    /// Cell centers at their coordinate values, ascending or descending.
    Values(&'a [f32]),
}

/// The visible axis minor ticks are placed on.
struct Axis {
    start: f64,
    span: f64,
    lo: f64,
    hi: f64,
    px_per_unit: f64,
    majors: bool,
}

impl Axis {
    fn fract(&self, v: f64) -> f32 {
        ((v - self.start) / self.span) as f32
    }

    /// Adds a cell tick at `v` unless a drawn major sits on it; false once full.
    fn push_cell(&self, ticks: &mut AxisTicks, v: f64) -> bool {
        let on_major = self.majors
            && ticks
                .as_slice()
                .iter()
                .any(|m| (m.val - v).abs() * self.px_per_unit < MAJOR_CLEARANCE_PX);
        on_major || ticks.push_minor(self.fract(v))
    }
}

/// Fills `ticks`'s minor ticks for `spec`.
pub(super) fn fill_minor(ticks: &mut AxisTicks, spec: &AxisSpec<'_>) {
    ticks.clear_minor();
    let span = spec.end - spec.start;
    let Some(placement) = spec.minor else {
        return;
    };
    if !span.is_finite() || span == 0.0 || spec.px <= 0.0 {
        return;
    }
    let axis = Axis {
        start: spec.start,
        span,
        lo: spec.start.min(spec.end),
        hi: spec.start.max(spec.end),
        px_per_unit: f64::from(spec.px) / span.abs(),
        majors: spec.majors,
    };
    if placement == MinorPlacement::Cells && fill_cells(ticks, spec.cells, &axis) {
        return;
    }
    ticks.clear_minor();
    if !is_time_axis(spec.title, spec.units) {
        fill_round(ticks, is_degree_axis(spec.title, spec.units), &axis);
    }
}

/// A tick at every visible cell center; false when cells are too narrow,
/// too many or unknown.
fn fill_cells(ticks: &mut AxisTicks, cells: CellTicks<'_>, axis: &Axis) -> bool {
    match cells {
        CellTicks::None => false,
        CellTicks::Even { plotted, detail } => {
            let finest = detail.map_or(plotted.step, |d| d.step.min(plotted.step));
            if finest * axis.px_per_unit < MIN_GAP_PX {
                return false;
            }
            let fine_ok = detail.is_none_or(|d| push_lattice(ticks, axis, &d, None));
            fine_ok && push_lattice(ticks, axis, &plotted, detail.map(|d| d.extent()))
        }
        CellTicks::Values(values) => fill_values(ticks, axis, values),
    }
}

/// `lattice`'s visible centers, those inside `skip` left out.
fn push_lattice(
    ticks: &mut AxisTicks,
    axis: &Axis,
    lattice: &Lattice,
    skip: Option<(f64, f64)>,
) -> bool {
    lattice
        .within(axis.lo, axis.hi)
        .map(|i| lattice.center(i))
        .filter(|c| !skip.is_some_and(|(a, b)| (a..=b).contains(c)))
        .all(|c| axis.push_cell(ticks, c))
}

/// An irregular axis's visible coordinate values, while no two sit closer
/// than `MIN_GAP_PX`.
fn fill_values(ticks: &mut AxisTicks, axis: &Axis, values: &[f32]) -> bool {
    let (Some(&a), Some(&b)) = (values.first(), values.last()) else {
        return false;
    };
    let (lo, hi) = (axis.lo, axis.hi);
    let visible = if a <= b {
        values.partition_point(|&c| f64::from(c) < lo)
            ..values.partition_point(|&c| f64::from(c) <= hi)
    } else {
        values.partition_point(|&c| f64::from(c) > hi)
            ..values.partition_point(|&c| f64::from(c) >= lo)
    };
    let Some(shown) = values.get(visible) else {
        return false;
    };
    let tight = shown
        .windows(2)
        .any(|w| (f64::from(w[1]) - f64::from(w[0])).abs() * axis.px_per_unit < MIN_GAP_PX);
    shown.len() <= MAX_MINOR && !tight && shown.iter().all(|&c| axis.push_cell(ticks, f64::from(c)))
}

/// Round subdivisions of the major step, where they stay `MIN_GAP_PX` apart.
fn fill_round(ticks: &mut AxisTicks, degrees: bool, axis: &Axis) {
    let n = subdivisions(ticks.step, degrees);
    let minor = ticks.step / f64::from(n);
    if minor.is_nan() || minor * axis.px_per_unit < MIN_GAP_PX {
        return;
    }
    let first = (axis.lo / minor - 1e-9).ceil() as i64;
    let last = (axis.hi / minor + 1e-9).floor() as i64;
    for k in first..=last {
        // Exact on integers: a minor never lands on a major through float noise.
        if axis.majors && k.rem_euclid(i64::from(n)) == 0 {
            continue;
        }
        if !ticks.push_minor(axis.fract(k as f64 * minor)) {
            return;
        }
    }
}

/// Parts a major step splits into: whole degrees on degree steps, else by
/// the step's 1, 2 or 5 leading digit.
pub(super) fn subdivisions(step: f64, degrees: bool) -> u32 {
    const DEGREES: [(f64, u32); 7] = [
        (10.0, 5),
        (15.0, 3),
        (30.0, 3),
        (45.0, 3),
        (60.0, 2),
        (90.0, 3),
        (180.0, 6),
    ];
    if degrees && let Some(&(_, n)) = DEGREES.iter().find(|(s, _)| (s - step).abs() < 1e-9) {
        return n;
    }
    let leading = step / 10f64.powf(step.log10().floor());
    if (leading - 2.0).abs() < 1e-6 { 4 } else { 5 }
}
