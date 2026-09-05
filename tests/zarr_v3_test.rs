use octant::data::backends::zarr::ZarrBlockStore;
use octant::data::block_store::BlockStore;
use octant::data::slice_request::{DimensionSelection, SliceRequest};
use std::sync::Arc;
use zarrs::array::{ArrayBuilder, ArraySubset, FillValue};
use zarrs::filesystem::FilesystemStore;
use zarrs::group::GroupBuilder;

#[test]
fn test_v3_inline_consolidated_metadata() {
    let temp_path = std::env::temp_dir().join(format!(
        "test_zarr_v3_inline_cons_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_path).unwrap();
    let store_path = temp_path.to_str().unwrap();

    let json_content = r#"{
  "attributes": {},
  "zarr_format": 3,
  "consolidated_metadata": {
    "kind": "inline",
    "must_understand": false,
    "metadata": {
      "eais": {
        "shape": [5, 10, 20, 30],
        "data_type": "float32",
        "chunk_grid": {
          "name": "regular",
          "configuration": {
            "chunk_shape": [5, 10, 10, 10]
          }
        },
        "chunk_key_encoding": {
          "name": "default",
          "configuration": {
            "separator": "/"
          }
        },
        "fill_value": "NaN",
        "codecs": [
          {
            "name": "bytes",
            "configuration": {
              "endian": "little"
            }
          },
          {
            "name": "numcodecs.blosc",
            "configuration": {
              "cname": "zstd",
              "clevel": 5,
              "shuffle": 2
            }
          }
        ],
        "attributes": {
          "units": "m",
          "long_name": "Regional eais sea-level projections"
        },
        "dimension_names": [
          "percentile",
          "time",
          "lat",
          "lon"
        ],
        "zarr_format": 3,
        "node_type": "array"
      },
      "lat": {
        "shape": [20],
        "data_type": "float64",
        "chunk_grid": {
          "name": "regular",
          "configuration": {
            "chunk_shape": [20]
          }
        },
        "chunk_key_encoding": {
          "name": "default",
          "configuration": {
            "separator": "/"
          }
        },
        "fill_value": "NaN",
        "codecs": [
          {
            "name": "bytes",
            "configuration": {
              "endian": "little"
            }
          }
        ],
        "attributes": {},
        "dimension_names": ["lat"],
        "zarr_format": 3,
        "node_type": "array"
      }
    }
  },
  "node_type": "group"
}"#;

    std::fs::write(temp_path.join("zarr.json"), json_content).unwrap();

    let block_store = ZarrBlockStore::open_local(store_path).expect("open_local should succeed");
    let metadata = block_store.inspect().expect("inspect should succeed");

    assert_eq!(
        metadata.variables.len(),
        2,
        "Should discover 2 variables from inline consolidated metadata"
    );
    let names: Vec<&str> = metadata.variables.iter().map(|v| v.name.as_str()).collect();
    assert!(names.contains(&"eais"));
    assert!(names.contains(&"lat"));

    let eais_var = metadata
        .variables
        .iter()
        .find(|v| v.name == "eais")
        .unwrap();
    assert_eq!(
        eais_var.dimension_names,
        vec!["percentile", "time", "lat", "lon"]
    );
    assert_eq!(eais_var.units.as_deref(), Some("m"));

    let _ = std::fs::remove_dir_all(temp_path);
}

#[test]
fn test_local_zarr_v3_nested_groups_and_data_types() {
    let temp_path = std::env::temp_dir().join(format!(
        "test_zarr_v3_nested_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_path).unwrap();
    let store_path = temp_path.to_str().unwrap();
    let store = Arc::new(FilesystemStore::new(store_path).unwrap());

    GroupBuilder::new()
        .build(store.clone(), "/")
        .unwrap()
        .store_metadata()
        .unwrap();
    GroupBuilder::new()
        .build(store.clone(), "/sensors")
        .unwrap()
        .store_metadata()
        .unwrap();
    GroupBuilder::new()
        .build(store.clone(), "/sensors/met")
        .unwrap()
        .store_metadata()
        .unwrap();

    let arr_temp = ArrayBuilder::new(
        vec![10, 20],
        vec![5, 10],
        "float32",
        FillValue::from(0.0f32),
    )
    .dimension_names(Some(vec![Some("lat".to_string()), Some("lon".to_string())]))
    .build(store.clone(), "/sensors/met/temperature")
    .unwrap();
    arr_temp.store_metadata().unwrap();

    let data_temp: Vec<f32> = (0..(10 * 20)).map(|v| v as f32).collect();
    arr_temp
        .store_array_subset(&ArraySubset::new_with_shape(vec![10, 20]), &data_temp)
        .unwrap();

    let arr_mask = ArrayBuilder::new(vec![10, 20], vec![5, 10], "uint8", FillValue::from(0u8))
        .build(store.clone(), "/sensors/met/quality_mask")
        .unwrap();
    arr_mask.store_metadata().unwrap();

    let data_mask: Vec<u8> = vec![1u8; 10 * 20];
    arr_mask
        .store_array_subset(&ArraySubset::new_with_shape(vec![10, 20]), &data_mask)
        .unwrap();

    // Open via file:// and inspect
    let file_url = format!("file://{}", store_path);
    let block_store =
        ZarrBlockStore::open_local(&file_url).expect("open_local with file:// should succeed");
    let metadata = block_store.inspect().expect("inspect should succeed");

    assert_eq!(
        metadata.variables.len(),
        2,
        "Should discover 2 nested variables"
    );

    let req_temp = SliceRequest {
        variable: "sensors/met/temperature".to_string(),
        selections: vec![
            DimensionSelection::Range { start: 0, end: 5 },
            DimensionSelection::Range { start: 0, end: 10 },
        ],
    };
    let block_temp = block_store
        .fetch_block(&req_temp)
        .expect("fetch_block for float32 should succeed");
    assert_eq!(block_temp.shape, vec![5, 10]);

    let req_mask = SliceRequest {
        variable: "sensors/met/quality_mask".to_string(),
        selections: vec![
            DimensionSelection::Range { start: 0, end: 5 },
            DimensionSelection::Range { start: 0, end: 10 },
        ],
    };
    let block_mask = block_store
        .fetch_block(&req_mask)
        .expect("fetch_block for uint8 should succeed");
    assert_eq!(block_mask.shape, vec![5, 10]);
    assert_eq!(block_mask.values[0], 1.0);

    let _ = std::fs::remove_dir_all(temp_path);
}

#[test]
fn test_local_zarr_v3_root_array() {
    let temp_path = std::env::temp_dir().join(format!(
        "test_zarr_v3_root_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_path).unwrap();
    let store_path = temp_path.to_str().unwrap();
    let store = Arc::new(FilesystemStore::new(store_path).unwrap());

    let array = ArrayBuilder::new(vec![5, 10], vec![5, 5], "float32", FillValue::from(0.0f32))
        .build(store.clone(), "/")
        .unwrap();
    array.store_metadata().unwrap();

    let data: Vec<f32> = (0..50).map(|v| v as f32).collect();
    array
        .store_array_subset(&ArraySubset::new_with_shape(vec![5, 10]), &data)
        .unwrap();

    // Path pointing directly to zarr.json file
    let zarr_json_path = format!("{}/zarr.json", store_path);
    let block_store = ZarrBlockStore::open_local(&zarr_json_path)
        .expect("open_local with zarr.json path should succeed");
    let metadata = block_store.inspect().expect("inspect should succeed");

    assert_eq!(metadata.variables.len(), 1);
    assert_eq!(metadata.variables[0].name, "data");

    let req = SliceRequest {
        variable: "data".to_string(),
        selections: vec![
            DimensionSelection::Range { start: 0, end: 5 },
            DimensionSelection::Range { start: 0, end: 10 },
        ],
    };
    let block = block_store
        .fetch_block(&req)
        .expect("fetch_block on root array with 'data' should succeed");
    assert_eq!(block.values.len(), 50);

    let _ = std::fs::remove_dir_all(temp_path);
}

#[test]
fn test_local_zarr_v3_nested_only_no_root_variables() {
    let temp_path = std::env::temp_dir().join(format!(
        "test_zarr_v3_nested_only_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_path).unwrap();
    let store_path = temp_path.to_str().unwrap();
    let store = Arc::new(FilesystemStore::new(store_path).unwrap());

    // Create root group
    GroupBuilder::new()
        .build(store.clone(), "/")
        .unwrap()
        .store_metadata()
        .unwrap();

    // Create nested groups: /atmosphere/forecast and /ocean
    GroupBuilder::new()
        .build(store.clone(), "/atmosphere")
        .unwrap()
        .store_metadata()
        .unwrap();
    GroupBuilder::new()
        .build(store.clone(), "/atmosphere/forecast")
        .unwrap()
        .store_metadata()
        .unwrap();
    GroupBuilder::new()
        .build(store.clone(), "/ocean")
        .unwrap()
        .store_metadata()
        .unwrap();

    // Atmosphere nested arrays: /atmosphere/forecast/u_wind with scoped lat & lon
    let arr_wind = ArrayBuilder::new(vec![4, 8], vec![4, 8], "float32", FillValue::from(1.5f32))
        .dimension_names(Some(vec![Some("lat".to_string()), Some("lon".to_string())]))
        .build(store.clone(), "/atmosphere/forecast/u_wind")
        .unwrap();
    arr_wind.store_metadata().unwrap();
    let data_wind = vec![1.5f32; 32];
    arr_wind
        .store_array_subset(&ArraySubset::new_with_shape(vec![4, 8]), &data_wind)
        .unwrap();

    // Nested coordinates: /atmosphere/forecast/lat & /atmosphere/forecast/lon
    let arr_lat = ArrayBuilder::new(vec![4], vec![4], "float32", FillValue::from(0.0f32))
        .build(store.clone(), "/atmosphere/forecast/lat")
        .unwrap();
    arr_lat.store_metadata().unwrap();
    let data_lat = vec![-30.0f32, -10.0f32, 10.0f32, 30.0f32];
    arr_lat
        .store_array_subset(&ArraySubset::new_with_shape(vec![4]), &data_lat)
        .unwrap();

    // Ocean nested array: /ocean/temperature
    let arr_ocean = ArrayBuilder::new(vec![6, 6], vec![6, 6], "float32", FillValue::from(20.0f32))
        .build(store.clone(), "/ocean/temperature")
        .unwrap();
    arr_ocean.store_metadata().unwrap();
    let data_ocean = vec![20.0f32; 36];
    arr_ocean
        .store_array_subset(&ArraySubset::new_with_shape(vec![6, 6]), &data_ocean)
        .unwrap();

    let block_store = ZarrBlockStore::open_local(store_path).expect("open_local should succeed");
    let metadata = block_store.inspect().expect("inspect should succeed");

    assert_eq!(
        metadata.variables.len(),
        3,
        "Should discover 3 nested variables"
    );

    // Verify group-scoped coordinate resolution for nested lat
    assert!(
        metadata.dimension_coordinates.contains_key("lat"),
        "Should resolve 'lat' coordinate bounds from nested group"
    );
    let lat_bounds = metadata.dimension_coordinates.get("lat").unwrap();
    assert_eq!(lat_bounds, &vec!["-30".to_string(), "30".to_string()]);

    let tree = metadata.build_variable_tree();
    assert!(
        tree.variable_indices.is_empty(),
        "Root should have no variables"
    );
    assert_eq!(
        tree.subgroups.len(),
        2,
        "Two top-level groups: atmosphere and ocean"
    );
    assert_eq!(tree.total_variable_count(), 3);

    // Fetch block from nested path
    let req = SliceRequest {
        variable: "atmosphere/forecast/u_wind".to_string(),
        selections: vec![
            DimensionSelection::Range { start: 0, end: 4 },
            DimensionSelection::Range { start: 0, end: 8 },
        ],
    };
    let block = block_store
        .fetch_block(&req)
        .expect("fetch_block from nested path should succeed");
    assert_eq!(block.values[0], 1.5);

    let _ = std::fs::remove_dir_all(temp_path);
}

#[test]
fn test_local_zarr_v3_mixed_root_and_nested() {
    let temp_path = std::env::temp_dir().join(format!(
        "test_zarr_v3_mixed_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp_path).unwrap();
    let store_path = temp_path.to_str().unwrap();
    let store = Arc::new(FilesystemStore::new(store_path).unwrap());

    // Root group
    GroupBuilder::new()
        .build(store.clone(), "/")
        .unwrap()
        .store_metadata()
        .unwrap();

    // Root-level array: /elevation
    let arr_root = ArrayBuilder::new(
        vec![10, 10],
        vec![5, 5],
        "float32",
        FillValue::from(100.0f32),
    )
    .build(store.clone(), "/elevation")
    .unwrap();
    arr_root.store_metadata().unwrap();
    let data_root = vec![150.0f32; 100];
    arr_root
        .store_array_subset(&ArraySubset::new_with_shape(vec![10, 10]), &data_root)
        .unwrap();

    // Nested array: /models/gfs/temperature
    GroupBuilder::new()
        .build(store.clone(), "/models")
        .unwrap()
        .store_metadata()
        .unwrap();
    GroupBuilder::new()
        .build(store.clone(), "/models/gfs")
        .unwrap()
        .store_metadata()
        .unwrap();
    let arr_nested = ArrayBuilder::new(vec![2, 4], vec![2, 4], "float32", FillValue::from(0.0f32))
        .build(store.clone(), "/models/gfs/temperature")
        .unwrap();
    arr_nested.store_metadata().unwrap();
    let data_nested = vec![273.15f32; 8];
    arr_nested
        .store_array_subset(&ArraySubset::new_with_shape(vec![2, 4]), &data_nested)
        .unwrap();

    let block_store = ZarrBlockStore::open_local(store_path).expect("open_local should succeed");
    let metadata = block_store.inspect().expect("inspect should succeed");

    assert_eq!(
        metadata.variables.len(),
        2,
        "Should discover 1 root variable and 1 nested variable"
    );

    let tree = metadata.build_variable_tree();
    assert_eq!(
        tree.variable_indices.len(),
        1,
        "Root should have 1 variable"
    );
    assert_eq!(
        tree.subgroups.len(),
        1,
        "Root should have 1 subgroup: models"
    );
    assert_eq!(tree.total_variable_count(), 2);

    let req_root = SliceRequest {
        variable: "elevation".to_string(),
        selections: vec![
            DimensionSelection::Range { start: 0, end: 10 },
            DimensionSelection::Range { start: 0, end: 10 },
        ],
    };
    let block_root = block_store.fetch_block(&req_root).unwrap();
    assert_eq!(block_root.values[0], 150.0);

    let req_nested = SliceRequest {
        variable: "models/gfs/temperature".to_string(),
        selections: vec![
            DimensionSelection::Range { start: 0, end: 2 },
            DimensionSelection::Range { start: 0, end: 4 },
        ],
    };
    let block_nested = block_store.fetch_block(&req_nested).unwrap();
    assert_eq!(block_nested.values[0], 273.15);

    let _ = std::fs::remove_dir_all(temp_path);
}
