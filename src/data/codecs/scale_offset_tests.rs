//! `scale_offset` read as CF calibration: dynamical.org's HRRR temperatures
//! (`{"offset": -273.15}` before their GRIB codec) read in °C.

use serde_json::{Value, json};

use super::normalize_v3_array_metadata;
use crate::data::DataCalibration;

fn array(codecs: Value, attributes: Value, fill: Value) -> Value {
    json!({
        "zarr_format": 3,
        "node_type": "array",
        "shape": [2, 2],
        "data_type": "float32",
        "chunk_grid": { "name": "regular", "configuration": { "chunk_shape": [2, 2] } },
        "chunk_key_encoding": { "name": "default", "configuration": { "separator": "/" } },
        "fill_value": fill,
        "codecs": codecs,
        "attributes": attributes
    })
}

fn calibration(meta: &Value) -> DataCalibration {
    let attrs = meta["attributes"].as_object().cloned().unwrap_or_default();
    DataCalibration::from_json_map(&attrs)
}

fn codec_names(meta: &Value) -> Vec<&str> {
    (meta["codecs"].as_array().into_iter().flatten())
        .filter_map(|c| c["name"].as_str())
        .collect()
}

#[test]
fn a_leading_scale_offset_becomes_calibration() {
    let codecs = json!([
        { "name": "scale_offset", "configuration": { "offset": -273.15 } },
        { "name": "gribberish", "configuration": { "var": "TMP" } }
    ]);
    let norm = normalize_v3_array_metadata(array(codecs, json!({}), json!("NaN")));
    assert_eq!(codec_names(&norm), ["gribberish"]);
    let cal = calibration(&norm);
    assert!((f64::from(cal.transform(300.0)) - 26.85).abs() < 1e-4);
    assert_eq!(norm["fill_value"], "NaN");
}

#[test]
fn scale_composes_with_cf_attributes_and_moves_the_fill() {
    // Decoding: x / 4 + 1, then CF: * 2 + 3.
    let codecs = json!([
        { "name": "scale_offset", "configuration": { "scale": 4.0, "offset": 1.0 } },
        { "name": "bytes", "configuration": { "endian": "little" } }
    ]);
    let attrs = json!({ "scale_factor": 2.0, "add_offset": 3.0 });
    let norm = normalize_v3_array_metadata(array(codecs, attrs, json!(5.0)));
    assert_eq!(codec_names(&norm), ["bytes"]);
    let cal = calibration(&norm);
    assert_eq!(cal.transform(8.0), ((8.0 / 4.0 + 1.0) * 2.0 + 3.0) as f32);
    // A fill of 5 decoded is (5 - 1) * 4 = 16 stored.
    assert_eq!(norm["fill_value"], 16.0);
}

#[test]
fn a_scale_offset_after_another_codec_is_left_for_zarrs() {
    let codecs = json!([
        { "name": "transpose", "configuration": { "order": [1, 0] } },
        { "name": "scale_offset", "configuration": { "offset": 1.0 } },
        { "name": "bytes", "configuration": { "endian": "little" } }
    ]);
    let norm = normalize_v3_array_metadata(array(codecs, json!({}), json!(0.0)));
    assert!(codec_names(&norm).contains(&"scale_offset"));
    assert_eq!(calibration(&norm), DataCalibration::default());
}
