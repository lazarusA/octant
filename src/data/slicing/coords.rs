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
    let full_coords = coordinates
        .get(&clean)
        .or_else(|| coordinates.get(dim_name))?;
    let full_len = shape.get(dim_idx).copied().unwrap_or(1);
    let (start, end) = (range.0.min(range.1), range.1.max(range.0).min(full_len));

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
}

/// Extracts a 2D sub-rectangle `(x_start..x_end, y_start..y_end)` from a `CurvilinearCoord2D`.
pub fn extract_sliced_curvilinear_coord_2d(
    coord: &crate::data::CurvilinearCoord2D,
    x_range: (usize, usize),
    y_range: (usize, usize),
) -> Option<crate::data::CurvilinearCoord2D> {
    let x_start = x_range.0.min(x_range.1).min(coord.width);
    let x_end = x_range.1.max(x_range.0).min(coord.width);
    let y_start = y_range.0.min(y_range.1).min(coord.height);
    let y_end = y_range.1.max(y_range.0).min(coord.height);

    let sliced_w = x_end.saturating_sub(x_start);
    let sliced_h = y_end.saturating_sub(y_start);

    if sliced_w == 0 || sliced_h == 0 {
        return None;
    }

    if x_start == 0 && x_end == coord.width && y_start == 0 && y_end == coord.height {
        return Some(coord.clone());
    }

    let mut out = Vec::with_capacity(sliced_w.checked_mul(sliced_h).unwrap_or(0));
    for y in y_start..y_end {
        let row_offset = y * coord.width;
        let row_slice = &coord.values[row_offset + x_start..row_offset + x_end];
        out.extend_from_slice(row_slice);
    }

    Some(crate::data::CurvilinearCoord2D {
        values: out.into(),
        width: sliced_w,
        height: sliced_h,
    })
}
