//! In-memory procedural and known-truth synthetic block store.

use std::collections::HashMap;

use crate::data::{
    block_request::BlockResult,
    block_store::{BlockStore, BlockStoreError},
    metadata::{DatasetMetadata, VariableInfo},
    octant_block::OctantBlock,
    procedural::{eval_known_truth_4d, generate_procedural_matrix},
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
        Ok(vec![
            "gaussian_wave_packet_4d".to_string(),
            "procedural_matrix_2d".to_string(),
        ])
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
            ]
        };

        Ok(DatasetMetadata {
            name: if is_4d {
                "4D Known-Truth Procedural Store".to_string()
            } else {
                "2D Procedural Store".to_string()
            },
            store_type: "Procedural / Ground Truth".to_string(),
            variables: vars,
            dimension_coordinates: HashMap::new(),
        })
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        mut on_progress: Option<&mut (dyn FnMut(u64) + Send)>,
    ) -> Result<OctantBlock, BlockStoreError> {
        let (nt_full, nz_full, ny_full, nx_full) = (20, 32, 32, 32);

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
