//! Color parameter resolution and bounds resetting for visualization shaders.

use crate::app::OctantApp;
use crate::app::layers::Layer;
use crate::plots::common::PlotColorParams;

impl OctantApp {
    /// Resets global and local colormap normalization limits.
    pub fn reset_variable_bounds(&mut self) {
        self.layers.base.color.reset_bounds();
    }

    /// Assembles the complete `PlotColorParams` uniform bundle for `layer`.
    pub fn get_color_params(&self, layer: &Layer) -> PlotColorParams {
        layer.color.params(
            self.layer_colormap(layer),
            self.colormaps.reversed,
            layer.composite.enabled,
            layer.data.matrix.as_ref(),
        )
    }
}
