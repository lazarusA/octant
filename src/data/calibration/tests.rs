use super::types::DataCalibration;

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

/// Numbers written as text read as numbers; `valid_range` gives both bounds.
#[test]
fn text_attributes_and_valid_range() {
    let attrs = serde_json::json!({
        "scale_factor": "0.5",
        "add_offset": "-5.0",
        "missing_value": "-32768",
        "valid_range": [-40000, 100],
    });
    let cal = DataCalibration::from_json_map(attrs.as_object().expect("map"));
    assert_eq!(cal.scale_factor, Some(0.5));
    assert_eq!(cal.add_offset, Some(-5.0));
    assert_eq!(cal.fill_value, Some(-32768.0));
    assert_eq!(
        (cal.valid_min, cal.valid_max),
        (Some(-40000.0), Some(100.0))
    );

    // 10 * 0.5 - 5.0 = 0.0
    assert_eq!(cal.transform(10.0), 0.0);
    assert!(cal.transform(-32768.0).is_nan());
    assert!(cal.transform(101.0).is_nan());

    // An inverted range (GOES-R's unsigned `0, -6` read as signed) masks nothing.
    let attrs = serde_json::json!({ "valid_range": [0, -6] });
    let cal = DataCalibration::from_json_map(attrs.as_object().expect("map"));
    assert_eq!((cal.valid_min, cal.valid_max), (None, None));
    // An `f32` value at a bound `f32` cannot hold is kept.
    let cal = DataCalibration {
        valid_max: Some(0.3),
        ..Default::default()
    };
    let mut slice = [0.3f32, 0.4];
    cal.transform_slice_in_place(&mut slice);
    assert!(!slice[0].is_nan() && slice[1].is_nan(), "{slice:?}");
}

#[test]
fn test_data_calibration_passthrough_when_empty() {
    let empty_map = serde_json::Map::new();
    let cal = DataCalibration::from_json_map(&empty_map);
    assert!(!cal.has_transformation());
    assert_eq!(cal.transform(42.5), 42.5f32);
}

/// A zero fill (or GeoTIFF nodata) masks only zero, not every value near it.
#[test]
fn zero_fill_keeps_tiny_values() {
    let cal = DataCalibration {
        fill_value: Some(0.0),
        ..Default::default()
    };
    assert!(cal.transform_f64(0.0).is_nan(), "0 is the fill value");
    assert_eq!(cal.transform_f64(1e-6), 1e-6);
    let mut slice = [0.0f32, -0.0, 1e-6, -5e-6];
    cal.transform_slice_in_place(&mut slice);
    assert!(slice[0].is_nan() && slice[1].is_nan(), "{slice:?}");
    assert_eq!(&slice[2..], &[1e-6, -5e-6]);

    use crate::data::backends::geotiff::blit::is_nodata;
    assert!(is_nodata(0.0, Some(0.0)));
    assert!(!is_nodata(5e-7, Some(0.0)));
    assert!(is_nodata(-9999.0, Some(-9999.0)));
}

/// GDAL's `nan` nodata masks only NaN samples, never finite ones (B1).
#[test]
fn nan_nodata_keeps_finite_samples() {
    use crate::data::backends::geotiff::blit::{ReadWindow, blit_chunk_to_window};
    use async_tiff::tags::SampleFormat;
    let raw: Vec<u8> = [1.0f32, f32::NAN, 3.0, 4.0]
        .iter()
        .flat_map(|v| v.to_ne_bytes())
        .collect();
    let win = ReadWindow {
        row_start: 0,
        row_end: 2,
        col_start: 0,
        col_end: 2,
        nodata_val: Some(f64::NAN),
    };
    let mut out = [0.0f32; 4];
    let (fmt, bits) = (SampleFormat::Float, 32);
    blit_chunk_to_window(
        &raw,
        2,
        2,
        1,
        false,
        0,
        0,
        fmt,
        bits,
        false,
        &win,
        &[0],
        &mut [&mut out],
    );
    assert_eq!(out.map(f32::is_nan), [false, true, false, false], "{out:?}");
}
