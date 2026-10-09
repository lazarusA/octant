//! Where colorbar panels sit on the canvas: each slot's pivot and point, the
//! panel and bar sizes per orientation, and dragged positions (canvas
//! fractions, `drag_grip`).

use crate::app::layers::{BarOrientation, ColorbarPlacement, Slot};
use crate::ui::drag_grip;
use egui::{Align2, Pos2, Rect};

/// Gap between a panel and the canvas edge.
pub const EDGE_GAP: f32 = 8.0;
/// Distance between stacked bottom panels.
pub const PANEL_STEP: f32 = 84.0;
/// Horizontal margin inside a panel's frame.
pub const MARGIN_X: f32 = 12.0;
/// Width of a vertical panel.
const VERTICAL_W: f32 = 124.0;
/// Longest vertical bar; shorter on small canvases.
const VERTICAL_BAR_MAX: f32 = 220.0;
/// Shortest bar of either orientation.
const BAR_MIN: f32 = 80.0;

/// The pivot of `slot`'s panel and where it sits on `canvas`.
pub fn anchor(slot: Slot, canvas: Rect) -> (Pos2, Align2) {
    let center = canvas.center();
    match slot {
        Slot::Bottom(n) => {
            let y = canvas.bottom() - EDGE_GAP - f32::from(n) * PANEL_STEP;
            (Pos2::new(center.x, y), Align2::CENTER_BOTTOM)
        }
        Slot::Top => (
            Pos2::new(center.x, canvas.top() + EDGE_GAP),
            Align2::CENTER_TOP,
        ),
        Slot::Right => (
            Pos2::new(canvas.right() - EDGE_GAP, center.y),
            Align2::RIGHT_CENTER,
        ),
        Slot::Left => (
            Pos2::new(canvas.left() + EDGE_GAP, center.y),
            Align2::LEFT_CENTER,
        ),
    }
}

/// Where `placement`'s panel sits: its dragged position, else its slot's.
pub fn position(placement: &ColorbarPlacement, canvas: Rect) -> (Pos2, Align2) {
    let (slot_point, pivot) = anchor(placement.slot, canvas);
    let point = placement
        .pos
        .map_or(slot_point, |f| drag_grip::from_fraction(f, canvas));
    (point, pivot)
}

/// Outer width of a panel in `orientation` on `canvas`.
pub fn panel_width(orientation: BarOrientation, canvas: Rect) -> f32 {
    match orientation {
        BarOrientation::Horizontal => 490.0_f32.min((canvas.width() - 32.0).max(180.0)),
        BarOrientation::Vertical => VERTICAL_W,
    }
}

/// Length of the bar of a panel `width` wide in `orientation` on `canvas`.
pub fn bar_length(orientation: BarOrientation, width: f32, canvas: Rect) -> f32 {
    match orientation {
        BarOrientation::Horizontal => (width - 90.0).max(BAR_MIN),
        BarOrientation::Vertical => (canvas.height() - 200.0).clamp(BAR_MIN, VERTICAL_BAR_MAX),
    }
}
