//! The x axis title, centered under (or, near the canvas bottom, above) the
//! bottom axis.

use egui::{FontId, Pos2, Rect};

use super::segments::{draw_tick_label, is_near_corner};

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_x_title(
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
