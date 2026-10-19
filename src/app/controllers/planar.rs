//! 2D and 1D planar pan/zoom plot controllers.

use super::traits::PlotController;
use crate::app::DimConfig;
use crate::app::state::NavigationState;
use crate::data::VariableInfo;
use crate::data::multiscale::{DimDemand, ResolutionDemand};
use crate::plots::PlotType;
use crate::utils::apply_zoom_pan_at_point;

/// Controller for planar 2D Heatmap plots.
pub struct HeatmapController;

impl PlotController for HeatmapController {
    fn plot_type(&self) -> PlotType {
        PlotType::Heatmap
    }

    /// No more cells than canvas pixels along x and y (each cell at least a
    /// pixel when the whole window shows), within one GPU buffer; other
    /// dimensions whole.
    fn resolution_demand(
        &self,
        var: &VariableInfo,
        dim_config: &[DimConfig],
        canvas_px: [u32; 2],
    ) -> ResolutionDemand {
        let mut demand = ResolutionDemand::finest(var.shape.len());
        if let (Some(x), Some(y), _) = var.resolve_spatial_dim_indices(dim_config)
            && x != y
        {
            demand.dims[x] = DimDemand::Pixels(canvas_px[0]);
            demand.dims[y] = DimDemand::Pixels(canvas_px[1]);
            demand.max_elements = Some(crate::plots::limits::max_storage_elements() as u64);
        }
        demand
    }

    fn reset_view(&self, nav: &mut NavigationState) {
        nav.heatmap_zoom = 1.0;
        nav.heatmap_pan = egui::Vec2::ZERO;
    }

    fn handle_drag(&self, nav: &mut NavigationState, delta: egui::Vec2) {
        nav.heatmap_pan += delta;
    }

    fn handle_zoom(
        &self,
        nav: &mut NavigationState,
        factor: f32,
        anchor: egui::Pos2,
        rect: egui::Rect,
    ) {
        let (zoom, pan) = apply_zoom_pan_at_point(
            nav.heatmap_zoom,
            nav.heatmap_pan,
            anchor,
            rect.center(),
            factor,
            self.min_zoom(),
            nav.heatmap_max_zoom,
        );
        nav.heatmap_zoom = zoom;
        nav.heatmap_pan = pan;
    }
}

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

    fn handle_zoom(
        &self,
        nav: &mut NavigationState,
        factor: f32,
        anchor: egui::Pos2,
        rect: egui::Rect,
    ) {
        let (zoom, pan) = apply_zoom_pan_at_point(
            nav.line_zoom,
            nav.line_pan,
            anchor,
            rect.center(),
            factor,
            self.min_zoom(),
            crate::app::state::navigation::MAX_ZOOM,
        );
        nav.line_zoom = zoom;
        nav.line_pan = pan;
    }
}
