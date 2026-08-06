use crate::stores::MatrixSlice;
use crate::utils::coordinates::get_cached_coord_bounds;
use crate::utils::grid::check_and_orient_axes_with_coords;
use std::error::Error;
use zarrs::array::{Array, ArraySubset};
use zarrs::storage::ReadableWritableListableStorage;

/// Fetches a 2D scalar matrix slice [timestep, lat, lon] from a Zarr storage backend.
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

        if let Ok(raw_values) = retrieve_array_subset_as_f32(&array, &subset) {
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

            return Ok(MatrixSlice {
                variable_name: variable.to_string(),
                width,
                height,
                values: oriented_values,
                min_val: min_v,
                max_val: max_v,
                shape: shape.to_vec(),
                current_timestep: timestep,
                max_timesteps,
                dataset_name: format!("Zarr Store [{}]", variable),
            });
        }
    }

    // Procedural fallback matrix if array slice fetch fails
    let (width, height) = (64, 64);
    let (raw_data, min_val, max_val) =
        crate::data::procedural::generate_procedural_matrix(width, height, timestep);

    Ok(MatrixSlice {
        variable_name: variable.to_string(),
        width,
        height,
        values: raw_data,
        min_val,
        max_val,
        shape: vec![height as u64, width as u64],
        current_timestep: timestep,
        max_timesteps: 1,
        dataset_name: format!("Zarr Sample [{}]", variable),
    })
}

/// Fetches a range of consecutive 2D matrix slices in a single request.
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

        if let Ok(raw_values) = retrieve_array_subset_as_f32(&array, &subset) {
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

                slices.push(MatrixSlice {
                    variable_name: variable.to_string(),
                    width,
                    height,
                    values: oriented_values,
                    min_val: min_v,
                    max_val: max_v,
                    shape: shape.to_vec(),
                    current_timestep: start_step + i,
                    max_timesteps,
                    dataset_name: format!("Zarr Store [{}]", variable),
                });
            }

            return Ok(slices);
        }
    }

    let single = fetch_slice(store, store_url, variable, start_step)?;
    Ok(vec![single])
}

fn retrieve_array_subset_as_f32(
    array: &Array<dyn zarrs::storage::ReadableWritableListableStorageTraits>,
    subset: &ArraySubset,
) -> Result<Vec<f32>, Box<dyn Error>> {
    let dt_str = array.data_type().to_string().to_lowercase();
    if dt_str.contains("float64") || dt_str.contains("f64") {
        let vals = array.retrieve_array_subset::<Vec<f64>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("int64") || dt_str.contains("i64") {
        let vals = array.retrieve_array_subset::<Vec<i64>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("int32") || dt_str.contains("i32") {
        let vals = array.retrieve_array_subset::<Vec<i32>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("uint64") || dt_str.contains("u64") {
        let vals = array.retrieve_array_subset::<Vec<u64>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("uint32") || dt_str.contains("u32") {
        let vals = array.retrieve_array_subset::<Vec<u32>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("int16") || dt_str.contains("i16") {
        let vals = array.retrieve_array_subset::<Vec<i16>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else if dt_str.contains("uint16") || dt_str.contains("u16") {
        let vals = array.retrieve_array_subset::<Vec<u16>>(subset)?;
        Ok(vals.into_iter().map(|v| v as f32).collect())
    } else {
        let vals = array.retrieve_array_subset::<Vec<f32>>(subset)?;
        Ok(vals)
    }
}
