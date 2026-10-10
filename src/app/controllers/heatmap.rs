//! 2D Heatmap plot controller.

use super::traits::PlotController;
use crate::app::state::NavigationState;
use crate::plots::PlotType;
use crate::utils::apply_zoom_pan_at_point;

/// Controller for planar 2D Heatmap plots.
pub struct HeatmapController;

impl PlotController for HeatmapController {
    fn plot_type(&self) -> PlotType {
        PlotType::Heatmap
    }

    fn reset_view(&self, nav: &mut NavigationState) {
        nav.heatmap_zoom = 1.0;
        nav.heatmap_pan = egui::Vec2::ZERO;
    }

    fn handle_drag(&self, nav: &mut NavigationState, delta: egui::Vec2) {
        nav.heatmap_pan += delta;
    }

    fn handle_scroll(
        &self,
        nav: &mut NavigationState,
        scroll: f32,
        mouse_pos: egui::Pos2,
        center: egui::Pos2,
    ) {
        let (zoom, pan) = apply_zoom_pan_at_point(
            nav.heatmap_zoom,
            nav.heatmap_pan,
            mouse_pos,
            center,
            scroll,
            self.min_zoom(),
            50.0,
        );
        nav.heatmap_zoom = zoom;
        nav.heatmap_pan = pan;
    }
}
