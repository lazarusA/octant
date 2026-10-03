//! Shared drawing surface for every procedural icon.
//!
//! Icons are designed on a 24-unit grid. [`IconCanvas`] maps grid units to
//! screen points, snaps straight geometry to physical pixels, rounds stroke
//! widths to whole physical pixels, and provides the shared weight and shade
//! tokens so every icon draws with one visual vocabulary. Drawing primitives
//! live in `draw.rs`, tokens in `tokens.rs`, polygon helpers in `geom.rs`.

mod draw;
mod geom;
mod tokens;

pub(crate) use draw::Caps;
pub(crate) use tokens::{Shade, Weight, key};

use super::style;
use egui::{Color32, Painter, Pos2, Rect, Stroke, pos2};

/// Painter wrapper that speaks grid units.
pub(crate) struct IconCanvas<'a> {
    painter: &'a Painter,
    rect: Rect,
    ppp: f32,
    base_px: f32,
    /// Icon color supplied by the caller.
    pub color: Color32,
    pub is_dark: bool,
}

impl<'a> IconCanvas<'a> {
    pub fn new(painter: &'a Painter, rect: Rect, color: Color32, is_dark: bool) -> Self {
        let ppp = painter.pixels_per_point();
        let dim = rect.width().min(rect.height());
        Self {
            painter,
            rect,
            ppp,
            base_px: style::stroke_width(dim) * ppp,
            color,
            is_dark,
        }
    }

    /// Points per grid unit.
    #[inline]
    pub fn unit(&self) -> f32 {
        self.rect.width() / 24.0
    }

    /// True below 16 px, where icons drop secondary detail.
    #[inline]
    pub fn compact(&self) -> bool {
        self.rect.width() < 16.0
    }

    /// Unsnapped grid point.
    #[inline]
    pub fn p(&self, gx: f32, gy: f32) -> Pos2 {
        pos2(
            self.rect.min.x + gx * self.unit(),
            self.rect.min.y + gy * self.unit(),
        )
    }

    /// Stroke in the icon color at `weight`, a whole number of physical pixels wide.
    pub fn stroke(&self, weight: Weight) -> Stroke {
        self.stroke_toned(weight, self.color)
    }

    /// [`Self::stroke`] in an explicit color.
    pub fn stroke_toned(&self, weight: Weight, color: Color32) -> Stroke {
        let mult = match weight {
            Weight::Detail => 0.75,
            Weight::Base => 1.0,
            Weight::Bold => 1.35,
        };
        let px = (self.base_px * mult).round().max(1.0);
        Stroke::new(px / self.ppp, color)
    }

    /// Detail stroke in a [`Shade`] of the icon color.
    pub fn detail(&self, shade: Shade) -> Stroke {
        self.stroke_toned(Weight::Detail, self.shade(shade))
    }

    /// Icon color at a [`Shade`] opacity.
    pub fn shade(&self, shade: Shade) -> Color32 {
        self.color.gamma_multiply(shade.alpha())
    }

    /// Barely-there body fill for outlined shapes, tuned per theme.
    pub fn body(&self) -> Color32 {
        let alpha = if self.is_dark { 0.10 } else { 0.08 };
        self.color.gamma_multiply(alpha)
    }

    /// Snap a coordinate (points) so a stroke of `width` points is pixel-crisp.
    fn snap_center(&self, v: f32, width: f32) -> f32 {
        let px = v * self.ppp;
        let w = (width * self.ppp).round() as i32;
        let snapped = if w % 2 == 1 {
            px.floor() + 0.5
        } else {
            px.round()
        };
        snapped / self.ppp
    }

    /// Snap a coordinate (points) to the nearest physical pixel boundary.
    fn snap_edge(&self, v: f32) -> f32 {
        (v * self.ppp).round() / self.ppp
    }

    /// Grid point snapped for a stroke of `width` points.
    pub fn ps(&self, gx: f32, gy: f32, width: f32) -> Pos2 {
        let q = self.p(gx, gy);
        pos2(self.snap_center(q.x, width), self.snap_center(q.y, width))
    }

    /// Grid point snapped to pixel boundaries, for fill edges.
    fn pe(&self, gx: f32, gy: f32) -> Pos2 {
        let q = self.p(gx, gy);
        pos2(self.snap_edge(q.x), self.snap_edge(q.y))
    }
}
