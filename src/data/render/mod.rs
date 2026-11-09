//! Plotted data containers and Level-of-Detail (LOD) resamplers.

pub mod decimate;
#[cfg(test)]
mod decimate_tests;
pub mod matrix;
pub mod pyramid;
pub mod pyramid_downsample;
#[cfg(test)]
mod pyramid_tests;
pub mod resampler;
pub mod volume;

pub use matrix::{MatrixData, SpatialLayout};
pub use pyramid::{AggregationOp, MatrixPyramid, PyramidLevel};
pub use resampler::{ResampledTile, ViewportRequest, ViewportResampler};
pub use volume::VolumeData;
