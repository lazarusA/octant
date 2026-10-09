//! Opacity curve editor state (not persisted) and its registration.

use super::app_state::OctantApp;
use crate::app::layers::Layer;
use crate::plots::PlotType;
use crate::plots::oit::{self, Transparency};
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

    /// Whether the base layer's colormapped values may be drawn translucent.
    pub fn has_color_alpha(&self) -> bool {
        self.layers.base.color.is_translucent()
    }

    /// Frees the OIT frames of the 3D renderers not drawn as `active`, whose
    /// callbacks do not run to free them.
    pub fn release_idle_oit_frames(&self, active: PlotType) {
        for layer in self.layers.iter() {
            layer.renderers.release_idle_oit_frames(active);
        }
    }

    /// How `layer`'s 3D meshes and point clouds draw: order-independent transparency
    /// with translucent colors when the device supports it, else without depth
    /// writes; opaque colors (including RGB composites, which ignore opacity)
    /// keep depth writes, so near parts hide far ones.
    pub fn transparency_mode(&self, layer: &Layer) -> Transparency {
        if !self.plot_transparency || layer.composite.enabled || !layer.color.is_translucent() {
            Transparency::Off
        } else if self
            .wgpu_render_state
            .as_ref()
            .is_some_and(|rs| oit::supported(&rs.adapter))
        {
            Transparency::Oit
        } else {
            Transparency::NoDepthWrite
        }
    }
}
