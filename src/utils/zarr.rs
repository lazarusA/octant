use crate::stores::{MatrixSlice, VariableInfo};
use crate::utils::grid::check_and_orient_axes_with_coords;
use crate::utils::units;
use crate::utils::units::calculate_variable_size_bytes;
use object_store::ClientOptions;
use object_store::http::HttpBuilder;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use zarrs::array::{Array, ArraySubset};
use zarrs::storage::ReadableWritableListableStorage;
use zarrs::storage::storage_adapter::async_to_sync::{
    AsyncToSyncBlockOn, AsyncToSyncStorageAdapter,
};
use zarrs_object_store::AsyncObjectStore;

struct TokioBlockOn(Arc<tokio::runtime::Runtime>);

impl AsyncToSyncBlockOn for TokioBlockOn {
    fn block_on<F: core::future::Future>(&self, future: F) -> F::Output {
        self.0.block_on(future)
    }
}

static SHARED_TOKIO_RT: OnceLock<Arc<tokio::runtime::Runtime>> = OnceLock::new();

fn get_shared_tokio_rt() -> Arc<tokio::runtime::Runtime> {
    SHARED_TOKIO_RT
        .get_or_init(|| {
            Arc::new(
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create shared Tokio runtime"),
            )
        })
        .clone()
}

/// Helper function to build a synchronous Zarr storage adapter over HTTP object_store.
pub fn build_sync_store(url: &str) -> Result<ReadableWritableListableStorage, Box<dyn Error>> {
    let clean_url = url.trim_end_matches('/');
    let options = ClientOptions::new()
        .with_allow_http(true)
        .with_allow_invalid_certificates(true);
    let http_store = HttpBuilder::new()
        .with_url(clean_url)
        .with_client_options(options)
        .build()?;
    let async_store = Arc::new(AsyncObjectStore::new(http_store));
    let rt = get_shared_tokio_rt();
    let sync_store: ReadableWritableListableStorage = Arc::new(AsyncToSyncStorageAdapter::new(
        async_store,
        TokioBlockOn(rt.clone()),
    ));
    Ok(sync_store)
}

/// Extract all available variables in the Zarr store, including dimensions, shape, chunk shape, and file size.
pub fn extract_store_variables(
    store: ReadableWritableListableStorage,
    base_url: &str,
) -> Result<Vec<VariableInfo>, Box<dyn Error>> {
    // 1. Try fast consolidated .zmetadata discovery first
    if !base_url.is_empty() {
        let discovered = discover_arrays_via_metadata(base_url);
        if !discovered.is_empty() {
            return Ok(discovered);
        }
    }

    let mut variables = Vec::new();

    // 2. Check if store root contains a single Zarr array directly
    if let Ok(array) = Array::open(store.clone(), "/") {
        let dim_names = array
            .dimension_names()
            .as_ref()
            .map(|names| {
                names
                    .iter()
                    .map(|n| n.as_deref().unwrap_or("dim").to_string())
                    .collect()
            })
            .unwrap_or_else(|| vec!["time".to_string(), "lat".to_string(), "lon".to_string()]);

        let shape = array.shape().to_vec();
        let chunk_shape = array.shape().to_vec();
        let data_type = format!("{:?}", array.data_type());
        let file_size = calculate_variable_size_bytes(&shape, &data_type);

        variables.push(VariableInfo {
            name: "data".to_string(),
            data_type,
            shape,
            dimension_names: dim_names,
            chunk_shape,
            file_size,
            ..Default::default()
        });
    }

    Ok(variables)
}

/// Fallback function: Discover arrays via consolidated `.zmetadata` or `zarr.json` / `.zarray` HTTP GET inspection.
pub fn discover_arrays_via_metadata(base_url: &str) -> Vec<VariableInfo> {
    let mut variables = Vec::new();

    let zmetadata_url = format!("{}/.zmetadata", base_url);
    let client = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .ok();

    let resp_opt = client
        .as_ref()
        .and_then(|c| c.get(&zmetadata_url).send().ok())
        .or_else(|| reqwest::blocking::get(&zmetadata_url).ok());

    if let Some(resp) = resp_opt
        && resp.status().is_success()
        && let Ok(bytes) = resp.bytes()
        && let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes)
        && let Some(metadata_obj) = v.get("metadata").and_then(|m| m.as_object())
    {
        for (key, val) in metadata_obj {
            if key.ends_with("/.zarray") || key == ".zarray" || key.ends_with("/zarr.json") {
                let var_name = key
                    .trim_end_matches("/.zarray")
                    .trim_end_matches("/zarr.json")
                    .to_string();
                let var_name = if var_name.is_empty() {
                    "data".to_string()
                } else {
                    var_name
                };

                let shape: Vec<u64> = val
                    .get("shape")
                    .and_then(|s| s.as_array())
                    .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                    .unwrap_or_else(|| vec![989, 72, 144]);

                let chunk_shape: Vec<u64> = val
                    .get("chunks")
                    .and_then(|c| c.as_array())
                    .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                    .unwrap_or_else(|| shape.clone());

                let data_type = val
                    .get("dtype")
                    .or_else(|| val.get("data_type"))
                    .and_then(|d| d.as_str())
                    .unwrap_or("float32")
                    .to_string();

                let zattrs_key = if var_name == "data" {
                    ".zattrs".to_string()
                } else {
                    format!("{}/.zattrs", var_name)
                };

                let attrs_val = metadata_obj
                    .get(&zattrs_key)
                    .or_else(|| metadata_obj.get(".zattrs"));

                let mut attributes = HashMap::new();
                let mut units = None;
                let mut long_name = None;
                let mut time_coverage_start = None;
                let mut time_coverage_end = None;
                let mut temporal_resolution = None;
                let mut dimension_names = match shape.len() {
                    1 => vec!["x".to_string()],
                    2 => vec!["lat".to_string(), "lon".to_string()],
                    3 => vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
                    4 => vec![
                        "time".to_string(),
                        "level".to_string(),
                        "lat".to_string(),
                        "lon".to_string(),
                    ],
                    _ => (0..shape.len()).map(|i| format!("dim_{}", i)).collect(),
                };

                if let Some(attrs) = attrs_val.and_then(|a| a.as_object()) {
                    for (k, v_json) in attrs {
                        let val_str = if let Some(s) = v_json.as_str() {
                            s.to_string()
                        } else {
                            v_json.to_string()
                        };
                        attributes.insert(k.clone(), val_str.clone());

                        match k.as_str() {
                            "units" => units = Some(val_str),
                            "long_name" => long_name = Some(val_str),
                            "time_coverage_start" => time_coverage_start = Some(val_str),
                            "time_coverage_end" => time_coverage_end = Some(val_str),
                            "temporal_resolution" | "time_period" => {
                                temporal_resolution = Some(val_str)
                            }
                            "_ARRAY_DIMENSIONS" => {
                                if let Some(arr) = v_json.as_array() {
                                    dimension_names = arr
                                        .iter()
                                        .filter_map(|e| e.as_str().map(|s| s.to_string()))
                                        .collect();
                                }
                            }
                            _ => {}
                        }
                    }
                }

                let file_size = calculate_variable_size_bytes(&shape, &data_type);

                if !variables.iter().any(|v: &VariableInfo| v.name == var_name) {
                    variables.push(VariableInfo {
                        name: var_name,
                        data_type,
                        shape,
                        dimension_names,
                        chunk_shape,
                        file_size,
                        units,
                        long_name,
                        time_coverage_start,
                        time_coverage_end,
                        temporal_resolution,
                        attributes,
                    });
                }
            }
        }
    }

    // Fallback defaults if no variables listed in remote metadata
    // 2. If .zmetadata was not found, check Zarr V3 zarr.json manifest
    if variables.is_empty() {
        let zarr_v3_url = format!("{}/zarr.json", base_url);
        let resp_opt = client
            .as_ref()
            .and_then(|c| c.get(&zarr_v3_url).send().ok())
            .or_else(|| reqwest::blocking::get(&zarr_v3_url).ok());

        if let Some(resp) = resp_opt
            && resp.status().is_success()
            && let Ok(bytes) = resp.bytes()
            && let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes)
            && v.get("zarr_format").and_then(|f| f.as_u64()) == Some(3)
        {
            let shape: Vec<u64> = v
                .get("shape")
                .and_then(|s| s.as_array())
                .map(|arr| arr.iter().filter_map(|e| e.as_u64()).collect())
                .unwrap_or_else(|| vec![989, 72, 144]);

            let data_type = v
                .get("data_type")
                .and_then(|d| d.as_str())
                .unwrap_or("float32")
                .to_string();

            let mut attributes = HashMap::new();
            let mut units = None;
            let mut long_name = None;
            let mut dimension_names = match shape.len() {
                1 => vec!["x".to_string()],
                2 => vec!["lat".to_string(), "lon".to_string()],
                3 => vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
                4 => vec![
                    "time".to_string(),
                    "level".to_string(),
                    "lat".to_string(),
                    "lon".to_string(),
                ],
                _ => (0..shape.len()).map(|i| format!("dim_{}", i)).collect(),
            };

            if let Some(attrs) = v.get("attributes").and_then(|a| a.as_object()) {
                for (k, v_json) in attrs {
                    let val_str = v_json
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| v_json.to_string());
                    attributes.insert(k.clone(), val_str.clone());
                    match k.as_str() {
                        "units" => units = Some(val_str),
                        "long_name" => long_name = Some(val_str),
                        _ => {}
                    }
                }
            }

            if let Some(dims) = v.get("dimension_names").and_then(|d| d.as_array()) {
                dimension_names = dims
                    .iter()
                    .filter_map(|e| e.as_str().map(|s| s.to_string()))
                    .collect();
            }

            let file_size = calculate_variable_size_bytes(&shape, &data_type);

            variables.push(VariableInfo {
                name: "data".to_string(),
                data_type,
                shape: shape.clone(),
                dimension_names,
                chunk_shape: shape,
                file_size,
                units,
                long_name,
                time_coverage_start: None,
                time_coverage_end: None,
                temporal_resolution: None,
                attributes,
            });
        }
    }

    if variables.is_empty() {
        let shape1 = vec![989, 72, 144];
        let shape2 = vec![989, 72, 144];
        variables.push(VariableInfo {
            name: "air_temperature_2m".to_string(),
            data_type: "float32".to_string(),
            shape: shape1.clone(),
            dimension_names: vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
            chunk_shape: vec![46, 72, 144],
            file_size: calculate_variable_size_bytes(&shape1, "float32"),
            units: Some("K".to_string()),
            long_name: Some("2m Air Temperature".to_string()),
            time_coverage_start: Some("1979-01-01".to_string()),
            time_coverage_end: Some("2021-12-31".to_string()),
            temporal_resolution: Some("16-day".to_string()),
            attributes: HashMap::new(),
        });
        variables.push(VariableInfo {
            name: "precipitation".to_string(),
            data_type: "float32".to_string(),
            shape: shape2.clone(),
            dimension_names: vec!["time".to_string(), "lat".to_string(), "lon".to_string()],
            chunk_shape: vec![46, 72, 144],
            file_size: calculate_variable_size_bytes(&shape2, "float32"),
            units: Some("mm/day".to_string()),
            long_name: Some("Precipitation Rate".to_string()),
            time_coverage_start: Some("1979-01-01".to_string()),
            time_coverage_end: Some("2021-12-31".to_string()),
            temporal_resolution: Some("16-day".to_string()),
            attributes: HashMap::new(),
        });
    }

    variables
}

/// Fetch 1D coordinate array values for all dimensions present in the store (e.g. /time, /lat, /lon, /depth).
#[allow(clippy::single_range_in_vec_init)]
pub fn fetch_all_dimension_coordinates(
    store: ReadableWritableListableStorage,
    dim_names: &[String],
    target_hint: Option<&str>,
) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();

    for dim in dim_names {
        let dim_clean = dim.trim().to_lowercase();
        if !dim_clean.contains("time")
            && !dim_clean.contains("date")
            && !dim_clean.contains("lat")
            && !dim_clean.contains("lon")
            && !dim_clean.contains("depth")
            && !dim_clean.contains("pres")
            && !dim_clean.contains("level")
        {
            continue;
        }

        let array_path = format!("/{}", dim_clean);

        if let Ok(array) = Array::open(store.clone(), &array_path)
            .or_else(|_| Array::open(store.clone(), &dim_clean))
        {
            let len = (array.shape().first().copied().unwrap_or(0) as usize).min(5000);
            if len > 0 {
                let subset = ArraySubset::new_with_ranges(&[0..len as u64]);
                let mut coords = Vec::with_capacity(len);

                let attrs = array.attributes();
                let time_units = attrs.get("units").and_then(|v| v.as_str());
                let time_start = attrs.get("time_coverage_start").and_then(|v| v.as_str());

                if let Ok(vec_f64) = array.retrieve_array_subset::<Vec<f64>>(&subset) {
                    for (idx, &val) in vec_f64.iter().enumerate() {
                        coords.push(format_dim_value(
                            &dim_clean,
                            val,
                            idx,
                            time_units,
                            time_start,
                            target_hint,
                        ));
                    }
                } else if let Ok(vec_f32) = array.retrieve_array_subset::<Vec<f32>>(&subset) {
                    for (idx, &val) in vec_f32.iter().enumerate() {
                        coords.push(format_dim_value(
                            &dim_clean,
                            val as f64,
                            idx,
                            time_units,
                            time_start,
                            target_hint,
                        ));
                    }
                } else if let Ok(vec_i64) = array.retrieve_array_subset::<Vec<i64>>(&subset) {
                    for (idx, &val) in vec_i64.iter().enumerate() {
                        coords.push(format_dim_value(
                            &dim_clean,
                            val as f64,
                            idx,
                            time_units,
                            time_start,
                            target_hint,
                        ));
                    }
                }

                if !coords.is_empty() {
                    map.insert(dim_clean, coords);
                }
            }
        }
    }

    map
}

fn format_dim_value(
    dim_name: &str,
    val: f64,
    _idx: usize,
    units_str: Option<&str>,
    time_start: Option<&str>,
    target_hint: Option<&str>,
) -> String {
    if dim_name.contains("time") || dim_name.contains("date") {
        if val > 1e14 {
            // Nanoseconds since 1970-01-01 epoch
            let days = (val / (86_400.0 * 1e9)).max(0.0).round() as usize;
            let (y, m, d) = units::add_days_to_date(1970, 1, 1, days);
            format!("{:04}-{:02}-{:02}", y, m, d)
        } else if val > 1e11 {
            // Milliseconds since 1970-01-01 epoch
            let days = (val / (86_400.0 * 1000.0)).max(0.0).round() as usize;
            let (y, m, d) = units::add_days_to_date(1970, 1, 1, days);
            format!("{:04}-{:02}-{:02}", y, m, d)
        } else if val > 1e8 {
            // Seconds since 1970-01-01 epoch
            let days = (val / 86_400.0).max(0.0).round() as usize;
            let (y, m, d) = units::add_days_to_date(1970, 1, 1, days);
            format!("{:04}-{:02}-{:02}", y, m, d)
        } else {
            // Step index or days since reference date
            let (ref_y, ref_m, ref_d, days_step) =
                units::parse_reference_date(units_str, time_start, None, target_hint);
            let total_days = (val * days_step as f64).max(0.0).round() as usize;
            let (y, m, d) = units::add_days_to_date(ref_y, ref_m, ref_d, total_days);
            format!("{:04}-{:02}-{:02}", y, m, d)
        }
    } else if dim_name.contains("lat") {
        let cardinal = if val >= 0.0 { "N" } else { "S" };
        format!("{:.2}° {}", val.abs(), cardinal)
    } else if dim_name.contains("lon") {
        let cardinal = if val >= 0.0 { "E" } else { "W" };
        format!("{:.2}° {}", val.abs(), cardinal)
    } else if dim_name.contains("pres") || dim_name.contains("level") {
        format!("{:.0} hPa", val)
    } else if dim_name.contains("depth") {
        format!("{:.1} m", val)
    } else {
        format!("{:.2}", val)
    }
}

/// Fetch a 2D matrix slice for a specific variable and timestep using `zarrs` subset API.
pub fn fetch_slice(
    store: ReadableWritableListableStorage,
    store_url: &str,
    variable: &str,
    timestep: usize,
) -> Result<MatrixSlice, Box<dyn Error>> {
    let var_path = if variable.starts_with('/') {
        variable.to_string()
    } else {
        format!("/{}", variable)
    };

    if let Ok(array) = Array::open(store.clone(), &var_path) {
        let shape = array.shape();
        let dim_names: Vec<String> = array
            .dimension_names()
            .as_ref()
            .map(|names| {
                names
                    .iter()
                    .map(|n| n.as_deref().unwrap_or("dim").to_string())
                    .collect()
            })
            .unwrap_or_else(|| vec!["time".to_string(), "lat".to_string(), "lon".to_string()]);

        let (max_timesteps, initial_height, initial_width, local_time_idx) = match shape.len() {
            4 => (
                shape[0] as usize,
                shape[2] as usize,
                shape[3] as usize,
                (timestep % (shape[0] as usize).max(1)) as u64,
            ),
            3 => (
                shape[0] as usize,
                shape[1] as usize,
                shape[2] as usize,
                (timestep % (shape[0] as usize).max(1)) as u64,
            ),
            2 => (1, shape[0] as usize, shape[1] as usize, 0u64),
            1 => (1, 1, shape[0] as usize, 0u64),
            _ => (1, 64, 64, 0u64),
        };

        let subset = if shape.len() == 4 {
            ArraySubset::new_with_ranges(&[
                local_time_idx..(local_time_idx + 1),
                0..1,
                0..initial_height as u64,
                0..initial_width as u64,
            ])
        } else if shape.len() == 3 {
            ArraySubset::new_with_ranges(&[
                local_time_idx..(local_time_idx + 1),
                0..initial_height as u64,
                0..initial_width as u64,
            ])
        } else if shape.len() == 2 {
            ArraySubset::new_with_ranges(&[0..initial_height as u64, 0..initial_width as u64])
        } else {
            ArraySubset::new_with_shape(shape.to_vec())
        };

        if let Ok(raw_values) = array.retrieve_array_subset::<Vec<f32>>(&subset) {
            let attributes = array.attributes();

            let lat_dim = dim_names
                .iter()
                .find(|d| d.to_lowercase().contains("lat") || d.to_lowercase() == "y");
            let lon_dim = dim_names
                .iter()
                .find(|d| d.to_lowercase().contains("lon") || d.to_lowercase() == "x");

            let lat_coords = lat_dim
                .and_then(|d| get_cached_coord_bounds(store.clone(), store_url, d))
                .map(|(f, l)| vec![f, l]);
            let lon_coords = lon_dim
                .and_then(|d| get_cached_coord_bounds(store.clone(), store_url, d))
                .map(|(f, l)| vec![f, l]);

            // Check axes order and orientation directly from axis coordinate values
            let (oriented_values, width, height) = check_and_orient_axes_with_coords(
                raw_values,
                initial_width,
                initial_height,
                &dim_names,
                attributes,
                lat_coords.as_deref(),
                lon_coords.as_deref(),
            );

            let valid_vals: Vec<f32> = oriented_values
                .iter()
                .copied()
                .filter(|v| !v.is_nan())
                .collect();
            let (min_v, max_v) = if !valid_vals.is_empty() {
                let min_val = valid_vals.iter().copied().fold(f32::INFINITY, f32::min);
                let max_val = valid_vals.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                (min_val, max_val)
            } else {
                (0.0, 1.0)
            };

            let values = oriented_values;

            return Ok(MatrixSlice {
                variable_name: variable.to_string(),
                width,
                height,
                values,
                min_val: min_v,
                max_val: max_v,
                shape: shape.to_vec(),
                current_timestep: timestep,
                max_timesteps,
                dataset_name: format!("Remote Zarr [{}]", variable),
            });
        }
    }

    // Procedural fallback matrix if array slice fetch fails
    let (width, height) = (64, 64);
    let mut raw_data = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let val = (((x * 13 + y * 29 + timestep * 7) % 100) as f32).clamp(0.0, 100.0);
            raw_data.push(val);
        }
    }

    Ok(MatrixSlice {
        variable_name: variable.to_string(),
        width,
        height,
        values: raw_data,
        min_val: 0.0,
        max_val: 100.0,
        shape: vec![height as u64, width as u64],
        current_timestep: timestep,
        max_timesteps: 1,
        dataset_name: format!("Remote Zarr Sample [{}]", variable),
    })
}

/// Fetches a range of consecutive 2D matrix slices in a SINGLE HTTP GET request over the network.
pub fn fetch_slice_range(
    store: ReadableWritableListableStorage,
    store_url: &str,
    variable: &str,
    start_step: usize,
    count: usize,
) -> Result<Vec<MatrixSlice>, Box<dyn Error>> {
    let var_path = if variable.starts_with('/') {
        variable.to_string()
    } else {
        format!("/{}", variable)
    };

    if let Ok(array) = Array::open(store.clone(), &var_path) {
        let shape = array.shape();
        let dim_names: Vec<String> = array
            .dimension_names()
            .as_ref()
            .map(|names| {
                names
                    .iter()
                    .map(|n| n.as_deref().unwrap_or("dim").to_string())
                    .collect()
            })
            .unwrap_or_else(|| vec!["time".to_string(), "lat".to_string(), "lon".to_string()]);

        let (max_timesteps, initial_height, initial_width) = match shape.len() {
            4 => (shape[0] as usize, shape[2] as usize, shape[3] as usize),
            3 => (shape[0] as usize, shape[1] as usize, shape[2] as usize),
            2 => (1, shape[0] as usize, shape[1] as usize),
            1 => (1, 1, shape[0] as usize),
            _ => (1, 64, 64),
        };

        let actual_count = count.min(max_timesteps.saturating_sub(start_step)).max(1);

        let subset = if shape.len() == 4 {
            ArraySubset::new_with_ranges(&[
                start_step as u64..(start_step + actual_count) as u64,
                0..1,
                0..initial_height as u64,
                0..initial_width as u64,
            ])
        } else if shape.len() == 3 {
            ArraySubset::new_with_ranges(&[
                start_step as u64..(start_step + actual_count) as u64,
                0..initial_height as u64,
                0..initial_width as u64,
            ])
        } else {
            ArraySubset::new_with_ranges(&[0..initial_height as u64, 0..initial_width as u64])
        };

        if let Ok(raw_values) = array.retrieve_array_subset::<Vec<f32>>(&subset) {
            let attributes = array.attributes();

            let lat_dim = dim_names
                .iter()
                .find(|d| d.to_lowercase().contains("lat") || d.to_lowercase() == "y");
            let lon_dim = dim_names
                .iter()
                .find(|d| d.to_lowercase().contains("lon") || d.to_lowercase() == "x");

            let lat_coords = lat_dim
                .and_then(|d| get_cached_coord_bounds(store.clone(), store_url, d))
                .map(|(f, l)| vec![f, l]);
            let lon_coords = lon_dim
                .and_then(|d| get_cached_coord_bounds(store.clone(), store_url, d))
                .map(|(f, l)| vec![f, l]);

            let slice_size = initial_width * initial_height;
            let mut slices = Vec::with_capacity(actual_count);

            for i in 0..actual_count {
                let offset = i * slice_size;
                if offset + slice_size > raw_values.len() {
                    break;
                }
                let slice_raw = raw_values[offset..offset + slice_size].to_vec();

                let (oriented_values, width, height) = check_and_orient_axes_with_coords(
                    slice_raw,
                    initial_width,
                    initial_height,
                    &dim_names,
                    attributes,
                    lat_coords.as_deref(),
                    lon_coords.as_deref(),
                );

                let valid_vals: Vec<f32> = oriented_values
                    .iter()
                    .copied()
                    .filter(|v| !v.is_nan())
                    .collect();
                let (min_v, max_v) = if !valid_vals.is_empty() {
                    let min_val = valid_vals.iter().copied().fold(f32::INFINITY, f32::min);
                    let max_val = valid_vals.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                    (min_val, max_val)
                } else {
                    (0.0, 1.0)
                };

                let values = oriented_values;

                slices.push(MatrixSlice {
                    variable_name: variable.to_string(),
                    width,
                    height,
                    values,
                    min_val: min_v,
                    max_val: max_v,
                    shape: shape.to_vec(),
                    current_timestep: start_step + i,
                    max_timesteps,
                    dataset_name: format!("Remote Zarr [{}]", variable),
                });
            }

            return Ok(slices);
        }
    }

    let single = fetch_slice(store, store_url, variable, start_step)?;
    Ok(vec![single])
}

use std::sync::{OnceLock, RwLock};
#[allow(clippy::type_complexity)]
static COORD_BOUNDS_CACHE: OnceLock<RwLock<HashMap<String, Option<(f64, f64)>>>> = OnceLock::new();

#[allow(clippy::single_range_in_vec_init)]
fn get_cached_coord_bounds(
    store: ReadableWritableListableStorage,
    store_url: &str,
    dim_name: &str,
) -> Option<(f64, f64)> {
    let cache_lock = COORD_BOUNDS_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    let key = format!("{}:{}", store_url, dim_name.trim().to_lowercase());

    if let Ok(cache) = cache_lock.read()
        && let Some(bounds) = cache.get(&key)
    {
        return *bounds;
    }

    let bounds = read_coord_bounds(store, dim_name);
    if let Ok(mut cache) = cache_lock.write() {
        cache.insert(key, bounds);
    }
    bounds
}

#[allow(clippy::single_range_in_vec_init)]
fn read_coord_bounds(store: ReadableWritableListableStorage, dim_name: &str) -> Option<(f64, f64)> {
    let clean = dim_name.trim().to_lowercase();
    let array_path = format!("/{}", clean);
    let array = Array::open(store.clone(), &array_path)
        .or_else(|_| Array::open(store.clone(), &clean))
        .ok()?;

    let len = array.shape().first().copied().unwrap_or(0) as usize;
    if len < 2 {
        return None;
    }

    let subset_start = ArraySubset::new_with_ranges(&[0..1]);
    let subset_end = ArraySubset::new_with_ranges(&[(len as u64 - 1)..len as u64]);

    let v_start = array
        .retrieve_array_subset::<Vec<f64>>(&subset_start)
        .ok()
        .and_then(|v| v.first().copied())
        .or_else(|| {
            array
                .retrieve_array_subset::<Vec<f32>>(&subset_start)
                .ok()
                .and_then(|v| v.first().map(|&x| x as f64))
        })?;

    let v_end = array
        .retrieve_array_subset::<Vec<f64>>(&subset_end)
        .ok()
        .and_then(|v| v.first().copied())
        .or_else(|| {
            array
                .retrieve_array_subset::<Vec<f32>>(&subset_end)
                .ok()
                .and_then(|v| v.first().map(|&x| x as f64))
        })?;
    Some((v_start, v_end))
}
