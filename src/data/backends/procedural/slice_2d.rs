//! 2D grid hyperslab extraction for procedural datasets.

use std::collections::HashMap;

use crate::data::blocks::BlockStoreError;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

pub fn slice_2d_grid_block(
    var_name: &str,
    shape: (usize, usize),
    full_data: &[f32],
    coords: (&[f64], &[f64]),
    request: &SliceRequest,
) -> Result<OctantBlock, BlockStoreError> {
    let (w_full, h_full) = shape;
    let (xs, ys) = coords;

    let (y_start, y_end) = request.clamped(0, h_full);
    let (x_start, x_end) = request.clamped(1, w_full);

    let block_h = y_end - y_start;
    let block_w = x_end - x_start;

    let mut values = Vec::with_capacity(block_h * block_w);
    for y in y_start..y_end {
        for x in x_start..x_end {
            let idx = y * w_full + x;
            values.push(full_data.get(idx).copied().unwrap_or(0.0));
        }
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
