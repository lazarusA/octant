//! The ticks of a plot's visible part, from its domains and screen rects.

use egui::Rect;

use super::draw::PlotAxisOptions;
use super::steps::{AxisSpec, AxisTicks, axis_ticks};

/// The ticks of the data's visible part: x left to right, y bottom to top.
pub(super) fn visible_ticks(
    plot_rect: Rect,
    visible: Rect,
    options: &PlotAxisOptions<'_>,
) -> (AxisTicks, AxisTicks) {
    let ((x0, x1), (y0, y1)) = visible_values(plot_rect, visible, options);
    let (minor, majors) = (options.style.minors(), options.style.majors());
    (
        axis_ticks(&AxisSpec {
            start: x0,
            end: x1,
            title: options.x_title,
            units: options.x_units,
            px: visible.width(),
            cells: options.x_cells,
            minor,
            majors,
        }),
        axis_ticks(&AxisSpec {
            start: y0,
            end: y1,
            title: options.y_title,
            units: options.y_units,
            px: visible.height(),
            cells: options.y_cells,
            minor,
            majors,
        }),
    )
}

/// The x values at `visible`'s left and right and the y values at its
/// bottom and top.
fn visible_values(
    plot_rect: Rect,
    visible: Rect,
    options: &PlotAxisOptions<'_>,
) -> ((f64, f64), (f64, f64)) {
    let (x_min, x_max) = min_max(options.x_domain);
    let full_x = (x_max - x_min).max(1e-9);
    let pw = plot_rect.width().max(1.0) as f64;
    let x_at = |screen_x: f32| x_min + ((screen_x - plot_rect.left()) as f64 / pw) * full_x;

    let (y_min, y_max) = min_max(options.y_domain);
    let full_y = (y_max - y_min).max(1e-9);
    let ph = plot_rect.height().max(1.0) as f64;
    // The y value at a screen height: growing upward, or downward for `y_down`.
    let y_at = |screen_y: f32| {
        let from_top = (screen_y - plot_rect.top()) as f64 / ph;
        y_min
            + if options.y_down {
                from_top
            } else {
                1.0 - from_top
            } * full_y
    };
    (
        (x_at(visible.left()), x_at(visible.right())),
        (y_at(visible.bottom()), y_at(visible.top())),
    )
}

#[inline]
fn min_max(domain: (f64, f64)) -> (f64, f64) {
    (domain.0.min(domain.1), domain.0.max(domain.1))
}
