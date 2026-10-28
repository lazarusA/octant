//! Unlabelled marks along the axes: minor ticks, and grid lines, which are
//! marks spanning the whole visible plot.

use egui::{Painter, Pos2, Rect, Stroke};

use super::steps::AxisTicks;
use super::style::AxisStyle;

/// Grid lines across `visible` at the majors and minors `style` shows.
pub fn draw_grid(
    painter: &Painter,
    visible: Rect,
    (x, y): (&AxisTicks, &AxisTicks),
    style: AxisStyle,
    (major, minor): (Stroke, Stroke),
) {
    let (top, left) = ((visible.top(), 1.0), (visible.left(), 1.0));
    let (h, w) = (visible.height(), visible.width());
    if style.minor_grid {
        let (xs, ys) = (x.minor().iter().copied(), y.minor().iter().copied());
        marks(painter, xs, visible, true, top, h, minor);
        marks(painter, ys, visible, false, left, w, minor);
    }
    if style.major_grid {
        let (xs, ys) = (x.as_slice().iter(), y.as_slice().iter());
        marks(painter, xs.map(|m| m.fract), visible, true, top, h, major);
        marks(painter, ys.map(|m| m.fract), visible, false, left, w, major);
    }
}

/// Minor ticks along an axis on its `main` and `other` sides (position and
/// direction), half as long as the majors.
pub fn draw_minor_ticks(
    painter: &Painter,
    visible: Rect,
    (ticks, x_axis): (&AxisTicks, bool),
    (main, other): ((f32, f32), (f32, f32)),
    len: f32,
    (stroke, faint): (Stroke, Stroke),
) {
    for (side, s) in [(main, stroke), (other, faint)] {
        let fracts = ticks.minor().iter().copied();
        marks(painter, fracts, visible, x_axis, side, len, s);
    }
}

/// A mark at each of `fracts` along `visible` (x: 0 at its left, 1 at its
/// right; y: 0 at its bottom, 1 at its top), across the axis from `at`
/// toward `dir` over `len`.
fn marks(
    painter: &Painter,
    fracts: impl Iterator<Item = f32>,
    visible: Rect,
    x_axis: bool,
    (at, dir): (f32, f32),
    len: f32,
    stroke: Stroke,
) {
    for f in fracts {
        let points = if x_axis {
            let x = (visible.left() + f * visible.width()).round();
            [Pos2::new(x, at), Pos2::new(x, at + dir * len)]
        } else {
            let y = (visible.bottom() - f * visible.height()).round();
            [Pos2::new(at, y), Pos2::new(at + dir * len, y)]
        };
        painter.line_segment(points, stroke);
    }
}
