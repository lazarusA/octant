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

/// Whether the spatial dimensions are stored lon-first (`(lon, lat)` or `(x, y)`), which
/// orientation transposes into `(lat, lon)`.
pub fn needs_transpose(dim_names: &[String]) -> bool {
    let mut spatial = dim_names
        .iter()
        .map(|d| d.to_lowercase())
        .filter(|d| d.contains("lat") || d.contains("lon") || d == "y" || d == "x");
    match (spatial.next(), spatial.next()) {
        (Some(first), Some(second)) => {
            (first.contains("lon") || first == "x") && (second.contains("lat") || second == "y")
        }
        _ => false,
    }
}

/// The dimensions a block's data is reversed along by its orientation: the row axis (second
/// to last) when `flip_y`, the column axis (last) when `flip_x`, each only when longer than
/// one index. Pass the names and shape after any transpose.
pub fn flipped_dims(
    dim_names: &[String],
    block_shape: &[usize],
    (flip_y, flip_x): (bool, bool),
) -> Vec<String> {
    let rank = dim_names.len().min(block_shape.len());
    if rank < 2 {
        return Vec::new();
    }
    [(flip_y, rank - 2), (flip_x, rank - 1)]
        .into_iter()
        .filter(|&(flipped, dim)| flipped && block_shape[dim] > 1)
        .map(|(_, dim)| dim_names[dim].clone())
        .collect()
}

/// Reverses the per-index coordinate vectors (one value per row or column) of the
/// `flipped` dimensions, so that value `i` stays the coordinate of data row or column `i`.
/// Two-value `[first, last]` vectors of longer axes describe only the extent and are kept.
pub fn reverse_flipped_coordinates(
    coordinates: &mut HashMap<String, Vec<f64>>,
    dim_names: &[String],
    block_shape: &[usize],
    flipped: &[String],
) {
    for name in flipped {
        let Some(dim) = dim_names.iter().position(|d| d == name) else {
            continue;
        };
        let len = block_shape.get(dim).copied().unwrap_or(0);
        if len <= 2 {
            continue;
        }
        let clean = name.trim().to_lowercase();
        let keys: &[&str] = if clean == *name {
            &[name]
        } else {
            &[name, &clean]
        };
        for key in keys {
            if let Some(values) = coordinates.get_mut(*key)
                && values.len() == len
            {
                values.reverse();
            }
        }
    }
}
