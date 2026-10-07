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

    // 1. Determine if dimensions are ordered (X, Y) / (lon, lat) instead of (Y, X) / (lat, lon)
    let needs_transpose = needs_transpose(dim_names);

    let (mut current_values, width, height) = if needs_transpose {
        let mut transposed = vec![0.0f32; in_width * in_height];
        for r in 0..in_height {
            for c in 0..in_width {
                transposed[c * in_height + r] = raw_values[r * in_width + c];
            }
        }
        (transposed, in_height, in_width)
    } else {
        (raw_values, in_width, in_height)
    };

    let (flip_y, flip_x) = axis_flips(dim_names, attributes, lat_coords, lon_coords);

    if flip_y && height > 1 {
        for r in 0..(height / 2) {
            let top_row_start = r * width;
            let bot_row_start = (height - 1 - r) * width;
            for c in 0..width {
                current_values.swap(top_row_start + c, bot_row_start + c);
            }
        }
    }

    if flip_x && width > 1 {
        for r in 0..height {
            let row_start = r * width;
            for c in 0..(width / 2) {
                current_values.swap(row_start + c, row_start + width - 1 - c);
            }
        }
    }

    (current_values, width, height)
}

/// Orients an N-dimensional block's 2D spatial grid slices and axes using
/// `check_and_orient_axes_with_coords`. Returns the oriented values and the dimensions they
/// were reversed along (see [`flipped_dims`]); per-row coordinates are reversed with them.
pub fn check_and_orient_block_grid(
    mut values: Vec<f32>,
    block_shape: &mut [usize],
    dimension_names: &mut [String],
    origin: &mut [usize],
    attributes: &serde_json::Map<String, serde_json::Value>,
    coordinates: &mut std::collections::HashMap<String, Vec<f64>>,
) -> (Vec<f32>, Vec<String>) {
    let rank = block_shape.len();
    if rank < 2 {
        return (values, Vec::new());
    }

    let lat_dim = dimension_names
        .iter()
        .find(|d| d.to_lowercase().contains("lat") || d.to_lowercase() == "y");
    let lon_dim = dimension_names
        .iter()
        .find(|d| d.to_lowercase().contains("lon") || d.to_lowercase() == "x");

    let lat_coords = lat_dim
        .and_then(|d| coordinates.get(d))
        .or_else(|| {
            coordinates
                .iter()
                .find(|(k, _)| {
                    let clean = k.to_lowercase();
                    clean.contains("lat") || clean == "y"
                })
                .map(|(_, v)| v)
        })
        .map(|v| v.as_slice());

    let lon_coords = lon_dim
        .and_then(|d| coordinates.get(d))
        .or_else(|| {
            coordinates
                .iter()
                .find(|(k, _)| {
                    let clean = k.to_lowercase();
                    clean.contains("lon") || clean == "x"
                })
                .map(|(_, v)| v)
        })
        .map(|v| v.as_slice());

    let flips = axis_flips(dimension_names, attributes, lat_coords, lon_coords);
    let transposed = needs_transpose(dimension_names);
    let in_height = block_shape[rank - 2];
    let in_width = block_shape[rank - 1];
    let slice_size = in_width * in_height;
    let oriented = slice_size > 0 && values.len().is_multiple_of(slice_size);

    if oriented {
        let num_slices = values.len() / slice_size;
        let mut final_values = Vec::with_capacity(values.len());
        let mut final_width = in_width;
        let mut final_height = in_height;

        for i in 0..num_slices {
            let slice_raw = values[i * slice_size..(i + 1) * slice_size].to_vec();
            let (slice_oriented, w, h) = check_and_orient_axes_with_coords(
                slice_raw,
                in_width,
                in_height,
                dimension_names,
                attributes,
                lat_coords,
                lon_coords,
            );
            final_width = w;
            final_height = h;
            final_values.extend(slice_oriented);
        }

        // Decided by the transpose itself: a square grid keeps its shape when transposed.
        if transposed {
            block_shape[rank - 2] = final_height;
            block_shape[rank - 1] = final_width;
            origin.swap(rank - 2, rank - 1);
            dimension_names.swap(rank - 2, rank - 1);
        }

        values = final_values;
        // After the slices: they judge their flips from these coordinates.
        let flipped = flipped_dims(dimension_names, block_shape, flips);
        reverse_flipped_coordinates(coordinates, dimension_names, block_shape, &flipped);
        return (values, flipped);
    }

    (values, Vec::new())
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
