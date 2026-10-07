//! WebAssembly inspection and metadata discovery for remote Icechunk repositories.

#[cfg(target_arch = "wasm32")]
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use icechunk_format::format_constants::SpecVersionBin;
#[cfg(target_arch = "wasm32")]
use icechunk_format::snapshot::{NodeData, Snapshot};

#[cfg(target_arch = "wasm32")]
use super::discovery::resolve_snapshot_id;
#[cfg(target_arch = "wasm32")]
use super::header::decompress_icechunk_file;
#[cfg(target_arch = "wasm32")]
use super::store::ArrayManifestInfo;
use super::store::WasmIcechunkBlockStore;
use crate::data::DatasetMetadata;
#[cfg(target_arch = "wasm32")]
use crate::data::VariableInfo;
#[cfg(target_arch = "wasm32")]
use crate::data::backends::http::fetch_url_bytes;
#[cfg(target_arch = "wasm32")]
use crate::utils::metadata::{open_or_instantiate_array_normalized, variable_info_from_array};
#[cfg(target_arch = "wasm32")]
use crate::utils::units::calculate_variable_size_bytes;

/// Asynchronously inspects a remote Icechunk repository in the browser and returns `DatasetMetadata`.
#[cfg(target_arch = "wasm32")]
#[allow(clippy::single_range_in_vec_init)]
pub async fn inspect_wasm_remote_icechunk(url: &str) -> Result<DatasetMetadata, String> {
    let clean_url = url.trim_start_matches("icechunk+").trim_end_matches('/');
    if clean_url.is_empty() {
        return Err("URL is empty".to_string());
    }

    let store = WasmIcechunkBlockStore::get_or_create(clean_url);
    let (snap_id, spec_version) = resolve_snapshot_id(clean_url).await?;

    log::info!("[WASM Icechunk] Resolved snapshot ID: {snap_id} (spec: {spec_version:?})");

    let snap_url = format!("{clean_url}/snapshots/{snap_id}");
    let raw_snap_bytes = fetch_url_bytes(&snap_url)
        .await
        .map_err(|e| format!("Failed fetching snapshot from '{snap_url}': {e}"))?;

    let (snap_spec_version, decomp_snap) = decompress_icechunk_file(&raw_snap_bytes)
        .map_err(|e| format!("Failed decompressing snapshot: {e}"))?;

    let actual_version =
        if snap_spec_version == SpecVersionBin::V1 && spec_version != SpecVersionBin::V1 {
            spec_version
        } else {
            snap_spec_version
        };

    let snapshot = Snapshot::from_buffer(actual_version, decomp_snap)
        .map_err(|e| format!("Failed parsing Icechunk snapshot FlatBuffers: {e:?}"))?;

    let mut variables = Vec::new();
    let mut manifest_map = HashMap::new();

    for node_res in snapshot.iter() {
        let node = node_res.map_err(|e| format!("Failed iterating snapshot nodes: {e:?}"))?;
        let node_path = node.path.to_string();
        let clean_path = node_path.trim_matches('/');

        if !node.user_data.is_empty() {
            let meta_key = if clean_path.is_empty() {
                "zarr.json".to_string()
            } else {
                format!("{clean_path}/zarr.json")
            };
            let _ = store
                .inner
                .insert_key_bytes(&meta_key, node.user_data.as_ref());
        }

        match node.node_data {
            NodeData::Array {
                shape,
                dimension_names,
                manifests,
            } => {
                let var_name = if clean_path.is_empty() {
                    "data".to_string()
                } else {
                    clean_path.to_string()
                };

                manifest_map.insert(
                    var_name.clone(),
                    ArrayManifestInfo {
                        node_id: node.id,
                        manifests,
                    },
                );

                let dim_names_vec: Vec<String> = dimension_names
                    .map(|dims| {
                        dims.into_iter()
                            .map(|d| match d {
                                icechunk_format::snapshot::DimensionName::Name(n) => n,
                                icechunk_format::snapshot::DimensionName::NotSpecified => {
                                    String::new()
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let shape_u64: Vec<u64> = shape.iter().map(|d| d.array_length()).collect();

                let mut var_info_opt = None;
                if let Ok(arr) = open_or_instantiate_array_normalized(
                    store.inner.memory_store.clone(),
                    &format!("/{clean_path}"),
                ) {
                    var_info_opt = variable_info_from_array(&arr, &var_name);
                }

                if let Some(mut var_info) = var_info_opt {
                    if !dim_names_vec.is_empty() && var_info.dimension_names.is_empty() {
                        var_info.dimension_names = dim_names_vec;
                    }
                    variables.push(var_info);
                } else {
                    let data_type = "float32".to_string();
                    let file_size = calculate_variable_size_bytes(&shape_u64, &data_type);

                    variables.push(VariableInfo {
                        name: var_name,
                        data_type,
                        shape: shape_u64.clone(),
                        dimension_names: dim_names_vec,
                        chunk_shape: shape_u64,
                        file_size,
                        units: None,
                        long_name: None,
                        time_coverage_start: None,
                        time_coverage_end: None,
                        temporal_resolution: None,
                        attributes: HashMap::new(),
                    });
                }
            }
            NodeData::Group => {}
        }
    }

    *store
        .array_manifests
        .write()
        .unwrap_or_else(|p| p.into_inner()) = manifest_map;

    // Preload 1D coordinate arrays to populate dimension_coordinates
    let coord_candidates =
        crate::data::backends::coord_bounds::collect_coordinate_candidates(&variables);

    for coord_name in &coord_candidates {
        if let Some(var_info) = variables.iter().find(|v| &v.name == coord_name) {
            let count = var_info.shape.first().copied().unwrap_or(0);
            store.preload_coordinate_chunks_1d(coord_name, count).await;
        }
    }

    let dimension_coordinates =
        crate::data::backends::coord_bounds::fetch_all_dimension_coordinates_for_variables(
            store.inner.memory_store.clone(),
            &variables,
            Some(clean_url),
        );

    let dataset_name = clean_url
        .split('/')
        .next_back()
        .unwrap_or(clean_url)
        .to_string();

    let dataset_metadata = DatasetMetadata {
        name: dataset_name,
        store_type: "icechunk".to_string(),
        variables,
        dimension_coordinates,
    };

    *store
        .inner
        .metadata
        .write()
        .unwrap_or_else(|p| p.into_inner()) = Some(dataset_metadata.clone());

    Ok(dataset_metadata)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn inspect_wasm_remote_icechunk(url: &str) -> Result<DatasetMetadata, String> {
    use crate::data::blocks::BlockStore;
    let clean_url = url.trim_start_matches("icechunk+").trim_end_matches('/');
    let store = WasmIcechunkBlockStore::get_or_create(clean_url);
    store.inner.inspect().map_err(|e| e.to_string())
}
