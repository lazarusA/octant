//! 3D Volume raymarching plot controller.

use super::traits::PlotController;
use crate::app::state::NavigationState;
use crate::plots::PlotType;

/// Controller for volumetric DVR and MIP raymarching plots.
pub struct VolumeController;

impl PlotController for VolumeController {
    fn plot_type(&self) -> PlotType {
        PlotType::Volume
    }

    fn reset_view(&self, nav: &mut NavigationState) {
        nav.reset_3d_camera();
    }

    fn handle_drag(&self, nav: &mut NavigationState, delta: egui::Vec2) {
        nav.sphere_rotation_y += delta.x * 0.008;
        nav.sphere_rotation_x = (nav.sphere_rotation_x + delta.y * 0.008).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.05,
            std::f32::consts::FRAC_PI_2 - 0.05,
        );
    }

    fn handle_scroll(
        &self,
        nav: &mut NavigationState,
        scroll: f32,
        _mouse_pos: egui::Pos2,
        _center: egui::Pos2,
    ) {
        nav.sphere_zoom = (nav.sphere_zoom - scroll * 0.003).clamp(self.min_zoom(), 8.0);
    }
}
