//! The 1D coordinate arrays a variable or one of its blocks reads, found in the in-memory
//! store the browser fills with metadata at inspect and with chunks on demand.

use zarrs::storage::ReadableWritableListableStorage;

use crate::data::VariableInfo;
use crate::data::backends::coord_bounds::{
    block_coordinate_candidates, variable_coordinate_candidates,
};
use crate::data::slice_request::SliceRequest;
use crate::utils::metadata::{attr_string, open_or_instantiate_array_normalized};

/// `(path, length)` of the 1D arrays named `names`, at the root or in `group`.
fn coordinate_arrays(
    memory: &ReadableWritableListableStorage,
    group: Option<&str>,
    names: Vec<String>,
) -> Vec<(String, u64)> {
    let mut found = Vec::new();
    for name in names {
        let paths = std::iter::once(name.clone()).chain(group.map(|gp| format!("{gp}/{name}")));
        for path in paths {
            if let Ok(array) =
                open_or_instantiate_array_normalized(memory.clone(), &format!("/{path}"))
                && let [len] = array.shape()
            {
                found.push((path, *len));
            }
        }
    }
    found
}

/// The coordinate arrays of `variable`'s dimensions, spatial ones first.
pub fn variable_coordinate_arrays(
    memory: &ReadableWritableListableStorage,
    variable: &VariableInfo,
) -> Vec<(String, u64)> {
    let names = variable_coordinate_candidates(variable);
    coordinate_arrays(memory, variable.group_path(), names)
}

/// The coordinate arrays a block for `request` reads: those of its range-selected
/// dimensions, as the block's own coordinates (`zarr::block_coords`) do.
pub fn block_coordinate_arrays(
    memory: &ReadableWritableListableStorage,
    request: &SliceRequest,
) -> Vec<(String, u64)> {
    let clean_var = request.variable.trim_matches('/');
    let Ok(array) = open_or_instantiate_array_normalized(memory.clone(), &format!("/{clean_var}"))
    else {
        return Vec::new();
    };
    let variable = VariableInfo {
        name: clean_var.to_string(),
        dimension_names: crate::utils::resolve_array_dimension_names(&array),
        attributes: array
            .attributes()
            .iter()
            .map(|(k, v)| (k.clone(), attr_string(v)))
            .collect(),
        ..Default::default()
    };
    let names = block_coordinate_candidates(&variable, &request.selections);
    coordinate_arrays(memory, variable.group_path(), names)
}
