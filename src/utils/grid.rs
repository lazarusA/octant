use std::collections::HashMap;

use super::grid_flips::{axis_flips, flipped_dims, needs_transpose, reverse_flipped_coordinates};

/// Function for checking axes order and orientation.
///
/// Accounts for:
/// 1. Axis ordering: If X/lon dimension precedes Y/lat (e.g. `(lon, lat)`), transposes the grid to `(height, width)` / `(lat, lon)`.
/// 2. Coordinate direction:
///    - If Y/latitude ascends from South (-90) to North (+90), flips rows vertically (y-flip) so North renders at top of map.
///    - If X/longitude descends from East to West, flips columns horizontally (x-flip) so East renders on the right.
pub fn check_and_orient_axes_with_coords(
    raw_values: Vec<f32>,
    in_width: usize,
    in_height: usize,
    dim_names: &[String],
    attributes: &serde_json::Map<String, serde_json::Value>,
    lat_coords: Option<&[f64]>,
    lon_coords: Option<&[f64]>,
) -> (Vec<f32>, usize, usize) {
    if raw_values.len() != in_width * in_height {
        return (raw_values, in_width, in_height);
    }
    let transpose = needs_transpose(dim_names);
    let flips = axis_flips(dim_names, attributes, lat_coords, lon_coords);
    let mut out = Vec::with_capacity(raw_values.len());
    let (width, height) = orient_slice_into(
        &raw_values,
        (in_width, in_height),
        transpose,
        flips,
        &mut out,
    );
    (out, width, height)
}

/// Appends the oriented copy of one `in_width x in_height` slice to `out`: transposed when
/// stored lon-first, then rows and columns flipped by `(flip_y, flip_x)`. Returns the
/// slice's oriented `(width, height)`.
fn orient_slice_into(
    raw: &[f32],
    (in_width, in_height): (usize, usize),
    transpose: bool,
    (flip_y, flip_x): (bool, bool),
    out: &mut Vec<f32>,
) -> (usize, usize) {
    let start = out.len();
    let (width, height) = if transpose {
        // Row `c` of the transposed slice is column `c` of the stored one.
        out.extend((0..in_width).flat_map(|c| (0..in_height).map(move |r| raw[r * in_width + c])));
        (in_height, in_width)
    } else {
        out.extend_from_slice(raw);
        (in_width, in_height)
    };
    let slice = &mut out[start..];
    if flip_y && height > 1 {
        for r in 0..height / 2 {
            let (top, bottom) = slice.split_at_mut((height - 1 - r) * width);
            top[r * width..(r + 1) * width].swap_with_slice(&mut bottom[..width]);
        }
    }
    if flip_x && width > 1 {
        slice.chunks_mut(width).for_each(<[f32]>::reverse);
    }
    (width, height)
}

/// Orients an N-dimensional block's 2D spatial grid slices and axes like
/// [`check_and_orient_axes_with_coords`], deciding the transpose and flips once for every
/// slice. Returns the oriented values and the dimensions they were reversed along (see
/// [`flipped_dims`]); coordinates of those dimensions are reversed with them.
pub fn check_and_orient_block_grid(
    values: Vec<f32>,
    block_shape: &mut [usize],
    dimension_names: &mut [String],
    origin: &mut [usize],
    attributes: &serde_json::Map<String, serde_json::Value>,
    coordinates: &mut HashMap<String, Vec<f64>>,
) -> (Vec<f32>, Vec<String>) {
    let rank = block_shape.len();
    let slice = (
        block_shape.get(rank.wrapping_sub(1)).copied().unwrap_or(0),
        block_shape.get(rank.wrapping_sub(2)).copied().unwrap_or(0),
    );
    let slice_size = slice.0 * slice.1;
    if rank < 2 || slice_size == 0 || !values.len().is_multiple_of(slice_size) {
        return (values, Vec::new());
    }
    let lat = axis_coords(coordinates, dimension_names, "lat", "y");
    let lon = axis_coords(coordinates, dimension_names, "lon", "x");
    let flips = axis_flips(dimension_names, attributes, lat, lon);
    let transpose = needs_transpose(dimension_names);

    let mut oriented = Vec::with_capacity(values.len());
    let mut size = slice;
    for raw in values.chunks(slice_size) {
        size = orient_slice_into(raw, slice, transpose, flips, &mut oriented);
    }
    // Decided by the transpose itself: a square grid keeps its shape when transposed.
    if transpose {
        (block_shape[rank - 1], block_shape[rank - 2]) = size;
        origin.swap(rank - 2, rank - 1);
        dimension_names.swap(rank - 2, rank - 1);
    }
    let flipped = flipped_dims(dimension_names, block_shape, flips);
    reverse_flipped_coordinates(coordinates, dimension_names, block_shape, &flipped);
    (oriented, flipped)
}

/// The coordinates of the axis named like `part` (or exactly `exact`): those of its
/// dimension, else of any coordinate key named like it.
fn axis_coords<'a>(
    coordinates: &'a HashMap<String, Vec<f64>>,
    dimension_names: &[String],
    part: &str,
    exact: &str,
) -> Option<&'a [f64]> {
    let named = |name: &str| {
        let name = name.to_lowercase();
        name.contains(part) || name == exact
    };
    dimension_names
        .iter()
        .find(|d| named(d))
        .and_then(|d| coordinates.get(d))
        .or_else(|| coordinates.iter().find(|(k, _)| named(k)).map(|(_, v)| v))
        .map(Vec::as_slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_orientation_ascending_axis_data() {
        // Ascending lat axis data: [-89.875, ..., 89.875] (Row 0 = South [10.0, 20.0], Row 1 = North [30.0, 40.0])
        let raw = vec![10.0, 20.0, 30.0, 40.0];
        let dim_names = vec!["lat".to_string(), "lon".to_string()];
        let attrs = serde_json::Map::new();
        let lat_axis = vec![-89.875, 89.875];

        let (oriented, w, h) =
            check_and_orient_axes_with_coords(raw, 2, 2, &dim_names, &attrs, Some(&lat_axis), None);
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        // Ascending lat axis SHOULD flip Y so North [30.0, 40.0] moves to Row 0 (top of map)
        assert_eq!(oriented, vec![30.0, 40.0, 10.0, 20.0]);
    }

    #[test]
    fn test_grid_orientation_descending_axis_data() {
        // Descending lat axis data: [89.875, ..., -89.875] (Row 0 = North [10.0, 20.0], Row 1 = South [30.0, 40.0])
        let raw = vec![10.0, 20.0, 30.0, 40.0];
        let dim_names = vec!["latitude".to_string(), "longitude".to_string()];
        let attrs = serde_json::Map::new();
        let lat_axis = vec![89.875, -89.875];

        let (oriented, w, h) = check_and_orient_axes_with_coords(
            raw.clone(),
            2,
            2,
            &dim_names,
            &attrs,
            Some(&lat_axis),
            None,
        );
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        // Descending lat axis should NOT flip Y, row 0 stays North [10.0, 20.0]
        assert_eq!(oriented, raw);
    }
}
