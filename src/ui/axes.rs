use egui::{Color32, FontId, Pos2, Rect, Stroke};

pub struct PlotAxisOptions<'a> {
    pub x_domain: (f64, f64),
    pub y_domain: (f64, f64),
    pub x_title: &'a str,
    pub y_title: &'a str,
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

    let vis_w = visible_right - visible_left;
    let vis_h = visible_bottom - visible_top;

    if vis_w <= 1.0 || vis_h <= 1.0 {
        return; // Plot is off-screen
    }

    // Calculate visible domain span for constant tick count updating on zoom
    let full_x_span = (options.x_domain.1 - options.x_domain.0).max(1e-9);
    let t_x_min = ((visible_left - plot_rect.left()) / plot_rect.width().max(1.0)) as f64;
    let t_x_max = ((visible_right - plot_rect.left()) / plot_rect.width().max(1.0)) as f64;
    let vis_x_min = options.x_domain.0 + t_x_min * full_x_span;
    let vis_x_max = options.x_domain.0 + t_x_max * full_x_span;

    let full_y_span = (options.y_domain.1 - options.y_domain.0).max(1e-9);
    let t_y_min = ((plot_rect.bottom() - visible_bottom) / plot_rect.height().max(1.0)) as f64;
    let t_y_max = ((plot_rect.bottom() - visible_top) / plot_rect.height().max(1.0)) as f64;
    let vis_y_min = options.y_domain.0 + t_y_min * full_y_span;
    let vis_y_max = options.y_domain.0 + t_y_max * full_y_span;

    // Generate constant count of ticks (7 ticks) for visible viewport
    let num_x_ticks = 7;
    let num_y_ticks = 7;
    let x_ticks = generate_constant_count_ticks(vis_x_min, vis_x_max, num_x_ticks);
    let y_ticks = generate_constant_count_ticks(vis_y_min, vis_y_max, num_y_ticks);

    // ==========================================
    // 1. BOTTOM & TOP X-AXES
    // ==========================================
    let is_bottom_close_to_nav = (canvas_rect.bottom() - plot_rect.bottom()).abs() < 28.0
        || plot_rect.bottom() >= canvas_rect.bottom() - 15.0;

    let is_bottom_inside = !is_bottom_close_to_nav
        && plot_rect.bottom() >= canvas_rect.top()
        && plot_rect.bottom() <= canvas_rect.bottom();

    let (bottom_axis_y, bottom_x_tick_dir) = if is_bottom_inside {
        (plot_rect.bottom(), 1.0) // Outward (pointing down)
    } else {
        (canvas_rect.bottom() - 1.0, -1.0) // Inward (pointing up into canvas)
    };

    // Draw Bottom X-Axis Line
    painter.line_segment(
        [
            Pos2::new(visible_left.round(), bottom_axis_y.round()),
            Pos2::new(visible_right.round(), bottom_axis_y.round()),
        ],
        stroke,
    );

    // Draw Top X-Axis Line
    let is_top_inside = plot_rect.top() >= canvas_rect.top()
        && plot_rect.top() <= canvas_rect.bottom()
        && (plot_rect.top() - canvas_rect.top()).abs() > 20.0;

    let (top_axis_y, top_x_tick_dir) = if is_top_inside {
        (plot_rect.top(), -1.0) // Outward (pointing up)
    } else {
        (canvas_rect.top() + 1.0, 1.0) // Inward (pointing down into canvas)
    };

    painter.line_segment(
        [
            Pos2::new(visible_left.round(), top_axis_y.round()),
            Pos2::new(visible_right.round(), top_axis_y.round()),
        ],
        secondary_stroke,
    );

    // Render X-Axis Ticks (Bottom & Top)
    for (i, tick) in x_ticks.iter().enumerate() {
        let fract = if num_x_ticks > 1 {
            i as f32 / (num_x_ticks - 1) as f32
        } else {
            0.5
        };
        let tick_x = (visible_left + fract * vis_w).round();

        if tick_x >= visible_left - 1.0 && tick_x <= visible_right + 1.0 {
            // Bottom tick mark & label
            let b_start = Pos2::new(tick_x, bottom_axis_y);
            let b_end = Pos2::new(tick_x, bottom_axis_y + bottom_x_tick_dir * tick_len);
            painter.line_segment([b_start, b_end], stroke);

            let b_label_pos = if bottom_x_tick_dir > 0.0 {
                Pos2::new(tick_x, bottom_axis_y + tick_len + 3.0)
            } else {
                Pos2::new(tick_x, bottom_axis_y - tick_len - 14.0)
            };

            if !is_near_corner(b_label_pos, canvas_rect) {
                draw_tick_label_aligned(
                    visuals,
                    painter,
                    b_label_pos,
                    &tick.label,
                    font_id.clone(),
                    text_color,
                    egui::Align2::CENTER_TOP,
                    bottom_x_tick_dir < 0.0, // use bg pill when inward
                );
            }

            // Top tick mark & label
            let t_start = Pos2::new(tick_x, top_axis_y);
            let t_end = Pos2::new(tick_x, top_axis_y + top_x_tick_dir * tick_len);
            painter.line_segment([t_start, t_end], secondary_stroke);

            if top_x_tick_dir > 0.0 {
                // Inward Top X-Axis tick label numbers in pills
                let t_label_pos = Pos2::new(tick_x, top_axis_y + tick_len + 3.0);
                if !is_near_corner(t_label_pos, canvas_rect) {
                    draw_tick_label_aligned(
                        visuals,
                        painter,
                        t_label_pos,
                        &tick.label,
                        font_id.clone(),
                        text_color,
                        egui::Align2::CENTER_TOP,
                        true, // use bg pill for inward top ticks
                    );
                }
            }
        }
    }

    // Draw X-Axis Title
    if !options.x_title.is_empty() {
        let title_x = ((visible_left + visible_right) * 0.5).round();
        let title_y = if bottom_x_tick_dir > 0.0 {
            bottom_axis_y + tick_len + 20.0
        } else {
            bottom_axis_y - tick_len - 30.0
        };

        if title_y >= canvas_rect.top() && title_y <= canvas_rect.bottom() {
            let title_pos = Pos2::new(title_x, title_y);
            if !is_near_corner(title_pos, canvas_rect) {
                draw_tick_label_aligned(
                    visuals,
                    painter,
                    title_pos,
                    options.x_title,
                    title_font_id.clone(),
                    text_color,
                    egui::Align2::CENTER_TOP,
                    bottom_x_tick_dir < 0.0,
                );
            }
        }
    }

    // ==========================================
    // 2. RIGHT & LEFT Y-AXES
    // ==========================================
    let is_right_inside =
        plot_rect.right() >= canvas_rect.left() && plot_rect.right() <= canvas_rect.right() - 20.0;

    let (right_axis_x, right_y_tick_dir) = if is_right_inside {
        (plot_rect.right(), 1.0) // Outward (pointing right)
    } else {
        (canvas_rect.right() - 1.0, -1.0) // Inward (pointing left into canvas)
    };

    // Draw Right Vertical Axis Line
    painter.line_segment(
        [
            Pos2::new(right_axis_x.round(), visible_top.round()),
            Pos2::new(right_axis_x.round(), visible_bottom.round()),
        ],
        stroke,
    );

    // Draw Left Vertical Axis Line
    let is_left_inside =
        plot_rect.left() >= canvas_rect.left() + 20.0 && plot_rect.left() <= canvas_rect.right();

    let (left_axis_x, left_y_tick_dir) = if is_left_inside {
        (plot_rect.left(), -1.0) // Outward (pointing left)
    } else {
        (canvas_rect.left() + 1.0, 1.0) // Inward (pointing right into canvas)
    };

    painter.line_segment(
        [
            Pos2::new(left_axis_x.round(), visible_top.round()),
            Pos2::new(left_axis_x.round(), visible_bottom.round()),
        ],
        secondary_stroke,
    );

    // Render Y-Axis Ticks (Right & Left)
    for (j, tick) in y_ticks.iter().enumerate() {
        let fract = if num_y_ticks > 1 {
            j as f32 / (num_y_ticks - 1) as f32
        } else {
            0.5
        };
        // Screen Y decreases upwards
        let tick_y = (visible_bottom - fract * vis_h).round();

        if tick_y >= visible_top - 1.0 && tick_y <= visible_bottom + 1.0 {
            // Right Y tick mark & label
            let r_start = Pos2::new(right_axis_x, tick_y);
            let r_end = Pos2::new(right_axis_x + right_y_tick_dir * tick_len, tick_y);
            painter.line_segment([r_start, r_end], stroke);

            let r_label_pos = if right_y_tick_dir > 0.0 {
                Pos2::new(right_axis_x + tick_len + 4.0, tick_y)
            } else {
                Pos2::new(right_axis_x - tick_len - 4.0, tick_y)
            };

            let align = if right_y_tick_dir > 0.0 {
                egui::Align2::LEFT_CENTER
            } else {
                egui::Align2::RIGHT_CENTER
            };

            if !is_near_corner(r_label_pos, canvas_rect) {
                draw_tick_label_aligned(
                    visuals,
                    painter,
                    r_label_pos,
                    &tick.label,
                    font_id.clone(),
                    text_color,
                    align,
                    right_y_tick_dir < 0.0, // use bg pill when inward
                );
            }

            // Left Y tick mark & label
            let l_start = Pos2::new(left_axis_x, tick_y);
            let l_end = Pos2::new(left_axis_x + left_y_tick_dir * tick_len, tick_y);
            painter.line_segment([l_start, l_end], secondary_stroke);

            if left_y_tick_dir > 0.0 {
                // Inward Left Y-Axis tick label numbers in pills
                let l_label_pos = Pos2::new(left_axis_x + tick_len + 4.0, tick_y);
                if !is_near_corner(l_label_pos, canvas_rect) {
                    draw_tick_label_aligned(
                        visuals,
                        painter,
                        l_label_pos,
                        &tick.label,
                        font_id.clone(),
                        text_color,
                        egui::Align2::LEFT_CENTER,
                        true, // use bg pill for inward left ticks
                    );
                }
            }
        }
    }
}

pub struct TickMark {
    pub val: f64,
    pub label: String,
}

fn generate_constant_count_ticks(min_val: f64, max_val: f64, count: usize) -> Vec<TickMark> {
    let count = count.max(2);
    let range = max_val - min_val;
    let step = range / (count - 1) as f64;

    (0..count)
        .map(|i| {
            let val = min_val + i as f64 * step;
            TickMark {
                val,
                label: format_tick_value(val, step),
            }
        })
        .collect()
}

fn format_tick_value(val: f64, step: f64) -> String {
    let abs_val = val.abs();
    if abs_val > 0.0 && !(1e-3..1e5).contains(&abs_val) {
        format!("{:.2e}", val)
    } else if step.abs() < 1.0 {
        let decimals = (-step.abs().log10()).ceil().max(0.0) as usize + 1;
        format!("{:.1$}", val, decimals.min(4))
    } else if val.fract().abs() < 1e-6 {
        format!("{:.0}", val)
    } else {
        format!("{:.2}", val)
    }
}

/// Helper function to detect if a tick label position lands near any of the 4 canvas corners.
fn is_near_corner(pos: Pos2, rect: Rect) -> bool {
    let margin = 32.0;
    let near_left = (pos.x - rect.left()).abs() < margin;
    let near_right = (pos.x - rect.right()).abs() < margin;
    let near_top = (pos.y - rect.top()).abs() < margin;
    let near_bottom = (pos.y - rect.bottom()).abs() < margin;

    (near_left && near_top)
        || (near_right && near_top)
        || (near_left && near_bottom)
        || (near_right && near_bottom)
}

#[allow(clippy::too_many_arguments)]
fn draw_tick_label_aligned(
    visuals: &egui::Visuals,
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
        let expanded = rect.expand(3.0);

        // System theme-aware pill background & subtle border
        let bg_color = if visuals.dark_mode {
            Color32::from_black_alpha(200)
        } else {
            Color32::from_white_alpha(225)
        };
        let border_color = visuals
            .widgets
            .noninteractive
            .bg_stroke
            .color
            .linear_multiply(0.6);

        painter.rect(
            expanded,
            4.0,
            bg_color,
            Stroke::new(1.0, border_color),
            egui::StrokeKind::Inside,
        );
    }

    painter.text(pos, align, text, font_id, color);
}
