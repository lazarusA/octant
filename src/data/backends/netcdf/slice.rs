//! NetCDF hyperslab reading and the `BlockStore` implementation, over whichever reader the
//! store opens (`NetCdfBlockStore::with_reader`).

use super::NetCdfBlockStore;
use super::inspect::inspect_reader;
use super::reader::{NcReader, NcVar, Raw};
use crate::data::blocks::{BlockStore, BlockStoreError};
use crate::data::metadata::DatasetMetadata;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;

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
            oriented_block(reader, request, &var, values, window)
        })
    }
}

/// The dimension names, origin and shape of a block window.
struct BlockWindow {
    dim_names: Vec<String>,
    origin: Vec<usize>,
    shape: Vec<usize>,
}

/// The window `request` reads from `var` (`SliceRequest::window`).
fn selection_window(var: &NcVar, request: &SliceRequest) -> Result<BlockWindow, BlockStoreError> {
    let lens: Vec<u64> = var.dims.iter().map(|(_, len)| *len as u64).collect();
    let (origin, shape) = request.window(&lens)?;
    let dim_names = var.dims.iter().map(|(name, _)| name.clone()).collect();
    Ok(BlockWindow {
        dim_names,
        origin,
        shape,
    })
}

/// The block of `raw_values` read from `var` over `window`, with its window coordinates,
/// oriented north-up and west-left.
fn oriented_block(
    reader: &dyn NcReader,
    request: &SliceRequest,
    var: &NcVar,
    raw_values: Vec<f32>,
    window: BlockWindow,
) -> Result<OctantBlock, BlockStoreError> {
    let coords = super::coords::extract_sliced_coordinates(
        reader,
        &request.variable,
        &window.dim_names,
        &window.origin,
        &window.shape,
    );
    coords.oriented_block(
        request.variable.clone(),
        raw_values,
        (window.dim_names, window.origin, window.shape),
        var.attributes.clone(),
    )
}
