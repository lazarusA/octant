//! How a layer colors its data: colormap, range, clipping and composites.

use crate::data::matrix_data::MatrixData;
use crate::plots::common::PlotColorParams;
use crate::utils::colormap::{COLORMAP_RGB_COMPOSITE, NO_ALPHA_ROW, registry};

/// Colormapping of a scalar layer.
#[derive(Debug, Clone)]
pub struct ColorStyle {
    /// Row id in `utils::colormap::registry` (and the GPU colormap atlas).
    pub colormap: u32,
    pub range_min: f32,
    pub range_max: f32,
    /// Keeps the range while new data arrives.
    pub lock_bounds: bool,
    /// Data extent seen across the steps shown so far.
    pub global_min: f32,
    pub global_max: f32,
    pub nan_color: [f32; 4],
    pub use_nan_color: bool,
    pub lowclip_color: [f32; 4],
    pub use_lowclip: bool,
    pub highclip_color: [f32; 4],
    pub use_highclip: bool,
    /// Opacity of colormapped colors, in [0, 1].
    pub opacity: f32,
    pub scale_type: u32,
    pub scale_param: f32,
    pub categorical: bool,
    pub custom_label: Option<String>,
    pub volume_cmin: f32,
    pub volume_cmax: f32,
}

impl Default for ColorStyle {
    /// The shader defaults (`PlotColorParams::default`): range, clip and NaN
    /// colors, scale; with every clip color off.
    fn default() -> Self {
        let shader = PlotColorParams::default();
        Self {
            colormap: registry::default_id(),
            range_min: shader.cmin,
            range_max: shader.cmax,
            lock_bounds: false,
            global_min: f32::INFINITY,
            global_max: f32::NEG_INFINITY,
            nan_color: shader.nan_color,
            use_nan_color: false,
            lowclip_color: shader.lowclip_color,
            use_lowclip: false,
            highclip_color: shader.highclip_color,
            use_highclip: false,
            opacity: shader.opacity,
            scale_type: shader.scale_type,
            scale_param: shader.scale_param,
            categorical: false,
            custom_label: None,
            volume_cmin: 5.0,
            volume_cmax: 100.0,
        }
    }
}

impl ColorStyle {
    /// Forgets the data extent seen so far, unless the range is locked.
    pub fn reset_bounds(&mut self) {
        if !self.lock_bounds {
            self.global_min = f32::MAX;
            self.global_max = f32::MIN;
        }
    }

    /// Starts the data extent and range over at a new variable's `min`..`max`
    /// (non-finite ends, as in an empty volume, keep the old ones) and unlocks
    /// the range.
    pub fn reset_to_extent(&mut self, min: f32, max: f32) {
        if min.is_finite() {
            self.global_min = min;
            self.range_min = min;
            self.volume_cmin = min;
        }
        if max.is_finite() {
            self.global_max = max;
            self.range_max = max;
            self.volume_cmax = max;
        }
        self.lock_bounds = false;
    }

    /// Widens the data extent seen so far by another step's `min`..`max`, and
    /// moves the range there unless it is locked. Non-finite ends are skipped.
    pub fn follow_extent(&mut self, min: f32, max: f32) {
        if min.is_finite() {
            self.global_min = self.global_min.min(min);
            if !self.lock_bounds {
                self.range_min = min;
                self.volume_cmin = min;
            }
        }
        if max.is_finite() {
            self.global_max = self.global_max.max(max);
            if !self.lock_bounds {
                self.range_max = max;
                self.volume_cmax = max;
            }
        }
    }

    /// The shader color uniforms drawing atlas row `shown` (the colormap after
    /// preview and smoothing), `reversed`, as an RGB composite when `composite`;
    /// `matrix` counts categories for categorical colors.
    pub fn params(
        &self,
        shown: u32,
        reversed: bool,
        composite: bool,
        matrix: Option<&MatrixData>,
    ) -> PlotColorParams {
        // Ids that are no atlas row (e.g. a stale selection) draw the default,
        // so the GPU never reads a padding row the CPU would not.
        let row = Some(shown)
            .filter(|&id| registry::is_row(id))
            .unwrap_or_else(registry::default_id);
        let num_categories = matrix
            .and_then(MatrixData::detect_unique_values)
            .map_or(10, |unique| unique.len() as u32);
        PlotColorParams {
            colormap: if composite {
                COLORMAP_RGB_COMPOSITE
            } else {
                row
            },
            cmin: self.range_min,
            cmax: self.range_max,
            use_nan_color: u32::from(self.use_nan_color),
            use_lowclip: u32::from(self.use_lowclip),
            use_highclip: u32::from(self.use_highclip),
            scale_type: self.scale_type,
            scale_param: self.scale_param,
            is_categorical: u32::from(self.categorical),
            num_categories: if self.categorical { num_categories } else { 10 },
            reverse: u32::from(reversed),
            nearest: u32::from(registry::is_stepped(row)),
            fallback_colormap: row,
            opacity: self.opacity.clamp(0.0, 1.0),
            alpha_row: registry::alpha_row().unwrap_or(NO_ALPHA_ROW),
            _pad: 0,
            nan_color: self.nan_color,
            lowclip_color: self.lowclip_color,
            highclip_color: self.highclip_color,
        }
    }
}

/// Channels drawn together as one color image.
#[derive(Debug, Clone)]
pub struct CompositeStyle {
    pub enabled: bool,
    pub rgb_channels: [usize; 3],
    pub channel_configs: Vec<crate::data::slicing::ChannelColorConfig>,
}

impl Default for CompositeStyle {
    fn default() -> Self {
        Self {
            enabled: false,
            rgb_channels: [0, 1, 2],
            channel_configs: Vec::new(),
        }
    }
}
