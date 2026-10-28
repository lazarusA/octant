//! Dynamic plot axis rendering on egui canvas.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Visuals};

use super::segments::{draw_x_ticks, draw_y_ticks};
use super::steps::{AxisTicks, generate_ticks};
use super::ticks::TickMark;
use super::title::draw_x_title;

pub struct PlotAxisOptions<'a> {
    pub x_domain: (f64, f64),
    pub y_domain: (f64, f64),
    pub x_title: &'a str,
    pub y_title: &'a str,
    pub x_units: Option<&'a str>,
    pub y_units: Option<&'a str>,
    /// Whether y grows downward (raster rows, row 0 at the top) instead of upward.
    pub y_down: bool,
}

/// Dynamic plot axis renderer with auto-attaching canvas borders, inward/outward ticks,
/// and theme-aware overlay pills for Top, Bottom, Right, and Left axes.
pub fn draw_plot_axes(
    ui: &mut egui::Ui,
    canvas_rect: Rect,
    plot_rect: Rect,
    options: &PlotAxisOptions<'_>,
) {
    let visible = plot_rect.intersect(canvas_rect);
    if visible.width() <= 1.0 || visible.height() <= 1.0 {
        return;
    }
    let (x_ticks, y_ticks) = visible_ticks(plot_rect, visible, options);
    let axes = Axes::new(ui.painter(), ui.visuals(), canvas_rect, plot_rect, visible);
    axes.draw_x(x_ticks.as_slice(), options.x_title);
    axes.draw_y(y_ticks.as_slice());
}

/// The ticks of the data's visible part: x left to right, y bottom to top.
fn visible_ticks(
    plot_rect: Rect,
    visible: Rect,
    options: &PlotAxisOptions<'_>,
) -> (AxisTicks, AxisTicks) {
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
    let (x_title, y_title) = (options.x_title, options.y_title);
    (
        generate_ticks(
            x_at(visible.left()),
            x_at(visible.right()),
            x_title,
            options.x_units,
        ),
        generate_ticks(
            y_at(visible.bottom()),
            y_at(visible.top()),
            y_title,
            options.y_units,
        ),
    )
}

#[inline]
fn min_max(domain: (f64, f64)) -> (f64, f64) {
    (domain.0.min(domain.1), domain.0.max(domain.1))
}

/// The four axes of a plot on its canvas, with their theme-aware style.
struct Axes<'a> {
    painter: &'a Painter,
    visuals: &'a Visuals,
    canvas: Rect,
    plot: Rect,
    /// The part of the plot inside the canvas.
    visible: Rect,
    stroke: Stroke,
    secondary: Stroke,
    font: FontId,
    title_font: FontId,
    text: Color32,
    tick_len: f32,
}

impl<'a> Axes<'a> {
    fn new(
        painter: &'a Painter,
        visuals: &'a Visuals,
        canvas: Rect,
        plot: Rect,
        visible: Rect,
    ) -> Self {
        let line = visuals
            .widgets
            .noninteractive
            .fg_stroke
            .color
            .linear_multiply(0.85);
        Self {
            painter,
            visuals,
            canvas,
            plot,
            visible,
            stroke: Stroke::new(1.5, line),
            secondary: Stroke::new(1.0, line.linear_multiply(0.5)),
            font: FontId::proportional(11.0),
            title_font: FontId::proportional(12.0),
            text: visuals.text_color(),
            tick_len: 6.0,
        }
    }

    /// Where the bottom and top x axes sit, with their ticks' directions: on
    /// the plot's edges, else on the canvas's.
    fn x_axes(&self) -> ((f32, f32), (f32, f32)) {
        let (canvas, plot) = (self.canvas, self.plot);
        let close_to_nav = (canvas.bottom() - plot.bottom()).abs() < 28.0
            || plot.bottom() >= canvas.bottom() - 15.0;
        let bottom_inside =
            !close_to_nav && plot.bottom() >= canvas.top() && plot.bottom() <= canvas.bottom();
        let bottom = if bottom_inside {
            (plot.bottom(), 1.0)
        } else {
            (canvas.bottom() - 1.0, -1.0)
        };
        let top_inside = plot.top() >= canvas.top()
            && plot.top() <= canvas.bottom()
            && (plot.top() - canvas.top()).abs() > 20.0;
        let top = if top_inside {
            (plot.top(), -1.0)
        } else {
            (canvas.top() + 1.0, 1.0)
        };
        (bottom, top)
    }

    /// The bottom and top x axes, their ticks and the x title.
    fn draw_x(&self, ticks: &[TickMark], title: &str) {
        let (canvas, visible) = (self.canvas, self.visible);
        let ((bottom_y, b_dir), (top_y, t_dir)) = self.x_axes();
        self.line(
            visible.left(),
            bottom_y,
            visible.right(),
            bottom_y,
            self.stroke,
        );
        self.line(
            visible.left(),
            top_y,
            visible.right(),
            top_y,
            self.secondary,
        );
        draw_x_ticks(
            self.painter,
            self.visuals,
            ticks,
            visible.left(),
            visible.width(),
            bottom_y,
            top_y,
            b_dir,
            t_dir,
            self.tick_len,
            canvas,
            self.stroke,
            self.secondary,
            &self.font,
            self.text,
        );
        draw_x_title(
            self.painter,
            self.visuals,
            title,
            visible.left(),
            visible.right(),
            bottom_y,
            b_dir,
            self.tick_len,
            canvas,
            &self.title_font,
            self.text,
        );
    }

    /// The right and left y axes and their ticks.
    fn draw_y(&self, ticks: &[TickMark]) {
        let (canvas, plot, visible) = (self.canvas, self.plot, self.visible);
        let right_inside = plot.right() >= canvas.left() && plot.right() <= canvas.right() - 20.0;
        let (right_x, r_dir) = if right_inside {
            (plot.right(), 1.0)
        } else {
            (canvas.right() - 1.0, -1.0)
        };
        let left_inside = plot.left() >= canvas.left() + 20.0 && plot.left() <= canvas.right();
        let (left_x, l_dir) = if left_inside {
            (plot.left(), -1.0)
        } else {
            (canvas.left() + 1.0, 1.0)
        };
        self.line(
            right_x,
            visible.top(),
            right_x,
            visible.bottom(),
            self.stroke,
        );
        self.line(
            left_x,
            visible.top(),
            left_x,
            visible.bottom(),
            self.secondary,
        );
        draw_y_ticks(
            self.painter,
            self.visuals,
            ticks,
            visible.bottom(),
            visible.height(),
            right_x,
            left_x,
            r_dir,
            l_dir,
            self.tick_len,
            canvas,
            self.stroke,
            self.secondary,
            &self.font,
            self.text,
        );
    }

    /// A pixel-snapped axis line.
    fn line(&self, x0: f32, y0: f32, x1: f32, y1: f32, stroke: Stroke) {
        self.painter.line_segment(
            [
                Pos2::new(x0.round(), y0.round()),
                Pos2::new(x1.round(), y1.round()),
            ],
            stroke,
        );
    }
}
