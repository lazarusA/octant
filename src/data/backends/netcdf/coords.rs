//! Coordinate extraction for NetCDF sliced hyperslabs.

use std::collections::HashMap;

use netcdf::{Extent, Extents};

use super::coord_read::{group_ancestors, group_of, read_numbers_f64, text_dimension};
use super::slice::with_netcdf_variable;
use crate::data::blocks::BlockStoreError;
use crate::utils::grid_flips::{is_lat_name, is_lon_name};

/// Window coordinates of a block, keyed by dimension name (and its lowercase form), plus
/// the first and last coordinate of the whole latitude and longitude dimensions.
#[derive(Default)]
pub struct SlicedCoordinates {
    pub coordinates: HashMap<String, Vec<f64>>,
    pub lat_extent: Option<(f64, f64)>,
    pub lon_extent: Option<(f64, f64)>,
}

/// A numeric coordinate's block window and the whole coordinate's first and last value.
struct WindowRead {
    values: Vec<f64>,
    extent: Option<(f64, f64)>,
}

/// What a group holds under a coordinate's name.
enum Found {
    Numbers(WindowRead),
    /// A text coordinate: it labels the dimension, so ancestors are not searched.
    Text,
    /// A variable that is not a readable 1D coordinate.
    Unreadable,
}

/// Extracts the coordinates of the block window `origin..origin + block_shape` of variable
/// `var_path`, looking for each dimension's coordinate variable from the variable's group
/// up to the root. Text coordinates give no block coordinates.
pub fn extract_sliced_coordinates(
    file: &netcdf::File,
    var_path: &str,
    dim_names: &[String],
    origin: &[usize],
    block_shape: &[usize],
) -> SlicedCoordinates {
    let mut out = SlicedCoordinates::default();
    for (i, name) in dim_names.iter().enumerate() {
        let window = (
            origin.get(i).copied().unwrap_or(0),
            block_shape.get(i).copied().unwrap_or(1),
        );
        let Some(read) = find_coordinate(file, var_path, name, window) else {
            continue;
        };
        if is_lat_name(name) && out.lat_extent.is_none() {
            out.lat_extent = read.extent;
        } else if is_lon_name(name) && out.lon_extent.is_none() {
            out.lon_extent = read.extent;
        }
        let clean = name.trim().to_lowercase();
        if clean != *name {
            out.coordinates.insert(clean, read.values.clone());
        }
        out.coordinates.insert(name.clone(), read.values);
    }
    out
}

/// The window of the numeric coordinate named `name` (or its lowercase form) nearest to
/// `var_path`'s group, or `None` when none is found or the nearest one is text.
fn find_coordinate(
    file: &netcdf::File,
    var_path: &str,
    name: &str,
    window: (usize, usize),
) -> Option<WindowRead> {
    let clean = name.trim().to_lowercase();
    let candidates: &[&str] = if clean == name {
        &[name]
    } else {
        &[name, &clean]
    };
    for group in group_ancestors(group_of(var_path)) {
        for candidate in candidates {
            let path = if group.is_empty() {
                (*candidate).to_string()
            } else {
                format!("{group}/{candidate}")
            };
            match with_netcdf_variable(file, &path, |var| Ok(classify(var, window))) {
                Ok(Found::Numbers(read)) => return Some(read),
                Ok(Found::Text) => return None,
                Ok(Found::Unreadable) | Err(_) => {}
            }
        }
    }
    None
}

fn classify(var: &netcdf::Variable<'_>, window: (usize, usize)) -> Found {
    if text_dimension(var).is_some() {
        return Found::Text;
    }
    read_window(var, window).map_or(Found::Unreadable, Found::Numbers)
}

/// The values `start..start + count` of a 1D numeric coordinate variable, and its first
/// and last value.
fn read_window(
    var: &netcdf::Variable<'_>,
    (start, count): (usize, usize),
) -> Result<WindowRead, BlockStoreError> {
    let [dim] = var.dimensions() else {
        return Err("coordinate variable is not 1D".into());
    };
    let dim_len = dim.len();
    let start = start.min(dim_len.saturating_sub(1));
    let end = start.saturating_add(count).min(dim_len);
    let values = read_slice(var, start, end.saturating_sub(start).max(1))?;
    let (Some(&first), Some(&last)) = (values.first(), values.last()) else {
        return Err("empty coordinate window".into());
    };
    let extent = if start == 0 && end == dim_len {
        Some((first, last))
    } else {
        let at = |i: usize| read_slice(var, i, 1).ok()?.first().copied();
        at(0).zip(at(dim_len - 1))
    };
    Ok(WindowRead { values, extent })
}

fn read_slice(
    var: &netcdf::Variable<'_>,
    start: usize,
    count: usize,
) -> Result<Vec<f64>, BlockStoreError> {
    let extents = Extents::from(vec![Extent::SliceCount {
        start,
        count,
        stride: 1,
    }]);
    read_numbers_f64(var, &extents)
}
