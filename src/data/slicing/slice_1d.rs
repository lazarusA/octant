//! 1D and scalar (0D) slice extraction formatted as MatrixData.

use crate::data::CoordinateGrid;
use crate::data::MatrixData;
use crate::data::octant_block::OctantBlock;
use crate::data::slicing::common::{clamp_slice_range, compute_fixed_dims_offset, resolve_min_max};
use crate::data::slicing::coords::extract_sliced_coords_for_dim;

/// Extracts a 1D slice representation formatted as MatrixData (width x 1).
pub fn slice_1d(
    block: &OctantBlock,
    x_dim: usize,
    x_range: (usize, usize),
    fixed_indices: Option<&[usize]>,
    max_timesteps: usize,
    dataset_name: &str,
    compute_bounds: bool,
) -> Option<MatrixData> {
    if x_dim >= block.rank() {
        return None;
    }
    let full_x = block.shape.get(x_dim).copied().unwrap_or(1);
    let (x_start, x_end, width) = clamp_slice_range(x_range, full_x);
    if width == 0 {
        return None;
    }

    let stride_x = block.strides.get(x_dim).copied().unwrap_or(1);
    let base_offset = if let Some(fixed) = fixed_indices {
        compute_fixed_dims_offset(fixed, &block.shape, &block.strides, x_dim, x_dim, None)
    } else {
        0
    };

    let values: Vec<f32> = if stride_x == 1 {
        let start = base_offset + x_start;
        let end = base_offset + x_end;
        if end <= block.values.len() {
            block.values[start..end].to_vec()
        } else {
            (x_start..x_end)
                .map(|x| {
                    block
                        .values
                        .get(base_offset + x * stride_x)
                        .copied()
                        .unwrap_or(f32::NAN)
                })
                .collect()
        }
    } else {
        (x_start..x_end)
            .map(|x| {
                block
                    .values
                    .get(base_offset + x * stride_x)
                    .copied()
                    .unwrap_or(f32::NAN)
            })
            .collect()
    };

    let (min_val, max_val) =
        resolve_min_max(compute_bounds, &values, block.min_value, block.max_value);
    let x_name = block
        .dimension_names
        .get(x_dim)
        .map(|s| s.as_str())
        .unwrap_or("x");
    let x_coords = extract_sliced_coords_for_dim(
        &block.coordinates,
        &block.dimension_names,
        &block.shape,
        x_dim,
        (x_start, x_end),
    );
    let grid = CoordinateGrid::detect_grid_from_block(
        block,
        x_name,
        "y",
        x_coords.as_deref(),
        None,
        width,
        1,
    );

    Some(MatrixData::new_with_grid(
        width,
        1,
        values,
        min_val,
        max_val,
        dataset_name.to_string(),
        max_timesteps,
        grid,
    ))
}

/// Extracts a scalar (0D) slice formatted as 1x1 MatrixData.
pub fn slice_0d(
    block: &OctantBlock,
    max_timesteps: usize,
    dataset_name: &str,
) -> Option<MatrixData> {
    let val = block.values.first().copied().unwrap_or(0.0);
    Some(MatrixData::new(
        1,
        1,
        vec![val],
        val,
        val,
        dataset_name.to_string(),
        max_timesteps,
    ))
}
