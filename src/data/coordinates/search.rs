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
