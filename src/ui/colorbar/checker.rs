//! Transparency backdrop for the colorbar when colormapped values are translucent.

use egui::{Color32, Painter, Rect, Vec2};

/// Paints a two-tone checkerboard of square cells half the bar height.
pub fn paint_checker(painter: &Painter, rect: Rect, dark_mode: bool) {
    let (light, dark) = if dark_mode {
        (Color32::from_gray(90), Color32::from_gray(55))
    } else {
        (Color32::from_gray(250), Color32::from_gray(205))
    };
    painter.rect_filled(rect, 0.0, light);
    let cell = (rect.height() / 2.0).max(1.0);
    let cols = (rect.width() / cell).ceil() as usize;
    for col in 0..cols {
        for row in 0..2 {
            if (col + row) % 2 == 0 {
                continue;
            }
            let min = rect.min + Vec2::new(col as f32 * cell, row as f32 * cell);
            let cell_rect = Rect::from_min_size(min, Vec2::splat(cell)).intersect(rect);
            painter.rect_filled(cell_rect, 0.0, dark);
        }
    }
}

/// `color` with its alpha replaced by `alpha` in [0, 1].
pub fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    let [r, g, b, _] = color.to_array();
    Color32::from_rgba_unmultiplied(r, g, b, crate::utils::colormap::lut::unit_to_u8(alpha))
}
