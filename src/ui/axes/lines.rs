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
        x_marks(painter, x.minor().iter().copied(), visible, top, h, minor);
        y_marks(painter, y.minor().iter().copied(), visible, left, w, minor);
    }
    if style.major_grid {
        let (xs, ys) = (x.as_slice().iter(), y.as_slice().iter());
        x_marks(painter, xs.map(|m| m.fract), visible, top, h, major);
        y_marks(painter, ys.map(|m| m.fract), visible, left, w, major);
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
        if x_axis {
            x_marks(painter, fracts, visible, side, len, s);
        } else {
            y_marks(painter, fracts, visible, side, len, s);
        }
    }
}

/// A vertical mark at each of `fracts` (0 at `visible`'s left, 1 at its
/// right), from `y` toward `dir` over `len`.
fn x_marks(
    painter: &Painter,
    fracts: impl Iterator<Item = f32>,
    visible: Rect,
    (y, dir): (f32, f32),
    len: f32,
    stroke: Stroke,
) {
    for f in fracts {
        let x = (visible.left() + f * visible.width()).round();
        painter.line_segment([Pos2::new(x, y), Pos2::new(x, y + dir * len)], stroke);
    }
}

/// A horizontal mark at each of `fracts` (0 at `visible`'s bottom, 1 at its
/// top), from `x` toward `dir` over `len`.
fn y_marks(
    painter: &Painter,
    fracts: impl Iterator<Item = f32>,
    visible: Rect,
    (x, dir): (f32, f32),
    len: f32,
    stroke: Stroke,
) {
    for f in fracts {
        let y = (visible.bottom() - f * visible.height()).round();
        painter.line_segment([Pos2::new(x, y), Pos2::new(x + dir * len, y)], stroke);
    }
}
