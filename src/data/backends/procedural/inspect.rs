//! Synthetic dataset metadata generator for procedural benchmarks.

use std::collections::HashMap;

use super::healpix_meta::build_healpix_metadata;
use crate::data::blocks::BlockStoreError;
use crate::data::metadata::{CoordValues, DatasetMetadata, VariableInfo};
use crate::data::procedural::{
    generate_clenshaw_curtis_coords, generate_gaussian_coords, generate_stepped_resolution_coords,
    generate_stretched_regional_coords,
};

fn make_var(
    name: &str,
    shape: Vec<u64>,
    chunk: Vec<u64>,
    dims: &[&str],
    units: &str,
    long_name: &str,
    temporal: Option<&str>,
) -> VariableInfo {
    let file_size = shape.iter().product::<u64>() * 4;
    VariableInfo {
        name: name.to_string(),
        data_type: "float32".to_string(),
        shape,
        chunk_shape: chunk,
        dimension_names: dims.iter().map(|s| s.to_string()).collect(),
        units: Some(units.to_string()),
        long_name: Some(long_name.to_string()),
        temporal_resolution: temporal.map(String::from),
        time_coverage_start: None,
        time_coverage_end: None,
        file_size,
        attributes: HashMap::new(),
    }
}

pub fn inspect_procedural(uri: &str) -> Result<DatasetMetadata, BlockStoreError> {
    if uri.contains("healpix") {
        return Ok(build_healpix_metadata());
    }

    let is_4d = uri.contains("volume") || uri.contains("4d");

    let vars = if is_4d {
        vec![
            make_var(
                "gaussian_wave_packet_4d",
                vec![20, 32, 32, 32],
                vec![1, 32, 32, 32],
                &["time", "depth", "lat", "lon"],
                "K",
                "4D Known-Truth Gaussian Wave Packet (Procedural)",
                Some("1 day"),
            ),
            make_var(
                "procedural_matrix_2d",
                vec![64, 64],
                vec![64, 64],
                &["y", "x"],
                "dimensionless",
                "2D Procedural Wave Field",
                None,
            ),
        ]
    } else {
        vec![
            make_var(
                "clenshaw_curtis_2d",
                vec![64, 128],
                vec![64, 128],
                &["lat", "lon"],
                "dimensionless",
                "2D Clenshaw-Curtis Grid (Boundary Compressed)",
                None,
            ),
            make_var(
                "gaussian_grid_2d",
                vec![64, 128],
                vec![64, 128],
                &["lat", "lon"],
                "K",
                "2D Gaussian Latitude Grid (Poles Compressed)",
                None,
            ),
            make_var(
                "stretched_regional_2d",
                vec![32, 48],
                vec![32, 48],
                &["lat", "lon"],
                "dimensionless",
                "2D Geometrically Stretched Regional Grid [10E..50E, 30N..60N]",
                None,
            ),
            make_var(
                "stepped_resolution_2d",
                vec![32, 64],
                vec![32, 64],
                &["lat", "lon"],
                "dimensionless",
                "2D Stepped Multi-Resolution Grid (5x Resolution Jump)",
                None,
            ),
            make_var(
                "gaussian_wave_packet_4d",
                vec![20, 32, 32, 32],
                vec![1, 32, 32, 32],
                &["time", "depth", "lat", "lon"],
                "K",
                "4D Known-Truth Gaussian Wave Packet (Procedural)",
                Some("1 day"),
            ),
            make_var(
                "procedural_matrix_2d",
                vec![64, 64],
                vec![64, 64],
                &["y", "x"],
                "dimensionless",
                "2D Procedural Wave Field",
                None,
            ),
        ]
    };

    let mut dim_coords = HashMap::new();
    let mut insert = |key: &str, values: &CoordValues| {
        dim_coords.insert(key.to_string(), values.clone());
    };
    let regular = |start: f64, step: f64, len: usize| CoordValues::Regular { start, step, len };
    let time = regular(0.0, 1.0, 20);
    let depth = regular(0.0, 1000.0 / 31.0, 32);
    let lat = regular(90.0, -180.0 / 31.0, 32);
    let lon = regular(-180.0, 360.0 / 31.0, 32);
    let xy = regular(0.0, 1.0, 64);

    for (dim, values) in [("time", &time), ("depth", &depth)] {
        insert(&format!("gaussian_wave_packet_4d/{dim}"), values);
        insert(dim, values);
    }
    insert("gaussian_wave_packet_4d/lat", &lat);
    insert("gaussian_wave_packet_4d/lon", &lon);
    for dim in ["y", "x"] {
        insert(&format!("procedural_matrix_2d/{dim}"), &xy);
        insert(dim, &xy);
    }
    if is_4d {
        insert("lat", &lat);
        insert("lon", &lon);
    } else {
        let grids = [
            (
                "clenshaw_curtis_2d",
                generate_clenshaw_curtis_coords(128, 64),
            ),
            ("gaussian_grid_2d", generate_gaussian_coords(128, 64)),
            (
                "stretched_regional_2d",
                generate_stretched_regional_coords(48, 32),
            ),
            (
                "stepped_resolution_2d",
                generate_stepped_resolution_coords(64, 32),
            ),
        ];
        for (var, (x, y)) in grids {
            for (dim, values) in [("lon", x), ("lat", y)] {
                let Some(values) = CoordValues::from_values(values, false) else {
                    continue;
                };
                // The unscoped axes are the Clenshaw-Curtis grid's.
                if var == "clenshaw_curtis_2d" {
                    insert(dim, &values);
                }
                insert(&format!("{var}/{dim}"), &values);
            }
        }
    }

    Ok(DatasetMetadata {
        name: if is_4d {
            "4D Known-Truth Procedural Store".to_string()
        } else {
            "Ground Truth Irregular & Procedural Store".to_string()
        },
        store_type: "Procedural / Ground Truth".to_string(),
        variables: vars,
        dimension_coordinates: dim_coords,
    })
}
