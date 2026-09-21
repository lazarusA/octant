use std::collections::HashMap;

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
