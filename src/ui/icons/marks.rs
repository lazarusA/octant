//! Single-glyph marks: check, cross, warning, info, bullet and chevron.

use super::canvas::{IconCanvas, Weight, key};
use egui::{Color32, Stroke};

/// Check: bold tick with round caps (16x12 keyline).
pub fn draw_check(c: &IconCanvas) {
    c.path_round(
        &[(4.5, 12.5), (9.5, 17.5), (19.5, 6.5)],
        c.stroke(Weight::Bold),
    );
}

/// Cross: bold diagonal cross with round caps (14x14 keyline).
pub fn draw_cross(c: &IconCanvas) {
    let s = c.stroke(Weight::Bold);
    c.path_round(&[(5.0, 5.0), (19.0, 19.0)], s);
    c.path_round(&[(19.0, 5.0), (5.0, 19.0)], s);
}

/// Warning: hazard triangle with an exclamation mark (19x17 keyline).
pub fn draw_warning(c: &IconCanvas) {
    let tri = [(12.0, 3.0), (21.5, 20.0), (2.5, 20.0)];
    c.fill(&tri, c.body());
    c.closed(&tri, c.stroke(Weight::Base));
    c.path_round(&[(12.0, 9.5), (12.0, 14.0)], c.stroke(Weight::Bold));
    c.dot((12.0, 17.0), 1.2, c.color);
}

/// Info: circle with a dot and stem (20-unit circle).
pub fn draw_info(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    c.circle((key::C, key::C), key::CIRCLE_R, Color32::TRANSPARENT, s);
    c.dot((12.0, 7.5), 1.3, c.color);
    c.path_round(&[(12.0, 11.0), (12.0, 17.0)], c.stroke(Weight::Bold));
}

/// Bullet: small solid diamond (9-unit diamond).
pub fn draw_bullet(c: &IconCanvas) {
    c.diamond((key::C, key::C), 4.5, 4.5, c.color, Stroke::NONE);
}

/// ChevronRight: bold right-pointing chevron with round caps (7x14 keyline).
pub fn draw_chevron_right(c: &IconCanvas) {
    c.path_round(
        &[(9.0, 5.0), (16.0, 12.0), (9.0, 19.0)],
        c.stroke(Weight::Bold),
    );
}
