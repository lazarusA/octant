//! Which way a lat/lon grid flips so north renders at the top and west on the left, and
//! keeping per-index coordinates in the flipped order.

use std::collections::HashMap;

use crate::data::coordinates::naming::contains_ascii_case_insensitive;

/// What orientation reads besides the block itself: the variable's attributes and the
/// first and last coordinate of the whole latitude and longitude dimensions. Deciding the
/// flips from the whole dimension, not from a block's window, flips every block of a
/// variable alike, including edge blocks one row or column long.
#[derive(Clone, Copy)]
pub struct OrientHints<'a> {
    pub attributes: &'a serde_json::Map<String, serde_json::Value>,
    pub lat_extent: Option<(f64, f64)>,
    pub lon_extent: Option<(f64, f64)>,
}

impl<'a> OrientHints<'a> {
    /// Hints without dimension extents: flips are judged from the block's coordinates.
    pub fn new(attributes: &'a serde_json::Map<String, serde_json::Value>) -> Self {
        Self {
            attributes,
            lat_extent: None,
            lon_extent: None,
        }
    }
}

/// Whether a dimension or coordinate name denotes latitude (`*lat*` or `y`).
pub fn is_lat_name(name: &str) -> bool {
    contains_ascii_case_insensitive(name, "lat") || name.eq_ignore_ascii_case("y")
}

/// Whether a dimension or coordinate name denotes longitude (`*lon*` or `x`).
pub fn is_lon_name(name: &str) -> bool {
    contains_ascii_case_insensitive(name, "lon") || name.eq_ignore_ascii_case("x")
}

/// Whether orientation applies to the last two dimensions: no latitude or longitude
/// dimension comes before them (`(y, x, band)` is left as stored).
pub fn spatial_dims_last(dim_names: &[String]) -> bool {
    let head = dim_names.len().saturating_sub(2);
    !dim_names[..head]
        .iter()
        .any(|d| is_lat_name(d) || is_lon_name(d))
}

/// `(flip_y, flip_x)`: rows flip when latitude ascends (row 0 south), columns when
/// longitude descends (column 0 east), judged from the whole dimension's extent when
/// known, else the block's coordinates, else orientation attributes and dimension names.
pub fn axis_flips(
    dim_names: &[String],
    hints: OrientHints<'_>,
    lat_coords: Option<&[f64]>,
    lon_coords: Option<&[f64]>,
) -> (bool, bool) {
    let lat = hints.lat_extent.or_else(|| lat_coords.map(ends));
    let lon = hints.lon_extent.or_else(|| lon_coords.map(ends));
    (
        lat_flip(dim_names, hints.attributes, lat),
        lon_flip(hints.attributes, lon),
    )
}

/// First and last of `coords`; equal ends (never flipping) for fewer than two values.
fn ends(coords: &[f64]) -> (f64, f64) {
    match (coords.first(), coords.last()) {
        (Some(&f), Some(&l)) if coords.len() >= 2 => (f, l),
        _ => (0.0, 0.0),
    }
}

/// Rows flip when latitude ascends, so north renders at the top: from the coordinates, else
/// `latitude_orientation`, else `positive = "up"`, else whenever a latitude dimension exists.
fn lat_flip(
    dim_names: &[String],
    attributes: &serde_json::Map<String, serde_json::Value>,
    lat: Option<(f64, f64)>,
) -> bool {
    if let Some((first, last)) = lat {
        return first < last;
    }
    let attr = |key: &str| attributes.get(key).and_then(|v| v.as_str());
    if let Some(orientation) = attr("latitude_orientation") {
        return orientation.eq_ignore_ascii_case("ascending");
    }
    if attr("positive").is_some_and(|p| p.eq_ignore_ascii_case("up")) {
        return true;
    }
    dim_names.iter().any(|d| is_lat_name(d))
}

/// Columns flip when longitude descends, so west renders on the left: from the
/// coordinates, else `longitude_orientation = "descending"`.
fn lon_flip(
    attributes: &serde_json::Map<String, serde_json::Value>,
    lon: Option<(f64, f64)>,
) -> bool {
    match lon {
        Some((first, last)) => first > last,
        None => attributes
            .get("longitude_orientation")
            .and_then(|v| v.as_str())
            .is_some_and(|o| o.eq_ignore_ascii_case("descending")),
    }
}

/// Whether the last two dimensions are stored lon-first (`(lon, lat)` or `(x, y)`), which
/// orientation transposes into `(lat, lon)`.
pub fn needs_transpose(dim_names: &[String]) -> bool {
    match dim_names {
        [.., first, second] => is_lon_name(first) && is_lat_name(second),
        _ => false,
    }
}

/// The dimensions a block's data is reversed along by its orientation: the row axis (second
/// to last) when `flip_y`, the column axis (last) when `flip_x`. A dimension one index long
/// is listed too, so an edge block one row long is placed like the rest of its variable.
/// Pass the names after any transpose.
pub fn flipped_dims(dim_names: &[String], (flip_y, flip_x): (bool, bool)) -> Vec<String> {
    let rank = dim_names.len();
    if rank < 2 {
        return Vec::new();
    }
    [(flip_y, rank - 2), (flip_x, rank - 1)]
        .into_iter()
        .filter(|&(flipped, _)| flipped)
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
