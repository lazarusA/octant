//! Procedural and analytical synthetic data generators for tests and benchmarking.

pub mod fields;
pub mod grids;
pub mod known_truth;

#[cfg(test)]
mod tests;

pub use fields::{
    generate_procedural_matrix, generate_procedural_volume_3d, generate_procedural_volume_4d,
};
pub use grids::{
    generate_clenshaw_curtis_2d, generate_clenshaw_curtis_coords, generate_gaussian_coords,
    generate_gaussian_grid_2d, generate_stepped_resolution_2d, generate_stepped_resolution_coords,
    generate_stretched_regional_2d, generate_stretched_regional_coords,
};
pub use known_truth::{
    KnownTruth4DParams, eval_known_truth_4d, generate_known_truth_4d_block,
    get_known_truth_4d_center,
};
