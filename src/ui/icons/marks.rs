//! Single-glyph marks: check, cross, warning, info, bullet, chevrons, eyes,
//! the drag grip and the orientation flip.

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
    c.polygon(&tri, c.body(), c.stroke(Weight::Base));
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

/// ChevronDown: bold down-pointing chevron with round caps (14x7 keyline).
pub fn draw_chevron_down(c: &IconCanvas) {
    c.path_round(
        &[(5.0, 9.0), (12.0, 16.0), (19.0, 9.0)],
        c.stroke(Weight::Bold),
    );
}

/// ChevronUp: bold up-pointing chevron with round caps (14x7 keyline).
pub fn draw_chevron_up(c: &IconCanvas) {
    c.path_round(
        &[(5.0, 15.0), (12.0, 8.0), (19.0, 15.0)],
        c.stroke(Weight::Bold),
    );
}

/// Eye outline: an almond of two lid arcs, with the iris (20x12 keyline).
fn eye(c: &IconCanvas) {
    use std::f32::consts::PI;
    let s = c.stroke(Weight::Base);
    let lids = (10.0, 6.0);
    let mut outline = c.arc((key::C, key::C), lids, PI, 2.0 * PI);
    outline.extend(c.arc((key::C, key::C), lids, 0.0, PI).into_iter().skip(1));
    c.curve_closed(outline, s);
    if c.compact() {
        c.dot((key::C, key::C), 2.6, c.color);
    } else {
        c.circle((key::C, key::C), 3.0, Color32::TRANSPARENT, s);
        c.dot((key::C, key::C), 1.2, c.color);
    }
}

/// Eye: shown (20x12 keyline).
pub fn draw_eye(c: &IconCanvas) {
    eye(c);
}

/// EyeOff: the eye struck through (20x18 keyline).
pub fn draw_eye_off(c: &IconCanvas) {
    eye(c);
    c.path_round(&[(4.0, 20.0), (20.0, 4.0)], c.stroke(Weight::Bold));
}

/// Grip: a 2x3 grid of dots, the handle that drags a panel (8x14 keyline).
pub fn draw_grip(c: &IconCanvas) {
    for x in [8.0, 16.0] {
        for y in [5.0, 12.0, 19.0] {
            c.dot((x, y), 1.9, c.color);
        }
    }
}

/// Orientation: a horizontal bar turning into a vertical one along a
/// quarter-turn arrow (18x18 keyline).
pub fn draw_orientation(c: &IconCanvas) {
    use std::f32::consts::PI;
    let s = c.stroke(Weight::Base);
    let fill = if c.compact() {
        Color32::TRANSPARENT
    } else {
        c.body()
    };
    c.rrect((key::SQ_MIN, 16.0), (13.0, key::SQ_MAX), 1.0, fill, s);
    c.rrect((16.0, key::SQ_MIN), (key::SQ_MAX, 13.0), 1.0, fill, s);
    c.curve(c.arc((11.5, 11.5), (6.5, 6.5), PI, 1.5 * PI), s);
    c.path_round(&[(9.5, 2.8), (11.7, 5.0), (9.5, 7.2)], s);
}
