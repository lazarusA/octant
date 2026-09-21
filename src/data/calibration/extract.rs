//! CF conventions metadata attribute parsers for calibration rules.

use std::collections::HashMap;

use super::types::DataCalibration;

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
}
