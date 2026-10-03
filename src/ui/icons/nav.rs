//! Navigation, panel and theme icons.

use super::canvas::{IconCanvas, Shade, Weight, key};
use egui::Color32;
use std::f32::consts::{PI, TAU};

/// Globe: circle with an equator and one meridian ellipse (20-unit circle).
pub fn draw_globe(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    c.circle((key::C, key::C), key::CIRCLE_R, Color32::TRANSPARENT, s);
    c.line((2.0, key::C), (22.0, key::C), s);
    c.curve_closed(c.arc((key::C, key::C), (4.5, key::CIRCLE_R), 0.0, TAU), s);
}

/// Variables: `(x)` with elliptical parentheses (20x18 keyline).
pub fn draw_variables(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let sweep = 0.9;
    c.curve(c.arc((11.0, key::C), (7.0, 9.0), PI - sweep, PI + sweep), s);
    c.curve(c.arc((13.0, key::C), (7.0, 9.0), -sweep, sweep), s);
    c.line((8.5, 8.5), (15.5, 15.5), s);
    c.line((15.5, 8.5), (8.5, 15.5), s);
}

/// Dimensions: two slider rails with diamond knobs at different positions (16x18 keyline).
pub fn draw_dimensions(c: &IconCanvas) {
    let rail = c.stroke_toned(Weight::Base, c.shade(Shade::Mid));
    c.line((8.0, key::SQ_MIN), (8.0, key::SQ_MAX), rail);
    c.line((16.0, key::SQ_MIN), (16.0, key::SQ_MAX), rail);

    if c.compact() {
        c.diamond((8.0, 9.0), 3.0, 3.0, c.color, egui::Stroke::NONE);
        c.diamond((16.0, 15.0), 3.0, 3.0, c.color, egui::Stroke::NONE);
    } else {
        let s = c.stroke(Weight::Base);
        c.diamond((8.0, 9.0), 3.0, 3.0, c.body(), s);
        c.diamond((16.0, 15.0), 3.0, 3.0, c.shade(Shade::Soft), s);
    }
}

/// Settings: six-tooth cog with a center bore; a ring with solid teeth when compact (20-unit circle).
pub fn draw_settings(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let ctr = (key::C, key::C);
    let (teeth, r_out) = (6, key::CIRCLE_R);
    let pitch = TAU / teeth as f32;
    let half = pitch / 4.0;
    let at = |a: f32, r: f32| c.polar(ctr, r, a);
    let tooth = |a: f32, r_root: f32, root_w: f32, tip_w: f32| {
        vec![
            at(a - half * root_w, r_root),
            at(a - half * tip_w, r_out),
            at(a + half * tip_w, r_out),
            at(a + half * root_w, r_root),
        ]
    };

    if c.compact() {
        // A six-tooth outline cannot resolve at 12-14 px: use a ring with solid teeth.
        let ring = 5.5;
        c.circle(ctr, ring, Color32::TRANSPARENT, s);
        for i in 0..teeth {
            c.fill_pts(tooth(i as f32 * pitch, ring, 1.1, 0.8), c.color);
        }
        return;
    }

    // Outline: root chord into each tooth, flat tip, back to the root.
    let r_root = 7.0;
    let mut outline = Vec::with_capacity(teeth * 4);
    for i in 0..teeth {
        let pts = tooth(i as f32 * pitch, r_root, 1.25, 0.7);
        // The cog is concave, so fill each tooth and the hub separately.
        c.fill_pts(pts.clone(), c.body());
        outline.extend(pts);
    }
    c.circle(ctr, r_root, c.body(), egui::Stroke::NONE);
    c.curve_closed(outline, s);
    c.circle(ctr, 3.0, Color32::TRANSPARENT, s);
}

/// Cache: memory chip with two pins per side and a shaded die (18-unit square).
pub fn draw_cache(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    c.rrect((6.0, 6.0), (18.0, 18.0), 1.5, c.body(), s);
    for o in [10.0, 14.0] {
        c.line((o, key::SQ_MIN), (o, 6.0), s);
        c.line((o, 18.0), (o, key::SQ_MAX), s);
        c.line((key::SQ_MIN, o), (6.0, o), s);
        c.line((18.0, o), (key::SQ_MAX, o), s);
    }
    if !c.compact() {
        c.rrect_fill((9.5, 9.5), (14.5, 14.5), 0.75, c.shade(Shade::Mid));
    }
}

/// Sun: open core ring with eight rays, cardinals longer (20-unit circle).
pub fn draw_sun(c: &IconCanvas) {
    let s = c.stroke(Weight::Base);
    let ctr = (key::C, key::C);
    c.circle(ctr, 4.0, Color32::TRANSPARENT, s);
    for i in 0..8 {
        let a = i as f32 * TAU / 8.0;
        let r1 = if i % 2 == 0 { key::CIRCLE_R } else { 8.75 };
        c.seg(c.polar(ctr, 6.5, a), c.polar(ctr, r1, a), s);
    }
}

/// Moon: shaded lunar disc with craters; one crater at small sizes (20-unit circle).
pub fn draw_moon(c: &IconCanvas) {
    let ctr = (key::C, key::C);
    c.circle(
        ctr,
        key::CIRCLE_R,
        c.shade(Shade::Faint),
        c.stroke(Weight::Base),
    );
    let crater = c.shade(Shade::Soft);
    if c.compact() {
        c.circle((10.0, 10.0), 3.0, crater, egui::Stroke::NONE);
    } else {
        c.circle((9.0, 9.5), 2.6, crater, egui::Stroke::NONE);
        c.circle((15.0, 14.5), 1.9, crater, egui::Stroke::NONE);
        c.circle((8.5, 15.5), 1.2, crater, egui::Stroke::NONE);
    }
}
