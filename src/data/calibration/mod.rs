//! Format-agnostic data calibration, scaling, and nodata fill masking.
//!
//! Evaluates standard CF (Climate and Forecast) and xarray conventions:
//! - `scale_factor` (multiplicative scaling, default 1.0)
//! - `add_offset` (additive offset, default 0.0)
//! - `_FillValue`, `missing_value`, `fill_value` (nodata masking to NaN)
//! - `valid_min`, `valid_max`, `valid_range` (out-of-range masking to NaN)

pub mod extract;
pub mod transform;
pub mod types;

#[cfg(test)]
mod tests;

pub use types::DataCalibration;
