//! Coordinate Lookup Table (LUT) generation for GPU shader index mapping.

use super::search::find_coord_cell_1d;

/// Computes the optimal lookup table (LUT) size for a 1D coordinate axis.
/// Scales dynamically as clamp(dim_len * 2, 4096, 65536) to prevent cell undersampling
/// while providing sub-pixel precision on 4K displays.
#[inline]
pub fn compute_coord_lut_size(dim_len: usize) -> usize {
    (dim_len.saturating_mul(2)).clamp(4096, 65536)
}

/// Builds a 1D coordinate lookup table mapping normalized [0, 1] positions to cell indices.
/// Returns a `Vec<f32>` where each element is the cell index (stored as f32 for GPU storage buffers).
pub fn build_1d_coord_lut(coords: &[f32], lut_size: usize) -> Vec<f32> {
    if coords.is_empty() {
        return Vec::new();
    }
    let m = lut_size.max(1);
    if coords.len() == 1 {
        return vec![0.0; m];
    }
    let first = coords[0];
    let last = coords[coords.len() - 1];
    let span = last - first;
    let mut lut = Vec::with_capacity(m);
    let m_denom = (m - 1).max(1) as f32;
    for k in 0..m {
        let u = k as f32 / m_denom;
        let target = first + u * span;
        let cell_idx = find_coord_cell_1d(coords, target) as f32;
        lut.push(cell_idx);
    }
    lut
}
