//! Format-agnostic data calibration, scaling, and nodata fill masking types.

/// Format-agnostic representation of variable calibration and fill-value masking rules.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DataCalibration {
    pub scale_factor: Option<f64>,
    pub add_offset: Option<f64>,
    pub fill_value: Option<f64>,
    pub valid_min: Option<f64>,
    pub valid_max: Option<f64>,
}
