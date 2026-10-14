//! Dynamic plot axis rendering on egui canvas.

use egui::{FontId, Pos2, Rect, Stroke};

use super::segments::{draw_tick_label, draw_x_ticks, draw_y_ticks, is_near_corner};
use super::ticks::generate_constant_count_ticks;

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
    let painter = ui.painter();
    let visuals = ui.visuals();
    let line_color = visuals
        .widgets
        .noninteractive
        .fg_stroke
        .color
        .linear_multiply(0.85);
    let text_color = visuals.text_color();
    let stroke = Stroke::new(1.5, line_color);
    let secondary_stroke = Stroke::new(1.0, line_color.linear_multiply(0.5));
    let font_id = FontId::proportional(11.0);
    let title_font_id = FontId::proportional(12.0);
    let tick_len = 6.0;

    let visible_left = plot_rect
        .left()
        .clamp(canvas_rect.left(), canvas_rect.right());
    let visible_right = plot_rect
        .right()
        .clamp(canvas_rect.left(), canvas_rect.right());
    let visible_top = plot_rect
        .top()
        .clamp(canvas_rect.top(), canvas_rect.bottom());
    let visible_bottom = plot_rect
        .bottom()
        .clamp(canvas_rect.top(), canvas_rect.bottom());

    let vis_w = visible_right - visible_left;
    let vis_h = visible_bottom - visible_top;
    if vis_w <= 1.0 || vis_h <= 1.0 {
        return;
    }

    let (x_min, x_max) = min_max(options.x_domain);
    let full_x = (x_max - x_min).max(1e-9);
    let pw = plot_rect.width().max(1.0) as f64;
    let vis_x_min = x_min + ((visible_left - plot_rect.left()) as f64 / pw) * full_x;
    let vis_x_max = x_min + ((visible_right - plot_rect.left()) as f64 / pw) * full_x;

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

    let x_ticks =
        generate_constant_count_ticks(vis_x_min, vis_x_max, options.x_title, options.x_units);
    // Ticks are laid out bottom to top.
    let y_ticks = generate_constant_count_ticks(
        y_at(visible_bottom),
        y_at(visible_top),
        options.y_title,
        options.y_units,
    );

    // 1. BOTTOM & TOP X-AXES
    let is_bottom_close_to_nav = (canvas_rect.bottom() - plot_rect.bottom()).abs() < 28.0
        || plot_rect.bottom() >= canvas_rect.bottom() - 15.0;
    let is_bottom_inside = !is_bottom_close_to_nav
        && plot_rect.bottom() >= canvas_rect.top()
        && plot_rect.bottom() <= canvas_rect.bottom();
    let (bottom_axis_y, b_tick_dir) = if is_bottom_inside {
        (plot_rect.bottom(), 1.0)
    } else {
        (canvas_rect.bottom() - 1.0, -1.0)
    };

    painter.line_segment(
        [
            Pos2::new(visible_left.round(), bottom_axis_y.round()),
            Pos2::new(visible_right.round(), bottom_axis_y.round()),
        ],
        stroke,
    );

    let is_top_inside = plot_rect.top() >= canvas_rect.top()
        && plot_rect.top() <= canvas_rect.bottom()
        && (plot_rect.top() - canvas_rect.top()).abs() > 20.0;
    let (top_axis_y, t_tick_dir) = if is_top_inside {
        (plot_rect.top(), -1.0)
    } else {
        (canvas_rect.top() + 1.0, 1.0)
    };

    painter.line_segment(
        [
            Pos2::new(visible_left.round(), top_axis_y.round()),
            Pos2::new(visible_right.round(), top_axis_y.round()),
        ],
        secondary_stroke,
    );

    draw_x_ticks(
        painter,
        visuals,
        &x_ticks,
        visible_left,
        vis_w,
        bottom_axis_y,
        top_axis_y,
        b_tick_dir,
        t_tick_dir,
        tick_len,
        canvas_rect,
        stroke,
        secondary_stroke,
        &font_id,
        text_color,
    );

    draw_x_title(
        painter,
        visuals,
        options.x_title,
        visible_left,
        visible_right,
        bottom_axis_y,
        b_tick_dir,
        tick_len,
        canvas_rect,
        &title_font_id,
        text_color,
    );

    // 2. RIGHT & LEFT Y-AXES
    let is_right_inside =
        plot_rect.right() >= canvas_rect.left() && plot_rect.right() <= canvas_rect.right() - 20.0;
    let (right_axis_x, r_tick_dir) = if is_right_inside {
        (plot_rect.right(), 1.0)
    } else {
        (canvas_rect.right() - 1.0, -1.0)
    };

    painter.line_segment(
        [
            Pos2::new(right_axis_x.round(), visible_top.round()),
            Pos2::new(right_axis_x.round(), visible_bottom.round()),
        ],
        stroke,
    );

    let is_left_inside =
        plot_rect.left() >= canvas_rect.left() + 20.0 && plot_rect.left() <= canvas_rect.right();
    let (left_axis_x, l_tick_dir) = if is_left_inside {
        (plot_rect.left(), -1.0)
    } else {
        (canvas_rect.left() + 1.0, 1.0)
    };

    painter.line_segment(
        [
            Pos2::new(left_axis_x.round(), visible_top.round()),
            Pos2::new(left_axis_x.round(), visible_bottom.round()),
        ],
        secondary_stroke,
    );

    draw_y_ticks(
        painter,
        visuals,
        &y_ticks,
        visible_bottom,
        vis_h,
        right_axis_x,
        left_axis_x,
        r_tick_dir,
        l_tick_dir,
        tick_len,
        canvas_rect,
        stroke,
        secondary_stroke,
        &font_id,
        text_color,
    );
}

#[inline]
fn min_max(domain: (f64, f64)) -> (f64, f64) {
    (domain.0.min(domain.1), domain.0.max(domain.1))
}

#[allow(clippy::too_many_arguments)]
fn draw_x_title(
    painter: &egui::Painter,
    visuals: &egui::Visuals,
    title: &str,
    visible_left: f32,
    visible_right: f32,
    bottom_axis_y: f32,
    b_tick_dir: f32,
    tick_len: f32,
    canvas_rect: Rect,
    title_font_id: &FontId,
    text_color: egui::Color32,
) {
    if title.is_empty() {
        return;
    }
    let title_x = ((visible_left + visible_right) * 0.5).round();
    let (title_y, title_align, title_use_pill) = if b_tick_dir > 0.0 {
        (
            bottom_axis_y + tick_len + 20.0,
            egui::Align2::CENTER_TOP,
            false,
        )
    } else {
        (
            bottom_axis_y - tick_len - 28.0,
            egui::Align2::CENTER_BOTTOM,
            true,
        )
    };
    if title_y >= canvas_rect.top() && title_y <= canvas_rect.bottom() {
        let title_pos = Pos2::new(title_x, title_y);
        if !is_near_corner(title_pos, canvas_rect) {
            draw_tick_label(
                visuals,
                painter,
                title_pos,
                title,
                title_font_id,
                text_color,
                title_align,
                title_use_pill,
            );
        }
    }
}
