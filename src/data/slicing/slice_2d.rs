//! 2D hyperslab slicing for MatrixData representations.

use crate::data::CoordinateGrid;
use crate::data::MatrixData;
use crate::data::octant_block::OctantBlock;
use crate::data::slicing::common::{clamp_slice_range, compute_fixed_dims_offset, resolve_min_max};
use crate::data::slicing::coords::extract_sliced_coords_for_dim;
use crate::data::slicing::copy::{
    copy_contiguous_slice, copy_row_contiguous_slice, copy_strided_slice,
};
use crate::data::slicing::slice_1d::{slice_0d, slice_1d};

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
        return slice_1d(
            block,
            0,
            x_range,
            None,
            max_timesteps,
            dataset_name,
            compute_bounds,
        );
    }
    if block.rank() == 0 {
        return slice_0d(block, max_timesteps, dataset_name);
    }

    if x_dim >= block.rank() || fixed_indices.len() != block.rank() {
        return None;
    }

    if x_dim == y_dim || y_dim >= block.rank() {
        return slice_1d(
            block,
            x_dim,
            x_range,
            Some(fixed_indices),
            max_timesteps,
            dataset_name,
            compute_bounds,
        );
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

    let grid = CoordinateGrid::detect_grid_from_block(
        block,
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
