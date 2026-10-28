//! Ticks at round values: multiples of a 1/2/5 x 10^n step (degree steps on
//! degree axes), so 0 and ±90 show whenever they are in view.

use super::minor::{CellTicks, MAX_MINOR, fill_minor};
use super::style::MinorPlacement;
use super::ticks::{TickMark, is_time_axis};
use crate::data::coordinates::naming::contains_ascii_case_insensitive;

/// Most ticks an axis can get (the step aims at about six intervals).
pub(super) const MAX_TICKS: usize = 12;

/// Intervals the step aims for over the visible axis.
const TARGET_INTERVALS: f64 = 6.0;

/// Degree steps from 10° up, which divide 90 and 180 (or 360).
const DEGREE_STEPS: [f64; 7] = [10.0, 15.0, 30.0, 45.0, 60.0, 90.0, 180.0];

/// An axis's labelled ticks and its minor ones (positions along the visible
/// axis) in fixed stack arrays.
#[derive(Clone, Copy)]
pub struct AxisTicks {
    marks: [TickMark; MAX_TICKS],
    len: usize,
    /// The labelled ticks' step (0 without ticks).
    pub(super) step: f64,
    minor: [f32; MAX_MINOR],
    minor_len: usize,
}

impl AxisTicks {
    const EMPTY: Self = Self {
        marks: [TickMark::EMPTY; MAX_TICKS],
        len: 0,
        step: 0.0,
        minor: [0.0; MAX_MINOR],
        minor_len: 0,
    };

    pub fn as_slice(&self) -> &[TickMark] {
        &self.marks[..self.len]
    }

    /// The labelled ticks to draw: none while `shown` is off.
    pub(super) fn labelled(&self, shown: bool) -> &[TickMark] {
        if shown { self.as_slice() } else { &[] }
    }

    /// The minor ticks' positions: 0 at the axis's start, 1 at its end.
    pub fn minor(&self) -> &[f32] {
        &self.minor[..self.minor_len]
    }

    /// Adds a minor tick; false once the array is full.
    pub(super) fn push_minor(&mut self, fract: f32) -> bool {
        let Some(slot) = self.minor.get_mut(self.minor_len) else {
            return false;
        };
        *slot = fract;
        self.minor_len += 1;
        true
    }

    pub(super) fn clear_minor(&mut self) {
        self.minor_len = 0;
    }
}

/// What an axis's ticks are generated from.
pub struct AxisSpec<'a> {
    /// The values at the visible axis's start (fract 0) and end (fract 1).
    pub start: f64,
    pub end: f64,
    pub title: &'a str,
    pub units: Option<&'a str>,
    /// The visible axis's length in screen pixels.
    pub px: f32,
    pub cells: CellTicks<'a>,
    /// Where minors go, when any are drawn.
    pub minor: Option<MinorPlacement>,
    /// Whether anything is drawn at the majors, which minors then keep off.
    pub majors: bool,
}

/// An axis's labelled and minor ticks.
pub fn axis_ticks(spec: &AxisSpec<'_>) -> AxisTicks {
    let mut ticks = generate_ticks(spec.start, spec.end, spec.title, spec.units);
    fill_minor(&mut ticks, spec);
    ticks
}

/// The ticks between `start` (the axis's start, fract 0) and `end` (fract 1),
/// in either order.
pub fn generate_ticks(start: f64, end: f64, title: &str, units: Option<&str>) -> AxisTicks {
    let mut ticks = AxisTicks::EMPTY;
    let span = end - start;
    if !start.is_finite() || !end.is_finite() || span == 0.0 {
        return ticks;
    }
    let step = nice_step(span.abs(), is_degree_axis(title, units));
    ticks.step = step;
    let time = is_time_axis(title, units);
    let (lo, hi) = (start.min(end), start.max(end));
    // A small tolerance keeps a tick that sits on an end within float noise.
    let first = (lo / step - 1e-9).ceil() as i64;
    let last = (hi / step + 1e-9).floor() as i64;
    for k in first..=last {
        if ticks.len == MAX_TICKS {
            break;
        }
        let val = k as f64 * step;
        let mut mark = TickMark::on_axis(val, step, span.abs(), title, units, time);
        mark.fract = ((val - start) / span) as f32;
        ticks.marks[ticks.len] = mark;
        ticks.len += 1;
    }
    ticks
}

/// The round step giving about `TARGET_INTERVALS` intervals over `span`.
fn nice_step(span: f64, degrees: bool) -> f64 {
    // 10% slack: -90.125..90.125 still ticks every 30°, not 45°.
    let raw = span / TARGET_INTERVALS * 0.9;
    if degrees
        && raw >= DEGREE_STEPS[0]
        && let Some(&step) = DEGREE_STEPS.iter().find(|&&s| s >= raw)
    {
        return step;
    }
    let decade = 10f64.powf(raw.log10().floor());
    [1.0, 2.0, 5.0, 10.0]
        .into_iter()
        .map(|m| m * decade)
        .find(|&s| s >= raw)
        .unwrap_or(10.0 * decade)
}

/// Whether an axis is in degrees: its units say so, or its title carries `°`
/// (`format_dimension_axis_title` gives lon/lat names one).
pub(super) fn is_degree_axis(title: &str, units: Option<&str>) -> bool {
    units.is_some_and(|u| contains_ascii_case_insensitive(u, "degree")) || title.contains('°')
}
