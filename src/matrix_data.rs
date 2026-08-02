use std::sync::Arc;
use zarrs::array::{Array, ArrayBuilder, DataType, FillValue};
use zarrs::array_subset::ArraySubset;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::WritableStorageTraits;

pub struct MatrixData {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
    pub shape: Vec<u64>,
    pub dataset_name: String,
    pub current_timestep: usize,
    pub max_timesteps: usize,
}

impl MatrixData {
    /// Generates a random 2D scalar field for visualization
    pub fn create_random_matrix(width: usize, height: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut raw_data = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                let fx = x as f32 / width as f32;
                let fy = y as f32 / height as f32;
                let wave1 = (fx * 14.2 + fy * 9.7).sin() * 0.5 + 0.5;
                let wave2 = ((fx * 28.4 - fy * 19.1).cos() * 0.5 + 0.5) * 0.5;
                let hash = (((x * 1597 + y * 28491) % 1000) as f32 / 1000.0) * 0.25;
                let val = ((wave1 * 0.5 + wave2 + hash) * 100.0).clamp(0.0, 100.0);
                raw_data.push(val);
            }
        }

        Ok(Self {
            width,
            height,
            values: raw_data,
            shape: vec![height as u64, width as u64],
            dataset_name: format!("Random Matrix ({}x{})", width, height),
            current_timestep: 0,
            max_timesteps: 1,
        })
    }

    /// Generates a sample 2D Gaussian heat source in an in-memory Zarr v3 store
    pub fn create_sample_heatmap(width: usize, height: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let store = Arc::new(MemoryStore::new());
        let array_path = "/sample_matrix";

        let shape = vec![height as u64, width as u64];
        let chunk_shape = vec![height as u64, width as u64];

        let array = ArrayBuilder::new(
            shape.clone(),
            DataType::Float32,
            chunk_shape.try_into()?,
            FillValue::from(0.0f32),
        )
        .build(store.clone(), array_path)?;

        array.store_metadata()?;

        let mut raw_data = Vec::with_capacity(width * height);
        let center_x1 = width as f32 * 0.35;
        let center_y1 = height as f32 * 0.35;
        let center_x2 = width as f32 * 0.70;
        let center_y2 = height as f32 * 0.65;

        for y in 0..height {
            for x in 0..width {
                let dx1 = x as f32 - center_x1;
                let dy1 = y as f32 - center_y1;
                let dist1 = (dx1 * dx1 + dy1 * dy1).sqrt();

                let dx2 = x as f32 - center_x2;
                let dy2 = y as f32 - center_y2;
                let dist2 = (dx2 * dx2 + dy2 * dy2).sqrt();

                let val1 = 100.0 * (-dist1 * 0.25).exp();
                let val2 = 75.0 * (-dist2 * 0.30).exp();
                let val = (val1 + val2).min(100.0);

                raw_data.push(val);
            }
        }

        let subset = ArraySubset::new_with_shape(shape.clone());
        array.store_array_subset_elements(&subset, &raw_data)?;

        let read_values = array.retrieve_array_subset_elements::<f32>(&subset)?;

        Ok(Self {
            width,
            height,
            values: read_values,
            shape,
            dataset_name: "Gaussian 2D Matrix (32x32)".to_string(),
            current_timestep: 0,
            max_timesteps: 365,
        })
    }

    /// Fetches remote Zarr array slice over HTTP
    pub fn fetch_remote_esdc_temperature(store_url: &str, timestep: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let base_url = store_url.trim_end_matches('/');
        let zarray_url = format!("{}/air_temperature_2m/.zarray", base_url);
        let zarray_bytes = reqwest::blocking::get(&zarray_url)?.bytes()?;

        let time_chunk_size = 46;
        let chunk_time_idx = timestep / time_chunk_size;
        let local_time_idx = (timestep % time_chunk_size) as u64;

        let chunk_url = format!("{}/air_temperature_2m/{}.0.0", base_url, chunk_time_idx);
        let chunk_bytes = reqwest::blocking::get(&chunk_url)?.bytes()?;

        let key_zarray = zarrs::storage::StoreKey::new("air_temperature_2m/.zarray")?;
        let key_chunk = zarrs::storage::StoreKey::new(format!("air_temperature_2m/{}.0.0", chunk_time_idx))?;

        let store = Arc::new(MemoryStore::new());
        store.set(&key_zarray, zarray_bytes.to_vec().into())?;
        store.set(&key_chunk, chunk_bytes.to_vec().into())?;

        let array = Array::open(store, "/air_temperature_2m")?;
        let shape = array.shape();
        let max_timesteps = shape.first().copied().unwrap_or(989) as usize;
        let height = shape.get(1).copied().unwrap_or(72) as usize;
        let width = shape.get(2).copied().unwrap_or(144) as usize;

        let subset = ArraySubset::new_with_ranges(&[
            local_time_idx..(local_time_idx + 1),
            0..height as u64,
            0..width as u64,
        ]);

        let raw_values = array.retrieve_array_subset_elements::<f32>(&subset)?;

        let valid_vals: Vec<f32> = raw_values
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

        let range = if max_v > min_v { max_v - min_v } else { 1.0 };

        let values = raw_values
            .into_iter()
            .map(|val| {
                if val.is_nan() {
                    0.0
                } else {
                    ((val - min_v) / range * 100.0).clamp(0.0, 100.0)
                }
            })
            .collect();

        Ok(Self {
            width,
            height,
            values,
            shape: shape.to_vec(),
            dataset_name: format!("ESDC air_temperature_2m [Step {}/{}]", timestep + 1, max_timesteps),
            current_timestep: timestep,
            max_timesteps,
        })
    }
}
