//! CPU colormap sampling backed by the shared LUT registry (mirrors WGSL `sample_colormap`).

use egui::Color32;

/// Samples colormap `colormap_id` at normalized `t` in [0.0, 1.0].
pub fn sample_colormap_rgb(colormap_id: u32, t: f32) -> Color32 {
    super::registry::sample(colormap_id, t)
}
