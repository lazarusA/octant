/// Function for checking axes order and orientation.
///
/// Accounts for:
/// 1. Axis ordering: If X/lon dimension precedes Y/lat (e.g. `(lon, lat)`), transposes the grid to `(height, width)` / `(lat, lon)`.
/// 2. Coordinate direction:
///    - If Y/latitude ascends from South (-90) to North (+90), flips rows vertically (y-flip) so North renders at top of map.
///    - If X/longitude descends from East to West, flips columns horizontally (x-flip) so East renders on the right.
pub fn check_and_orient_axes(
    raw_values: Vec<f32>,
    in_width: usize,
    in_height: usize,
    dim_names: &[String],
    attributes: &serde_json::Map<String, serde_json::Value>,
) -> (Vec<f32>, usize, usize) {
    if raw_values.len() != in_width * in_height {
        return (raw_values, in_width, in_height);
    }

    // Determine if dimensions are ordered (X, Y) / (lon, lat) instead of (Y, X) / (lat, lon)
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
                // Input index: r * in_width + c
                // Output transposed index: c * in_height + r
                transposed[c * in_height + r] = raw_values[r * in_width + c];
            }
        }
        (transposed, in_height, in_width)
    } else {
        (raw_values, in_width, in_height)
    };

    // Check latitude orientation (South-to-North ascending vs North-to-South descending)
    // Standard map texture display expects Row 0 = North (top).
    // If attributes or metadata indicate ascending latitude (-90 to +90), flip Y vertically.
    let mut flip_y = false;

    // Check attribute hints if present (e.g., "_ARRAY_DIMENSIONS" or "positive" / coordinate direction)
    if let Some(positive_attr) = attributes.get("positive").and_then(|v| v.as_str()) {
        if positive_attr.to_lowercase() == "up" || positive_attr.to_lowercase() == "north" {
            flip_y = true;
        }
    }

    // Default heuristic for geospatial lat dimension: in most NetCDF/Zarr climate datasets with lat dimension,
    // latitude coordinates are stored ascending [-90..90]. Unless specified as descending, perform Y-flip.
    if dim_names.iter().any(|d| d.to_lowercase().contains("lat") || d.to_lowercase() == "y") {
        let is_descending = attributes
            .get("latitude_orientation")
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase() == "descending")
            .unwrap_or(false);

        if !is_descending {
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

    // Check longitude orientation (East-to-West descending)
    let flip_x = attributes
        .get("longitude_orientation")
        .and_then(|v| v.as_str())
        .map(|s| s.to_lowercase() == "descending")
        .unwrap_or(false);

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
