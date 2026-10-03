//! Octant brand marks: shared cube face shading and the block wordmark.
//!
//! The hero cube and the wordmark draw their solid faces from the same
//! shade steps so the logo and the name read as one lockup.

#[cfg(test)]
mod tests;
mod wordmark;

pub use wordmark::Wordmark;

use egui::{Color32, Visuals};

/// Brightness of the three visible faces of a solid cube: front/top, right
/// side and bottom/left side.
pub const FACE_SHADES: [f32; 3] = [1.0, 0.72, 0.52];

/// Scale the RGB channels of `c` by `f`, keeping alpha.
pub fn darken(c: Color32, f: f32) -> Color32 {
    let ch = |v: u8| (f32::from(v) * f).round() as u8;
    Color32::from_rgba_unmultiplied(ch(c.r()), ch(c.g()), ch(c.b()), c.a())
}

/// Face colors for a block in `base`, from [`FACE_SHADES`].
///
/// Dark theme darkens the side faces like the hero cube. Light theme blends
/// them toward the panel instead, since darkening a near-black base would
/// leave all three faces the same.
pub fn face_colors(visuals: &Visuals, base: Color32) -> [Color32; 3] {
    if visuals.dark_mode {
        return FACE_SHADES.map(|f| darken(base, f));
    }
    let panel = visuals.panel_fill;
    FACE_SHADES.map(|f| {
        let t = ((1.0 - f) * 1.4).min(1.0);
        let mix = |a: u8, b: u8| (f32::from(a) * (1.0 - t) + f32::from(b) * t).round() as u8;
        Color32::from_rgb(
            mix(base.r(), panel.r()),
            mix(base.g(), panel.g()),
            mix(base.b(), panel.b()),
        )
    })
}
