//! Color parameter resolution and bounds resetting for visualization shaders.

use crate::app::OctantApp;
use crate::plots::common::PlotColorParams;

impl OctantApp {
    /// Resets global and local colormap normalization limits.
    pub fn reset_variable_bounds(&mut self) {
        self.global_data_min = f32::MAX;
        self.global_data_max = f32::MIN;
        self.lock_color_bounds = false;
    }

    /// Assembles the complete `PlotColorParams` uniform bundle from current application state.
    pub fn get_color_params(&self) -> PlotColorParams {
        let effective_colormap = self.preview_colormap.unwrap_or(self.active_colormap);

        let (is_cat, num_cats) = if self.is_categorical {
            if let Some(mdata) = &self.matrix_data {
                if let Some(unique) = mdata.detect_unique_values() {
                    (1, unique.len() as u32)
                } else {
                    (1, 10)
                }
            } else {
                (1, 10)
            }
        } else {
            (0, 10)
        };

        PlotColorParams {
            colormap: effective_colormap,
            cmin: self.color_range_min,
            cmax: self.color_range_max,
            use_nan_color: if self.use_nan_color { 1 } else { 0 },
            use_lowclip: if self.use_lowclip { 1 } else { 0 },
            use_highclip: if self.use_highclip { 1 } else { 0 },
            scale_type: self.active_scale_type,
            scale_param: self.scale_param,
            is_categorical: is_cat,
            num_categories: num_cats,
            _pad0: 0,
            _pad1: 0,
            nan_color: self.nan_color,
            lowclip_color: self.lowclip_color,
            highclip_color: self.highclip_color,
        }
    }
}
