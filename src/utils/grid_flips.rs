//! Which way a lat/lon grid flips so north renders at the top and west on the left, and
//! keeping per-index coordinates in the flipped order.

use std::collections::HashMap;

/// `(flip_y, flip_x)`: rows flip when latitude ascends (row 0 south), columns when
/// longitude descends (column 0 east), judged from the coordinates' first and last value,
/// else from orientation attributes and dimension names.
pub fn axis_flips(
    dim_names: &[String],
    attributes: &serde_json::Map<String, serde_json::Value>,
    lat_coords: Option<&[f64]>,
    lon_coords: Option<&[f64]>,
) -> (bool, bool) {
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

    (flip_y, flip_x)
}

/// Reverses the per-index coordinate vectors (one value per row or column) of the axes the
/// data is flipped along, so that value `i` stays the coordinate of data row or column `i`.
/// Two-value `[first, last]` vectors of longer axes describe only the extent and are kept.
pub fn reverse_flipped_coordinates(
    coordinates: &mut HashMap<String, Vec<f64>>,
    dim_names: &[String],
    block_shape: &[usize],
    (flip_y, flip_x): (bool, bool),
) {
    let axes = [(flip_y, ["lat", "y"]), (flip_x, ["lon", "x"])];
    for (flipped, [part, exact]) in axes {
        let Some(dim) = dim_names.iter().position(|d| {
            let d = d.to_lowercase();
            d.contains(part) || d == exact
        }) else {
            continue;
        };
        let len = block_shape.get(dim).copied().unwrap_or(0);
        if !flipped || len <= 2 {
            continue;
        }
        let name = &dim_names[dim];
        let keys = [name.clone(), name.trim().to_lowercase()];
        for (i, key) in keys.iter().enumerate() {
            if i == 1 && key == name {
                break;
            }
            if let Some(values) = coordinates.get_mut(key)
                && values.len() == len
            {
                values.reverse();
            }
        }
    }
}
