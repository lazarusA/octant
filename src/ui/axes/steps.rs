//! Ticks at round values: multiples of a 1/2/5 x 10^n step (degree steps on
//! degree axes), so 0 and ±90 show whenever they are in view.

use super::ticks::TickMark;
use crate::data::coordinates::naming::contains_ascii_case_insensitive;

/// Most ticks an axis can get (the step aims at about six intervals).
pub const MAX_TICKS: usize = 12;

/// Intervals the step aims for over the visible axis.
const TARGET_INTERVALS: f64 = 6.0;

/// Degree steps from 10° up, which divide 90 and 180 (or 360).
const DEGREE_STEPS: [f64; 7] = [10.0, 15.0, 30.0, 45.0, 60.0, 90.0, 180.0];

/// An axis's ticks in a fixed stack array, `len` of them used.
#[derive(Clone, Copy)]
pub struct AxisTicks {
    marks: [TickMark; MAX_TICKS],
    len: usize,
}

impl AxisTicks {
    pub fn as_slice(&self) -> &[TickMark] {
        &self.marks[..self.len]
    }
}

/// The ticks between `start` (the axis's start, fract 0) and `end` (fract 1),
/// in either order.
pub fn generate_ticks(start: f64, end: f64, title: &str, units: Option<&str>) -> AxisTicks {
    let mut ticks = AxisTicks {
        marks: [TickMark::EMPTY; MAX_TICKS],
        len: 0,
    };
    let span = end - start;
    if !start.is_finite() || !end.is_finite() || span == 0.0 {
        return ticks;
    }
    let step = nice_step(span.abs(), is_degree_axis(title, units));
    let (lo, hi) = (start.min(end), start.max(end));
    // A small tolerance keeps a tick that sits on an end within float noise.
    let first = (lo / step - 1e-9).ceil() as i64;
    let last = (hi / step + 1e-9).floor() as i64;
    for k in first..=last {
        if ticks.len == MAX_TICKS {
            break;
        }
        let val = k as f64 * step;
        let mut mark = TickMark::new_with_context(val, step, span.abs(), title, units);
        mark.fract = ((val - start) / span) as f32;
        ticks.marks[ticks.len] = mark;
        ticks.len += 1;
    }
    ticks
}

/// The round step giving about `TARGET_INTERVALS` intervals over `span`.
pub fn nice_step(span: f64, degrees: bool) -> f64 {
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
fn is_degree_axis(title: &str, units: Option<&str>) -> bool {
    units.is_some_and(|u| contains_ascii_case_insensitive(u, "degree")) || title.contains('°')
}
