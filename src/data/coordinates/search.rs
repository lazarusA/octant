//! Binary search utilities for 1D and 2D coordinate arrays.

/// Binary searches a 1D monotonic (ascending or descending) coordinate array for the nearest cell index.
/// Matches `find_coord_cell_x` and `find_coord_cell_y` in WGSL shaders.
pub fn find_coord_cell_1d(coords: &[f32], query_val: f32) -> usize {
    let len = coords.len();
    if len <= 1 {
        return 0;
    }
    let max_idx = len - 1;
    let first = coords[0];
    let last = coords[max_idx];
    let is_descending = first > last;

    let mut low = 0;
    let mut high = max_idx.saturating_sub(1);

    if is_descending {
        while low < high {
            let mid = (low + high).div_ceil(2);
            if coords[mid] >= query_val {
                low = mid;
            } else {
                high = mid.saturating_sub(1);
            }
        }
    } else {
        while low < high {
            let mid = (low + high).div_ceil(2);
            if coords[mid] <= query_val {
                low = mid;
            } else {
                high = mid.saturating_sub(1);
            }
        }
    }

    let next = (low + 1).min(max_idx);
    if (query_val - coords[low]).abs() <= (query_val - coords[next]).abs() {
        low
    } else {
        next
    }
}

/// Finds the nearest cell index `(px, py)` in a 2D curvilinear grid using a fast 2-tier spatial chord distance search.
pub fn find_curvilinear_cell_2d(
    lons: &[f32],
    lats: &[f32],
    lon_rad: f32,
    lat_rad: f32,
    width: usize,
    height: usize,
) -> (usize, usize) {
    if width == 0 || height == 0 || lons.is_empty() || lats.is_empty() {
        return (0, 0);
    }
    let total = lons.len().min(lats.len()).min(width * height);
    let tx = lat_rad.cos() * lon_rad.sin();
    let ty = lat_rad.sin();
    let tz = lat_rad.cos() * lon_rad.cos();

    let stride = if width >= 64 && height >= 64 { 8 } else { 4 };
    let mut best_dist_sq = f32::MAX;
    let mut best_coarse_x = 0usize;
    let mut best_coarse_y = 0usize;

    let mut y = 0;
    while y < height {
        let row_offset = y * width;
        let mut x = 0;
        while x < width {
            let idx = row_offset + x;
            if idx < total {
                let clon_rad = lons[idx].to_radians();
                let clat_rad = lats[idx].to_radians();
                let cx = clat_rad.cos() * clon_rad.sin();
                let cy = clat_rad.sin();
                let cz = clat_rad.cos() * clon_rad.cos();
                let dist_sq = (cx - tx).powi(2) + (cy - ty).powi(2) + (cz - tz).powi(2);
                if dist_sq < best_dist_sq {
                    best_dist_sq = dist_sq;
                    best_coarse_x = x;
                    best_coarse_y = y;
                }
            }
            x += stride;
        }
        y += stride;
    }

    let y_min = best_coarse_y.saturating_sub(stride);
    let y_max = (best_coarse_y + stride).min(height.saturating_sub(1));
    let x_min = best_coarse_x.saturating_sub(stride);
    let x_max = (best_coarse_x + stride).min(width.saturating_sub(1));

    let mut final_px = best_coarse_x;
    let mut final_py = best_coarse_y;

    for fy in y_min..=y_max {
        let row_offset = fy * width;
        for fx in x_min..=x_max {
            let idx = row_offset + fx;
            if idx < total {
                let clon_rad = lons[idx].to_radians();
                let clat_rad = lats[idx].to_radians();
                let cx = clat_rad.cos() * clon_rad.sin();
                let cy = clat_rad.sin();
                let cz = clat_rad.cos() * clon_rad.cos();
                let dist_sq = (cx - tx).powi(2) + (cy - ty).powi(2) + (cz - tz).powi(2);
                if dist_sq < best_dist_sq {
                    best_dist_sq = dist_sq;
                    final_px = fx;
                    final_py = fy;
                }
            }
        }
    }

    (final_px, final_py)
}
