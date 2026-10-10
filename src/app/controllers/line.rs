//! 1D Line plot controller.

use super::traits::PlotController;
use crate::app::state::NavigationState;
use crate::plots::PlotType;
use crate::utils::apply_zoom_pan_at_point;

/// Controller for 1D profile Line plots.
pub struct LineController;

impl PlotController for LineController {
    fn plot_type(&self) -> PlotType {
        PlotType::Line
    }

    fn reset_view(&self, nav: &mut NavigationState) {
        nav.line_zoom = 1.0;
        nav.line_pan = egui::Vec2::ZERO;
    }

    fn handle_drag(&self, nav: &mut NavigationState, delta: egui::Vec2) {
        nav.line_pan += delta;
    }

    fn handle_scroll(
        &self,
        nav: &mut NavigationState,
        scroll: f32,
        mouse_pos: egui::Pos2,
        center: egui::Pos2,
    ) {
        let (zoom, pan) = apply_zoom_pan_at_point(
            nav.line_zoom,
            nav.line_pan,
            mouse_pos,
            center,
            scroll,
            self.min_zoom(),
            50.0,
        );
        nav.line_zoom = zoom;
        nav.line_pan = pan;
    }
}
