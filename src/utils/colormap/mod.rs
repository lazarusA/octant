//! Colormap definitions, scale transformations, and CPU color evaluation.

pub mod eval;
pub mod sample;
pub mod scale;

#[cfg(test)]
mod tests;

pub use eval::evaluate_color_cpu;
pub use sample::sample_colormap_rgb;
pub use scale::{apply_color_scale_cpu, unscale_norm_to_value};
