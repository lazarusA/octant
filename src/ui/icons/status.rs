//! Tool and status icons: scissors, padlocks, bolt and hourglass.
//! Single-glyph marks (check, cross, warning, ...) live in `marks.rs`.

use super::canvas::{IconCanvas, Shade, Weight};
use egui::Color32;
use std::f32::consts::PI;

/// Scissors: two finger rings, crossing blades and a pivot dot (18x18 keyline).
pub fn draw_scissors(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    c.circle((7.0, 17.5), 2.8, Color32::TRANSPARENT, s);
    c.circle((17.0, 17.5), 2.8, Color32::TRANSPARENT, s);
    c.seg(c.p(8.8, 15.3), c.p(18.0, 4.0), s);
    c.seg(c.p(15.2, 15.3), c.p(6.0, 4.0), s);
    c.dot((12.0, 11.6), 1.1, c.color);
}

/// Shared padlock: body, keyhole and a shackle whose arc is centered at
/// `arc_y` with its right leg ending at `right_end`.
fn padlock(c: &IconCanvas, arc_y: f32, right_end: f32) {
    let s = c.stroke(Weight::Base);
    // Snap the legs first and build the arc between them in screen space so
    // the vertical legs stay pixel-crisp and meet the arc without a kink.
    let (left, right) = (c.ps(8.0, 11.0, s.width), c.ps(16.0, right_end, s.width));
    let top = c.ps(12.0, arc_y, s.width);
    let (cx, r) = ((left.x + right.x) * 0.5, (right.x - left.x) * 0.5);
    let mut shackle = vec![left];
    shackle.extend((0..=16).map(|i| {
        let a = PI + PI * i as f32 / 16.0;
        egui::pos2(cx + r * a.cos(), top.y + r * a.sin())
    }));
    shackle.push(right);
    c.curve(shackle, s);

    c.rrect((5.0, 11.0), (19.0, 21.0), 2.0, c.body(), s);
    if c.compact() {
        c.dot((12.0, 16.0), 1.6, c.color);
    } else {
        c.dot((12.0, 15.0), 1.4, c.color);
        c.line((12.0, 15.0), (12.0, 18.0), s);
    }
}

/// Lock: closed padlock (14x18 keyline).
pub fn draw_lock(c: &IconCanvas) {
    padlock(c, 7.5, 11.0);
}

/// Unlock: padlock with the shackle lifted and its right leg free (14x18 keyline).
pub fn draw_unlock(c: &IconCanvas) {
    padlock(c, 5.0, 7.0);
}

/// Bolt: solid lightning bolt, filled as two overlapping convex halves (14x19 keyline).
pub fn draw_bolt(c: &IconCanvas) {
    // The bolt is concave at its two inner corners. Split it along the seam
    // between them and push the upper half 0.35 units across the seam so the
    // halves overlap instead of leaving an anti-aliased hairline.
    let (seam_lo, seam_hi) = ((11.5, 13.0), (12.5, 10.5));
    let push = (0.33, 0.13);
    let over = |p: (f32, f32)| (p.0 + push.0, p.1 + push.1);
    c.fill(
        &[(14.5, 2.5), (5.5, 13.0), over(seam_lo), over(seam_hi)],
        c.color,
    );
    c.fill(&[seam_hi, (18.5, 10.5), (9.5, 21.5), seam_lo], c.color);
}

/// Hourglass glass profile in grid units: plates at `GLASS_TOP` and
/// `GLASS_BOTTOM`, half widths `GLASS_OUTER` at the plates and `GLASS_INNER`
/// along the flat waist.
const GLASS_TOP: f32 = 4.0;
const GLASS_BOTTOM: f32 = 20.0;
const WAIST_TOP: f32 = 11.25;
const WAIST_BOT: f32 = 12.75;
const GLASS_OUTER: f32 = 5.5;
const GLASS_INNER: f32 = 0.75;

/// Half width of the hourglass glass at height `y`.
fn glass_half(y: f32) -> f32 {
    let slope = (GLASS_OUTER - GLASS_INNER) / (WAIST_TOP - GLASS_TOP);
    if y <= WAIST_TOP {
        GLASS_OUTER - (y - GLASS_TOP) * slope
    } else if y >= WAIST_BOT {
        GLASS_INNER + (y - WAIST_BOT) * slope
    } else {
        GLASS_INNER
    }
}

/// Hourglass: sand timer with a flat waist and sand settling in both chambers (14x16 keyline).
pub fn draw_hourglass(c: &IconCanvas) {
    // Glass fill in three bands: upper chamber, waist, lower chamber.
    let bands = [
        (GLASS_TOP, WAIST_TOP),
        (WAIST_TOP, WAIST_BOT),
        (WAIST_BOT, GLASS_BOTTOM),
    ];
    for (y0, y1) in bands {
        let (h0, h1) = (glass_half(y0), glass_half(y1));
        let quad = [
            (12.0 - h0, y0),
            (12.0 + h0, y0),
            (12.0 + h1, y1),
            (12.0 - h1, y1),
        ];
        c.fill(&quad, c.body());
    }
    draw_sand(c);

    // Glass sides as two open strokes so nothing doubles up at the waist.
    let s = c.stroke(Weight::Base);
    for side in [-1.0_f32, 1.0] {
        let x = |dx: f32| 12.0 + side * dx;
        let profile = [
            (x(GLASS_OUTER), GLASS_TOP),
            (x(GLASS_INNER), WAIST_TOP),
            (x(GLASS_INNER), WAIST_BOT),
            (x(GLASS_OUTER), GLASS_BOTTOM),
        ];
        c.path(&profile, s);
    }

    // End plates, only slightly wider than the glass.
    c.line((5.0, GLASS_TOP), (19.0, GLASS_TOP), s);
    c.line((5.0, GLASS_BOTTOM), (19.0, GLASS_BOTTOM), s);
}

/// Sand inset one unit inside the glass: a remnant above the waist (full size
/// only) and a mounded pile below it.
fn draw_sand(c: &IconCanvas) {
    let at = |y: f32, side: f32| (12.0 + side * (glass_half(y) - 1.0), y);
    let sand = c.shade(Shade::Mid);
    if !c.compact() {
        c.fill(&[at(8.0, -1.0), at(8.0, 1.0), (12.0, 10.5)], sand);
    }
    let pile = [
        at(17.0, -1.0),
        (12.0, 15.5),
        at(17.0, 1.0),
        at(19.0, 1.0),
        at(19.0, -1.0),
    ];
    c.fill(&pile, sand);
}
