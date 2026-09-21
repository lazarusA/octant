use super::array_open::normalize_v3_array_metadata;
use super::cf::ParsedCfAttributes;
use zarrs::array::ArrayMetadata;

#[test]
fn test_normalize_v3_metadata_zlib_and_shuffle() {
    let raw_meta = serde_json::json!({
        "zarr_format": 3,
        "node_type": "array",
        "shape": [5, 3600, 7200],
        "data_type": "int16",
        "chunk_grid": {
            "name": "regular",
            "configuration": {
                "chunk_shape": [1, 1200, 2400]
            }
        },
        "chunk_key_encoding": {
            "name": "default",
            "configuration": {
                "separator": "/"
            }
        },
        "fill_value": -9999,
        "codecs": [
            {
                "name": "numcodecs.shuffle",
                "configuration": {
                    "elementsize": 2
                }
            },
            {
                "name": "numcodecs.zlib",
                "configuration": {
                    "level": 1
                }
            }
        ]
    });

    let normalized = normalize_v3_array_metadata(raw_meta);
    let array_meta: Result<ArrayMetadata, _> = serde_json::from_value(normalized);
    assert!(
        array_meta.is_ok(),
        "ArrayMetadata should successfully parse normalized zlib and shuffle metadata: {:?}",
        array_meta.err()
    );
}

#[test]
fn test_parsed_cf_attributes_resolution() {
    let attrs_json = serde_json::json!({
        "units": "m/s",
        "long_name": "Wind Speed",
        "time_coverage_start": "2024-01-01T00:00:00Z",
        "time_coverage_end": "2024-01-02T00:00:00Z",
        "temporal_resolution": "PT1H",
        "_ARRAY_DIMENSIONS": ["time", "lat", "lon"],
        "custom_tag": 42
    });

    let cf_attrs = ParsedCfAttributes::from_json_map(attrs_json.as_object().unwrap());
    assert_eq!(cf_attrs.units.as_deref(), Some("m/s"));
    assert_eq!(cf_attrs.long_name.as_deref(), Some("Wind Speed"));
    assert_eq!(
        cf_attrs.time_coverage_start.as_deref(),
        Some("2024-01-01T00:00:00Z")
    );
    assert_eq!(
        cf_attrs.time_coverage_end.as_deref(),
        Some("2024-01-02T00:00:00Z")
    );
    assert_eq!(cf_attrs.temporal_resolution.as_deref(), Some("PT1H"));
    assert_eq!(
        cf_attrs.attributes.get("custom_tag").map(|s| s.as_str()),
        Some("42")
    );

    let dims = cf_attrs.resolve_dimension_names(None, 3);
    assert_eq!(dims, vec!["time", "lat", "lon"]);

    // Explicit names take precedence
    let explicit = vec![
        Some("t".to_string()),
        Some("y".to_string()),
        Some("x".to_string()),
    ];
    let dims_explicit = cf_attrs.resolve_dimension_names(Some(&explicit), 3);
    assert_eq!(dims_explicit, vec!["t", "y", "x"]);

    // Fallback when neither explicit nor _ARRAY_DIMENSIONS present
    let empty_cf = ParsedCfAttributes::default();
    let dims_fallback = empty_cf.resolve_dimension_names(None, 2);
    assert_eq!(dims_fallback, vec!["lat", "lon"]);
}
