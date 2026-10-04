//! Evaluates final pixel colors for data values using active plot parameters.

use crate::plots::common::PlotColorParams;
use egui::Color32;

/// Evaluates the RGBA color for a given data scalar matching shader semantics.
pub fn evaluate_color_cpu(val: f32, params: &PlotColorParams) -> Color32 {
    if val.is_nan() || !val.is_finite() || val.abs() > 1e30 {
        if params.use_nan_color != 0 {
            rgba_to_color32(params.nan_color)
        } else {
            Color32::TRANSPARENT
        }
    } else if params.colormap == super::COLORMAP_RGB_COMPOSITE {
        // RGB packed u32
        let packed = val.max(0.0) as u32;
        let r = (packed & 0xFF) as u8;
        let g = ((packed >> 8) & 0xFF) as u8;
        let b = ((packed >> 16) & 0xFF) as u8;
        Color32::from_rgb(r, g, b)
    } else if params.use_lowclip != 0 && val < params.cmin {
        rgba_to_color32(params.lowclip_color)
    } else if params.use_highclip != 0 && val > params.cmax {
        rgba_to_color32(params.highclip_color)
    } else {
        let mut t = super::scale::apply_color_scale_cpu(
            val,
            params.cmin,
            params.cmax,
            params.scale_type,
            params.scale_param,
        );
        if params.is_categorical != 0 {
            let num_cats = params.num_categories.max(1) as f32;
            let bin_idx = (t.clamp(0.0, 0.999999) * num_cats).floor();
            t = (bin_idx + 0.5) / num_cats;
        }
        super::sample::sample_colormap_rgb(params.colormap, super::orient(t, params.reverse != 0))
    }
}

fn rgba_to_color32(rgba: [f32; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (rgba[0] * 255.0).clamp(0.0, 255.0) as u8,
        (rgba[1] * 255.0).clamp(0.0, 255.0) as u8,
        (rgba[2] * 255.0).clamp(0.0, 255.0) as u8,
        (rgba[3] * 255.0).clamp(0.0, 255.0) as u8,
    )
}
