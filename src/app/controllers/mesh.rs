//! 3D Mesh plot controller (Surface and Sphere).

use super::traits::PlotController;
use crate::app::state::NavigationState;
use crate::plots::PlotType;

/// Controller for 3D Mesh plots (topological Surface and global Sphere).
pub struct MeshController {
    plot_type: PlotType,
}

impl MeshController {
    /// Constructs a controller for either Surface or Sphere plot type.
    pub const fn new(plot_type: PlotType) -> Self {
        Self { plot_type }
    }
}

impl PlotController for MeshController {
    fn plot_type(&self) -> PlotType {
        self.plot_type
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
