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
    let spatial_dims: Vec<String> = dim_names
        .iter()
        .map(|d| d.to_lowercase())
        .filter(|d| d.contains("lat") || d.contains("lon") || d == "y" || d == "x")
        .collect();

    let mut needs_transpose = false;
    if spatial_dims.len() >= 2 {
        let first = &spatial_dims[0];
        let second = &spatial_dims[1];
        if (first.contains("lon") || first == "x") && (second.contains("lat") || second == "y") {
            needs_transpose = true;
        }
    }

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

    let mut flip_y = false;

    if let Some(coords) = lat_coords {
        if coords.len() >= 2 {
            let first = coords.first().copied().unwrap_or(0.0);
            let last = coords.last().copied().unwrap_or(0.0);
            if first < last {
                // Axis data ascends from South to North (Row 0 is South).
                // Flip Y so North (+90) renders at top of screen (Row 0).
                flip_y = true;
            } else if first > last {
                // Axis data descends from North to South (Row 0 is North).
                // Row 0 is already at top of screen (North), do not flip Y.
                flip_y = false;
            }
        }
    } else if let Some(orientation) = attributes
        .get("latitude_orientation")
        .and_then(|v| v.as_str())
    {
        if orientation.to_lowercase() == "ascending" {
            flip_y = true;
        } else if orientation.to_lowercase() == "descending" {
            flip_y = false;
        }
    } else if let Some(positive_attr) = attributes.get("positive").and_then(|v| v.as_str())
        && positive_attr.to_lowercase() == "up"
    {
        flip_y = true;
    } else {
        let is_lat_dim = dim_names
            .iter()
            .any(|d| d.to_lowercase().contains("lat") || d.to_lowercase() == "y");
        if is_lat_dim {
            flip_y = true;
        }
    }

    if flip_y && height > 1 {
        for r in 0..(height / 2) {
            let top_row_start = r * width;
            let bot_row_start = (height - 1 - r) * width;
            for c in 0..width {
                current_values.swap(top_row_start + c, bot_row_start + c);
            }
        }
    }

    // 3. Determine X (Longitude) orientation directly from axis coordinate values or explicit metadata
    let mut flip_x = false;

    if let Some(coords) = lon_coords {
        if coords.len() >= 2 {
            let first = coords.first().copied().unwrap_or(0.0);
            let last = coords.last().copied().unwrap_or(0.0);
            if first > last {
                // Axis data descends (East to West). Flip X so West renders on left.
                flip_x = true;
            }
        }
    } else if let Some(orientation) = attributes
        .get("longitude_orientation")
        .and_then(|v| v.as_str())
        && orientation.to_lowercase() == "descending"
    {
        flip_x = true;
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

/// Orients an N-dimensional block's 2D spatial grid slices and axes using `check_and_orient_axes_with_coords`.
pub fn check_and_orient_block_grid(
    mut values: Vec<f32>,
    block_shape: &mut [usize],
    dimension_names: &mut [String],
    origin: &mut [usize],
    attributes: &serde_json::Map<String, serde_json::Value>,
    coordinates: &std::collections::HashMap<String, Vec<f64>>,
) -> Vec<f32> {
    let rank = block_shape.len();
    if rank < 2 {
        return values;
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
            dimension_names
                .get(rank - 2)
                .and_then(|d| coordinates.get(d))
        })
        .or_else(|| {
            coordinates
                .iter()
                .find(|(k, _)| k.contains("lat") || *k == "y")
                .map(|(_, v)| v)
        })
        .map(|v| v.as_slice());

    let lon_coords = lon_dim
        .and_then(|d| coordinates.get(d))
        .or_else(|| {
            dimension_names
                .get(rank - 1)
                .and_then(|d| coordinates.get(d))
        })
        .or_else(|| {
            coordinates
                .iter()
                .find(|(k, _)| k.contains("lon") || *k == "x")
                .map(|(_, v)| v)
        })
        .map(|v| v.as_slice());

    let in_height = block_shape[rank - 2];
    let in_width = block_shape[rank - 1];
    let slice_size = in_width * in_height;

    if slice_size > 0 && values.len().is_multiple_of(slice_size) {
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

        if final_width != in_width || final_height != in_height {
            block_shape[rank - 2] = final_height;
            block_shape[rank - 1] = final_width;
            origin.swap(rank - 2, rank - 1);
            dimension_names.swap(rank - 2, rank - 1);
        }

        values = final_values;
    }

    values
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
