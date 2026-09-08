//! 2D Hyperslab Slicing from N-Dimensional Resident Blocks.

use crate::data::CoordinateGrid;
use crate::data::matrix_data::MatrixData;
use crate::data::octant_block::OctantBlock;
use crate::data::slicing::common::{clamp_slice_range, compute_fixed_dims_offset, resolve_min_max};
use crate::data::slicing::coords::extract_sliced_coords_for_dim;

/// Extracts a 1D slice representation formatted as MatrixData (width x 1).
fn slice_1d(
    block: &OctantBlock,
    x_range: (usize, usize),
    max_timesteps: usize,
    dataset_name: &str,
    compute_bounds: bool,
) -> Option<MatrixData> {
    let full_x = block.shape.first().copied().unwrap_or(1);
    let (x_start, x_end, width) = clamp_slice_range(x_range, full_x);
    if width == 0 {
        return None;
    }

    let values: Vec<f32> = if x_start + width <= block.values.len() {
        block.values[x_start..x_start + width].to_vec()
    } else {
        (0..width)
            .map(|x| block.values.get(x_start + x).copied().unwrap_or(f32::NAN))
            .collect()
    };

    let (min_val, max_val) =
        resolve_min_max(compute_bounds, &values, block.min_value, block.max_value);
    let x_name = block
        .dimension_names
        .first()
        .map(|s| s.as_str())
        .unwrap_or("x");
    let x_coords = extract_sliced_coords_for_dim(
        &block.coordinates,
        &block.dimension_names,
        &block.shape,
        0,
        (x_start, x_end),
    );
    let grid = CoordinateGrid::detect_grid(x_name, "y", x_coords.as_deref(), None, width, 1);

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
fn slice_0d(block: &OctantBlock, max_timesteps: usize, dataset_name: &str) -> Option<MatrixData> {
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

/// Copies values for contiguous row and column slices.
fn copy_contiguous_slice(
    values: &[f32],
    base_offset: usize,
    y_start: usize,
    y_end: usize,
    stride_y: usize,
    width: usize,
) -> Vec<f32> {
    let slice_len = width * (y_end - y_start);
    let slice_start = base_offset + y_start * stride_y;
    let slice_end = slice_start + slice_len;

    if slice_end <= values.len() {
        values[slice_start..slice_end].to_vec()
    } else {
        let mut result = Vec::with_capacity(slice_len);
        for y in y_start..y_end {
            let row_start = base_offset + y * stride_y;
            let row_end = row_start + width;
            if row_end <= values.len() {
                result.extend_from_slice(&values[row_start..row_end]);
            } else {
                for x in 0..width {
                    result.push(values.get(row_start + x).copied().unwrap_or(f32::NAN));
                }
            }
        }
        result
    }
}

/// Copies values when rows are contiguous in X (stride_x == 1).
#[allow(clippy::too_many_arguments)]
fn copy_row_contiguous_slice(
    values: &[f32],
    base_offset: usize,
    y_start: usize,
    y_end: usize,
    stride_y: usize,
    x_start: usize,
    width: usize,
) -> Vec<f32> {
    let slice_len = width * (y_end - y_start);
    let mut result = Vec::with_capacity(slice_len);

    for y in y_start..y_end {
        let row_start = base_offset + y * stride_y + x_start;
        let row_end = row_start + width;
        if row_end <= values.len() {
            result.extend_from_slice(&values[row_start..row_end]);
        } else {
            for x in 0..width {
                result.push(values.get(row_start + x).copied().unwrap_or(f32::NAN));
            }
        }
    }

    result
}

/// Copies values for arbitrary strided X and Y slices.
#[allow(clippy::too_many_arguments)]
fn copy_strided_slice(
    values: &[f32],
    base_offset: usize,
    y_start: usize,
    y_end: usize,
    stride_y: usize,
    x_start: usize,
    x_end: usize,
    stride_x: usize,
) -> Vec<f32> {
    let width = x_end.saturating_sub(x_start);
    let height = y_end.saturating_sub(y_start);
    let mut result = Vec::with_capacity(width * height);

    for y in y_start..y_end {
        let row_start = base_offset + y * stride_y;
        for x in x_start..x_end {
            let idx = row_start + x * stride_x;
            result.push(values.get(idx).copied().unwrap_or(f32::NAN));
        }
    }

    result
}

/// Slices a 2D matrix from an OctantBlock given X/Y dimension indices and fixed indices.
#[allow(clippy::too_many_arguments)]
pub fn slice_2d_with_ranges(
    block: &OctantBlock,
    x_dim: usize,
    y_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
    fixed_indices: &[usize],
    max_timesteps: usize,
    dataset_name: &str,
    compute_bounds: bool,
) -> Option<MatrixData> {
    if block.rank() == 1 {
        return slice_1d(block, x_range, max_timesteps, dataset_name, compute_bounds);
    }
    if block.rank() == 0 {
        return slice_0d(block, max_timesteps, dataset_name);
    }

    if x_dim == y_dim
        || x_dim >= block.rank()
        || y_dim >= block.rank()
        || fixed_indices.len() != block.rank()
    {
        return None;
    }

    let full_x = block.shape[x_dim];
    let full_y = block.shape[y_dim];

    let (x_start, x_end, width) = clamp_slice_range(x_range, full_x);
    let (y_start, y_end, height) = clamp_slice_range(y_range, full_y);

    if width == 0 || height == 0 {
        return None;
    }

    let stride_x = block.strides[x_dim];
    let stride_y = block.strides[y_dim];
    let base_offset = compute_fixed_dims_offset(
        fixed_indices,
        &block.shape,
        &block.strides,
        x_dim,
        y_dim,
        None,
    );

    let values = if stride_x == 1 && x_start == 0 && width == full_x && stride_y == width {
        copy_contiguous_slice(&block.values, base_offset, y_start, y_end, stride_y, width)
    } else if stride_x == 1 {
        copy_row_contiguous_slice(
            &block.values,
            base_offset,
            y_start,
            y_end,
            stride_y,
            x_start,
            width,
        )
    } else {
        copy_strided_slice(
            &block.values,
            base_offset,
            y_start,
            y_end,
            stride_y,
            x_start,
            x_end,
            stride_x,
        )
    };

    let (min_val, max_val) =
        resolve_min_max(compute_bounds, &values, block.min_value, block.max_value);
    let x_name = block
        .dimension_names
        .get(x_dim)
        .map(|s| s.as_str())
        .unwrap_or("x");
    let y_name = block
        .dimension_names
        .get(y_dim)
        .map(|s| s.as_str())
        .unwrap_or("y");

    let x_coords = extract_sliced_coords_for_dim(
        &block.coordinates,
        &block.dimension_names,
        &block.shape,
        x_dim,
        (x_start, x_end),
    );
    let y_coords = extract_sliced_coords_for_dim(
        &block.coordinates,
        &block.dimension_names,
        &block.shape,
        y_dim,
        (y_start, y_end),
    );

    let grid = CoordinateGrid::detect_grid(
        x_name,
        y_name,
        x_coords.as_deref(),
        y_coords.as_deref(),
        width,
        height,
    );

    Some(MatrixData::new_with_grid(
        width,
        height,
        values,
        min_val,
        max_val,
        dataset_name.to_string(),
        max_timesteps,
        grid,
    ))
}
