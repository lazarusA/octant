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
        if let Some(fv) = self.fill_value
            && ((val - fv).abs() <= 1e-5 * fv.abs().max(1.0) || (val.is_nan() && fv.is_nan()))
        {
            return f32::NAN;
        }
        if let Some(vmin) = self.valid_min
            && val < vmin
        {
            return f32::NAN;
        }
        if let Some(vmax) = self.valid_max
            && val > vmax
        {
            return f32::NAN;
        }

        let scale = self.scale_factor.unwrap_or(1.0);
        let offset = self.add_offset.unwrap_or(0.0);

        if self.scale_factor.is_some() || self.add_offset.is_some() {
            (val * scale + offset) as f32
        } else {
            val as f32
        }
    }

    /// Transforms a slice of values in-place if any transformation is active.
    /// Uses an optimized branchless loop for standard scale/offset transformations to enable SIMD vectorization.
    pub fn transform_slice_in_place(&self, slice: &mut [f32]) {
        if !self.has_transformation() {
            return;
        }

        let scale = self.scale_factor.map(|s| s as f32);
        let offset = self.add_offset.map(|o| o as f32);
        let fill = self.fill_value.map(|f| f as f32);
        let valid_min = self.valid_min.map(|m| m as f32);
        let valid_max = self.valid_max.map(|m| m as f32);

        // Fast-path: pure linear scale & offset without fill masking (SIMD-vectorizable)
        if fill.is_none() && valid_min.is_none() && valid_max.is_none() {
            let s = scale.unwrap_or(1.0);
            let o = offset.unwrap_or(0.0);
            for elem in slice.iter_mut() {
                *elem = *elem * s + o;
            }
            return;
        }

        // Masking path with nodata / fill value and valid range checks
        for elem in slice.iter_mut() {
            let val = *elem;
            if let Some(fv) = fill
                && ((val - fv).abs() <= 1e-5 * fv.abs().max(1.0) || (val.is_nan() && fv.is_nan()))
            {
                *elem = f32::NAN;
                continue;
            }
            if let Some(vmin) = valid_min
                && val < vmin
            {
                *elem = f32::NAN;
                continue;
            }
            if let Some(vmax) = valid_max
                && val > vmax
            {
                *elem = f32::NAN;
                continue;
            }

            if let Some(s) = scale {
                *elem = val * s + offset.unwrap_or(0.0);
            } else if let Some(o) = offset {
                *elem = val + o;
            }
        }
    }
}
