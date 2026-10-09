//! Slot anchors, dragged positions as canvas fractions, and bar geometry in
//! both orientations.

use super::axis::BarAxis;
use super::layout::{self, EDGE_GAP, PANEL_STEP};
use crate::app::layers::{BarOrientation, ColorbarPlacement, Slot};
use crate::ui::drag_grip;
use egui::{Align2, Pos2, Rect, pos2};

const CANVAS: Rect = Rect::from_min_max(pos2(100.0, 50.0), pos2(1100.0, 850.0));

#[test]
fn slots_anchor_on_their_canvas_edge() {
    let at = |slot| layout::anchor(slot, CANVAS);
    let bottom = CANVAS.bottom() - EDGE_GAP;
    assert_eq!(
        at(Slot::Bottom(0)),
        (pos2(600.0, bottom), Align2::CENTER_BOTTOM)
    );
    let stacked = pos2(600.0, bottom - PANEL_STEP);
    assert_eq!(at(Slot::Bottom(1)), (stacked, Align2::CENTER_BOTTOM));
    let top = pos2(600.0, CANVAS.top() + EDGE_GAP);
    assert_eq!(at(Slot::Top), (top, Align2::CENTER_TOP));
    let right = pos2(CANVAS.right() - EDGE_GAP, 450.0);
    assert_eq!(at(Slot::Right), (right, Align2::RIGHT_CENTER));
    let left = pos2(CANVAS.left() + EDGE_GAP, 450.0);
    assert_eq!(at(Slot::Left), (left, Align2::LEFT_CENTER));
}

#[test]
fn dragged_panels_keep_their_place_as_a_canvas_fraction() {
    let mut placement = ColorbarPlacement::at(Slot::Right);
    let point = pos2(350.0, 250.0);
    placement.pos = Some(drag_grip::to_fraction(point, CANVAS));
    let (at, pivot) = layout::position(&placement, CANVAS);
    assert!((at - point).length() < 1e-3);
    assert_eq!(
        pivot,
        Align2::RIGHT_CENTER,
        "a dragged panel keeps its pivot"
    );

    let resized = Rect::from_min_max(CANVAS.min, pos2(2100.0, 1650.0));
    let (at, _) = layout::position(&placement, resized);
    assert!((at - pos2(600.0, 450.0)).length() < 1e-3);

    let outside = drag_grip::to_fraction(pos2(-500.0, 5000.0), CANVAS);
    assert_eq!(outside, Pos2::new(0.0, 1.0), "fractions stay on the canvas");
}

#[test]
fn vertical_bars_fit_small_canvases() {
    let small = Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 250.0));
    let width = layout::panel_width(BarOrientation::Vertical, small);
    let len = layout::bar_length(BarOrientation::Vertical, width, small);
    assert!((80.0..=220.0).contains(&len));
    let tall = layout::bar_length(BarOrientation::Vertical, width, CANVAS);
    assert_eq!(tall, 220.0);
}

#[test]
fn horizontal_axis_runs_left_to_right_with_labels_below() {
    let rect = Rect::from_min_max(pos2(10.0, 20.0), pos2(110.0, 33.0));
    let axis = BarAxis::new(rect, BarOrientation::Horizontal);
    assert_eq!(axis.edge(0.25, 2.0), pos2(35.0, 35.0));
    assert_eq!(axis.across(1.0), [pos2(110.0, 20.0), pos2(110.0, 33.0)]);
    assert_eq!(axis.label(0.5, 4.0), (pos2(60.0, 37.0), Align2::CENTER_TOP));
    assert_eq!(axis.t_at(pos2(85.0, 0.0)), 0.75);
    assert_eq!(axis.t_at(pos2(500.0, 0.0)), 1.0);
}

#[test]
fn vertical_axis_has_its_high_end_on_top_and_labels_right() {
    let rect = Rect::from_min_max(pos2(10.0, 20.0), pos2(23.0, 220.0));
    let axis = BarAxis::new(rect, BarOrientation::Vertical);
    assert_eq!(axis.edge(0.0, 0.0), pos2(23.0, 220.0));
    assert_eq!(axis.edge(1.0, 3.0), pos2(26.0, 20.0));
    assert_eq!(axis.across(0.5), [pos2(10.0, 120.0), pos2(23.0, 120.0)]);
    assert_eq!(
        axis.label(0.5, 4.0),
        (pos2(27.0, 120.0), Align2::LEFT_CENTER)
    );
    assert_eq!(
        axis.t_at(pos2(0.0, 70.0)),
        0.75,
        "hovering near the top reads high values"
    );
    assert_eq!(axis.t_at(pos2(0.0, 900.0)), 0.0);
    let [a, b, c, d] = axis.quad(0.0, 0.5);
    assert_eq!([a.y, b.y, c.y, d.y], [220.0, 220.0, 120.0, 120.0]);
}
