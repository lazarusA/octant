//! Shared mathematical and indexing utilities for N-dimensional hyperslab slicing.

/// Clamps a `(start, end)` slice range to the dimension length `full_len`.
/// Returns `(start, end, span)`.
pub fn clamp_slice_range(range: (usize, usize), full_len: usize) -> (usize, usize, usize) {
    let start = range.0.min(full_len);
    let end = range.1.clamp(start, full_len);
    let span = end.saturating_sub(start);
    (start, end, span)
}

/// Computes the linear base offset contributed by fixed (pinned) dimension indices.
pub fn compute_fixed_dims_offset(
    fixed_indices: &[usize],
    shape: &[usize],
    strides: &[usize],
    skip_dim_a: usize,
    skip_dim_b: usize,
    skip_dim_c: Option<usize>,
) -> usize {
    let mut base_offset = 0usize;
    let rank = shape.len();

    for (i, &fixed_idx) in fixed_indices.iter().enumerate().take(rank) {
        if i != skip_dim_a && i != skip_dim_b && skip_dim_c != Some(i) {
            let idx = fixed_idx.min(shape[i].saturating_sub(1));
            base_offset += idx * strides[i];
        }
    }

    base_offset
}

/// Resolves min and max values based on computed or pre-existing block bounds.
pub fn resolve_min_max(
    compute_bounds: bool,
    values: &[f32],
    block_min: f32,
    block_max: f32,
) -> (f32, f32) {
    if compute_bounds {
        crate::utils::compute_finite_min_max(values)
    } else if block_min.is_finite() && block_max.is_finite() {
        (block_min, block_max)
    } else {
        (0.0, 1.0)
    }
}
