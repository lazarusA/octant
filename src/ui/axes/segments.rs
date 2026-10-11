//! Tick and label rendering segments for plot axes.

use egui::{Color32, FontId, Pos2, Rect, Stroke};

use super::ticks::TickMark;

#[allow(clippy::too_many_arguments)]
pub fn draw_x_ticks(
    painter: &egui::Painter,
    visuals: &egui::Visuals,
    x_ticks: &[TickMark; 7],
    visible_left: f32,
    vis_w: f32,
    bottom_y: f32,
    top_y: f32,
    b_dir: f32,
    t_dir: f32,
    tick_len: f32,
    canvas_rect: Rect,
    stroke: Stroke,
    sec_stroke: Stroke,
    font_id: &FontId,
    text_color: Color32,
) {
    for (i, tick) in x_ticks.iter().enumerate() {
        let fract = i as f32 / 6.0;
        let tick_x = (visible_left + fract * vis_w).round();

        // Bottom tick
        painter.line_segment(
            [
                Pos2::new(tick_x, bottom_y),
                Pos2::new(tick_x, bottom_y + b_dir * tick_len),
            ],
            stroke,
        );
        let (b_pos, b_align, b_pill) = if b_dir > 0.0 {
            (
                Pos2::new(tick_x, bottom_y + tick_len + 4.0),
                egui::Align2::CENTER_TOP,
                false,
            )
        } else {
            (
                Pos2::new(tick_x, bottom_y - tick_len - 4.0),
                egui::Align2::CENTER_BOTTOM,
                true,
            )
        };
        if !is_near_corner(b_pos, canvas_rect) {
            draw_tick_label(
                visuals,
                painter,
                b_pos,
                tick.as_str(),
                font_id,
                text_color,
                b_align,
                b_pill,
            );
        }

        // Top tick
        painter.line_segment(
            [
                Pos2::new(tick_x, top_y),
                Pos2::new(tick_x, top_y + t_dir * tick_len),
            ],
            sec_stroke,
        );
        if t_dir > 0.0 {
            let t_pos = Pos2::new(tick_x, top_y + tick_len + 4.0);
            if !is_near_corner(t_pos, canvas_rect) {
                draw_tick_label(
                    visuals,
                    painter,
                    t_pos,
                    tick.as_str(),
                    font_id,
                    text_color,
                    egui::Align2::CENTER_TOP,
                    true,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_y_ticks(
    painter: &egui::Painter,
    visuals: &egui::Visuals,
    y_ticks: &[TickMark; 7],
    visible_bottom: f32,
    vis_h: f32,
    right_x: f32,
    left_x: f32,
    r_dir: f32,
    l_dir: f32,
    tick_len: f32,
    canvas_rect: Rect,
    stroke: Stroke,
    sec_stroke: Stroke,
    font_id: &FontId,
    text_color: Color32,
) {
    for (j, tick) in y_ticks.iter().enumerate() {
        let fract = j as f32 / 6.0;
        let tick_y = (visible_bottom - fract * vis_h).round();

        // Right tick
        painter.line_segment(
            [
                Pos2::new(right_x, tick_y),
                Pos2::new(right_x + r_dir * tick_len, tick_y),
            ],
            stroke,
        );
        let (r_pos, r_align) = if r_dir > 0.0 {
            (
                Pos2::new(right_x + tick_len + 4.0, tick_y),
                egui::Align2::LEFT_CENTER,
            )
        } else {
            (
                Pos2::new(right_x - tick_len - 4.0, tick_y),
                egui::Align2::RIGHT_CENTER,
            )
        };
        if !is_near_corner(r_pos, canvas_rect) {
            draw_tick_label(
                visuals,
                painter,
                r_pos,
                tick.as_str(),
                font_id,
                text_color,
                r_align,
                r_dir < 0.0,
            );
        }

        // Left tick
        painter.line_segment(
            [
                Pos2::new(left_x, tick_y),
                Pos2::new(left_x + l_dir * tick_len, tick_y),
            ],
            sec_stroke,
        );
        if l_dir > 0.0 {
            let l_pos = Pos2::new(left_x + tick_len + 4.0, tick_y);
            if !is_near_corner(l_pos, canvas_rect) {
                draw_tick_label(
                    visuals,
                    painter,
                    l_pos,
                    tick.as_str(),
                    font_id,
                    text_color,
                    egui::Align2::LEFT_CENTER,
                    true,
                );
            }
        }
    }
}

pub fn is_near_corner(pos: Pos2, rect: Rect) -> bool {
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
pub fn draw_tick_label(
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
