use egui::{Color32, FontId, Pos2, Rect, Stroke};

pub struct PlotAxisOptions<'a> {
    pub x_domain: (f64, f64),
    pub y_domain: (f64, f64),
    pub x_title: &'a str,
    pub y_title: &'a str,
}

/// Dynamic plot axis renderer with auto-attaching canvas borders and inward/outward ticks.
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
    let font_id = FontId::proportional(11.0);
    let title_font_id = FontId::proportional(12.0);
    let tick_len = 6.0;

    // Visible intersection of plot within canvas
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

    if visible_right <= visible_left || visible_bottom <= visible_top {
        return; // Plot is completely off-screen
    }

    // --- 1. Bottom X-Axis ---
    let is_bottom_inside =
        plot_rect.bottom() >= canvas_rect.top() && plot_rect.bottom() <= canvas_rect.bottom();

    let (x_axis_y, x_tick_dir) = if is_bottom_inside {
        (plot_rect.bottom(), 1.0) // Outward (pointing down)
    } else if plot_rect.bottom() > canvas_rect.bottom() {
        (canvas_rect.bottom() - 1.0, -1.0) // Attached to canvas bottom border, inward (pointing up)
    } else {
        (canvas_rect.top() + 1.0, 1.0) // Attached to canvas top border, inward (pointing down)
    };

    // Draw horizontal axis line
    painter.line_segment(
        [
            Pos2::new(visible_left, x_axis_y),
            Pos2::new(visible_right, x_axis_y),
        ],
        stroke,
    );

    // Generate X ticks
    let x_ticks = generate_nice_ticks(options.x_domain.0, options.x_domain.1, 7);
    let x_range = (options.x_domain.1 - options.x_domain.0).max(1e-9);

    for tick in &x_ticks {
        let t = (tick.val - options.x_domain.0) / x_range;
        let tick_x = plot_rect.left() + t as f32 * plot_rect.width();

        if tick_x >= visible_left && tick_x <= visible_right {
            let start = Pos2::new(tick_x, x_axis_y);
            let end = Pos2::new(tick_x, x_axis_y + x_tick_dir * tick_len);
            painter.line_segment([start, end], stroke);

            let label_pos = if x_tick_dir > 0.0 {
                Pos2::new(tick_x, x_axis_y + tick_len + 4.0)
            } else {
                Pos2::new(tick_x, x_axis_y - tick_len - 14.0)
            };

            draw_tick_label(
                painter,
                label_pos,
                &tick.label,
                font_id.clone(),
                text_color,
                x_tick_dir < 0.0, // use bg pill when inward
            );
        }
    }

    // Draw X-Axis Title
    if !options.x_title.is_empty() {
        let title_x = (visible_left + visible_right) * 0.5;
        let title_y = if x_tick_dir > 0.0 {
            x_axis_y + tick_len + 22.0
        } else {
            x_axis_y - tick_len - 32.0
        };

        if title_y >= canvas_rect.top() && title_y <= canvas_rect.bottom() {
            painter.text(
                Pos2::new(title_x, title_y),
                egui::Align2::CENTER_CENTER,
                options.x_title,
                title_font_id.clone(),
                text_color,
            );
        }
    }

    // --- 2. Left Y-Axis ---
    let is_left_inside =
        plot_rect.left() >= canvas_rect.left() && plot_rect.left() <= canvas_rect.right();

    let (y_axis_x, y_tick_dir) = if is_left_inside {
        (plot_rect.left(), -1.0) // Outward (pointing left)
    } else if plot_rect.left() < canvas_rect.left() {
        (canvas_rect.left() + 1.0, 1.0) // Attached to canvas left border, inward (pointing right)
    } else {
        (canvas_rect.right() - 1.0, -1.0) // Attached to canvas right border, inward (pointing left)
    };

    // Draw vertical axis line
    painter.line_segment(
        [
            Pos2::new(y_axis_x, visible_top),
            Pos2::new(y_axis_x, visible_bottom),
        ],
        stroke,
    );

    // Generate Y ticks
    let y_ticks = generate_nice_ticks(options.y_domain.0, options.y_domain.1, 7);
    let y_range = (options.y_domain.1 - options.y_domain.0).max(1e-9);

    for tick in &y_ticks {
        let t = (tick.val - options.y_domain.0) / y_range;
        // Screen Y decreases upwards
        let tick_y = plot_rect.bottom() - t as f32 * plot_rect.height();

        if tick_y >= visible_top && tick_y <= visible_bottom {
            let start = Pos2::new(y_axis_x, tick_y);
            let end = Pos2::new(y_axis_x + y_tick_dir * tick_len, tick_y);
            painter.line_segment([start, end], stroke);

            let label_pos = if y_tick_dir < 0.0 {
                Pos2::new(y_axis_x - tick_len - 4.0, tick_y)
            } else {
                Pos2::new(y_axis_x + tick_len + 4.0, tick_y)
            };

            let align = if y_tick_dir < 0.0 {
                egui::Align2::RIGHT_CENTER
            } else {
                egui::Align2::LEFT_CENTER
            };

            draw_tick_label_aligned(
                painter,
                label_pos,
                &tick.label,
                font_id.clone(),
                text_color,
                align,
                y_tick_dir > 0.0, // use bg pill when inward
            );
        }
    }

    // Draw Y-Axis Title
    if !options.y_title.is_empty() {
        let title_y = (visible_top + visible_bottom) * 0.5;
        let title_x = if y_tick_dir < 0.0 {
            y_axis_x - tick_len - 38.0
        } else {
            y_axis_x + tick_len + 38.0
        };

        if title_x >= canvas_rect.left() && title_x <= canvas_rect.right() {
            painter.text(
                Pos2::new(title_x, title_y),
                if y_tick_dir < 0.0 {
                    egui::Align2::RIGHT_CENTER
                } else {
                    egui::Align2::LEFT_CENTER
                },
                options.y_title,
                title_font_id,
                text_color,
            );
        }
    }
}

pub struct TickMark {
    pub val: f64,
    pub label: String,
}

fn generate_nice_ticks(min_val: f64, max_val: f64, max_ticks: usize) -> Vec<TickMark> {
    if (max_val - min_val).abs() < 1e-12 {
        return vec![TickMark {
            val: min_val,
            label: format_tick_value(min_val, 1.0),
        }];
    }

    let range = (max_val - min_val).abs();
    let raw_step = range / (max_ticks as f64).max(1.0);
    let mag = 10.0f64.powf(raw_step.log10().floor());
    let residual = raw_step / mag;

    let step_mult = if residual < 1.5 {
        1.0
    } else if residual < 3.0 {
        2.0
    } else if residual < 7.0 {
        5.0
    } else {
        10.0
    };

    let step = step_mult * mag;
    let start_tick = (min_val / step).ceil() * step;
    let mut ticks = Vec::new();

    let mut current = start_tick;
    // Guard against floating point infinite loop
    let max_iter = 50;
    let mut iter = 0;
    while current <= max_val + step * 1e-6 && iter < max_iter {
        ticks.push(TickMark {
            val: current,
            label: format_tick_value(current, step),
        });
        current += step;
        iter += 1;
    }

    ticks
}

fn format_tick_value(val: f64, step: f64) -> String {
    let abs_val = val.abs();
    if abs_val > 0.0 && !(1e-3..1e5).contains(&abs_val) {
        format!("{:.2e}", val)
    } else if step < 1.0 {
        let decimals = (-step.log10()).ceil().max(0.0) as usize + 1;
        format!("{:.1$}", val, decimals.min(4))
    } else if val.fract().abs() < 1e-6 {
        format!("{:.0}", val)
    } else {
        format!("{:.2}", val)
    }
}

fn draw_tick_label(
    painter: &egui::Painter,
    pos: Pos2,
    text: &str,
    font_id: FontId,
    color: Color32,
    use_pill_bg: bool,
) {
    draw_tick_label_aligned(
        painter,
        pos,
        text,
        font_id,
        color,
        egui::Align2::CENTER_TOP,
        use_pill_bg,
    );
}

fn draw_tick_label_aligned(
    painter: &egui::Painter,
    pos: Pos2,
    text: &str,
    font_id: FontId,
    color: Color32,
    align: egui::Align2,
    use_pill_bg: bool,
) {
    if use_pill_bg {
        let galley = painter.layout_no_wrap(text.to_string(), font_id.clone(), color);
        let rect = align.anchor_rect(Rect::from_min_size(pos, galley.size()));
        let expanded = rect.expand(2.5);
        let bg_color = Color32::from_black_alpha(170);
        painter.rect_filled(expanded, 3.0, bg_color);
    }

    painter.text(pos, align, text, font_id, color);
}
