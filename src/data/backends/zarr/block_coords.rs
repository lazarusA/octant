//! Coordinates of a Zarr block's window, and the extents of the whole latitude and
//! longitude dimensions that decide how the block is oriented.

use std::collections::HashMap;

use zarrs::storage::ReadableWritableListableStorage;

use crate::data::CoordValues;
use crate::data::backends::coord_bounds::get_cached_coord_values_scoped;
use crate::data::coordinates::naming::{is_spatial_x_name, is_spatial_y_name};
use crate::utils::grid_flips::{is_lat_name, is_lon_name};

/// A block's window along every dimension of its array.
pub(super) struct BlockWindow<'a> {
    pub dim_names: &'a [String],
    pub full_shape: &'a [u64],
    pub origin: &'a [usize],
    pub block_shape: &'a [usize],
}

impl BlockWindow<'_> {
    /// The full length of dimension `i`.
    fn full_len(&self, i: usize) -> usize {
        self.full_shape
            .get(i)
            .map(|&n| usize::try_from(n).unwrap_or(usize::MAX))
            .or_else(|| self.block_shape.get(i).copied())
            .unwrap_or(1)
    }

    /// The window coordinates of dimension `i` read from its whole coordinate `coords`.
    fn coords(&self, i: usize, coords: &CoordValues) -> Option<Vec<f64>> {
        window_coords(
            coords,
            self.full_len(i),
            self.origin.get(i).copied().unwrap_or(0),
            self.block_shape.get(i).copied().unwrap_or(1),
        )
    }

    /// The generic (`dim_N`) dimension a fallback x or y coordinate of `len` values
    /// belongs to: the last dimension for x, the second to last for y.
    fn generic_dim(&self, is_x: bool, len: usize) -> Option<usize> {
        let back = if is_x { 1 } else { 2 };
        let i = self.dim_names.len().checked_sub(back)?;
        (self.dim_names[i].starts_with("dim_") && self.full_len(i) == len).then_some(i)
    }
}

/// Window coordinates of a block, keyed by dimension name (and its lowercase form), plus
/// the first and last coordinate of the whole latitude and longitude dimensions.
#[derive(Default)]
pub(super) struct BlockCoords {
    pub coordinates: HashMap<String, Vec<f64>>,
    pub lat_extent: Option<(f64, f64)>,
    pub lon_extent: Option<(f64, f64)>,
}

impl BlockCoords {
    fn insert(&mut self, name: &str, values: Vec<f64>) {
        let clean = name.trim().to_lowercase();
        if clean != name {
            self.coordinates.insert(clean, values.clone());
        }
        self.coordinates.insert(name.to_string(), values);
    }

    /// Records the extent of `coords` when `name` is the first latitude or longitude seen.
    fn note_extent(&mut self, name: &str, coords: &CoordValues) {
        let extent = coords.first_number().zip(coords.last_number());
        if is_lat_name(name) && self.lat_extent.is_none() {
            self.lat_extent = extent;
        } else if is_lon_name(name) && self.lon_extent.is_none() {
            self.lon_extent = extent;
        }
    }
}

/// Reads the window coordinates of every dimension of a block of array `variable`, from its
/// group up, falling back to the store's spatial coordinates for generic (`dim_N`)
/// dimension names.
pub(super) fn block_coordinates(
    store: &ReadableWritableListableStorage,
    store_url: &str,
    variable: &str,
    window: &BlockWindow<'_>,
) -> BlockCoords {
    let group_path = variable.rfind('/').map(|idx| &variable[..idx]);
    let total_dims = window.dim_names.len();
    let coord_values = |name: &str, dim_idx: usize| {
        get_cached_coord_values_scoped(
            store.clone(),
            store_url,
            name,
            group_path,
            &[],
            dim_idx,
            total_dims,
        )
    };
    let mut out = BlockCoords::default();
    for (i, name) in window.dim_names.iter().enumerate() {
        let Some(coords) = coord_values(name, i) else {
            continue;
        };
        out.note_extent(name, &coords);
        if let Some(values) = window.coords(i, &coords) {
            out.insert(name, values);
        }
    }
    if out.coordinates.is_empty() || window.dim_names.iter().any(|d| d.starts_with("dim_")) {
        add_spatial_fallback(&mut out, window, |name| coord_values(name, usize::MAX));
    }
    out
}

/// Adds the store's latitude and longitude coordinates for dimensions not named after
/// them: windowed along the dimension named or positioned like them (also keyed by a
/// generic `dim_N` name, so orientation reverses them with the data), else as the whole
/// coordinate's `[first, last]`.
fn add_spatial_fallback(
    out: &mut BlockCoords,
    window: &BlockWindow<'_>,
    coord_values: impl Fn(&str) -> Option<CoordValues>,
) {
    for candidate in ["lat", "latitude", "y", "lon", "longitude", "x"] {
        let Some(coords) = coord_values(candidate) else {
            continue;
        };
        out.note_extent(candidate, &coords);
        let is_x = is_spatial_x_name(candidate);
        let dim_i = window
            .dim_names
            .iter()
            .position(|d| {
                if is_x {
                    is_spatial_x_name(d)
                } else {
                    is_spatial_y_name(d)
                }
            })
            .or_else(|| window.generic_dim(is_x, coords.len()));
        let values = match dim_i {
            Some(i) => window.coords(i, &coords),
            None => coords
                .first_number()
                .zip(coords.last_number())
                .map(|(first, last)| vec![first, last]),
        };
        let Some(values) = values else {
            continue;
        };
        if let Some(dim) = dim_i.and_then(|i| window.dim_names.get(i))
            && !out.coordinates.contains_key(dim)
        {
            out.coordinates.insert(dim.clone(), values.clone());
        }
        out.coordinates.insert(candidate.to_string(), values);
    }
}

/// Coordinates of the block window `start..start + count` along a dimension of `full_len`:
/// every value of an uneven coordinate (so the plot grid places each row), else the
/// window's exact first and last value.
pub(crate) fn window_coords(
    coords: &CoordValues,
    full_len: usize,
    start: usize,
    count: usize,
) -> Option<Vec<f64>> {
    let end = start.checked_add(count.max(1) - 1)?;
    if matches!(coords, CoordValues::Values(_)) && coords.matches(full_len) {
        return (start..=end).map(|i| coords.number(i)).collect();
    }
    Some(vec![
        coords.number_for(start, full_len)?,
        coords.number_for(end, full_len)?,
    ])
}
