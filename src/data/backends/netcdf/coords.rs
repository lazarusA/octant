//! Coordinate extraction for NetCDF sliced hyperslabs.

use std::collections::HashMap;

use netcdf::{Extent, Extents};

use super::coord_read::{group_ancestors, group_of, read_numbers_f64};
use super::slice::with_netcdf_variable;

/// Extracts the coordinates of the block window `origin..origin + block_shape` of variable
/// `var_path`, looking for each dimension's coordinate variable from the variable's group
/// up to the root. Text coordinates give no block coordinates.
pub fn extract_sliced_coordinates(
    file: &netcdf::File,
    var_path: &str,
    dim_names: &[String],
    origin: &[usize],
    block_shape: &[usize],
) -> HashMap<String, Vec<f64>> {
    let mut coordinates: HashMap<String, Vec<f64>> = HashMap::new();
    for (i, name) in dim_names.iter().enumerate() {
        let clean = name.trim().to_lowercase();
        let window = (origin[i], block_shape[i]);
        let found = group_ancestors(group_of(var_path)).find_map(|group| {
            [name.as_str(), clean.as_str()]
                .into_iter()
                .find_map(|candidate| {
                    let path = if group.is_empty() {
                        candidate.to_string()
                    } else {
                        format!("{group}/{candidate}")
                    };
                    with_netcdf_variable(file, &path, |var| read_window(var, window)).ok()
                })
        });
        if let Some(values) = found {
            if clean != *name {
                coordinates.insert(clean, values.clone());
            }
            coordinates.insert(name.clone(), values);
        }
    }
    coordinates
}

/// The values `start..start + count` of a 1D numeric coordinate variable.
fn read_window(
    var: &netcdf::Variable<'_>,
    (start, count): (usize, usize),
) -> Result<Vec<f64>, crate::data::blocks::BlockStoreError> {
    let [dim] = var.dimensions() else {
        return Err("coordinate variable is not 1D".into());
    };
    let dim_len = dim.len();
    let start = start.min(dim_len.saturating_sub(1));
    let end = (start + count).min(dim_len);
    let extents = Extents::from(vec![Extent::SliceCount {
        start,
        count: end.saturating_sub(start).max(1),
        stride: 1,
    }]);
    let values = read_numbers_f64(var, &extents)?;
    if values.is_empty() {
        return Err("empty coordinate window".into());
    }
    Ok(values)
}
