use egui::{Color32, FontId, Pos2, Rect, Stroke};

pub struct PlotAxisOptions<'a> {
    pub x_domain: (f64, f64),
    pub y_domain: (f64, f64),
    pub x_title: &'a str,
    pub y_title: &'a str,
    pub x_units: Option<&'a str>,
    pub y_units: Option<&'a str>,
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
    let (x_min_domain, x_max_domain) = (
        options.x_domain.0.min(options.x_domain.1),
        options.x_domain.0.max(options.x_domain.1),
    );
    let full_x_span = (x_max_domain - x_min_domain).max(1e-9);
    let t_x_min = ((visible_left - plot_rect.left()) / plot_rect.width().max(1.0)) as f64;
    let t_x_max = ((visible_right - plot_rect.left()) / plot_rect.width().max(1.0)) as f64;
    let vis_x_min = x_min_domain + t_x_min * full_x_span;
    let vis_x_max = x_min_domain + t_x_max * full_x_span;

    let (y_min_domain, y_max_domain) = (
        options.y_domain.0.min(options.y_domain.1),
        options.y_domain.0.max(options.y_domain.1),
    );
    let full_y_span = (y_max_domain - y_min_domain).max(1e-9);
    let t_y_min = ((plot_rect.bottom() - visible_bottom) / plot_rect.height().max(1.0)) as f64;
    let t_y_max = ((plot_rect.bottom() - visible_top) / plot_rect.height().max(1.0)) as f64;
    let vis_y_min = y_min_domain + t_y_min * full_y_span;
    let vis_y_max = y_min_domain + t_y_max * full_y_span;

    // Generate constant count of ticks (7 ticks) for visible viewport using stack buffers
    const NUM_TICKS: usize = 7;
    let x_ticks =
        generate_constant_count_ticks(vis_x_min, vis_x_max, options.x_title, options.x_units);
    let y_ticks =
        generate_constant_count_ticks(vis_y_min, vis_y_max, options.y_title, options.y_units);

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
        let fract = i as f32 / (NUM_TICKS - 1) as f32;
        let tick_x = (visible_left + fract * vis_w).round();

        if tick_x >= visible_left - 1.0 && tick_x <= visible_right + 1.0 {
            // Bottom tick mark & label
            let b_start = Pos2::new(tick_x, bottom_axis_y);
            let b_end = Pos2::new(tick_x, bottom_axis_y + bottom_x_tick_dir * tick_len);
            painter.line_segment([b_start, b_end], stroke);

            let (b_label_pos, b_align, b_use_pill) = if bottom_x_tick_dir > 0.0 {
                (
                    Pos2::new(tick_x, bottom_axis_y + tick_len + 4.0),
                    egui::Align2::CENTER_TOP,
                    false,
                )
            } else {
                (
                    Pos2::new(tick_x, bottom_axis_y - tick_len - 4.0),
                    egui::Align2::CENTER_BOTTOM,
                    true, // Inward overlay pill
                )
            };

            if !is_near_corner(b_label_pos, canvas_rect) {
                draw_tick_label_aligned(
                    visuals,
                    painter,
                    b_label_pos,
                    tick.as_str(),
                    &font_id,
                    text_color,
                    b_align,
                    b_use_pill,
                );
            }

            // Top tick mark & label
            let t_start = Pos2::new(tick_x, top_axis_y);
            let t_end = Pos2::new(tick_x, top_axis_y + top_x_tick_dir * tick_len);
            painter.line_segment([t_start, t_end], secondary_stroke);

            if top_x_tick_dir > 0.0 {
                // Inward Top X-Axis tick label numbers in overlay pills
                let t_label_pos = Pos2::new(tick_x, top_axis_y + tick_len + 4.0);
                if !is_near_corner(t_label_pos, canvas_rect) {
                    draw_tick_label_aligned(
                        visuals,
                        painter,
                        t_label_pos,
                        tick.as_str(),
                        &font_id,
                        text_color,
                        egui::Align2::CENTER_TOP,
                        true, // Inward overlay pill
                    );
                }
            }
        }
    }

    // Draw X-Axis Title
    if !options.x_title.is_empty() {
        let title_x = ((visible_left + visible_right) * 0.5).round();
        let (title_y, title_align, title_use_pill) = if bottom_x_tick_dir > 0.0 {
            (
                bottom_axis_y + tick_len + 20.0,
                egui::Align2::CENTER_TOP,
                false,
            )
        } else {
            (
                bottom_axis_y - tick_len - 28.0,
                egui::Align2::CENTER_BOTTOM,
                true, // Inward overlay pill
            )
        };

        if title_y >= canvas_rect.top() && title_y <= canvas_rect.bottom() {
            let title_pos = Pos2::new(title_x, title_y);
            if !is_near_corner(title_pos, canvas_rect) {
                draw_tick_label_aligned(
                    visuals,
                    painter,
                    title_pos,
                    options.x_title,
                    &title_font_id,
                    text_color,
                    title_align,
                    title_use_pill,
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
        let fract = j as f32 / (NUM_TICKS - 1) as f32;
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
                    tick.as_str(),
                    &font_id,
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
                        tick.as_str(),
                        &font_id,
                        text_color,
                        egui::Align2::LEFT_CENTER,
                        true, // use bg pill for inward left ticks
                    );
                }
            }
        }
    }
}

/// Zero-allocation tick mark storing its formatted label in a fixed stack buffer.
#[derive(Clone, Copy)]
pub struct TickMark {
    pub val: f64,
    len: u8,
    buf: [u8; 32],
}

impl TickMark {
    pub fn new(val: f64, step: f64) -> Self {
        let mut mark = Self {
            val,
            len: 0,
            buf: [0u8; 32],
        };
        mark.format(step);
        mark
    }

    pub fn new_with_context(
        val: f64,
        step: f64,
        total_span: f64,
        title: &str,
        units: Option<&str>,
    ) -> Self {
        let mut mark = Self {
            val,
            len: 0,
            buf: [0u8; 32],
        };
        mark.format_with_context(step, total_span, title, units);
        mark
    }

    fn format_with_context(
        &mut self,
        step: f64,
        total_span: f64,
        title: &str,
        units: Option<&str>,
    ) {
        let is_time = crate::data::coordinates::naming::is_animated_time_name(title)
            || units.is_some_and(crate::utils::units::is_cf_time_unit);

        if is_time && self.try_format_time(total_span, title, units) {
            return;
        }

        self.format(step);
    }

    fn try_format_time(&mut self, total_span: f64, title: &str, units: Option<&str>) -> bool {
        use std::io::Write;
        let mut cursor = std::io::Cursor::new(&mut self.buf[..]);

        // 1. CF Relative Units (e.g. "hours since 2024-01-01")
        if let Some(units_str) = units {
            let clean_units = units_str.trim();
            if let Some((unit_part, ref_date_str)) = crate::utils::units::split_since(clean_units) {
                let (y, m, d) =
                    crate::utils::units::parse_iso_date(ref_date_str).unwrap_or((1970, 1, 1));
                let scale_ms =
                    crate::utils::units::unit_to_milliseconds(unit_part).unwrap_or(1_000) as f64;
                let total_ms = self.val * scale_ms;
                let total_hours = (total_ms / 3_600_000.0).round() as i64;
                let days_added = total_hours.div_euclid(24);
                let hour_of_day = total_hours.rem_euclid(24) as usize;
                let (res_y, res_m, res_d) =
                    crate::utils::units::add_days_to_date(y, m, d, days_added);

                let span_days = (total_span.abs() * scale_ms) / 86_400_000.0;
                let _ = if span_days > 730.0 {
                    write!(cursor, "{:04}", res_y)
                } else if span_days > 60.0 {
                    write!(cursor, "{:04}-{:02}", res_y, res_m)
                } else if span_days > 2.0 {
                    write!(cursor, "{:02}-{:02}", res_m, res_d)
                } else {
                    write!(cursor, "{:02}:{:02}", hour_of_day, 0)
                };
                self.len = cursor.position() as u8;
                return self.len > 0;
            }

            // 2. ISO reference date in units_str (e.g. "2024-01-01" from time_coverage_start)
            if let Some((y, m, d)) = crate::utils::units::parse_iso_date(clean_units) {
                let days_added = self.val.round() as i64;
                let (res_y, res_m, res_d) =
                    crate::utils::units::add_days_to_date(y, m, d, days_added);
                let span_days = total_span.abs();
                let _ = if span_days > 730.0 {
                    write!(cursor, "{:04}", res_y)
                } else if span_days > 60.0 {
                    write!(cursor, "{:04}-{:02}", res_y, res_m)
                } else {
                    write!(cursor, "{:02}-{:02}", res_m, res_d)
                };
                self.len = cursor.position() as u8;
                return self.len > 0;
            }

            // 3. Simple duration units (h, d, min, s)
            if clean_units.eq_ignore_ascii_case("h")
                || clean_units.eq_ignore_ascii_case("hr")
                || clean_units.eq_ignore_ascii_case("hours")
                || clean_units.eq_ignore_ascii_case("hour")
            {
                let _ = write!(cursor, "{:.0} h", self.val);
                self.len = cursor.position() as u8;
                return self.len > 0;
            }
            if clean_units.eq_ignore_ascii_case("d")
                || clean_units.eq_ignore_ascii_case("day")
                || clean_units.eq_ignore_ascii_case("days")
            {
                let _ = write!(cursor, "{:.0} d", self.val);
                self.len = cursor.position() as u8;
                return self.len > 0;
            }
        }

        // 4. Unix epoch timestamps (> 100M seconds or > 100B ms)
        if self.val.abs() > 1e8 && self.val.abs() < 1e13 {
            let total_secs = if self.val.abs() > 1e11 {
                (self.val / 1000.0).round() as i64
            } else {
                self.val.round() as i64
            };
            let total_hours = total_secs.div_euclid(3600);
            let days_added = total_hours.div_euclid(24);
            let hour_of_day = total_hours.rem_euclid(24) as usize;
            let (res_y, res_m, res_d) =
                crate::utils::units::add_days_to_date(1970, 1, 1, days_added);

            let span_secs = if total_span.abs() > 1e11 {
                total_span.abs() / 1000.0
            } else {
                total_span.abs()
            };
            let span_days = span_secs / 86400.0;
            let _ = if span_days > 730.0 {
                write!(cursor, "{:04}", res_y)
            } else if span_days > 60.0 {
                write!(cursor, "{:04}-{:02}", res_y, res_m)
            } else if span_days > 2.0 {
                write!(cursor, "{:02}-{:02}", res_m, res_d)
            } else {
                write!(cursor, "{:02}:{:02}", hour_of_day, 0)
            };
            self.len = cursor.position() as u8;
            return self.len > 0;
        }

        // 5. Step indices
        if title.eq_ignore_ascii_case("step") || title.eq_ignore_ascii_case("timestep") {
            let _ = write!(cursor, "t={:.0}", self.val);
            self.len = cursor.position() as u8;
            return self.len > 0;
        }

        false
    }

    fn format(&mut self, step: f64) {
        use std::io::Write;
        let mut cursor = std::io::Cursor::new(&mut self.buf[..]);
        let abs_val = self.val.abs();
        let _ = if abs_val > 0.0 && !(1e-3..1e5).contains(&abs_val) {
            write!(cursor, "{:.2e}", self.val)
        } else if step.abs() < 1.0 {
            let decimals = ((-step.abs().log10()).ceil().max(0.0) as usize + 1).min(4);
            write!(cursor, "{:.*}", decimals, self.val)
        } else if self.val.fract().abs() < 1e-6 {
            write!(cursor, "{:.0}", self.val)
        } else {
            write!(cursor, "{:.2}", self.val)
        };
        self.len = cursor.position() as u8;
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[..self.len as usize]).unwrap_or("")
    }
}

/// Generates a constant 7-count stack array of ticks with zero heap allocation.
fn generate_constant_count_ticks(
    min_val: f64,
    max_val: f64,
    title: &str,
    units: Option<&str>,
) -> [TickMark; 7] {
    let range = max_val - min_val;
    let step = range / 6.0;

    let mut ticks = [TickMark {
        val: 0.0,
        len: 0,
        buf: [0u8; 32],
    }; 7];

    for (i, tick) in ticks.iter_mut().enumerate() {
        let val = min_val + i as f64 * step;
        *tick = TickMark::new_with_context(val, step, range, title, units);
    }

    ticks
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
    font_id: &FontId,
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

    painter.text(pos, align, text, font_id.clone(), color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_tick_formatting_cf_units() {
        let tick =
            TickMark::new_with_context(10.0, 1.0, 30.0, "time", Some("days since 2024-01-01"));
        assert_eq!(tick.as_str(), "01-11");
    }

    #[test]
    fn test_time_tick_formatting_iso_units() {
        let tick = TickMark::new_with_context(15.0, 1.0, 30.0, "time", Some("2024-01-01"));
        assert_eq!(tick.as_str(), "01-16");
    }

    #[test]
    fn test_time_tick_formatting_durations() {
        let tick_h = TickMark::new_with_context(24.0, 4.0, 48.0, "time", Some("hours"));
        assert_eq!(tick_h.as_str(), "24 h");

        let tick_d = TickMark::new_with_context(7.0, 1.0, 14.0, "time", Some("days"));
        assert_eq!(tick_d.as_str(), "7 d");
    }

    #[test]
    fn test_time_tick_formatting_step() {
        let tick_step = TickMark::new_with_context(42.0, 1.0, 100.0, "step", None);
        assert_eq!(tick_step.as_str(), "t=42");
    }

    #[test]
    fn test_standard_numeric_tick_formatting() {
        let tick_num = TickMark::new_with_context(123.45, 1.0, 100.0, "lon", None);
        assert_eq!(tick_num.as_str(), "123.45");
    }
}
