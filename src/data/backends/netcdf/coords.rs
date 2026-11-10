//! Coordinate extraction for NetCDF sliced hyperslabs.

use super::coord_read::{group_ancestors, group_of, read_numbers_f64, text_dimension};
use super::reader::{NcReader, NcVar};
use crate::data::block_orientation::BlockCoords;
use crate::data::blocks::BlockStoreError;
use crate::data::slice_request::DimensionSelection;

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
    reader: &dyn NcReader,
    var_path: &str,
    dim_names: &[String],
    origin: &[usize],
    block_shape: &[usize],
) -> BlockCoords {
    let mut out = BlockCoords::default();
    for (i, name) in dim_names.iter().enumerate() {
        let window = (
            origin.get(i).copied().unwrap_or(0),
            block_shape.get(i).copied().unwrap_or(1),
        );
        let Some(read) = find_coordinate(reader, var_path, name, window) else {
            continue;
        };
        out.note_extent(name, read.extent);
        out.insert(name, read.values);
    }
    out
}

/// The window of the numeric coordinate named `name` (or its lowercase form) nearest to
/// `var_path`'s group, or `None` when none is found or the nearest one is text.
fn find_coordinate(
    reader: &dyn NcReader,
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
            match reader
                .variable(&path)
                .map(|var| classify(reader, &var, window))
            {
                Some(Found::Numbers(read)) => return Some(read),
                Some(Found::Text) => return None,
                Some(Found::Unreadable) | None => {}
            }
        }
    }
    None
}

fn classify(reader: &dyn NcReader, var: &NcVar, window: (usize, usize)) -> Found {
    if text_dimension(var).is_some() {
        return Found::Text;
    }
    read_window(reader, var, window).map_or(Found::Unreadable, Found::Numbers)
}

/// The values `start..start + count` of a 1D numeric coordinate variable, and its first
/// and last value.
fn read_window(
    reader: &dyn NcReader,
    var: &NcVar,
    (start, count): (usize, usize),
) -> Result<WindowRead, BlockStoreError> {
    let [(_, dim_len)] = var.dims.as_slice() else {
        return Err("coordinate variable is not 1D".into());
    };
    let dim_len = *dim_len;
    let window = DimensionSelection::range(start, start.saturating_add(count));
    let (start, end) = window.clamp(dim_len);
    let read = |start: usize, count: usize| read_numbers_f64(reader, var, &[start], &[count]);
    let values = read(start, end - start)?;
    let (Some(&first), Some(&last)) = (values.first(), values.last()) else {
        return Err("empty coordinate window".into());
    };
    let extent = if start == 0 && end == dim_len {
        Some((first, last))
    } else {
        let at = |i: usize| read(i, 1).ok()?.first().copied();
        at(0).zip(at(dim_len - 1))
    };
    Ok(WindowRead { values, extent })
}
