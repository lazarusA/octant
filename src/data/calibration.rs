//! Format-agnostic data calibration, scaling, and nodata fill masking.
//!
//! Evaluates standard CF (Climate and Forecast) and xarray conventions:
//! - `scale_factor` (multiplicative scaling, default 1.0)
//! - `add_offset` (additive offset, default 0.0)
//! - `_FillValue`, `missing_value`, `fill_value` (nodata masking to NaN)
//! - `valid_min`, `valid_max`, `valid_range` (out-of-range masking to NaN)

use std::collections::HashMap;

/// Format-agnostic representation of variable calibration and fill-value masking rules.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DataCalibration {
    pub scale_factor: Option<f64>,
    pub add_offset: Option<f64>,
    pub fill_value: Option<f64>,
    pub valid_min: Option<f64>,
    pub valid_max: Option<f64>,
}

fn parse_f64_from_json(val: &serde_json::Value) -> Option<f64> {
    match val {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.trim().parse::<f64>().ok(),
        serde_json::Value::Array(arr) => arr.first().and_then(parse_f64_from_json),
        _ => None,
    }
}

fn parse_f64_from_str(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok()
}

impl DataCalibration {
    /// Creates a `DataCalibration` from a JSON key-value map (Zarr v2, Zarr v3, Icechunk).
    pub fn from_json_map(attributes: &serde_json::Map<String, serde_json::Value>) -> Self {
        let scale_factor = attributes.get("scale_factor").and_then(parse_f64_from_json);
        let add_offset = attributes.get("add_offset").and_then(parse_f64_from_json);
        let fill_value = attributes
            .get("_FillValue")
            .or_else(|| attributes.get("missing_value"))
            .or_else(|| attributes.get("fill_value"))
            .and_then(parse_f64_from_json);

        let mut valid_min = attributes.get("valid_min").and_then(parse_f64_from_json);
        let mut valid_max = attributes.get("valid_max").and_then(parse_f64_from_json);

        if let Some(range) = attributes.get("valid_range").and_then(|v| v.as_array())
            && range.len() >= 2
        {
            if let Some(min_v) = parse_f64_from_json(&range[0]) {
                valid_min = Some(min_v);
            }
            if let Some(max_v) = parse_f64_from_json(&range[1]) {
                valid_max = Some(max_v);
            }
        }

        Self {
            scale_factor,
            add_offset,
            fill_value,
            valid_min,
            valid_max,
        }
    }

    /// Creates a `DataCalibration` from a string HashMap (GeoTIFF, NetCDF, GRIB, generic stores).
    pub fn from_string_map(attributes: &HashMap<String, String>) -> Self {
        let scale_factor = attributes
            .get("scale_factor")
            .and_then(|s| parse_f64_from_str(s));
        let add_offset = attributes
            .get("add_offset")
            .and_then(|s| parse_f64_from_str(s));
        let fill_value = attributes
            .get("_FillValue")
            .or_else(|| attributes.get("missing_value"))
            .or_else(|| attributes.get("fill_value"))
            .or_else(|| attributes.get("nodata"))
            .and_then(|s| parse_f64_from_str(s));

        let valid_min = attributes
            .get("valid_min")
            .and_then(|s| parse_f64_from_str(s));
        let valid_max = attributes
            .get("valid_max")
            .and_then(|s| parse_f64_from_str(s));

        Self {
            scale_factor,
            add_offset,
            fill_value,
            valid_min,
            valid_max,
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_calibration_json_extraction_and_transform() {
        let mut map = serde_json::Map::new();
        map.insert(
            "scale_factor".to_string(),
            serde_json::Value::from(0.0001f64),
        );
        map.insert("add_offset".to_string(), serde_json::Value::from(10.0f64));
        map.insert("_FillValue".to_string(), serde_json::Value::from(-9999i64));
        map.insert("valid_min".to_string(), serde_json::Value::from(-5000i64));
        map.insert("valid_max".to_string(), serde_json::Value::from(20000i64));

        let cal = DataCalibration::from_json_map(&map);
        assert!(cal.has_transformation());
        assert_eq!(cal.scale_factor, Some(0.0001));
        assert_eq!(cal.add_offset, Some(10.0));
        assert_eq!(cal.fill_value, Some(-9999.0));
        assert_eq!(cal.valid_min, Some(-5000.0));
        assert_eq!(cal.valid_max, Some(20000.0));

        // Test normal value transformation: (10000 * 0.0001) + 10.0 = 11.0
        let transformed = cal.transform(10000.0);
        assert!((transformed - 11.0).abs() < 1e-4);

        // Test fill value -> NaN
        let fv_transformed = cal.transform(-9999.0);
        assert!(fv_transformed.is_nan());

        // Test valid_min clipping -> NaN
        let out_min_transformed = cal.transform(-6000.0);
        assert!(out_min_transformed.is_nan());

        // Test valid_max clipping -> NaN
        let out_max_transformed = cal.transform(25000.0);
        assert!(out_max_transformed.is_nan());
    }

    #[test]
    fn test_data_calibration_string_map_extraction() {
        let mut map = HashMap::new();
        map.insert("scale_factor".to_string(), "0.5".to_string());
        map.insert("add_offset".to_string(), "-5.0".to_string());
        map.insert("missing_value".to_string(), "-32768".to_string());

        let cal = DataCalibration::from_string_map(&map);
        assert!(cal.has_transformation());
        assert_eq!(cal.scale_factor, Some(0.5));
        assert_eq!(cal.add_offset, Some(-5.0));
        assert_eq!(cal.fill_value, Some(-32768.0));

        // 10 * 0.5 - 5.0 = 0.0
        assert_eq!(cal.transform(10.0), 0.0);
        assert!(cal.transform(-32768.0).is_nan());
    }

    #[test]
    fn test_data_calibration_passthrough_when_empty() {
        let empty_map = serde_json::Map::new();
        let cal = DataCalibration::from_json_map(&empty_map);
        assert!(!cal.has_transformation());
        assert_eq!(cal.transform(42.5), 42.5f32);
    }
}
