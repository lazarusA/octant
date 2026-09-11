//! Coordinate slicing and linear interpolation utilities for N-D tensors.

use std::collections::HashMap;

/// Extracts or linearly interpolates 1D dimension coordinates for a sub-range `(start, end)`.
pub fn extract_sliced_coords_for_dim(
    coordinates: &HashMap<String, Vec<f64>>,
    dimension_names: &[String],
    shape: &[usize],
    dim_idx: usize,
    range: (usize, usize),
) -> Option<Vec<f64>> {
    let dim_name = dimension_names.get(dim_idx)?;
    let clean = dim_name.trim().to_lowercase();
    let full_len = shape.get(dim_idx).copied().unwrap_or(1);
    let (start, end) = (range.0.min(range.1), range.1.max(range.0).min(full_len));

    if let Some(full_coords) = coordinates
        .get(&clean)
        .or_else(|| coordinates.get(dim_name))
    {
        if full_coords.len() >= full_len && full_coords.len() >= end {
            Some(full_coords[start..end].to_vec())
        } else if full_coords.len() >= 2 && full_len > 1 {
            let first = full_coords[0];
            let last = full_coords[full_coords.len() - 1];
            let t_start = start as f64 / (full_len - 1) as f64;
            let t_end = (end.saturating_sub(1)) as f64 / (full_len - 1) as f64;
            let val_start = first + t_start * (last - first);
            let val_end = first + t_end * (last - first);
            Some(vec![val_start, val_end])
        } else {
            Some(full_coords.clone())
        }
    } else if full_len > 1 {
        // Fallback for spatial dimensions without explicit coordinate tables
        let is_spatial_x = crate::utils::coordinates::is_spatial_x_name(&clean);
        let is_spatial_y = crate::utils::coordinates::is_spatial_y_name(&clean);

        if is_spatial_x {
            let t_start = start as f64 / (full_len - 1) as f64;
            let t_end = (end.saturating_sub(1)) as f64 / (full_len - 1) as f64;
            let val_start = -180.0 + t_start * 360.0;
            let val_end = -180.0 + t_end * 360.0;
            Some(vec![val_start, val_end])
        } else if is_spatial_y {
            let t_start = start as f64 / (full_len - 1) as f64;
            let t_end = (end.saturating_sub(1)) as f64 / (full_len - 1) as f64;
            let val_start = 90.0 - t_start * 180.0;
            let val_end = 90.0 - t_end * 180.0;
            Some(vec![val_start, val_end])
        } else {
            None
        }
    } else {
        None
    }
}
