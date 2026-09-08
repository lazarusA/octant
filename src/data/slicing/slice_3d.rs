//! 3D Volume Slicing from N-Dimensional Resident Blocks.

use crate::data::octant_block::OctantBlock;
use crate::data::slicing::common::{clamp_slice_range, compute_fixed_dims_offset, resolve_min_max};
use crate::data::volume_data::VolumeData;

/// Computes the clamped effective depth ensuring the total voxel count stays within GPU storage limits.
pub fn clamp_gpu_volume_depth(nx: usize, ny: usize, nz: usize) -> usize {
    let slice_elements = nx.saturating_mul(ny).max(1);
    let max_z =
        (crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS / slice_elements).clamp(1, nz);
    nz.min(max_z)
}

/// Checks if the volume slice represents the entire 3D tensor buffer contiguously.
#[allow(clippy::too_many_arguments)]
fn is_full_contiguous_3d(
    block: &OctantBlock,
    x_dim: usize,
    y_dim: usize,
    z_dim: usize,
    x_start: usize,
    nx: usize,
    y_start: usize,
    ny: usize,
    z_start: usize,
    nz: usize,
    eff_nz: usize,
) -> bool {
    block.rank() == 3
        && x_dim == 0
        && y_dim == 1
        && z_dim == 2
        && x_start == 0
        && nx == block.shape[0]
        && y_start == 0
        && ny == block.shape[1]
        && z_start == 0
        && nz == block.shape[2]
        && block.strides[0] == block.shape[1] * block.shape[2]
        && block.strides[1] == block.shape[2]
        && block.strides[2] == 1
        && nz == eff_nz
}

/// Copies voxels when rows in X are contiguous (stride_x == 1).
#[allow(clippy::too_many_arguments)]
fn copy_row_contiguous_3d(
    values: &[f32],
    base_offset: usize,
    z_start: usize,
    eff_nz: usize,
    stride_z: usize,
    y_start: usize,
    y_end: usize,
    stride_y: usize,
    x_start: usize,
    nx: usize,
) -> Vec<f32> {
    let mut result = Vec::with_capacity(nx * (y_end - y_start) * eff_nz);

    for z in z_start..(z_start + eff_nz) {
        let z_offset = base_offset + z * stride_z;
        for y in y_start..y_end {
            let row_start = z_offset + y * stride_y + x_start;
            let row_end = row_start + nx;
            if row_end <= values.len() {
                result.extend_from_slice(&values[row_start..row_end]);
            } else {
                for x in 0..nx {
                    result.push(values.get(row_start + x).copied().unwrap_or(f32::NAN));
                }
            }
        }
    }

    result
}

/// Copies voxels for arbitrary strided 3D slices.
#[allow(clippy::too_many_arguments)]
fn copy_strided_3d(
    values: &[f32],
    base_offset: usize,
    z_start: usize,
    eff_nz: usize,
    stride_z: usize,
    y_start: usize,
    y_end: usize,
    stride_y: usize,
    x_start: usize,
    x_end: usize,
    stride_x: usize,
) -> Vec<f32> {
    let nx = x_end.saturating_sub(x_start);
    let ny = y_end.saturating_sub(y_start);
    let mut result = Vec::with_capacity(nx * ny * eff_nz);

    for z in z_start..(z_start + eff_nz) {
        let z_offset = base_offset + z * stride_z;
        for y in y_start..y_end {
            let row_start = z_offset + y * stride_y;
            for x in x_start..x_end {
                let idx = row_start + x * stride_x;
                result.push(values.get(idx).copied().unwrap_or(f32::NAN));
            }
        }
    }

    result
}

/// Extracts a 3D VolumeData from an OctantBlock given X, Y, Z dimension indices and ranges.
#[allow(clippy::too_many_arguments)]
pub fn volume_with_ranges(
    block: &OctantBlock,
    x_dim: usize,
    y_dim: usize,
    z_dim: usize,
    x_range: (usize, usize),
    y_range: (usize, usize),
    z_range: (usize, usize),
    fixed_indices: &[usize],
    dataset_name: &str,
    compute_bounds: bool,
) -> Option<VolumeData> {
    let has_z = z_dim < block.rank();

    if x_dim == y_dim
        || x_dim >= block.rank()
        || y_dim >= block.rank()
        || (has_z && (z_dim == x_dim || z_dim == y_dim))
        || fixed_indices.len() != block.rank()
    {
        return None;
    }

    let full_x = block.shape[x_dim];
    let full_y = block.shape[y_dim];
    let full_z = if has_z { block.shape[z_dim] } else { 1 };

    let (x_start, x_end, nx) = clamp_slice_range(x_range, full_x);
    let (y_start, y_end, ny) = clamp_slice_range(y_range, full_y);
    let (z_start, _z_end, nz) = if has_z {
        clamp_slice_range(z_range, full_z)
    } else {
        (0, 1, 1)
    };

    if nx == 0 || ny == 0 || nz == 0 {
        return None;
    }

    let eff_nz = clamp_gpu_volume_depth(nx, ny, nz);

    if is_full_contiguous_3d(
        block, x_dim, y_dim, z_dim, x_start, nx, y_start, ny, z_start, nz, eff_nz,
    ) {
        let (min_val, max_val) = resolve_min_max(
            compute_bounds,
            &block.values,
            block.min_value,
            block.max_value,
        );
        return Some(VolumeData::new(
            nx,
            ny,
            nz,
            block.values.to_vec(),
            min_val,
            max_val,
            dataset_name.to_string(),
        ));
    }

    let stride_x = block.strides[x_dim];
    let stride_y = block.strides[y_dim];
    let stride_z = if has_z { block.strides[z_dim] } else { 0 };

    let base_offset = compute_fixed_dims_offset(
        fixed_indices,
        &block.shape,
        &block.strides,
        x_dim,
        y_dim,
        if has_z { Some(z_dim) } else { None },
    );

    let values = if stride_x == 1 {
        copy_row_contiguous_3d(
            &block.values,
            base_offset,
            z_start,
            eff_nz,
            stride_z,
            y_start,
            y_end,
            stride_y,
            x_start,
            nx,
        )
    } else {
        copy_strided_3d(
            &block.values,
            base_offset,
            z_start,
            eff_nz,
            stride_z,
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

    Some(VolumeData::new(
        nx,
        ny,
        eff_nz,
        values,
        min_val,
        max_val,
        dataset_name.to_string(),
    ))
}
