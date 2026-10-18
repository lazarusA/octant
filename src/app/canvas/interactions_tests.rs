//! Canvas gestures, headless: two-finger pinches zoom and pan every plot
//! type without the first finger's drag turning or panning on its own, and
//! secondary button drags pan the 3D plots.

use super::handle_canvas_interactions;
use crate::app::OctantApp;
use crate::plots::PlotType;
use crate::ui::test_input::Harness;
use egui::{Event, PointerButton, Pos2, Rect, Vec2, pos2, vec2};

const CANVAS: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(800.0, 600.0));

fn run(app: &mut OctantApp, plot_type: PlotType) -> impl FnMut(&mut egui::Ui) + '_ {
    move |ui| {
        let response = ui.allocate_rect(CANVAS, egui::Sense::drag());
        handle_canvas_interactions(app, ui, &response, CANVAS, plot_type);
    }
}

fn pinch(plot_type: PlotType, spread: (f32, f32), shift: Vec2) -> OctantApp {
    let mut app = OctantApp::default();
    let mut h = Harness::new(CANVAS);
    h.pinch(
        CANVAS.center(),
        spread,
        shift,
        &mut run(&mut app, plot_type),
    );
    app
}

#[test]
fn pinch_zooms_the_heatmap_about_the_fingers() {
    let app = pinch(PlotType::Heatmap, (100.0, 300.0), Vec2::ZERO);
    assert!(app.nav.heatmap_zoom > 2.5, "zoom {}", app.nav.heatmap_zoom);
    // Spread about the canvas center: nothing pans, whatever the first
    // finger's pointer did.
    assert!(
        app.nav.heatmap_pan.length() < 1.0,
        "{:?}",
        app.nav.heatmap_pan
    );
}

#[test]
fn two_fingers_pan_the_line_plot() {
    let app = pinch(PlotType::Line, (200.0, 200.0), vec2(60.0, -30.0));
    // egui reports a gesture's motion from its second move on: the pan
    // follows the fingers, short of their first of ten steps.
    let followed = app.nav.line_pan / vec2(60.0, -30.0);
    assert!(
        (0.85..=1.0).contains(&followed.x) && (followed.x - followed.y).abs() < 1e-3,
        "{:?}",
        app.nav.line_pan
    );
    assert!((app.nav.line_zoom - 1.0).abs() < 1e-3);
}

#[test]
fn pinch_zooms_3d_plots_without_turning_them() {
    for plot_type in [
        PlotType::Surface,
        PlotType::Volume,
        PlotType::PointCloud,
        PlotType::Sphere,
    ] {
        let start = OctantApp::default().nav;
        let nav = pinch(plot_type, (100.0, 300.0), Vec2::ZERO).nav;
        assert!(nav.sphere_zoom < start.sphere_zoom, "{plot_type:?} zoom");
        assert_eq!(
            nav.sphere_rotation_x, start.sphere_rotation_x,
            "{plot_type:?}"
        );
        assert_eq!(
            nav.sphere_rotation_y, start.sphere_rotation_y,
            "{plot_type:?}"
        );
    }
}

#[test]
fn two_fingers_pan_3d_plots() {
    for plot_type in [PlotType::Surface, PlotType::Volume, PlotType::PointCloud] {
        let nav = pinch(plot_type, (200.0, 200.0), vec2(80.0, 0.0)).nav;
        assert!(
            nav.orbit_target[0] < -0.05,
            "{plot_type:?} {:?}",
            nav.orbit_target
        );
        assert_eq!(nav.sphere_rotation_y, 0.0, "{plot_type:?}");
    }
    // The sphere keeps its center and turns under the fingers instead.
    let nav = pinch(PlotType::Sphere, (200.0, 200.0), vec2(80.0, 0.0)).nav;
    assert_eq!(nav.orbit_target, [0.0; 3]);
    assert!(nav.sphere_rotation_y > 0.0);
}

#[test]
fn secondary_drag_pans_the_surface() {
    let mut app = OctantApp::default();
    let mut h = Harness::new(CANVAS);
    let from = CANVAS.center();
    let button = |pos: Pos2, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Secondary,
        pressed,
        modifiers: Default::default(),
    };
    let mut frame = |events| h.frame(events, &mut run(&mut app, PlotType::Surface));
    frame(vec![Event::PointerMoved(from)]);
    frame(vec![button(from, true)]);
    for step in 1..=10 {
        frame(vec![Event::PointerMoved(
            from + vec2(6.0 * step as f32, 0.0),
        )]);
    }
    frame(vec![button(from + vec2(60.0, 0.0), false)]);
    assert!(
        app.nav.orbit_target[0] < -0.05,
        "{:?}",
        app.nav.orbit_target
    );
    assert_eq!(app.nav.sphere_rotation_y, 0.0);
}
