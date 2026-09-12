//! In-memory procedural and known-truth synthetic block store.

use std::collections::HashMap;

use crate::data::{
    block_request::BlockResult,
    block_store::{BlockStore, BlockStoreError},
    metadata::{DatasetMetadata, VariableInfo},
    octant_block::OctantBlock,
    procedural::{
        eval_known_truth_4d, generate_clenshaw_curtis_2d, generate_clenshaw_curtis_coords,
        generate_gaussian_coords, generate_gaussian_grid_2d, generate_procedural_matrix,
        generate_stepped_resolution_2d, generate_stepped_resolution_coords,
        generate_stretched_regional_2d, generate_stretched_regional_coords,
    },
    slice_request::SliceRequest,
};

pub struct ProceduralBlockStore {
    pub uri: String,
}

impl ProceduralBlockStore {
    pub fn open(uri: &str) -> Result<Self, BlockStoreError> {
        Ok(Self {
            uri: uri.to_string(),
        })
    }
}

fn slice_2d_grid_block(
    var_name: &str,
    shape: (usize, usize),
    full_data: &[f32],
    coords: (&[f64], &[f64]),
    request: &SliceRequest,
    on_progress: &mut crate::data::block_store::ProgressCallback,
) -> OctantBlock {
    let (w_full, h_full) = shape;
    let (xs, ys) = coords;

    let (y_start, y_end) = request
        .selections
        .first()
        .map(|s| s.bounds())
        .unwrap_or((0, h_full));
    let (x_start, x_end) = request
        .selections
        .get(1)
        .map(|s| s.bounds())
        .unwrap_or((0, w_full));

    let y_start = y_start.min(h_full);
    let y_end = y_end.min(h_full).max(y_start);
    let x_start = x_start.min(w_full);
    let x_end = x_end.min(w_full).max(x_start);

    let block_h = y_end - y_start;
    let block_w = x_end - x_start;

    let mut values = Vec::with_capacity(block_h * block_w);
    for y in y_start..y_end {
        for x in x_start..x_end {
            let idx = y * w_full + x;
            values.push(full_data.get(idx).copied().unwrap_or(0.0));
        }
    }

    if let Some(cb) = on_progress {
        cb((values.len() * 4) as u64);
    }

    let mut coords_map = HashMap::new();
    coords_map.insert(
        "lon".to_string(),
        xs.get(x_start..x_end)
            .map(|s| s.to_vec())
            .unwrap_or_else(|| xs.to_vec()),
    );
    coords_map.insert(
        "lat".to_string(),
        ys.get(y_start..y_end)
            .map(|s| s.to_vec())
            .unwrap_or_else(|| ys.to_vec()),
    );

    OctantBlock::new(
        var_name.to_string(),
        vec![block_h, block_w],
        vec!["lat".to_string(), "lon".to_string()],
        vec![y_start, x_start],
        values,
        coords_map,
        HashMap::new(),
    )
}

impl BlockStore for ProceduralBlockStore {
    fn backend_name(&self) -> &str {
        "Procedural / Known-Truth"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        if self.uri.contains("healpix") {
            return Ok(vec![
                "temp".to_string(),
                "mslp".to_string(),
                "lat".to_string(),
                "lon".to_string(),
                "ring".to_string(),
            ]);
        }
        let is_4d = self.uri.contains("volume") || self.uri.contains("4d");
        if is_4d {
            Ok(vec![
                "gaussian_wave_packet_4d".to_string(),
                "procedural_matrix_2d".to_string(),
            ])
        } else {
            Ok(vec![
                "clenshaw_curtis_2d".to_string(),
                "gaussian_grid_2d".to_string(),
                "stretched_regional_2d".to_string(),
                "stepped_resolution_2d".to_string(),
                "gaussian_wave_packet_4d".to_string(),
                "procedural_matrix_2d".to_string(),
            ])
        }
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        if self.uri.contains("healpix") {
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
                    dimension_names: vec![
                        "time".to_string(),
                        "layer".to_string(),
                        "cell".to_string(),
                    ],
                    units: Some("K".to_string()),
                    long_name: Some("Atmospheric Temperature (HEALPix Nside=16)".to_string()),
                    temporal_resolution: Some("6 hours".to_string()),
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: (12 * 8 * npix * 4) as u64,
                    attributes: global_attrs.clone(),
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
                },
            ];

            let mut dim_coords = HashMap::new();
            let mut lons_vec = Vec::with_capacity(npix);
            let mut lats_vec = Vec::with_capacity(npix);
            for p in 0..npix {
                let (lon_rad, lat_rad) = crate::data::coordinates::healpix::pix2ang_ring(nside, p);
                lons_vec.push(format!("{:.4}", lon_rad.to_degrees()));
                lats_vec.push(format!("{:.4}", lat_rad.to_degrees()));
            }
            dim_coords.insert("lon".to_string(), lons_vec);
            dim_coords.insert("lat".to_string(), lats_vec);

            return Ok(DatasetMetadata {
                name: "SpeedyWeather HEALPix Grid (Nside=16)".to_string(),
                store_type: "Procedural / HEALPix".to_string(),
                variables: vars,
                dimension_coordinates: dim_coords,
            });
        }

        let is_4d = self.uri.contains("volume") || self.uri.contains("4d");

        let vars = if is_4d {
            vec![
                VariableInfo {
                    name: "gaussian_wave_packet_4d".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![20, 32, 32, 32],
                    chunk_shape: vec![1, 32, 32, 32],
                    dimension_names: vec![
                        "time".to_string(),
                        "depth".to_string(),
                        "lat".to_string(),
                        "lon".to_string(),
                    ],
                    units: Some("K".to_string()),
                    long_name: Some("4D Known-Truth Gaussian Wave Packet (Procedural)".to_string()),
                    temporal_resolution: Some("1 day".to_string()),
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 20 * 32 * 32 * 32 * 4,
                    attributes: HashMap::new(),
                },
                VariableInfo {
                    name: "procedural_matrix_2d".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![64, 64],
                    chunk_shape: vec![64, 64],
                    dimension_names: vec!["y".to_string(), "x".to_string()],
                    units: Some("dimensionless".to_string()),
                    long_name: Some("2D Procedural Wave Field".to_string()),
                    temporal_resolution: None,
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 64 * 64 * 4,
                    attributes: HashMap::new(),
                },
            ]
        } else {
            vec![
                VariableInfo {
                    name: "clenshaw_curtis_2d".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![64, 128],
                    chunk_shape: vec![64, 128],
                    dimension_names: vec!["lat".to_string(), "lon".to_string()],
                    units: Some("dimensionless".to_string()),
                    long_name: Some("2D Clenshaw-Curtis Grid (Boundary Compressed)".to_string()),
                    temporal_resolution: None,
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 64 * 128 * 4,
                    attributes: HashMap::new(),
                },
                VariableInfo {
                    name: "gaussian_grid_2d".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![64, 128],
                    chunk_shape: vec![64, 128],
                    dimension_names: vec!["lat".to_string(), "lon".to_string()],
                    units: Some("K".to_string()),
                    long_name: Some("2D Gaussian Latitude Grid (Poles Compressed)".to_string()),
                    temporal_resolution: None,
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 64 * 128 * 4,
                    attributes: HashMap::new(),
                },
                VariableInfo {
                    name: "stretched_regional_2d".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![32, 48],
                    chunk_shape: vec![32, 48],
                    dimension_names: vec!["lat".to_string(), "lon".to_string()],
                    units: Some("dimensionless".to_string()),
                    long_name: Some(
                        "2D Geometrically Stretched Regional Grid [10E..50E, 30N..60N]".to_string(),
                    ),
                    temporal_resolution: None,
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 32 * 48 * 4,
                    attributes: HashMap::new(),
                },
                VariableInfo {
                    name: "stepped_resolution_2d".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![32, 64],
                    chunk_shape: vec![32, 64],
                    dimension_names: vec!["lat".to_string(), "lon".to_string()],
                    units: Some("dimensionless".to_string()),
                    long_name: Some(
                        "2D Stepped Multi-Resolution Grid (5x Resolution Jump)".to_string(),
                    ),
                    temporal_resolution: None,
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 32 * 64 * 4,
                    attributes: HashMap::new(),
                },
                VariableInfo {
                    name: "gaussian_wave_packet_4d".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![20, 32, 32, 32],
                    chunk_shape: vec![1, 32, 32, 32],
                    dimension_names: vec![
                        "time".to_string(),
                        "depth".to_string(),
                        "lat".to_string(),
                        "lon".to_string(),
                    ],
                    units: Some("K".to_string()),
                    long_name: Some("4D Known-Truth Gaussian Wave Packet (Procedural)".to_string()),
                    temporal_resolution: Some("1 day".to_string()),
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 20 * 32 * 32 * 32 * 4,
                    attributes: HashMap::new(),
                },
                VariableInfo {
                    name: "procedural_matrix_2d".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![64, 64],
                    chunk_shape: vec![64, 64],
                    dimension_names: vec!["y".to_string(), "x".to_string()],
                    units: Some("dimensionless".to_string()),
                    long_name: Some("2D Procedural Wave Field".to_string()),
                    temporal_resolution: None,
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 64 * 64 * 4,
                    attributes: HashMap::new(),
                },
            ]
        };

        let mut dim_coords = HashMap::new();

        if is_4d {
            let t_coords: Vec<String> = (0..20).map(|t| format!("{t}")).collect();
            let z_coords: Vec<String> = (0..32)
                .map(|z| format!("{:.1}", z as f64 * (1000.0 / 31.0)))
                .collect();
            let lat_coords: Vec<String> = (0..32)
                .map(|j| format!("{:.3}", 90.0 - j as f64 * (180.0 / 31.0)))
                .collect();
            let lon_coords: Vec<String> = (0..32)
                .map(|i| format!("{:.3}", -180.0 + i as f64 * (360.0 / 31.0)))
                .collect();
            let xy_coords: Vec<String> = (0..64).map(|i| format!("{i}")).collect();

            dim_coords.insert("gaussian_wave_packet_4d/time".to_string(), t_coords.clone());
            dim_coords.insert(
                "gaussian_wave_packet_4d/depth".to_string(),
                z_coords.clone(),
            );
            dim_coords.insert(
                "gaussian_wave_packet_4d/lat".to_string(),
                lat_coords.clone(),
            );
            dim_coords.insert(
                "gaussian_wave_packet_4d/lon".to_string(),
                lon_coords.clone(),
            );
            dim_coords.insert("procedural_matrix_2d/y".to_string(), xy_coords.clone());
            dim_coords.insert("procedural_matrix_2d/x".to_string(), xy_coords.clone());

            dim_coords.insert("time".to_string(), t_coords);
            dim_coords.insert("depth".to_string(), z_coords);
            dim_coords.insert("lat".to_string(), lat_coords);
            dim_coords.insert("lon".to_string(), lon_coords);
            dim_coords.insert("y".to_string(), xy_coords.clone());
            dim_coords.insert("x".to_string(), xy_coords);
        } else {
            let (clenshaw_x, clenshaw_y) = generate_clenshaw_curtis_coords(128, 64);
            let (gauss_x, gauss_y) = generate_gaussian_coords(128, 64);
            let (stretched_x, stretched_y) = generate_stretched_regional_coords(48, 32);
            let (stepped_x, stepped_y) = generate_stepped_resolution_coords(64, 32);

            let clenshaw_x_str: Vec<String> =
                clenshaw_x.iter().map(|v| format!("{v:.3}")).collect();
            let clenshaw_y_str: Vec<String> =
                clenshaw_y.iter().map(|v| format!("{v:.3}")).collect();
            let gauss_x_str: Vec<String> = gauss_x.iter().map(|v| format!("{v:.3}")).collect();
            let gauss_y_str: Vec<String> = gauss_y.iter().map(|v| format!("{v:.3}")).collect();
            let stretched_x_str: Vec<String> =
                stretched_x.iter().map(|v| format!("{v:.3}")).collect();
            let stretched_y_str: Vec<String> =
                stretched_y.iter().map(|v| format!("{v:.3}")).collect();
            let stepped_x_str: Vec<String> = stepped_x.iter().map(|v| format!("{v:.3}")).collect();
            let stepped_y_str: Vec<String> = stepped_y.iter().map(|v| format!("{v:.3}")).collect();

            let t_coords: Vec<String> = (0..20).map(|t| format!("{t}")).collect();
            let z_coords: Vec<String> = (0..32)
                .map(|z| format!("{:.1}", z as f64 * (1000.0 / 31.0)))
                .collect();
            let lat_coords_32: Vec<String> = (0..32)
                .map(|j| format!("{:.3}", 90.0 - j as f64 * (180.0 / 31.0)))
                .collect();
            let lon_coords_32: Vec<String> = (0..32)
                .map(|i| format!("{:.3}", -180.0 + i as f64 * (360.0 / 31.0)))
                .collect();
            let xy_coords: Vec<String> = (0..64).map(|i| format!("{i}")).collect();

            dim_coords.insert("clenshaw_curtis_2d/lon".to_string(), clenshaw_x_str.clone());
            dim_coords.insert("clenshaw_curtis_2d/lat".to_string(), clenshaw_y_str.clone());
            dim_coords.insert("gaussian_grid_2d/lon".to_string(), gauss_x_str);
            dim_coords.insert("gaussian_grid_2d/lat".to_string(), gauss_y_str);
            dim_coords.insert("stretched_regional_2d/lon".to_string(), stretched_x_str);
            dim_coords.insert("stretched_regional_2d/lat".to_string(), stretched_y_str);
            dim_coords.insert("stepped_resolution_2d/lon".to_string(), stepped_x_str);
            dim_coords.insert("stepped_resolution_2d/lat".to_string(), stepped_y_str);
            dim_coords.insert("gaussian_wave_packet_4d/time".to_string(), t_coords.clone());
            dim_coords.insert(
                "gaussian_wave_packet_4d/depth".to_string(),
                z_coords.clone(),
            );
            dim_coords.insert("gaussian_wave_packet_4d/lat".to_string(), lat_coords_32);
            dim_coords.insert("gaussian_wave_packet_4d/lon".to_string(), lon_coords_32);
            dim_coords.insert("procedural_matrix_2d/y".to_string(), xy_coords.clone());
            dim_coords.insert("procedural_matrix_2d/x".to_string(), xy_coords.clone());

            dim_coords.insert("lon".to_string(), clenshaw_x_str);
            dim_coords.insert("lat".to_string(), clenshaw_y_str);
            dim_coords.insert("time".to_string(), t_coords);
            dim_coords.insert("depth".to_string(), z_coords);
            dim_coords.insert("y".to_string(), xy_coords.clone());
            dim_coords.insert("x".to_string(), xy_coords);
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

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        mut on_progress: crate::data::block_store::ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        if self.uri.contains("healpix") {
            let nside = 16;
            let npix = 3072;
            let mut lons_f64 = Vec::with_capacity(npix);
            let mut lats_f64 = Vec::with_capacity(npix);
            for p in 0..npix {
                let (lon_rad, lat_rad) = crate::data::coordinates::healpix::pix2ang_ring(nside, p);
                lons_f64.push(lon_rad.to_degrees() as f64);
                lats_f64.push(lat_rad.to_degrees() as f64);
            }

            let mut coords_map = HashMap::new();
            coords_map.insert("lon".to_string(), lons_f64);
            coords_map.insert("lat".to_string(), lats_f64);

            let t_idx = request
                .selections
                .first()
                .map(|s| s.bounds().0)
                .unwrap_or(0);
            let layer_idx = if request.selections.len() >= 3 {
                request.selections.get(1).map(|s| s.bounds().0).unwrap_or(0)
            } else {
                0
            };

            let values: Vec<f32> = match request.variable.as_str() {
                "lat" => (0..npix)
                    .map(|p| {
                        let (_, lat_rad) =
                            crate::data::coordinates::healpix::pix2ang_ring(nside, p);
                        lat_rad.to_degrees()
                    })
                    .collect(),
                "lon" => (0..npix)
                    .map(|p| {
                        let (lon_rad, _) =
                            crate::data::coordinates::healpix::pix2ang_ring(nside, p);
                        lon_rad.to_degrees()
                    })
                    .collect(),
                "ring" => (0..npix)
                    .map(|p| crate::data::coordinates::healpix::pix2ring(nside, p).0 as f32)
                    .collect(),
                "mslp" => (0..npix)
                    .map(|p| {
                        let (lon_rad, lat_rad) =
                            crate::data::coordinates::healpix::pix2ang_ring(nside, p);
                        let phase = t_idx as f32 * 0.3;
                        1013.25 + 25.0 * (4.0 * lon_rad - phase).cos() * lat_rad.cos().powi(2)
                            - 15.0 * lat_rad.sin()
                    })
                    .collect(),
                _ => (0..npix)
                    .map(|p| {
                        let (lon_rad, lat_rad) =
                            crate::data::coordinates::healpix::pix2ang_ring(nside, p);
                        let phase = t_idx as f32 * 0.3;
                        let alt_decay = layer_idx as f32 * 6.5;
                        285.0 + 35.0 * lat_rad.cos() - 15.0 * lat_rad.sin().powi(2)
                            + 18.0
                                * (4.0 * lon_rad - phase).cos()
                                * lat_rad.cos().powi(2)
                                * (2.0 * lat_rad).sin()
                            - alt_decay
                    })
                    .collect(),
            };

            if let Some(cb) = on_progress {
                cb((values.len() * 4) as u64);
            }

            return Ok(OctantBlock::new(
                request.variable.clone(),
                vec![npix],
                vec!["cell".to_string()],
                vec![0],
                values,
                coords_map,
                HashMap::new(),
            ));
        }

        let (nt_full, nz_full, ny_full, nx_full) = (20, 32, 32, 32);

        if request.variable == "clenshaw_curtis_2d" {
            let (h_full, w_full) = (64, 128);
            let (full_data, _, _) = generate_clenshaw_curtis_2d(w_full, h_full, 0);
            let (xs, ys) = generate_clenshaw_curtis_coords(w_full, h_full);
            return Ok(slice_2d_grid_block(
                &request.variable,
                (w_full, h_full),
                &full_data,
                (&xs, &ys),
                request,
                &mut on_progress,
            ));
        }

        if request.variable == "gaussian_grid_2d" {
            let (h_full, w_full) = (64, 128);
            let (full_data, _, _) = generate_gaussian_grid_2d(w_full, h_full, 0);
            let (xs, ys) = generate_gaussian_coords(w_full, h_full);
            return Ok(slice_2d_grid_block(
                &request.variable,
                (w_full, h_full),
                &full_data,
                (&xs, &ys),
                request,
                &mut on_progress,
            ));
        }

        if request.variable == "stretched_regional_2d" {
            let (h_full, w_full) = (32, 48);
            let (full_data, _, _) = generate_stretched_regional_2d(w_full, h_full);
            let (xs, ys) = generate_stretched_regional_coords(w_full, h_full);
            return Ok(slice_2d_grid_block(
                &request.variable,
                (w_full, h_full),
                &full_data,
                (&xs, &ys),
                request,
                &mut on_progress,
            ));
        }

        if request.variable == "stepped_resolution_2d" {
            let (h_full, w_full) = (32, 64);
            let (full_data, _, _) = generate_stepped_resolution_2d(w_full, h_full);
            let (xs, ys) = generate_stepped_resolution_coords(w_full, h_full);
            return Ok(slice_2d_grid_block(
                &request.variable,
                (w_full, h_full),
                &full_data,
                (&xs, &ys),
                request,
                &mut on_progress,
            ));
        }

        if request.variable == "procedural_matrix_2d" {
            let (h_full, w_full) = (64, 64);
            let (y_start, y_end) = request
                .selections
                .first()
                .map(|s| s.bounds())
                .unwrap_or((0, h_full));
            let (x_start, x_end) = request
                .selections
                .get(1)
                .map(|s| s.bounds())
                .unwrap_or((0, w_full));

            let y_start = y_start.min(h_full);
            let y_end = y_end.min(h_full).max(y_start);
            let x_start = x_start.min(w_full);
            let x_end = x_end.min(w_full).max(x_start);

            let block_h = y_end - y_start;
            let block_w = x_end - x_start;
            let (full_matrix, _, _) = generate_procedural_matrix(w_full, h_full, 0);

            let mut values = Vec::with_capacity(block_h * block_w);
            for y in y_start..y_end {
                for x in x_start..x_end {
                    let idx = y * w_full + x;
                    values.push(full_matrix.get(idx).copied().unwrap_or(0.0));
                }
            }

            if let Some(ref mut cb) = on_progress {
                cb((values.len() * 4) as u64);
            }

            let mut coords = HashMap::new();
            let t_start_x = if w_full > 1 {
                x_start as f64 / (w_full - 1) as f64
            } else {
                0.0
            };
            let t_end_x = if w_full > 1 {
                (x_end.saturating_sub(1)) as f64 / (w_full - 1) as f64
            } else {
                1.0
            };
            coords.insert(
                "x".to_string(),
                vec![-180.0 + t_start_x * 360.0, -180.0 + t_end_x * 360.0],
            );

            let t_start_y = if h_full > 1 {
                y_start as f64 / (h_full - 1) as f64
            } else {
                0.0
            };
            let t_end_y = if h_full > 1 {
                (y_end.saturating_sub(1)) as f64 / (h_full - 1) as f64
            } else {
                1.0
            };
            coords.insert(
                "y".to_string(),
                vec![90.0 - t_start_y * 180.0, 90.0 - t_end_y * 180.0],
            );

            return Ok(OctantBlock::new(
                request.variable.clone(),
                vec![block_h, block_w],
                vec!["y".to_string(), "x".to_string()],
                vec![y_start, x_start],
                values,
                coords,
                HashMap::new(),
            ));
        }

        // 4D Known-Truth Block slicing
        let (t_start, t_end) = request
            .selections
            .first()
            .map(|s| s.bounds())
            .unwrap_or((0, nt_full));
        let (z_start, z_end) = request
            .selections
            .get(1)
            .map(|s| s.bounds())
            .unwrap_or((0, nz_full));
        let (y_start, y_end) = request
            .selections
            .get(2)
            .map(|s| s.bounds())
            .unwrap_or((0, ny_full));
        let (x_start, x_end) = request
            .selections
            .get(3)
            .map(|s| s.bounds())
            .unwrap_or((0, nx_full));

        let t_start = t_start.min(nt_full);
        let t_end = t_end.min(nt_full).max(t_start);
        let z_start = z_start.min(nz_full);
        let z_end = z_end.min(nz_full).max(z_start);
        let y_start = y_start.min(ny_full);
        let y_end = y_end.min(ny_full).max(y_start);
        let x_start = x_start.min(nx_full);
        let x_end = x_end.min(nx_full).max(x_start);

        let dt = (t_end - t_start).max(1);
        let dz = (z_end - z_start).max(1);
        let dy = (y_end - y_start).max(1);
        let dx = (x_end - x_start).max(1);

        let total = dt * dz * dy * dx;
        let mut values = Vec::with_capacity(total);

        for t in t_start..t_end {
            for z in z_start..z_end {
                for y in y_start..y_end {
                    for x in x_start..x_end {
                        let val = eval_known_truth_4d(
                            t, nt_full, z, nz_full, y, ny_full, x, nx_full, None,
                        );
                        values.push(val);
                    }
                }
            }
        }

        if let Some(ref mut cb) = on_progress {
            cb((values.len() * 4) as u64);
        }

        let mut coords = HashMap::new();
        let t_start_lon = if nx_full > 1 {
            x_start as f64 / (nx_full - 1) as f64
        } else {
            0.0
        };
        let t_end_lon = if nx_full > 1 {
            (x_end.saturating_sub(1)) as f64 / (nx_full - 1) as f64
        } else {
            1.0
        };
        coords.insert(
            "lon".to_string(),
            vec![-180.0 + t_start_lon * 360.0, -180.0 + t_end_lon * 360.0],
        );

        let t_start_lat = if ny_full > 1 {
            y_start as f64 / (ny_full - 1) as f64
        } else {
            0.0
        };
        let t_end_lat = if ny_full > 1 {
            (y_end.saturating_sub(1)) as f64 / (ny_full - 1) as f64
        } else {
            1.0
        };
        coords.insert(
            "lat".to_string(),
            vec![90.0 - t_start_lat * 180.0, 90.0 - t_end_lat * 180.0],
        );

        let z_start_m = if nz_full > 1 {
            z_start as f64 * (1000.0 / (nz_full - 1) as f64)
        } else {
            0.0
        };
        let z_end_m = if nz_full > 1 {
            (z_end.saturating_sub(1)) as f64 * (1000.0 / (nz_full - 1) as f64)
        } else {
            1000.0
        };
        coords.insert("depth".to_string(), vec![z_start_m, z_end_m]);
        coords.insert(
            "time".to_string(),
            vec![t_start as f64, (t_end.saturating_sub(1)) as f64],
        );

        Ok(OctantBlock::new(
            request.variable.clone(),
            vec![
                t_end - t_start,
                z_end - z_start,
                y_end - y_start,
                x_end - x_start,
            ],
            vec![
                "time".to_string(),
                "depth".to_string(),
                "lat".to_string(),
                "lon".to_string(),
            ],
            vec![t_start, z_start, y_start, x_start],
            values,
            coords,
            HashMap::new(),
        ))
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        let mut blocks = Vec::with_capacity(requests.len());
        for req in requests {
            blocks.push(self.fetch_block(req)?);
        }
        Ok(BlockResult::new(blocks))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::coordinates::CoordinateGrid;
    use crate::data::slice_request::DimensionSelection;

    #[test]
    fn test_procedural_store_metadata_bounds() {
        let store = ProceduralBlockStore::open("procedural://test").expect("open store");
        let meta = store.inspect().expect("inspect store");

        // Stretched regional bounds in metadata
        let stretched_lon = meta.get_coord_bounds_for_var(Some("stretched_regional_2d"), "lon");
        assert!(stretched_lon.is_some());
        let (min_lon, max_lon) = stretched_lon.unwrap();
        assert!((min_lon - 10.0).abs() < 1e-2);
        assert!((max_lon - 50.0).abs() < 1e-2);

        let stretched_lat = meta.get_coord_bounds_for_var(Some("stretched_regional_2d"), "lat");
        assert!(stretched_lat.is_some());
        let (min_lat, max_lat) = stretched_lat.unwrap();
        assert!((min_lat - 30.0).abs() < 1e-2);
        assert!((max_lat - 60.0).abs() < 1e-2);

        // Stepped resolution bounds in metadata
        let stepped_lon = meta.get_coord_bounds_for_var(Some("stepped_resolution_2d"), "lon");
        assert!(stepped_lon.is_some());
        let (s_min_lon, s_max_lon) = stepped_lon.unwrap();
        assert!((s_min_lon - (-40.0)).abs() < 1e-2);
        assert!((s_max_lon - 40.0).abs() < 1e-2);

        let stepped_lat = meta.get_coord_bounds_for_var(Some("stepped_resolution_2d"), "lat");
        assert!(stepped_lat.is_some());
        let (s_min_lat, s_max_lat) = stepped_lat.unwrap();
        assert!((s_min_lat - (-20.0)).abs() < 1e-2);
        assert!((s_max_lat - 20.0).abs() < 1e-2);
    }

    #[test]
    fn test_procedural_stretched_regional_slicing_and_grid_detection() {
        let store = ProceduralBlockStore::open("procedural://test").expect("open store");
        let mut request = SliceRequest::full_range("stretched_regional_2d", &[32, 48]);
        // Sub-slice: lat 10..20, lon 12..36
        request.selections = vec![
            DimensionSelection::range(10, 20),
            DimensionSelection::range(12, 36),
        ];

        let block = store.fetch_block(&request).expect("fetch block");
        assert_eq!(block.shape, vec![10, 24]);

        let matrix = block
            .slice_2d(1, 0, &[0, 0], 1, "test", true)
            .expect("slice 2d");
        assert_eq!(matrix.width, 24);
        assert_eq!(matrix.height, 10);

        match &matrix.grid {
            CoordinateGrid::Irregular1D {
                coords_x,
                coords_y,
                lon_bounds,
                lat_bounds,
            } => {
                assert_eq!(coords_x.len(), 24);
                assert_eq!(coords_y.len(), 10);
                assert!(lon_bounds.0 >= 10.0 && lon_bounds.1 <= 50.0);
                assert!(lat_bounds.0 >= 30.0 && lat_bounds.1 <= 60.0);
            }
            other => {
                panic!("Expected Irregular1D grid for sliced stretched regional, got {other:?}")
            }
        }
    }

    #[test]
    fn test_procedural_stepped_resolution_slicing_and_grid_detection() {
        let store = ProceduralBlockStore::open("procedural://test").expect("open store");

        // 1. Across-jump sub-slice: lat 0..16, lon 16..48 (spans across the 5x resolution jump at index 32)
        let mut request_jump = SliceRequest::full_range("stepped_resolution_2d", &[32, 64]);
        request_jump.selections = vec![
            DimensionSelection::range(0, 16),
            DimensionSelection::range(16, 48),
        ];

        let block_jump = store.fetch_block(&request_jump).expect("fetch block");
        assert_eq!(block_jump.shape, vec![16, 32]);

        let matrix_jump = block_jump
            .slice_2d(1, 0, &[0, 0], 1, "test", true)
            .expect("slice 2d");
        assert_eq!(matrix_jump.width, 32);
        assert_eq!(matrix_jump.height, 16);

        match &matrix_jump.grid {
            CoordinateGrid::Irregular1D {
                coords_x,
                coords_y,
                lon_bounds,
                lat_bounds,
            } => {
                assert_eq!(coords_x.len(), 32);
                assert_eq!(coords_y.len(), 16);
                assert!(lon_bounds.0 < 0.0 && lon_bounds.1 > 0.0);
                assert!((lat_bounds.1 - 20.0).abs() < 1e-2);
            }
            other => panic!(
                "Expected Irregular1D grid for across-jump stepped resolution, got {other:?}"
            ),
        }

        // 2. Uniform sub-slice: lat 0..16, lon 0..32 (entirely within fine uniform left half)
        let mut request_uniform = SliceRequest::full_range("stepped_resolution_2d", &[32, 64]);
        request_uniform.selections = vec![
            DimensionSelection::range(0, 16),
            DimensionSelection::range(0, 32),
        ];

        let block_uniform = store.fetch_block(&request_uniform).expect("fetch block");
        let matrix_uniform = block_uniform
            .slice_2d(1, 0, &[0, 0], 1, "test", true)
            .expect("slice 2d");

        match &matrix_uniform.grid {
            CoordinateGrid::RegionalRegular {
                lon_bounds,
                lat_bounds,
            } => {
                assert!((lon_bounds.0 - (-40.0)).abs() < 1e-2);
                assert!((lat_bounds.1 - 20.0).abs() < 1e-2);
            }
            other => panic!("Expected RegionalRegular grid for uniform half slice, got {other:?}"),
        }
    }

    #[test]
    fn test_procedural_gaussian_wave_packet_4d_slicing() {
        let store = ProceduralBlockStore::open("procedural://volume").expect("open store");
        let mut request = SliceRequest::full_range("gaussian_wave_packet_4d", &[20, 32, 32, 32]);
        request.selections = vec![
            DimensionSelection::index(2),
            DimensionSelection::index(5),
            DimensionSelection::range(4, 20),
            DimensionSelection::range(8, 24),
        ];

        let block = store.fetch_block(&request).expect("fetch block");
        assert_eq!(block.shape, vec![1, 1, 16, 16]);

        let matrix = block
            .slice_2d(3, 2, &[0, 0, 0, 0], 1, "test", true)
            .expect("slice 2d");
        assert_eq!(matrix.width, 16);
        assert_eq!(matrix.height, 16);

        match &matrix.grid {
            CoordinateGrid::RegionalRegular {
                lon_bounds,
                lat_bounds,
            } => {
                assert!(lon_bounds.0 > -180.0 && lon_bounds.1 < 180.0);
                assert!(lat_bounds.0 > -90.0 && lat_bounds.1 < 90.0);
            }
            other => panic!("Expected RegionalRegular grid for sliced 4d packet, got {other:?}"),
        }
    }
}
