//! NetCDF hyperslab reading and the `BlockStore` implementation, over whichever reader the
//! store opens (`NetCdfBlockStore::with_reader`).

use std::sync::Arc;

use super::NetCdfBlockStore;
use super::inspect::inspect_reader;
use super::reader::{NcReader, NcVar, Raw};
use crate::data::blocks::{BlockStore, BlockStoreError};
use crate::data::metadata::DatasetMetadata;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::utils::grid::check_and_orient_block_grid;
use crate::utils::grid_flips::OrientHints;

/// Reads the window of `var` as `f32`, applying scale/offset/fill masking.
pub fn read_variable_hyperslab_as_f32(
    reader: &dyn NcReader,
    var: &NcVar,
    start: &[usize],
    count: &[usize],
) -> Result<Vec<f32>, BlockStoreError> {
    let calibration = &var.calibration;
    let has_tx = calibration.has_transformation();
    Ok(match reader.read(var, start, count)? {
        Raw::F32(mut values) => {
            if has_tx {
                calibration.transform_slice_in_place(&mut values);
            }
            values
        }
        Raw::F64(values) if has_tx => values
            .into_iter()
            .map(|v| calibration.transform(v))
            .collect(),
        Raw::F64(values) => values.into_iter().map(|v| v as f32).collect(),
    })
}

impl BlockStore for NetCdfBlockStore {
    fn backend_name(&self) -> &str {
        "NetCDF"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        let meta = self.inspect()?;
        Ok(meta.variables.into_iter().map(|v| v.name).collect())
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        self.with_reader(|reader| Ok(inspect_reader(reader, self.file_path())))
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.with_reader(|reader| {
            let var = reader.variable(&request.variable).ok_or_else(|| {
                format!("Variable '{}' not found in NetCDF file", request.variable)
            })?;
            let window = selection_window(&var, request)?;
            let values =
                read_variable_hyperslab_as_f32(reader, &var, &window.origin, &window.shape)?;
            Ok(oriented_block(reader, request, &var, values, window))
        })
    }
}

/// The dimension names, origin and shape of a block window.
struct BlockWindow {
    dim_names: Vec<String>,
    origin: Vec<usize>,
    shape: Vec<usize>,
}

/// The window `request` reads from `var`, clamped to its dimensions.
fn selection_window(var: &NcVar, request: &SliceRequest) -> Result<BlockWindow, BlockStoreError> {
    if request.selections.len() != var.dims.len() {
        return Err(format!(
            "fetch_block: selection has {} dimension(s) but '{}' has rank {}",
            request.selections.len(),
            request.variable,
            var.dims.len()
        )
        .into());
    }
    let mut window = BlockWindow {
        dim_names: Vec::with_capacity(var.dims.len()),
        origin: Vec::with_capacity(var.dims.len()),
        shape: Vec::with_capacity(var.dims.len()),
    };
    for ((name, dim_len), sel) in var.dims.iter().zip(&request.selections) {
        let (start, end) = sel.bounds();
        let start = start.min(dim_len.saturating_sub(1));
        let end = end.max(start.saturating_add(1)).min(*dim_len);
        window.dim_names.push(name.clone());
        window.origin.push(start);
        window.shape.push(end.saturating_sub(start).max(1));
    }
    Ok(window)
}

/// The block of `raw_values` read from `var` over `window`, with its window coordinates,
/// oriented north-up and west-left.
fn oriented_block(
    reader: &dyn NcReader,
    request: &SliceRequest,
    var: &NcVar,
    raw_values: Vec<f32>,
    mut window: BlockWindow,
) -> OctantBlock {
    let attributes = var.attributes.clone();
    let mut coords = super::coords::extract_sliced_coordinates(
        reader,
        &request.variable,
        &window.dim_names,
        &window.origin,
        &window.shape,
    );
    let json_attrs: serde_json::Map<String, serde_json::Value> = attributes
        .iter()
        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        .collect();
    let hints = OrientHints {
        attributes: &json_attrs,
        lat_extent: coords.lat_extent,
        lon_extent: coords.lon_extent,
    };
    let (oriented_values, flipped_dims) = check_and_orient_block_grid(
        raw_values,
        &mut window.shape,
        &mut window.dim_names,
        &mut window.origin,
        hints,
        &mut coords.coordinates,
    );
    let mut block = OctantBlock::new(
        request.variable.clone(),
        window.shape,
        window.dim_names,
        window.origin,
        Arc::from(oriented_values.into_boxed_slice()),
        coords.coordinates,
        attributes,
    );
    block.flipped_dims = flipped_dims;
    block
}
