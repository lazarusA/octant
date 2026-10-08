use std::collections::HashMap;

use octant::app::{OctantApp, SpatialRole};
use octant::data::coordinates::detect_grid_from_block;
use octant::data::coordinates::dggs::DggsMetadata;
use octant::data::coordinates::healpix::HealpixOrder;
use octant::data::coordinates::types::CoordinateGrid;
use octant::data::{OctantBlock, VariableInfo};
use octant::ui::variables_panel::dimension_slider::init_variable_dimension_defaults;

#[test]
fn test_dggs_metadata_parsing_from_json_string() {
    let dggs_json = r#"{
        "name": "healpix",
        "refinement_level": 6,
        "indexing_scheme": "nested",
        "spatial_dimension": "cells",
        "coordinate": "cell_ids",
        "compression": "none",
        "ellipsoid": {
            "name": "wgs84",
            "semi_major_axis": 6378137.0,
            "inverse_flattening": 298.257223563
        }
    }"#;

    let mut attrs = HashMap::new();
    attrs.insert("dggs".to_string(), dggs_json.to_string());

    let dggs = DggsMetadata::from_attributes(&attrs).expect("Should parse DGGS metadata");
    assert_eq!(dggs.name, "healpix");
    assert!(dggs.is_healpix());
    assert_eq!(dggs.refinement_level, Some(6));
    assert_eq!(dggs.healpix_nside(49152), Some(64));
    assert_eq!(dggs.healpix_ordering(), HealpixOrder::Nested);
    assert_eq!(dggs.spatial_dimension, "cells");
    assert_eq!(dggs.coordinate.as_deref(), Some("cell_ids"));
    assert_eq!(dggs.compression.as_deref(), Some("none"));

    let ellipsoid = dggs.ellipsoid.expect("Ellipsoid should be present");
    assert_eq!(ellipsoid.name, "wgs84");
    assert_eq!(ellipsoid.semi_major_axis, Some(6378137.0));
    assert_eq!(ellipsoid.inverse_flattening, Some(298.257223563));
}

#[test]
fn test_dggs_metadata_ring_ordering_and_level() {
    let dggs_json = r#"{
        "name": "healpix",
        "refinement_level": 10,
        "indexing_scheme": "ring",
        "spatial_dimension": "pixels"
    }"#;

    let mut attrs = HashMap::new();
    attrs.insert("dggs".to_string(), dggs_json.to_string());

    let dggs = DggsMetadata::from_attributes(&attrs).expect("Should parse DGGS metadata");
    assert_eq!(dggs.healpix_nside(0), Some(1024)); // 2^10 = 1024
    assert_eq!(dggs.healpix_ordering(), HealpixOrder::Ring);
    assert_eq!(dggs.spatial_dimension, "pixels");
}

#[test]
fn test_detect_grid_from_block_with_dggs() {
    let dggs_json = r#"{
        "name": "healpix",
        "refinement_level": 6,
        "indexing_scheme": "nested",
        "spatial_dimension": "cells"
    }"#;

    let mut attrs = HashMap::new();
    attrs.insert("dggs".to_string(), dggs_json.to_string());

    let block = OctantBlock::new(
        "measurements/aod/aod550".to_string(),
        vec![1, 49152],
        vec!["time".to_string(), "cells".to_string()],
        vec![0, 0],
        vec![0.5f32; 49152],
        HashMap::new(),
        attrs,
    );

    let grid = detect_grid_from_block(&block, "cells", "time", None, None, 49152, 1);
    match grid {
        CoordinateGrid::Healpix {
            nside,
            ordering,
            npix,
            ..
        } => {
            assert_eq!(nside, 64);
            assert_eq!(ordering, HealpixOrder::Nested);
            assert_eq!(npix, 49152);
        }
        _ => panic!("Expected CoordinateGrid::Healpix, got {:?}", grid),
    }
}

#[test]
fn test_dimension_slider_defaults_with_dggs_custom_dim_name() {
    let mut app = OctantApp::default();

    let dggs_json = r#"{
        "name": "healpix",
        "refinement_level": 6,
        "indexing_scheme": "nested",
        "spatial_dimension": "my_custom_cells"
    }"#;

    let mut attrs = HashMap::new();
    attrs.insert("dggs".to_string(), dggs_json.to_string());

    let var_info = VariableInfo {
        name: "measurements/aod/aod550".to_string(),
        data_type: "float32".to_string(),
        shape: vec![1, 49152],
        dimension_names: vec!["time".to_string(), "my_custom_cells".to_string()],
        chunk_shape: vec![1, 256],
        file_size: 49152 * 4,
        units: Some("1".to_string()),
        long_name: Some("Total AOD at 550 nm".to_string()),
        time_coverage_start: None,
        time_coverage_end: None,
        temporal_resolution: None,
        attributes: attrs,
    };

    init_variable_dimension_defaults(&mut app, &var_info);

    // Dimension 0 ("time") should be animated, Dimension 1 ("my_custom_cells") should be SpatialRole::Grid
    assert_eq!(app.selected.dim_config[1].spatial, SpatialRole::Grid);
    assert_eq!(app.selected.spatial_dims, vec![1]);
}

#[test]
fn test_consolidated_metadata_group_to_array_inheritance() {
    use octant::utils::metadata::extract_store_variables_from_consolidated_metadata;
    use zarrs::node::NodeMetadata;

    let group_meta_json = serde_json::json!({
        "zarr_format": 3,
        "node_type": "group",
        "attributes": {
            "dggs": {
                "name": "healpix",
                "refinement_level": 6,
                "indexing_scheme": "nested",
                "spatial_dimension": "cells",
                "coordinate": "cell_ids"
            }
        }
    });

    let array_meta_json = serde_json::json!({
        "zarr_format": 3,
        "node_type": "array",
        "shape": [1, 49152],
        "data_type": "float32",
        "chunk_grid": {
            "name": "regular",
            "configuration": {
                "chunk_shape": [1, 256]
            }
        },
        "chunk_key_encoding": {
            "name": "default",
            "configuration": {
                "separator": "/"
            }
        },
        "fill_value": 0.0,
        "codecs": [
            {
                "name": "bytes",
                "configuration": {
                    "endian": "little"
                }
            }
        ],
        "dimension_names": ["time", "cells"],
        "attributes": {
            "long_name": "Total AOD at 550 nm",
            "units": "1"
        }
    });

    let group_node: NodeMetadata =
        serde_json::from_value(group_meta_json).expect("Group NodeMetadata should parse");
    let array_node: NodeMetadata =
        serde_json::from_value(array_meta_json).expect("Array NodeMetadata should parse");

    let mut meta_map = HashMap::new();
    meta_map.insert("measurements/aod".to_string(), group_node);
    meta_map.insert("measurements/aod/aod550".to_string(), array_node);

    let variables = extract_store_variables_from_consolidated_metadata(&meta_map);
    assert_eq!(variables.len(), 1);

    let var = &variables[0];
    assert_eq!(var.name, "measurements/aod/aod550");
    assert_eq!(var.long_name.as_deref(), Some("Total AOD at 550 nm"));
    assert!(var.attributes.contains_key("dggs"));

    let dggs =
        DggsMetadata::from_attributes(&var.attributes).expect("Should inherit DGGS metadata");
    assert_eq!(dggs.name, "healpix");
    assert_eq!(dggs.refinement_level, Some(6));
    assert_eq!(dggs.spatial_dimension, "cells");
}

#[test]
fn test_consolidated_metadata_multi_level_group_inheritance() {
    use octant::utils::metadata::extract_store_variables_from_consolidated_metadata;
    use zarrs::node::NodeMetadata;

    // Root group has DGGS convention
    let root_meta_json = serde_json::json!({
        "zarr_format": 3,
        "node_type": "group",
        "attributes": {
            "dggs": {
                "name": "healpix",
                "refinement_level": 7,
                "indexing_scheme": "nested",
                "spatial_dimension": "cells"
            },
            "institution": "Octant Org"
        }
    });

    // Subgroup has dataset-level metadata
    let sub_meta_json = serde_json::json!({
        "zarr_format": 3,
        "node_type": "group",
        "attributes": {
            "dataset_version": "v1.2"
        }
    });

    let array_meta_json = serde_json::json!({
        "zarr_format": 3,
        "node_type": "array",
        "shape": [1, 49152],
        "data_type": "float32",
        "chunk_grid": {
            "name": "regular",
            "configuration": { "chunk_shape": [1, 256] }
        },
        "chunk_key_encoding": {
            "name": "default",
            "configuration": { "separator": "/" }
        },
        "fill_value": 0.0,
        "codecs": [{ "name": "bytes", "configuration": { "endian": "little" } }],
        "dimension_names": ["time", "cells"],
        "attributes": {
            "units": "K"
        }
    });

    let root_node: NodeMetadata = serde_json::from_value(root_meta_json).unwrap();
    let sub_node: NodeMetadata = serde_json::from_value(sub_meta_json).unwrap();
    let array_node: NodeMetadata = serde_json::from_value(array_meta_json).unwrap();

    let mut meta_map = HashMap::new();
    meta_map.insert("".to_string(), root_node);
    meta_map.insert("atmosphere/model_run_1".to_string(), sub_node);
    meta_map.insert("atmosphere/model_run_1/temp".to_string(), array_node);

    let variables = extract_store_variables_from_consolidated_metadata(&meta_map);
    assert_eq!(variables.len(), 1);

    let var = &variables[0];
    assert_eq!(var.name, "atmosphere/model_run_1/temp");
    assert_eq!(var.units.as_deref(), Some("K"));
    assert_eq!(
        var.attributes.get("dataset_version").map(|s| s.as_str()),
        Some("v1.2")
    );
    assert_eq!(
        var.attributes.get("institution").map(|s| s.as_str()),
        Some("Octant Org")
    );

    let dggs = DggsMetadata::from_attributes(&var.attributes)
        .expect("Should inherit DGGS from root group");
    assert_eq!(dggs.name, "healpix");
    assert_eq!(dggs.refinement_level, Some(7));
}
