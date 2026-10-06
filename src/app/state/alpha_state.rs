//! Opacity curve editor state (not persisted) and its registration.

use super::app_state::OctantApp;
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

    /// Frees the OIT frames of the 3D renderers not drawn as `active`, whose
    /// callbacks do not run to free them.
    pub fn release_idle_oit_frames(&self, active: PlotType) {
        let meshes = [
            (PlotType::Sphere, &self.sphere_renderer),
            (PlotType::Surface, &self.surface_renderer),
        ];
        for (kind, renderer) in meshes {
            if kind != active
                && let Some(renderer) = renderer
            {
                renderer.release_oit_frame();
            }
        }
        if active != PlotType::PointCloud
            && let Some(renderer) = &self.point_cloud_renderer
        {
            renderer.release_oit_frame();
        }
    }

    /// How 3D meshes and point clouds draw: order-independent transparency
    /// with translucent colors when the device supports it, else without depth
    /// writes; opaque colors (including RGB composites, which ignore opacity)
    /// keep depth writes, so near parts hide far ones.
    pub fn transparency_mode(&self) -> Transparency {
        if !self.plot_transparency || self.rgb_composite_mode || !self.has_color_alpha() {
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
