//! The `scale_offset` codec (zarr-extensions: decoding is `x / scale + offset`) read as CF
//! calibration instead of decoded: zarrs 0.23 has no such codec, and `DataCalibration`
//! already scales every Zarr read, data and coordinates alike. dynamical.org's HRRR
//! temperatures put it before their GRIB codec to read in °C.

use serde_json::{Value, json};

/// Moves a leading `scale_offset` codec into the array's `scale_factor`/`add_offset`
/// attributes, composed with those already there (the codec decodes first, then CF), and
/// its fill value into stored units. Only the first codec is lifted: it is the one decoded
/// last, so applying it after decoding is exact; one elsewhere is left for zarrs to refuse.
pub(super) fn lift(meta: &mut Value) {
    let Some(codecs) = meta.get_mut("codecs").and_then(Value::as_array_mut) else {
        return;
    };
    if codecs.first().and_then(|c| c["name"].as_str()) != Some("scale_offset") {
        return;
    }
    let config = &codecs[0]["configuration"];
    let scale = config["scale"].as_f64().unwrap_or(1.0);
    let offset = config["offset"].as_f64().unwrap_or(0.0);
    if scale == 0.0 || !scale.is_finite() || !offset.is_finite() {
        return;
    }
    codecs.remove(0);

    let integer = meta["data_type"]
        .as_str()
        .is_some_and(|t| t.starts_with("int") || t.starts_with("uint"));
    if let Some(fill) = meta["fill_value"].as_f64() {
        let stored = (fill - offset) * scale;
        meta["fill_value"] = json!(if integer { stored.round() } else { stored });
    }

    if !meta["attributes"].is_object() {
        meta["attributes"] = json!({});
    }
    let Some(attrs) = meta["attributes"].as_object_mut() else {
        return;
    };
    let cf = crate::data::DataCalibration::from_json_map(attrs);
    let (sf, ao) = (cf.scale_factor.unwrap_or(1.0), cf.add_offset.unwrap_or(0.0));
    let (sf, ao) = (sf / scale, offset * sf + ao);
    if sf != 1.0 || attrs.contains_key("scale_factor") {
        attrs.insert("scale_factor".into(), json!(sf));
    }
    if ao != 0.0 || attrs.contains_key("add_offset") {
        attrs.insert("add_offset".into(), json!(ao));
    }
}
