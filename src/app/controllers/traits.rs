//! Traits for plot-type specific controllers.

use crate::app::state::NavigationState;
use crate::plots::PlotType;

/// Polymorphic plot controller defining navigation, camera interactions, and capabilities.
#[allow(dead_code)]
pub trait PlotController: Send + Sync {
    /// Associated visualization plot type.
    fn plot_type(&self) -> PlotType;

    /// Human-readable display label.
    fn display_name(&self) -> &'static str {
        self.plot_type().display_name()
    }

    /// Whether this plot utilizes a 3D orbiting camera view.
    fn is_3d(&self) -> bool {
        self.plot_type().is_3d()
    }

    /// Resets pan, zoom, or camera angle to initial defaults.
    fn reset_view(&self, nav: &mut NavigationState);

    /// Handles mouse dragging (orbiting in 3D, translation pan in 2D/1D).
    fn handle_drag(&self, nav: &mut NavigationState, delta: egui::Vec2);

    /// Handles scroll wheel zoom events.
    fn handle_scroll(
        &self,
        nav: &mut NavigationState,
        scroll: f32,
        mouse_pos: egui::Pos2,
        center: egui::Pos2,
    );

    /// Whether this plot type supports coastline overlays.
    fn draws_coastlines(&self) -> bool {
        self.plot_type().draws_coastlines()
    }

    /// Whether this plot type supports categorical color maps.
    fn draws_categories(&self) -> bool {
        self.plot_type().draws_categories()
    }

    /// Minimum zoom level allowed for this plot type.
    fn min_zoom(&self) -> f32 {
        if self.plot_type() == PlotType::Sphere {
            1.1
        } else if self.is_3d() {
            0.2
        } else {
            0.1
        }
    }
}
