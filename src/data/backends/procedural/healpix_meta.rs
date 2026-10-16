//! Synthetic HEALPix Nside=16 dataset metadata generator.

use std::collections::HashMap;

use crate::data::metadata::{DatasetMetadata, VariableInfo};

pub fn build_healpix_metadata() -> DatasetMetadata {
    let nside = 16;
    let npix = 12 * nside * nside;
    let mut global_attrs = HashMap::new();
    global_attrs.insert("healpix_nside".to_string(), "16".to_string());
    global_attrs.insert("healpix_npix".to_string(), "3072".to_string());
    global_attrs.insert("healpix_order".to_string(), "ring".to_string());
    global_attrs.insert("grid_type".to_string(), "healpix".to_string());

    let vars = vec![
        VariableInfo {
            name: "temp".to_string(),
            data_type: "float32".to_string(),
            shape: vec![12, 8, npix as u64],
            chunk_shape: vec![1, 1, npix as u64],
            dimension_names: vec!["time".to_string(), "layer".to_string(), "cell".to_string()],
            units: Some("K".to_string()),
            long_name: Some("Atmospheric Temperature (HEALPix Nside=16)".to_string()),
            temporal_resolution: Some("6 hours".to_string()),
            time_coverage_start: None,
            time_coverage_end: None,
            file_size: (12 * 8 * npix * 4) as u64,
            attributes: global_attrs.clone(),
            ome: None,
            multiscale: None,
            shard_shape: None,
        },
        VariableInfo {
            name: "mslp".to_string(),
            data_type: "float32".to_string(),
            shape: vec![12, npix as u64],
            chunk_shape: vec![1, npix as u64],
            dimension_names: vec!["time".to_string(), "cell".to_string()],
            units: Some("hPa".to_string()),
            long_name: Some("Mean Sea Level Pressure (HEALPix Nside=16)".to_string()),
            temporal_resolution: Some("6 hours".to_string()),
            time_coverage_start: None,
            time_coverage_end: None,
            file_size: (12 * npix * 4) as u64,
            attributes: global_attrs.clone(),
            ome: None,
            multiscale: None,
            shard_shape: None,
        },
        VariableInfo {
            name: "lat".to_string(),
            data_type: "float32".to_string(),
            shape: vec![npix as u64],
            chunk_shape: vec![npix as u64],
            dimension_names: vec!["cell".to_string()],
            units: Some("degrees_north".to_string()),
            long_name: Some("Cell Center Latitude".to_string()),
            temporal_resolution: None,
            time_coverage_start: None,
            time_coverage_end: None,
            file_size: (npix * 4) as u64,
            attributes: global_attrs.clone(),
            ome: None,
            multiscale: None,
            shard_shape: None,
        },
        VariableInfo {
            name: "lon".to_string(),
            data_type: "float32".to_string(),
            shape: vec![npix as u64],
            chunk_shape: vec![npix as u64],
            dimension_names: vec!["cell".to_string()],
            units: Some("degrees_east".to_string()),
            long_name: Some("Cell Center Longitude".to_string()),
            temporal_resolution: None,
            time_coverage_start: None,
            time_coverage_end: None,
            file_size: (npix * 4) as u64,
            attributes: global_attrs.clone(),
            ome: None,
            multiscale: None,
            shard_shape: None,
        },
        VariableInfo {
            name: "ring".to_string(),
            data_type: "float32".to_string(),
            shape: vec![npix as u64],
            chunk_shape: vec![npix as u64],
            dimension_names: vec!["cell".to_string()],
            units: Some("index".to_string()),
            long_name: Some("Latitude Ring Index (1 to 4*Nside-1)".to_string()),
            temporal_resolution: None,
            time_coverage_start: None,
            time_coverage_end: None,
            file_size: (npix * 4) as u64,
            attributes: global_attrs,
            ome: None,
            multiscale: None,
            shard_shape: None,
        },
    ];

    DatasetMetadata {
        name: "HEALPix Discrete Global Grid (Nside=16)".to_string(),
        store_type: "Procedural / HEALPix".to_string(),
        variables: vars,
        dimension_coordinates: HashMap::new(),
        ome: None,
    }
}
