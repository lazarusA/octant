//! Dynamic plot axis rendering on egui canvas.

use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, Visuals};

use super::lines::{draw_grid, draw_minor_ticks};
use super::minor::CellTicks;
use super::segments::{draw_x_ticks, draw_y_ticks};
use super::steps::AxisTicks;
use super::style::AxisStyle;
use super::title::draw_x_title;
use super::visible::visible_ticks;

pub struct PlotAxisOptions<'a> {
    pub x_domain: (f64, f64),
    pub y_domain: (f64, f64),
    pub x_title: &'a str,
    pub y_title: &'a str,
    pub x_units: Option<&'a str>,
    pub y_units: Option<&'a str>,
    /// Whether y grows downward (raster rows, row 0 at the top) instead of upward.
    pub y_down: bool,
    /// The data cells along x and y, for cell ticks.
    pub x_cells: CellTicks<'a>,
    pub y_cells: CellTicks<'a>,
    pub style: AxisStyle,
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
    let (ticks, grid) = ((&x_ticks, &y_ticks), (axes.grid, axes.minor_grid));
    draw_grid(axes.painter, visible, ticks, options.style, grid);
    axes.draw_x(&x_ticks, options.x_title, options.style);
    axes.draw_y(&y_ticks, options.style);
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
    /// Minor ticks on the axes opposite the main ones.
    faint: Stroke,
    grid: Stroke,
    minor_grid: Stroke,
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
            faint: Stroke::new(1.0, line.linear_multiply(0.3)),
            grid: Stroke::new(1.0, line.linear_multiply(0.25)),
            minor_grid: Stroke::new(1.0, line.linear_multiply(0.1)),
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

    /// The labelled ticks' mark length: none while major ticks are off, which
    /// keeps their values.
    fn major_len(&self, style: AxisStyle) -> f32 {
        if style.major_ticks {
            self.tick_len
        } else {
            0.0
        }
    }

    /// Minor ticks on an axis's two sides.
    fn draw_minor(&self, ticks: &AxisTicks, x_axis: bool, main: (f32, f32), other: (f32, f32)) {
        let strokes = (self.secondary, self.faint);
        let (p, v, len) = (self.painter, self.visible, self.tick_len / 2.0);
        draw_minor_ticks(p, v, (ticks, x_axis), (main, other), len, strokes);
    }

    /// The bottom and top x axes, their ticks and the x title.
    fn draw_x(&self, ticks: &AxisTicks, title: &str, style: AxisStyle) {
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
        if style.minor_ticks {
            self.draw_minor(ticks, true, (bottom_y, b_dir), (top_y, t_dir));
        }
        draw_x_ticks(
            self.painter,
            self.visuals,
            ticks.as_slice(),
            visible.left(),
            visible.width(),
            bottom_y,
            top_y,
            b_dir,
            t_dir,
            self.major_len(style),
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
    fn draw_y(&self, ticks: &AxisTicks, style: AxisStyle) {
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
        if style.minor_ticks {
            self.draw_minor(ticks, false, (right_x, r_dir), (left_x, l_dir));
        }
        draw_y_ticks(
            self.painter,
            self.visuals,
            ticks.as_slice(),
            visible.bottom(),
            visible.height(),
            right_x,
            left_x,
            r_dir,
            l_dir,
            self.major_len(style),
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
