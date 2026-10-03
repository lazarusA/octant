//! Playback and timeline icons. Transport glyphs are solid fills; loop,
//! reset and gauge are outlined.

use super::canvas::{Caps, IconCanvas, Weight, key};
use egui::{Pos2, Vec2, vec2};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// Play: solid right-pointing triangle, nudged right for optical centering (13x16 keyline).
pub fn draw_play(c: &IconCanvas) {
    c.fill(&[(7.0, 4.0), (20.0, 12.0), (7.0, 20.0)], c.color);
}

/// Pause: two solid rounded bars (12x16 keyline).
pub fn draw_pause(c: &IconCanvas) {
    c.rrect_fill((6.0, 4.0), (10.0, 20.0), 1.0, c.color);
    c.rrect_fill((14.0, 4.0), (18.0, 20.0), 1.0, c.color);
}

/// Stop: solid rounded square (15-unit square).
pub fn draw_stop(c: &IconCanvas) {
    c.rrect_fill((4.5, 4.5), (19.5, 19.5), 2.0, c.color);
}

/// StepBackward: end bar and left-pointing triangle (16x16 keyline).
pub fn draw_step_backward(c: &IconCanvas) {
    c.rrect_fill((4.0, 4.0), (7.0, 20.0), 1.0, c.color);
    c.fill(&[(20.0, 4.0), (9.0, 12.0), (20.0, 20.0)], c.color);
}

/// StepForward: right-pointing triangle and end bar (16x16 keyline).
pub fn draw_step_forward(c: &IconCanvas) {
    c.fill(&[(4.0, 4.0), (15.0, 12.0), (4.0, 20.0)], c.color);
    c.rrect_fill((17.0, 4.0), (20.0, 20.0), 1.0, c.color);
}

/// SeekStart: limit bar and two left-pointing triangles (18x16 keyline).
pub fn draw_seek_start(c: &IconCanvas) {
    c.rrect_fill((3.0, 4.0), (6.0, 20.0), 1.0, c.color);
    c.fill(&[(13.0, 4.5), (7.0, 12.0), (13.0, 19.5)], c.color);
    c.fill(&[(21.0, 4.5), (14.5, 12.0), (21.0, 19.5)], c.color);
}

/// SeekEnd: two right-pointing triangles and limit bar (18x16 keyline).
pub fn draw_seek_end(c: &IconCanvas) {
    c.fill(&[(3.0, 4.5), (9.5, 12.0), (3.0, 19.5)], c.color);
    c.fill(&[(11.0, 4.5), (17.0, 12.0), (11.0, 19.5)], c.color);
    c.rrect_fill((18.0, 4.0), (21.0, 20.0), 1.0, c.color);
}

/// Solid arrowhead whose base is centered on `at`, pointing along `dir` (unit vector).
fn arrowhead(c: &IconCanvas, at: Pos2, dir: Vec2) {
    let u = c.unit();
    let normal = vec2(-dir.y, dir.x);
    let pts = vec![
        at + dir * (3.0 * u),
        at + normal * (2.6 * u),
        at - normal * (2.6 * u),
    ];
    c.fill_pts(pts, c.color);
}

/// Unit tangent of a clockwise screen-space arc at angle `a`.
fn cw_tangent(a: f32) -> Vec2 {
    vec2(-a.sin(), a.cos())
}

/// Loop: two arcs chasing each other around a circle, arrowheads at their ends (18-unit circle).
pub fn draw_loop(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let (ctr, r, gap, head) = ((key::C, key::C), 8.0, 0.25, 0.4);
    // Clockwise screen angles: upper arc left to right, lower arc right to left.
    for start in [PI + gap, gap] {
        let end = start + PI - 2.0 * gap - head;
        if let Some((_, tip)) = c.curve_capped(c.arc(ctr, (r, r), start, end), s, Caps::Start) {
            arrowhead(c, tip, cw_tangent(end));
        }
    }
}

/// Reset: counter-clockwise arc with an arrowhead at the top (18-unit circle).
pub fn draw_reset(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let (ctr, r) = ((key::C, key::C), 8.0);
    let top = -FRAC_PI_2;
    let arc = c.arc(ctr, (r, r), top, top + TAU * 0.78);
    if let Some((tip, _)) = c.curve_capped(arc, s, Caps::End) {
        arrowhead(c, tip, -cw_tangent(top));
    }
}

/// Gauge: 240-degree dial open at the bottom, three ticks, needle and hub (18x16 keyline).
pub fn draw_gauge(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let (ctr, r) = ((key::C, 13.5), 8.5);

    // Clockwise from lower left (150 deg) over the top to lower right (30 deg).
    let (a0, a1) = (PI * 5.0 / 6.0, PI * 13.0 / 6.0);
    c.curve_capped(c.arc(ctr, (r, r), a0, a1), s, Caps::Both);

    if !c.compact() {
        for a in [PI * 7.0 / 6.0, PI * 1.5, PI * 11.0 / 6.0] {
            c.seg(c.polar(ctr, r * 0.6, a), c.polar(ctr, r * 0.8, a), s);
        }
    }
    let needle = -PI * 0.3;
    c.seg(c.p(ctr.0, ctr.1), c.polar(ctr, r * 0.72, needle), s);
    c.dot(ctr, 1.6, c.color);
}
