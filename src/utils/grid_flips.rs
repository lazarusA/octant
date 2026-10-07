//! Which way a lat/lon grid flips so north renders at the top and west on the left, and
//! keeping per-index coordinates in the flipped order.

use std::collections::HashMap;

use crate::data::coordinates::naming::contains_ascii_case_insensitive;

/// `(flip_y, flip_x)`: rows flip when latitude ascends (row 0 south), columns when
/// longitude descends (column 0 east), judged from the coordinates' first and last value,
/// else from orientation attributes and dimension names.
pub fn axis_flips(
    dim_names: &[String],
    attributes: &serde_json::Map<String, serde_json::Value>,
    lat_coords: Option<&[f64]>,
    lon_coords: Option<&[f64]>,
) -> (bool, bool) {
    (
        lat_flip(dim_names, attributes, lat_coords),
        lon_flip(attributes, lon_coords),
    )
}

/// Rows flip when latitude ascends, so north renders at the top: from the coordinates, else
/// `latitude_orientation`, else `positive = "up"`, else whenever a latitude dimension exists.
fn lat_flip(
    dim_names: &[String],
    attributes: &serde_json::Map<String, serde_json::Value>,
    lat_coords: Option<&[f64]>,
) -> bool {
    if let Some(coords) = lat_coords {
        return matches!((coords.first(), coords.last()), (Some(f), Some(l)) if coords.len() >= 2 && f < l);
    }
    let attr = |key: &str| attributes.get(key).and_then(|v| v.as_str());
    if let Some(orientation) = attr("latitude_orientation") {
        return orientation.eq_ignore_ascii_case("ascending");
    }
    if attr("positive").is_some_and(|p| p.eq_ignore_ascii_case("up")) {
        return true;
    }
    dim_names
        .iter()
        .any(|d| contains_ascii_case_insensitive(d, "lat") || d.eq_ignore_ascii_case("y"))
}

/// Columns flip when longitude descends, so west renders on the left: from the
/// coordinates, else `longitude_orientation = "descending"`.
fn lon_flip(
    attributes: &serde_json::Map<String, serde_json::Value>,
    lon_coords: Option<&[f64]>,
) -> bool {
    match lon_coords {
        Some(coords) => {
            matches!((coords.first(), coords.last()), (Some(f), Some(l)) if coords.len() >= 2 && f > l)
        }
        None => attributes
            .get("longitude_orientation")
            .and_then(|v| v.as_str())
            .is_some_and(|o| o.eq_ignore_ascii_case("descending")),
    }
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

/// Reverses the coordinate vectors of the `flipped` dimensions so they follow the data:
/// per-index vectors (one value per row or column) keep value `i` on data row or column
/// `i`, and two-value `[first, last]` extents swap ends, so interpolating them at oriented
/// indices gives the right coordinates.
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
        if len <= 1 {
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
                && (values.len() == len || values.len() == 2)
            {
                values.reverse();
            }
        }
    }
}
