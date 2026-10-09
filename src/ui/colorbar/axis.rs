//! A bar's geometry in either orientation: data position `t` (0 at the low
//! end) along the bar, the quads and lines across it, and where ticks and
//! labels go on its outer edge (below a horizontal bar, right of a vertical
//! one). A vertical bar's high end is at the top.

use crate::app::layers::BarOrientation;
use egui::{Align2, Pos2, Rect};

#[derive(Clone, Copy)]
pub struct BarAxis {
    pub rect: Rect,
    pub orientation: BarOrientation,
}

impl BarAxis {
    pub fn new(rect: Rect, orientation: BarOrientation) -> Self {
        Self { rect, orientation }
    }

    fn vertical(&self) -> bool {
        self.orientation == BarOrientation::Vertical
    }

    /// The coordinate along the bar at data position `t`.
    fn along(&self, t: f32) -> f32 {
        let r = self.rect;
        if self.vertical() {
            r.max.y - t * r.height()
        } else {
            r.min.x + t * r.width()
        }
    }

    /// The point at data position `t`, `offset` past the outer edge (inward
    /// when negative).
    pub fn edge(&self, t: f32, offset: f32) -> Pos2 {
        let r = self.rect;
        if self.vertical() {
            Pos2::new(r.max.x + offset, self.along(t))
        } else {
            Pos2::new(self.along(t), r.max.y + offset)
        }
    }

    /// The line across the bar at data position `t`.
    pub fn across(&self, t: f32) -> [Pos2; 2] {
        let r = self.rect;
        let a = self.along(t);
        if self.vertical() {
            [Pos2::new(r.min.x, a), Pos2::new(r.max.x, a)]
        } else {
            [Pos2::new(a, r.min.y), Pos2::new(a, r.max.y)]
        }
    }

    /// The quad spanning the bar from `t0` to `t1`: the corners at `t0`, then
    /// those at `t1`.
    pub fn quad(&self, t0: f32, t1: f32) -> [Pos2; 4] {
        let [a, b] = self.across(t0);
        let [c, d] = self.across(t1);
        [a, b, c, d]
    }

    /// Where the label of a tick at `t` sits, `gap` past the outer edge.
    pub fn label(&self, t: f32, gap: f32) -> (Pos2, Align2) {
        let align = if self.vertical() {
            Align2::LEFT_CENTER
        } else {
            Align2::CENTER_TOP
        };
        (self.edge(t, gap), align)
    }

    /// The data position under `pos`, clamped onto the bar.
    pub fn t_at(&self, pos: Pos2) -> f32 {
        let r = self.rect;
        let t = if self.vertical() {
            (r.max.y - pos.y) / r.height()
        } else {
            (pos.x - r.min.x) / r.width()
        };
        t.clamp(0.0, 1.0)
    }
}
