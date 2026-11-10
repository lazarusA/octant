//! Numerical data transformations and in-place SIMD-friendly slice masking.

use super::types::DataCalibration;

impl DataCalibration {
    /// Returns `true` if any calibration or masking rule is active.
    #[inline]
    pub fn has_transformation(&self) -> bool {
        self.scale_factor.is_some()
            || self.add_offset.is_some()
            || self.fill_value.is_some()
            || self.valid_min.is_some()
            || self.valid_max.is_some()
    }

    /// Transforms a single numerical element according to CF convention masking and scaling rules.
    #[inline]
    pub fn transform(&self, val: f64) -> f32 {
        self.transform_f64(val) as f32
    }

    /// [`Self::transform`] in full precision, for values such as coordinates that `f32`
    /// cannot hold exactly (e.g. seconds since an epoch).
    #[inline]
    pub fn transform_f64(&self, val: f64) -> f64 {
        if self.fill_value.is_some_and(|fv| matches_fill(val, fv)) {
            return f64::NAN;
        }
        if let Some(vmin) = self.valid_min
            && val < vmin
        {
            return f64::NAN;
        }
        if let Some(vmax) = self.valid_max
            && val > vmax
        {
            return f64::NAN;
        }
        val * self.scale_factor.unwrap_or(1.0) + self.add_offset.unwrap_or(0.0)
    }

    /// Transforms a slice of values in-place if any transformation is active, in `f64` as
    /// [`Self::transform`] does for every other data type. Pure scale and offset take a
    /// branchless loop the compiler vectorizes.
    pub fn transform_slice_in_place(&self, slice: &mut [f32]) {
        if !self.has_transformation() {
            return;
        }
        if self.fill_value.is_none() && self.valid_min.is_none() && self.valid_max.is_none() {
            let s = self.scale_factor.unwrap_or(1.0);
            let o = self.add_offset.unwrap_or(0.0);
            for elem in slice.iter_mut() {
                *elem = (f64::from(*elem) * s + o) as f32;
            }
            return;
        }
        // `f32` values meet `f32` bounds, so one at a bound `f32` cannot hold stays.
        let bound = |b: Option<f64>| b.map(|b| f64::from(b as f32));
        let cal = Self {
            valid_min: bound(self.valid_min),
            valid_max: bound(self.valid_max),
            ..*self
        };
        for elem in slice.iter_mut() {
            *elem = cal.transform(f64::from(*elem));
        }
    }
}

/// Whether `val` is the fill (nodata) value `fill`: equal within a relative 1e-5, so a
/// fill stored in another precision than the data still matches, or both NaN. The one
/// fill rule of every reader (CF attributes and GeoTIFF nodata alike).
#[inline]
pub fn matches_fill(val: f64, fill: f64) -> bool {
    val == fill || (val - fill).abs() <= 1e-5 * fill.abs() || (val.is_nan() && fill.is_nan())
}
