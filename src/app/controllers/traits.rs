//! Traits for plot-type specific controllers.

use crate::app::DimConfig;
use crate::app::state::NavigationState;
use crate::data::VariableInfo;
use crate::data::multiscale::{DimDemand, ResolutionDemand};
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

    /// Handles a one-finger or primary button drag of `delta` screen points
    /// (orbiting in 3D, panning in 2D/1D).
    fn handle_drag(&self, nav: &mut NavigationState, delta: egui::Vec2);

    /// Moves the picture `delta` screen points with the pointer on canvas
    /// `rect`: two-finger drags, and in 3D secondary button or Shift drags.
    fn handle_pan(&self, nav: &mut NavigationState, delta: egui::Vec2, _rect: egui::Rect) {
        self.handle_drag(nav, delta);
    }

    /// Zooms by `factor` (above 1 magnifies) about `anchor` on canvas `rect`,
    /// keeping what is under it in place.
    fn handle_zoom(
        &self,
        nav: &mut NavigationState,
        factor: f32,
        anchor: egui::Pos2,
        rect: egui::Rect,
    );

    /// Handles `scroll` points of mouse wheel scrolling over `mouse_pos`.
    fn handle_scroll(
        &self,
        nav: &mut NavigationState,
        scroll: f32,
        mouse_pos: egui::Pos2,
        rect: egui::Rect,
    ) {
        let factor = crate::utils::scroll_zoom_factor(scroll);
        self.handle_zoom(nav, factor, mouse_pos, rect);
    }

    /// Whether this plot type supports coastline overlays.
    fn draws_coastlines(&self) -> bool {
        self.plot_type().draws_coastlines()
    }

    /// Whether this plot type supports categorical color maps.
    fn draws_categories(&self) -> bool {
        self.plot_type().draws_categories()
    }

    /// What this plot needs from a multiscale variable `var` with dimension
    /// roles `dim_config`, drawn on a canvas of `canvas_px` physical pixels
    /// (`crate::data::multiscale::fit_level` picks the level from it). By
    /// default level 0, or the finest level whose window fits one GPU
    /// storage buffer when level 0's does not.
    fn resolution_demand(
        &self,
        var: &VariableInfo,
        dim_config: &[DimConfig],
        _canvas_px: [u32; 2],
    ) -> ResolutionDemand {
        spatial_demand(var, dim_config, DimDemand::Any)
    }

    /// Minimum zoom level allowed for this plot type (in 3D, the closest
    /// camera distance).
    fn min_zoom(&self) -> f32 {
        if self.is_3d() {
            crate::plots::camera::min_distance(self.plot_type())
        } else {
            0.1
        }
    }
}

/// `want` along `var`'s x, y and z, its other dimensions whole, within one
/// GPU storage buffer.
pub(super) fn spatial_demand(
    var: &VariableInfo,
    dim_config: &[DimConfig],
    want: DimDemand,
) -> ResolutionDemand {
    let mut demand = ResolutionDemand::finest(var.shape.len());
    let (x, y, z) = var.resolve_spatial_dim_indices(dim_config);
    for d in [x, y, z].into_iter().flatten() {
        if let Some(slot) = demand.dims.get_mut(d) {
            *slot = want;
        }
    }
    demand.max_elements = Some(crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS as u64);
    demand
}
