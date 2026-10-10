//! The colorbar controls driven headlessly: shown only while hovered and
//! never during export; the grip drags and resets the panel, the flip turns
//! it.

use super::controls;
use super::show_colorbar_overlay;
use crate::app::OctantApp;
use crate::app::layers::{BarOrientation, LayerId};
use crate::ui::test_input::Harness;
use egui::epaint::ClippedShape;
use egui::{Rect, Ui, Vec2, pos2};

const SCREEN: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1200.0, 900.0));

fn app() -> OctantApp {
    let mut app = OctantApp::default();
    app.layout.show_colorbar = true;
    app
}

/// One frame of `app`'s colorbars.
fn run(app: &mut OctantApp) -> impl FnMut(&mut Ui) + '_ {
    |ui| show_colorbar_overlay(app, ui.ctx(), SCREEN)
}

/// A harness that has laid the colorbar out over a few frames.
fn harness(app: &mut OctantApp) -> Harness {
    let mut h = Harness::new(SCREEN);
    h.settle(3, &mut run(app));
    h
}

/// The base layer's colorbar panel.
fn panel(h: &Harness) -> Rect {
    h.area(egui::Id::new(("octant_colorbar_overlay", LayerId::BASE)))
}

/// How many circles `shapes` holds (the grip's dots).
fn circles(shapes: &[ClippedShape]) -> usize {
    shapes
        .iter()
        .filter(|c| matches!(c.shape, egui::Shape::Circle(_)))
        .count()
}

#[test]
fn controls_show_only_while_the_panel_is_hovered() {
    let mut app = app();
    let mut h = harness(&mut app);
    let away = h.pointer(pos2(5.0, 5.0), None, &mut run(&mut app));
    assert_eq!(circles(&away), 0);
    let inside = h.pointer(panel(&h).center(), None, &mut run(&mut app));
    assert!(circles(&inside) >= 6);
}

#[test]
fn controls_hide_during_export() {
    let mut app = app();
    let mut h = harness(&mut app);
    app.pending_export = Some(crate::export::PendingExportRequest {
        format: Default::default(),
        target: Default::default(),
        roi: Default::default(),
        jpeg_quality: 90,
        copy_to_clipboard: false,
        output_path: None,
        canvas_rect_in_points: Rect::NOTHING,
        pixels_per_point: 1.0,
    });
    let inside = h.pointer(panel(&h).center(), None, &mut run(&mut app));
    assert_eq!(circles(&inside), 0);
}

#[test]
fn dragging_the_grip_moves_the_panel_and_double_click_resets_it() {
    let mut app = app();
    let mut h = harness(&mut app);
    let before = panel(&h);
    let [grip, _] = controls::rects(before);
    let delta = Vec2::new(-200.0, -300.0);
    h.drag(grip.center(), delta, &mut run(&mut app));

    assert!(app.layers.base.colorbar.pos.is_some());
    let moved = panel(&h).center_bottom() - before.center_bottom();
    assert!((moved - delta).length() < 12.0, "moved by {moved:?}");

    let [grip, _] = controls::rects(panel(&h));
    h.click(grip.center(), &mut run(&mut app));
    h.click(grip.center(), &mut run(&mut app));
    assert_eq!(app.layers.base.colorbar.pos, None);
}

#[test]
fn the_flip_turns_the_bar() {
    let mut app = app();
    let mut h = harness(&mut app);
    let [_, flip] = controls::rects(panel(&h));
    h.click(flip.center(), &mut run(&mut app));
    assert_eq!(
        app.layers.base.colorbar.orientation,
        BarOrientation::Vertical
    );
    h.settle(3, &mut run(&mut app));
    let panel = panel(&h);
    assert!(panel.height() > panel.width(), "vertical panel: {panel:?}");
    assert!(SCREEN.contains_rect(panel));
}
