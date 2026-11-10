//! CF conventions attributes read into calibration rules, alike for every format.

use super::types::DataCalibration;

/// The `i`-th number of a JSON attribute: a number or numeric text (`i == 0`), or an
/// array's element.
fn nth_json(value: &serde_json::Value, i: usize) -> Option<f64> {
    match value {
        serde_json::Value::Number(n) if i == 0 => n.as_f64(),
        serde_json::Value::String(s) if i == 0 => s.trim().parse().ok(),
        serde_json::Value::Array(values) => nth_json(values.get(i)?, 0),
        _ => None,
    }
}

impl DataCalibration {
    /// CF scale, offset, fill and valid range, `get(name, i)` reading attribute `name`'s
    /// `i`-th number: the one reader of every format (NetCDF, HDF5 and MATLAB readers,
    /// and through [`Self::from_json_map`] Zarr and Icechunk). `valid_range` wins over
    /// `valid_min`/`valid_max`.
    pub fn from_numbers(get: impl Fn(&str, usize) -> Option<f64>) -> Self {
        let first = |name: &str| get(name, 0);
        // An inverted range is a packed unsigned one read as signed (`_Unsigned`, GOES-R):
        // ignored rather than masking everything.
        let range = get("valid_range", 0)
            .zip(get("valid_range", 1))
            .filter(|(min, max)| min <= max);
        Self {
            scale_factor: first("scale_factor"),
            add_offset: first("add_offset"),
            fill_value: first("_FillValue")
                .or_else(|| first("missing_value"))
                .or_else(|| first("fill_value")),
            valid_min: range.map(|r| r.0).or_else(|| first("valid_min")),
            valid_max: range.map(|r| r.1).or_else(|| first("valid_max")),
        }
    }

    /// [`Self::from_numbers`] over a JSON attribute map (Zarr v2, Zarr v3, Icechunk).
    pub fn from_json_map(attributes: &serde_json::Map<String, serde_json::Value>) -> Self {
        Self::from_numbers(|name, i| nth_json(attributes.get(name)?, i))
    }
}
