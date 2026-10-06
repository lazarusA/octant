//! Opacity curve editor state (not persisted) and its registration.

use super::app_state::OctantApp;
use crate::utils::colormap::{AlphaInterp, alpha, registry};

/// Text of the opacity curve as typed, its interpolation and parse error.
#[derive(Default)]
pub struct AlphaCurveState {
    pub text: String,
    pub interp: AlphaInterp,
    pub error: Option<String>,
}

impl OctantApp {
    /// Parses the curve text and registers (or clears) the curve row. On a
    /// parse error the previous curve stays active.
    pub fn apply_alpha_curve(&mut self) {
        let state = &mut self.colormaps.alpha;
        match alpha::parse(&state.text) {
            Ok(curve) => {
                state.error = None;
                registry::set_alpha_curve(curve.map(|c| alpha::bake(&c, state.interp)));
            }
            Err(e) => state.error = Some(e.to_string()),
        }
    }

    /// Alpha of colormapped values at data position `t` (global opacity times
    /// the curve), as the plots draw it.
    pub fn color_alpha_at(&self, t: f32) -> f32 {
        let curve = if registry::alpha_row().is_some() {
            registry::curve_alpha(t)
        } else {
            1.0
        };
        self.color_opacity.clamp(0.0, 1.0) * curve
    }

    /// Whether colormapped values may be drawn translucent.
    pub fn has_color_alpha(&self) -> bool {
        self.color_opacity < 1.0 || registry::alpha_row().is_some()
    }
}
