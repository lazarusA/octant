//! In-memory procedural and known-truth synthetic block store.

use std::collections::HashMap;

use crate::data::{
    CurvilinearCoord2D,
    block_request::BlockResult,
    block_store::{BlockStore, BlockStoreError},
    metadata::{DatasetMetadata, VariableInfo},
    octant_block::OctantBlock,
    procedural::{
        eval_known_truth_4d, generate_clenshaw_curtis_2d, generate_clenshaw_curtis_coords,
        generate_curvilinear_antimeridian_grid, generate_curvilinear_orca_grid,
        generate_curvilinear_swirl_grid, generate_gaussian_coords, generate_gaussian_grid_2d,
        generate_procedural_matrix, generate_stepped_resolution_2d,
        generate_stepped_resolution_coords, generate_stretched_regional_2d,
        generate_stretched_regional_coords,
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

impl BlockStore for ProceduralBlockStore {
    fn backend_name(&self) -> &str {
        "Procedural / Known-Truth"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        let is_4d = self.uri.contains("volume") || self.uri.contains("4d");
        let is_curv = self.uri.contains("curvilinear");
        if is_4d {
            Ok(vec![
                "gaussian_wave_packet_4d".to_string(),
                "procedural_matrix_2d".to_string(),
            ])
        } else if is_curv {
            Ok(vec![
                "curvilinear_orca_ocean".to_string(),
                "curvilinear_swirl_vortex".to_string(),
                "curvilinear_antimeridian_crossing".to_string(),
            ])
        } else {
            Ok(vec![
                "curvilinear_orca_ocean".to_string(),
                "curvilinear_swirl_vortex".to_string(),
                "curvilinear_antimeridian_crossing".to_string(),
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
                    name: "curvilinear_orca_ocean".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![64, 128],
                    chunk_shape: vec![64, 128],
                    dimension_names: vec!["y".to_string(), "x".to_string()],
                    units: Some("degC".to_string()),
                    long_name: Some("2D Tripolar / ORCA-like Ocean Grid (Procedural)".to_string()),
                    temporal_resolution: None,
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 64 * 128 * 4,
                    attributes: HashMap::new(),
                },
                VariableInfo {
                    name: "curvilinear_swirl_vortex".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![64, 64],
                    chunk_shape: vec![64, 64],
                    dimension_names: vec!["y".to_string(), "x".to_string()],
                    units: Some("m/s".to_string()),
                    long_name: Some(
                        "2D Swirling Sheared Atmospheric Mesh (Procedural)".to_string(),
                    ),
                    temporal_resolution: None,
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 64 * 64 * 4,
                    attributes: HashMap::new(),
                },
                VariableInfo {
                    name: "curvilinear_antimeridian_crossing".to_string(),
                    data_type: "float32".to_string(),
                    shape: vec![48, 64],
                    chunk_shape: vec![48, 64],
                    dimension_names: vec!["y".to_string(), "x".to_string()],
                    units: Some("hPa".to_string()),
                    long_name: Some(
                        "2D Curvilinear Antimeridian Crossing [150E..150W] (Procedural)"
                            .to_string(),
                    ),
                    temporal_resolution: None,
                    time_coverage_start: None,
                    time_coverage_end: None,
                    file_size: 48 * 64 * 4,
                    attributes: HashMap::new(),
                },
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

        let (clenshaw_x, clenshaw_y) = generate_clenshaw_curtis_coords(128, 64);
        let mut dim_coords = HashMap::new();
        dim_coords.insert(
            "lon".to_string(),
            clenshaw_x.iter().map(|v| format!("{v:.3}")).collect(),
        );
        dim_coords.insert(
            "lat".to_string(),
            clenshaw_y.iter().map(|v| format!("{v:.3}")).collect(),
        );

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
        let (nt_full, nz_full, ny_full, nx_full) = (20, 32, 32, 32);

        if request.variable == "curvilinear_orca_ocean" {
            let (h, w) = (64, 128);
            let (lons, lats, data, _, _) = generate_curvilinear_orca_grid(w, h, 0);
            let mut curv_coords = HashMap::new();
            curv_coords.insert(
                "nav_lon".to_string(),
                CurvilinearCoord2D {
                    values: lons.into(),
                    width: w,
                    height: h,
                },
            );
            curv_coords.insert(
                "nav_lat".to_string(),
                CurvilinearCoord2D {
                    values: lats.into(),
                    width: w,
                    height: h,
                },
            );

            if let Some(ref mut cb) = on_progress {
                cb((data.len() * 4) as u64);
            }
            return Ok(OctantBlock::new(
                request.variable.clone(),
                vec![h, w],
                vec!["y".to_string(), "x".to_string()],
                vec![0, 0],
                data,
                HashMap::new(),
                HashMap::new(),
            )
            .with_curvilinear_coordinates(curv_coords));
        }

        if request.variable == "curvilinear_swirl_vortex" {
            let (h, w) = (64, 64);
            let (lons, lats, data, _, _) = generate_curvilinear_swirl_grid(w, h, 0);
            let mut curv_coords = HashMap::new();
            curv_coords.insert(
                "nav_lon".to_string(),
                CurvilinearCoord2D {
                    values: lons.into(),
                    width: w,
                    height: h,
                },
            );
            curv_coords.insert(
                "nav_lat".to_string(),
                CurvilinearCoord2D {
                    values: lats.into(),
                    width: w,
                    height: h,
                },
            );

            if let Some(ref mut cb) = on_progress {
                cb((data.len() * 4) as u64);
            }
            return Ok(OctantBlock::new(
                request.variable.clone(),
                vec![h, w],
                vec!["y".to_string(), "x".to_string()],
                vec![0, 0],
                data,
                HashMap::new(),
                HashMap::new(),
            )
            .with_curvilinear_coordinates(curv_coords));
        }

        if request.variable == "curvilinear_antimeridian_crossing" {
            let (h, w) = (48, 64);
            let (lons, lats, data, _, _) = generate_curvilinear_antimeridian_grid(w, h, 0);
            let mut curv_coords = HashMap::new();
            curv_coords.insert(
                "nav_lon".to_string(),
                CurvilinearCoord2D {
                    values: lons.into(),
                    width: w,
                    height: h,
                },
            );
            curv_coords.insert(
                "nav_lat".to_string(),
                CurvilinearCoord2D {
                    values: lats.into(),
                    width: w,
                    height: h,
                },
            );

            if let Some(ref mut cb) = on_progress {
                cb((data.len() * 4) as u64);
            }
            return Ok(OctantBlock::new(
                request.variable.clone(),
                vec![h, w],
                vec!["y".to_string(), "x".to_string()],
                vec![0, 0],
                data,
                HashMap::new(),
                HashMap::new(),
            )
            .with_curvilinear_coordinates(curv_coords));
        }

        if request.variable == "clenshaw_curtis_2d" {
            let (h, w) = (64, 128);
            let (data, _, _) = generate_clenshaw_curtis_2d(w, h, 0);
            let (xs, ys) = generate_clenshaw_curtis_coords(w, h);
            let mut coords = HashMap::new();
            coords.insert("lon".to_string(), xs);
            coords.insert("lat".to_string(), ys);

            if let Some(ref mut cb) = on_progress {
                cb((data.len() * 4) as u64);
            }
            return Ok(OctantBlock::new(
                request.variable.clone(),
                vec![h, w],
                vec!["lat".to_string(), "lon".to_string()],
                vec![0, 0],
                data,
                coords,
                HashMap::new(),
            ));
        }

        if request.variable == "gaussian_grid_2d" {
            let (h, w) = (64, 128);
            let (data, _, _) = generate_gaussian_grid_2d(w, h, 0);
            let (xs, ys) = generate_gaussian_coords(w, h);
            let mut coords = HashMap::new();
            coords.insert("lon".to_string(), xs);
            coords.insert("lat".to_string(), ys);

            if let Some(ref mut cb) = on_progress {
                cb((data.len() * 4) as u64);
            }
            return Ok(OctantBlock::new(
                request.variable.clone(),
                vec![h, w],
                vec!["lat".to_string(), "lon".to_string()],
                vec![0, 0],
                data,
                coords,
                HashMap::new(),
            ));
        }

        if request.variable == "stretched_regional_2d" {
            let (h, w) = (32, 48);
            let (data, _, _) = generate_stretched_regional_2d(w, h);
            let (xs, ys) = generate_stretched_regional_coords(w, h);
            let mut coords = HashMap::new();
            coords.insert("lon".to_string(), xs);
            coords.insert("lat".to_string(), ys);

            if let Some(ref mut cb) = on_progress {
                cb((data.len() * 4) as u64);
            }
            return Ok(OctantBlock::new(
                request.variable.clone(),
                vec![h, w],
                vec!["lat".to_string(), "lon".to_string()],
                vec![0, 0],
                data,
                coords,
                HashMap::new(),
            ));
        }

        if request.variable == "stepped_resolution_2d" {
            let (h, w) = (32, 64);
            let (data, _, _) = generate_stepped_resolution_2d(w, h);
            let (xs, ys) = generate_stepped_resolution_coords(w, h);
            let mut coords = HashMap::new();
            coords.insert("lon".to_string(), xs);
            coords.insert("lat".to_string(), ys);

            if let Some(ref mut cb) = on_progress {
                cb((data.len() * 4) as u64);
            }
            return Ok(OctantBlock::new(
                request.variable.clone(),
                vec![h, w],
                vec!["lat".to_string(), "lon".to_string()],
                vec![0, 0],
                data,
                coords,
                HashMap::new(),
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

            return Ok(OctantBlock::new(
                request.variable.clone(),
                vec![block_h, block_w],
                vec!["y".to_string(), "x".to_string()],
                vec![y_start, x_start],
                values,
                HashMap::new(),
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
            HashMap::new(),
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
