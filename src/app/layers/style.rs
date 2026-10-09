//! How a layer colors its data: colormap, range, clipping and composites.

use crate::data::matrix_data::MatrixData;
use crate::plots::common::PlotColorParams;
use crate::utils::colormap::{AlphaInterp, COLORMAP_RGB_COMPOSITE, NO_ALPHA_ROW, registry};

/// Text of an opacity curve as typed, its interpolation and parse error.
#[derive(Debug, Clone, Default)]
pub struct AlphaCurveState {
    pub text: String,
    pub interp: AlphaInterp,
    pub error: Option<String>,
}

/// Colormapping of a scalar layer.
#[derive(Debug, Clone)]
pub struct ColorStyle {
    /// Row id in `utils::colormap::registry` (and the GPU colormap atlas).
    pub colormap: u32,
    /// Samples the colormap from its end.
    pub reversed: bool,
    /// Draws the smooth twin of a short categorical palette.
    pub smooth: bool,
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
    /// The opacity curve editor; the baked curve lives in the colormap
    /// registry under `alpha_key`.
    pub alpha: AlphaCurveState,
    /// The layer's curve key in the registry (its `LayerId`), set by `Layer::new`.
    pub(super) alpha_key: u32,
}

impl Default for ColorStyle {
    /// The shader defaults (`PlotColorParams::default`): range, clip and NaN
    /// colors, scale; with every clip color off.
    fn default() -> Self {
        let shader = PlotColorParams::default();
        Self {
            colormap: registry::default_id(),
            reversed: false,
            smooth: false,
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
            alpha: AlphaCurveState::default(),
            alpha_key: 0,
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
        }
        if max.is_finite() {
            self.global_max = max;
            self.range_max = max;
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
            }
        }
        if max.is_finite() {
            self.global_max = self.global_max.max(max);
            if !self.lock_bounds {
                self.range_max = max;
            }
        }
    }

    /// Alpha of colormapped values at data position `t`: the opacity times
    /// the opacity curve, as the plots draw it.
    pub fn alpha_at(&self, t: f32) -> f32 {
        self.opacity.clamp(0.0, 1.0) * registry::curve_alpha(self.alpha_key, t)
    }

    /// Whether colormapped values may be drawn translucent.
    pub fn is_translucent(&self) -> bool {
        self.opacity < 1.0 || self.alpha_row().is_some()
    }

    /// The atlas row of this style's opacity curve, if it has one.
    pub fn alpha_row(&self) -> Option<u32> {
        registry::alpha_row(self.alpha_key)
    }

    /// The registry key of this style's opacity curve.
    pub fn alpha_key(&self) -> u32 {
        self.alpha_key
    }

    /// The colormap row drawn for colormap `id` with this style: its smooth
    /// twin when `smooth` is on and the style's own colormap has one (the
    /// toggle only shows, so only applies, then).
    pub fn shown_row(&self, id: u32) -> u32 {
        if self.smooth && registry::smooth_variant(self.colormap).is_some() {
            registry::smooth_variant(id).unwrap_or(id)
        } else {
            id
        }
    }

    /// The shader color uniforms drawing atlas row `shown` (the colormap after
    /// preview and smoothing), as an RGB composite when `composite`; `matrix`
    /// counts categories for categorical colors.
    pub fn params(
        &self,
        shown: u32,
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
            reverse: u32::from(self.reversed),
            nearest: u32::from(registry::is_stepped(row)),
            fallback_colormap: row,
            opacity: self.opacity.clamp(0.0, 1.0),
            alpha_row: self.alpha_row().unwrap_or(NO_ALPHA_ROW),
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
