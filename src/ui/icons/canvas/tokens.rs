//! Shared keylines, stroke weights and shade steps.

/// Shared keylines, in grid units. Icons sit inside one of these frames so
/// they read at the same optical size side by side.
pub mod key {
    /// Grid center.
    pub const C: f32 = 12.0;
    /// Radius of round icons (20-unit circle).
    pub const CIRCLE_R: f32 = 10.0;
    /// Square icons span 3..21 on both axes.
    pub const SQ_MIN: f32 = 3.0;
    pub const SQ_MAX: f32 = 21.0;
}

/// Stroke weight relative to the size step's base weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
    /// Secondary detail lines.
    Detail,
    /// Outlines.
    Base,
    /// Single-glyph marks (check, cross, chevron) that need extra presence.
    Bold,
}

/// Opacity steps of the icon color for faces, details and secondary lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shade {
    Faint,
    Soft,
    Mid,
    Strong,
}

impl Shade {
    pub(super) const fn alpha(self) -> f32 {
        match self {
            Shade::Faint => 0.16,
            Shade::Soft => 0.30,
            Shade::Mid => 0.50,
            Shade::Strong => 0.70,
        }
    }
}
