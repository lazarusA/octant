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

#[test]
fn test_ome_ngff_v03_remote_dataset_attrs() {
    let raw_attrs = serde_json::json!({
        "_creator": {
            "name": "omero-zarr",
            "version": "0.1.dev219+g541c88e"
        },
        "multiscales": [
            {
                "axes": ["c", "z", "y", "x"],
                "datasets": [
                    { "path": "0" },
                    { "path": "1" },
                    { "path": "2" }
                ],
                "version": "0.3"
            }
        ],
        "omero": {
            "channels": [
                {
                    "active": true,
                    "color": "0000FF",
                    "label": "LaminB1",
                    "window": { "min": 0.0, "max": 65535.0, "start": 0.0, "end": 1500.0 }
                },
                {
                    "active": true,
                    "color": "FFFF00",
                    "label": "Dapi",
                    "window": { "min": 0.0, "max": 65535.0, "start": 0.0, "end": 1500.0 }
                }
            ],
            "id": 1,
            "rdefs": {
                "defaultT": 0,
                "defaultZ": 118,
                "model": "color"
            },
            "version": "0.3"
        }
    });

    let map = raw_attrs.as_object().unwrap().clone();
    let normalized = super::ome::normalize_ngff_attributes(map);
    let root_zattrs: Result<super::ome::RootZattrs, _> =
        serde_json::from_value(serde_json::Value::Object(normalized));

    assert!(root_zattrs.is_ok());
    let root = root_zattrs.unwrap();
    assert_eq!(root.multiscales.len(), 1);
    let ms = &root.multiscales[0];
    let axes: Vec<String> = ms.axes.iter().map(|a| a.name.clone()).collect();
    assert_eq!(axes, vec!["c", "z", "y", "x"]);
    assert_eq!(ms.datasets.len(), 3);
    assert_eq!(ms.datasets[0].path, "0");

    let omero = root.omero.expect("omero");
    assert_eq!(omero.channels.len(), 2);
    assert_eq!(omero.channels[0].label.as_deref(), Some("LaminB1"));
    assert_eq!(omero.channels[1].label.as_deref(), Some("Dapi"));
    assert_eq!(omero.rdefs.as_ref().and_then(|r| r.default_z), Some(118));
}

#[test]
fn test_open_checked_in_synthetic_5ch_ome_zarr_fixture() {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("dev/odon/fixtures/synthetic_5ch.ome.zarr");

    if !fixture_path.exists() {
        return;
    }

    let store =
        crate::data::backends::zarr::ZarrBlockStore::open_local(fixture_path.to_str().unwrap())
            .expect("open local synthetic_5ch.ome.zarr");

    use crate::data::blocks::BlockStore;
    let vars = store.variables().expect("list variables");
    assert_eq!(vars, vec!["0", "1", "2", "3"]);

    let meta = store.inspect().expect("inspect dataset");
    assert_eq!(meta.variables.len(), 4);

    let var0 = &meta.variables[0];
    assert_eq!(var0.name, "0");
    assert_eq!(var0.shape, vec![5, 512, 512]);
    assert_eq!(var0.dimension_names, vec!["c", "y", "x"]);
    assert_eq!(
        var0.attributes.get("omero_channels").map(|s| s.as_str()),
        Some("DAPI,CD3,PanCK,Ki67,Collagen")
    );

    // Fetch a 2D block
    use crate::data::slice_request::DimensionSelection;
    let req = crate::data::slice_request::SliceRequest::new(
        "0",
        vec![
            DimensionSelection::range(0, 5),
            DimensionSelection::range(0, 64),
            DimensionSelection::range(0, 64),
        ],
    );
    let block = store.fetch_block(&req).expect("fetch block");
    assert_eq!(block.shape, vec![5, 64, 64]);

    // Test RGB composite slicing on this 5-channel block
    let composite = crate::data::slicing::slice_rgb_composite(&block, [0, 1, 2], 1);
    assert!(composite.is_some());
    let Some(mdata) = composite else {
        panic!("RGB composite slice failed");
    };
    assert_eq!(mdata.width, 64);
    assert_eq!(mdata.height, 64);
    assert_eq!(mdata.values.len(), 64 * 64);

    // Test Multi-Channel 5-color additive overlay slicing
    let configs = vec![
        crate::data::slicing::ChannelColorConfig::new(0, "DAPI".to_string(), [0, 0, 255]),
        crate::data::slicing::ChannelColorConfig::new(1, "CD3".to_string(), [0, 255, 0]),
        crate::data::slicing::ChannelColorConfig::new(2, "PanCK".to_string(), [255, 0, 0]),
        crate::data::slicing::ChannelColorConfig::new(3, "Ki67".to_string(), [255, 255, 0]),
        crate::data::slicing::ChannelColorConfig::new(4, "Collagen".to_string(), [255, 0, 255]),
    ];

    let mc_composite = crate::data::slicing::slice_multichannel_composite_nd(
        &block,
        0,
        2,
        1,
        (0, 64),
        (0, 64),
        &[0, 0, 0],
        &configs,
        1,
    );

    assert!(mc_composite.is_some());
    let Some(mc_mdata) = mc_composite else {
        panic!("Multi-channel composite slice failed");
    };
    assert_eq!(mc_mdata.width, 64);
    assert_eq!(mc_mdata.height, 64);
    assert_eq!(mc_mdata.values.len(), 64 * 64);
}
