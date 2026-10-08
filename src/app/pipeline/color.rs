//! Color parameter resolution and bounds resetting for visualization shaders.

use crate::app::OctantApp;
use crate::plots::common::PlotColorParams;

impl OctantApp {
    /// Resets global and local colormap normalization limits.
    pub fn reset_variable_bounds(&mut self) {
        self.layers.base.color.reset_bounds();
    }

    /// Assembles the complete `PlotColorParams` uniform bundle for the plotted layer.
    pub fn get_color_params(&self) -> PlotColorParams {
        let base = &self.layers.base;
        base.color.params(
            self.effective_colormap(),
            self.colormaps.reversed,
            base.composite.enabled,
            base.data.matrix.as_ref(),
        )
    }
}
