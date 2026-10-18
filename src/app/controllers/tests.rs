//! Unit tests for PlotController implementations and dispatching.

use super::*;
use crate::app::state::NavigationState;
use crate::plots::PlotType;

#[test]
fn test_controller_dispatch_matches_plot_type() {
    let types = [
        PlotType::Heatmap,
        PlotType::Line,
        PlotType::Surface,
        PlotType::Volume,
        PlotType::Sphere,
        PlotType::PointCloud,
    ];

    for pt in types {
        let ctrl = controller_for(pt);
        assert_eq!(ctrl.plot_type(), pt);
        assert_eq!(ctrl.is_3d(), pt.is_3d());
        assert_eq!(ctrl.draws_coastlines(), pt.draws_coastlines());
        assert_eq!(ctrl.draws_categories(), pt.draws_categories());
    }
}

#[test]
fn test_heatmap_reset_and_drag() {
    let ctrl = controller_for(PlotType::Heatmap);
    let mut nav = NavigationState::default();
    ctrl.handle_drag(&mut nav, egui::vec2(10.0, -5.0));
    assert_eq!(nav.heatmap_pan, egui::vec2(10.0, -5.0));

    ctrl.reset_view(&mut nav);
    assert_eq!(nav.heatmap_pan, egui::vec2(0.0, 0.0));
    assert_eq!(nav.heatmap_zoom, 1.0);
}

#[test]
fn test_line_reset_and_drag() {
    let ctrl = controller_for(PlotType::Line);
    let mut nav = NavigationState::default();
    ctrl.handle_drag(&mut nav, egui::vec2(25.0, 15.0));
    assert_eq!(nav.line_pan, egui::vec2(25.0, 15.0));

    ctrl.reset_view(&mut nav);
    assert_eq!(nav.line_pan, egui::vec2(0.0, 0.0));
    assert_eq!(nav.line_zoom, 1.0);
}

#[test]
fn test_3d_rotation_clamp() {
    let ctrl = controller_for(PlotType::Volume);
    let mut nav = NavigationState::default();
    // Huge pitch drag that would exceed PI/2
    ctrl.handle_drag(&mut nav, egui::vec2(0.0, 1000.0));
    let max_pitch = std::f32::consts::FRAC_PI_2 - 0.05;
    assert!(nav.sphere_rotation_x <= max_pitch + 1e-4);

    ctrl.reset_view(&mut nav);
    assert_eq!(nav.sphere_rotation_x, 0.25);
    assert_eq!(nav.sphere_rotation_y, 0.0);
}

const RECT: egui::Rect = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(800.0, 600.0));

/// Zooming the heatmap about a point keeps the content under it in place.
#[test]
fn test_heatmap_zoom_keeps_anchor() {
    let ctrl = controller_for(PlotType::Heatmap);
    let mut nav = NavigationState {
        heatmap_pan: egui::vec2(30.0, -20.0),
        ..Default::default()
    };
    let anchor = egui::pos2(600.0, 150.0);
    let content =
        |nav: &NavigationState| (anchor - RECT.center() - nav.heatmap_pan) / nav.heatmap_zoom;
    let before = content(&nav);
    ctrl.handle_zoom(&mut nav, 1.7, anchor, RECT);
    assert!((nav.heatmap_zoom - 1.7).abs() < 1e-5);
    assert!((content(&nav) - before).length() < 1e-3);
}

/// Zooming a 3D plot about a point keeps the point under it, at the
/// target's depth, in place, and pulls the target toward it.
#[test]
fn test_3d_zoom_keeps_anchor() {
    let ctrl = controller_for(PlotType::Surface);
    let mut nav = NavigationState::default();
    let aspect = RECT.width() / RECT.height();
    let anchor = egui::pos2(620.0, 180.0);
    let clip = [
        (anchor.x - RECT.center().x) / (RECT.width() / 2.0),
        -(anchor.y - RECT.center().y) / (RECT.height() / 2.0),
    ];
    let before = nav.orbit_camera(PlotType::Surface, aspect);
    // At the target's depth, under the anchor.
    let from_center = anchor - RECT.center();
    let offset = before.screen_offset([from_center.x, from_center.y], RECT.height());
    let point = add(before.target, offset);
    ctrl.handle_zoom(&mut nav, 2.0, anchor, RECT);
    let after = nav.orbit_camera(PlotType::Surface, aspect);
    assert!((after.distance - before.distance / 2.0).abs() < 1e-5);
    assert_ne!(nav.orbit_target, [0.0; 3]);
    let moved = after.project(point).expect("visible");
    assert!((moved[0] - clip[0]).abs() < 1e-3 && (moved[1] - clip[1]).abs() < 1e-3);
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[test]
fn test_3d_zoom_and_pan_clamp_and_reset() {
    let ctrl = controller_for(PlotType::Volume);
    let mut nav = NavigationState::default();
    for _ in 0..200 {
        ctrl.handle_zoom(&mut nav, 1.5, RECT.center(), RECT);
    }
    assert_eq!(nav.sphere_zoom, ctrl.min_zoom());
    for _ in 0..200 {
        ctrl.handle_zoom(&mut nav, 0.5, RECT.center(), RECT);
    }
    assert_eq!(nav.sphere_zoom, crate::plots::camera::MAX_DISTANCE);
    for _ in 0..200 {
        ctrl.handle_pan(&mut nav, egui::vec2(-500.0, 0.0), RECT);
    }
    let limit = crate::plots::camera::TARGET_LIMIT;
    assert!(nav.orbit_target.iter().all(|t| t.abs() <= limit));
    assert!(nav.orbit_target.iter().any(|t| t.abs() > 1.0));

    ctrl.reset_view(&mut nav);
    assert_eq!(nav.orbit_target, [0.0; 3]);
    assert_eq!(nav.sphere_zoom, 2.5);
}

/// The sphere zooms by its height above the globe and turns slower as the
/// camera comes closer; it never pans off its center.
#[test]
fn test_sphere_zooms_by_height() {
    let ctrl = controller_for(PlotType::Sphere);
    let mut nav = NavigationState::default();
    ctrl.handle_zoom(&mut nav, 2.0, egui::pos2(700.0, 100.0), RECT);
    assert!((nav.sphere_zoom - 1.75).abs() < 1e-5, "{}", nav.sphere_zoom);
    assert_eq!(nav.orbit_target, [0.0; 3]);

    let turn = |nav: &NavigationState| {
        let mut nav = nav.clone();
        ctrl.handle_pan(&mut nav, egui::vec2(10.0, 0.0), RECT);
        nav.sphere_rotation_y
    };
    let far = turn(&NavigationState::default());
    let near = turn(&nav);
    assert!(near > 0.0 && near < far, "{near} {far}");
    for _ in 0..100 {
        ctrl.handle_zoom(&mut nav, 2.0, RECT.center(), RECT);
    }
    assert_eq!(nav.sphere_zoom, ctrl.min_zoom());
}
