//! Plotted data containers and Level-of-Detail (LOD) resamplers.

pub mod matrix;
pub mod payload;
pub mod pyramid;
pub mod pyramid_downsample;
pub mod resampler;
pub mod volume;

pub use matrix::{MatrixData, SpatialLayout};
pub use payload::RenderData;
pub use pyramid::{AggregationOp, MatrixPyramid, PyramidLevel};
pub use resampler::{ResampledTile, ViewportRequest, ViewportResampler};
pub use volume::VolumeData;
