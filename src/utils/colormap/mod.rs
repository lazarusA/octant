//! Colormap catalog, LUT sampling, scale transformations, and CPU color evaluation.
//!
//! Every colormap is a 256-entry RGBA8 LUT held by [`registry`]; the CPU samples it
//! through [`lut::sample_lut`] and the GPU reads the same rows from the atlas texture
//! uploaded by `crate::plots::colormap_atlas`.

pub mod catalog;
pub mod custom;
pub mod eval;
pub mod format;
pub mod kind;
pub mod lut;
pub mod registry;
pub mod scale;

#[cfg(test)]
mod catalog_tests;
#[cfg(test)]
mod smooth_tests;
#[cfg(test)]
mod tests;

pub use catalog::{ColormapEntry, ColormapFamily, LICENSES_TEXT, builtin};
pub use custom::{BlendSpace, CustomColormapSpec, Interpolation};
pub use eval::evaluate_color_cpu;
pub use kind::ColormapKind;
pub use lut::{LUT_SIZE, Lut, orient};
pub use registry::COLORMAP_RGB_COMPOSITE;
pub use scale::{apply_color_scale_cpu, unscale_norm_to_value};
