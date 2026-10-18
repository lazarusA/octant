//! Canvas keys, headless: arrows pan as a drag of the same length would,
//! orbit 3D plots, zoom and reset; the probe and the crop box take the
//! arrows while shown; nothing moves while another widget holds focus.

use super::{canvas_id, keys::handle_canvas_keys};
use crate::app::OctantApp;
use crate::app::controllers::controller_for;
use crate::plots::PlotType;
use crate::ui::hover::probe;
use crate::ui::test_input::Harness;
use egui::{Key, Modifiers, Rect, Vec2, pos2, vec2};

const CANVAS: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));

fn run(app: &mut OctantApp, plot_type: PlotType) -> impl FnMut(&mut egui::Ui) + '_ {
    move |ui| {
        let response = ui.interact(CANVAS, canvas_id(), egui::Sense::drag());
        handle_canvas_keys(app, ui, &response, CANVAS, plot_type);
    }
}

fn press(app: &mut OctantApp, plot_type: PlotType, keys: &[(Key, Modifiers)]) {
    let mut h = Harness::new(CANVAS);
    for &(key, modifiers) in keys {
        h.key(key, modifiers, &mut run(app, plot_type));
    }
}

const NONE: Modifiers = Modifiers::NONE;

#[test]
fn arrows_pan_the_heatmap_as_a_drag_would() {
    let mut app = OctantApp::default();
    press(&mut app, PlotType::Heatmap, &[(Key::ArrowRight, NONE)]);
    let mut dragged = OctantApp::default().nav;
    controller_for(PlotType::Heatmap).handle_pan(&mut dragged, vec2(-40.0, 0.0), CANVAS);
    assert_eq!(app.nav.heatmap_pan, dragged.heatmap_pan);
    assert!(app.nav.heatmap_pan.x < 0.0, "the view moves right");

    let mut fast = OctantApp::default();
    press(
        &mut fast,
        PlotType::Heatmap,
        &[(Key::ArrowRight, Modifiers::SHIFT)],
    );
    assert!(
        fast.nav.heatmap_pan.x < app.nav.heatmap_pan.x * 2.0,
        "Shift pans further"
    );
}

#[test]
fn plus_and_minus_zoom_and_zero_resets() {
    let mut app = OctantApp::default();
    let start = app.nav.heatmap_zoom;
    press(
        &mut app,
        PlotType::Heatmap,
        &[(Key::Plus, Modifiers::SHIFT)],
    );
    assert!(app.nav.heatmap_zoom > start);
    press(
        &mut app,
        PlotType::Heatmap,
        &[(Key::Minus, NONE), (Key::Minus, NONE)],
    );
    assert!(app.nav.heatmap_zoom < start);
    press(
        &mut app,
        PlotType::Heatmap,
        &[(Key::ArrowUp, NONE), (Key::Num0, NONE)],
    );
    assert_eq!(app.nav.heatmap_zoom, start);
    assert_eq!(app.nav.heatmap_pan, Vec2::ZERO);
}

#[test]
fn arrows_orbit_3d_plots_and_shift_pans_them() {
    let start = OctantApp::default().nav;
    let mut app = OctantApp::default();
    press(&mut app, PlotType::Surface, &[(Key::ArrowLeft, NONE)]);
    assert_ne!(app.nav.sphere_rotation_y, start.sphere_rotation_y);
    assert_eq!(app.nav.orbit_target, start.orbit_target);

    let mut panned = OctantApp::default();
    press(
        &mut panned,
        PlotType::Surface,
        &[(Key::ArrowLeft, Modifiers::SHIFT)],
    );
    assert_eq!(panned.nav.sphere_rotation_y, start.sphere_rotation_y);
    assert_ne!(panned.nav.orbit_target, start.orbit_target);
}

#[test]
fn the_probe_takes_the_arrows_until_escape() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(CANVAS);
    let mut run = run(&mut app, PlotType::Heatmap);
    h.key(Key::Enter, NONE, &mut run);
    assert_eq!(probe::position(&h.ctx), Some(CANVAS.center()));
    h.key(Key::ArrowRight, NONE, &mut run);
    h.key(Key::ArrowDown, NONE, &mut run);
    let moved = probe::position(&h.ctx).expect("the probe is on");
    assert!(moved.x > CANVAS.center().x && moved.y > CANVAS.center().y);
    h.key(Key::Escape, NONE, &mut run);
    assert_eq!(probe::position(&h.ctx), None);
    drop(run);
    assert_eq!(app.nav.heatmap_pan, Vec2::ZERO, "the view stayed put");
}

#[test]
fn arrows_move_the_crop_box_and_shift_resizes_it() {
    let mut app = OctantApp {
        show_crop_overlay: true,
        ..Default::default()
    };
    let start = app.roi_crop_box;
    press(&mut app, PlotType::Heatmap, &[(Key::ArrowRight, NONE)]);
    let moved = app.roi_crop_box;
    assert!(moved.u_min > start.u_min);
    assert!((moved.u_max - moved.u_min - (start.u_max - start.u_min)).abs() < 1e-5);
    press(
        &mut app,
        PlotType::Heatmap,
        &[(Key::ArrowRight, Modifiers::SHIFT)],
    );
    assert_eq!(app.roi_crop_box.u_min, moved.u_min);
    assert!(app.roi_crop_box.u_max > moved.u_max);
    assert_eq!(app.nav.heatmap_pan, Vec2::ZERO);
}

#[test]
fn keys_stay_with_another_focused_widget() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(CANVAS);
    let field = egui::Id::unique("another widget");
    let mut run = |ui: &mut egui::Ui| {
        ui.interact(
            Rect::from_min_size(pos2(5.0, 5.0), vec2(20.0, 20.0)),
            field,
            egui::Sense::click(),
        );
        let response = ui.interact(CANVAS, canvas_id(), egui::Sense::drag());
        handle_canvas_keys(&mut app, ui, &response, CANVAS, PlotType::Heatmap);
    };
    h.settle(1, &mut run);
    h.ctx.memory_mut(|m| m.request_focus(field));
    h.key(Key::ArrowRight, NONE, &mut run);
    assert_eq!(app.nav.heatmap_pan, Vec2::ZERO);
}
