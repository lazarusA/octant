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
