//! NetCDF hyperslab reading and BlockStore trait implementation.

use std::sync::Arc;

use netcdf::types::{FloatType, IntType, NcVariableType};
use netcdf::{Extent, Extents};

use super::attrs::{extract_variable_attributes, extract_variable_calibration};
use super::desktop::NetCdfBlockStore;
use super::inspect::inspect_netcdf_file;
use crate::data::blocks::{BlockResult, BlockStore, BlockStoreError, ProgressCallback};
use crate::data::metadata::DatasetMetadata;
use crate::data::octant_block::OctantBlock;
use crate::data::slice_request::SliceRequest;
use crate::utils::grid::check_and_orient_block_grid;
use crate::utils::grid_flips::OrientHints;

/// Read raw numeric values from a NetCDF variable, convert to `f32`, and apply scale/offset/fill masking.
pub fn read_variable_hyperslab_as_f32(
    var: &netcdf::Variable<'_>,
    extents: &Extents,
) -> Result<Vec<f32>, BlockStoreError> {
    let calibration = extract_variable_calibration(var);
    let has_tx = calibration.has_transformation();

    macro_rules! read_and_transform {
        ($var:expr, $extents:expr, $t:ty) => {{
            let raw_vals: Vec<$t> = $var
                .get_values::<$t, _>($extents)
                .map_err(|e| format!("Failed reading {} hyperslab: {e}", stringify!($t)))?;
            if !has_tx {
                Ok(raw_vals.into_iter().map(|v| v as f32).collect())
            } else {
                Ok(raw_vals
                    .into_iter()
                    .map(|v| calibration.transform(v as f64))
                    .collect())
            }
        }};
    }

    match var.vartype() {
        NcVariableType::Float(FloatType::F32) => {
            let mut raw_vals: Vec<f32> = var
                .get_values::<f32, _>(extents)
                .map_err(|e| format!("Failed reading float32 hyperslab: {e}"))?;

            if has_tx {
                calibration.transform_slice_in_place(&mut raw_vals);
            }
            Ok(raw_vals)
        }
        NcVariableType::Float(FloatType::F64) => read_and_transform!(var, extents, f64),
        NcVariableType::Int(IntType::I32) => read_and_transform!(var, extents, i32),
        NcVariableType::Int(IntType::I16) => read_and_transform!(var, extents, i16),
        NcVariableType::Int(IntType::I8) => read_and_transform!(var, extents, i8),
        NcVariableType::Int(IntType::U32) => read_and_transform!(var, extents, u32),
        NcVariableType::Int(IntType::U16) => read_and_transform!(var, extents, u16),
        NcVariableType::Int(IntType::U8) | NcVariableType::Char => {
            read_and_transform!(var, extents, u8)
        }
        NcVariableType::Int(IntType::I64) => read_and_transform!(var, extents, i64),
        NcVariableType::Int(IntType::U64) => read_and_transform!(var, extents, u64),
        other => Err(format!("Unsupported NetCDF variable type for plotting: {other:?}").into()),
    }
}

pub fn with_netcdf_variable<R>(
    file: &netcdf::File,
    name: &str,
    f: impl FnOnce(&netcdf::Variable) -> Result<R, BlockStoreError>,
) -> Result<R, BlockStoreError> {
    let clean = name.trim_start_matches('/').trim_end_matches('/');
    if let Some(var) = file.variable(clean) {
        return f(&var);
    }

    let segments: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() > 1 {
        fn find_in_group<R>(
            group: &netcdf::Group,
            segments: &[&str],
            f: impl FnOnce(&netcdf::Variable) -> Result<R, BlockStoreError>,
        ) -> Result<R, BlockStoreError> {
            if segments.len() == 1 {
                if let Some(var) = group.variable(segments[0]) {
                    return f(&var);
                }
            } else if let Some(sub) = group.group(segments[0]) {
                return find_in_group(&sub, &segments[1..], f);
            }
            Err(format!(
                "Variable '{}' not found in NetCDF group",
                segments.join("/")
            )
            .into())
        }

        if let Ok(Some(top_group)) = file.group(segments[0]) {
            return find_in_group(&top_group, &segments[1..], f);
        }
    }

    Err(format!("Variable '{name}' not found in NetCDF file").into())
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
        inspect_netcdf_file(&self.file_path)
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        self.fetch_block_with_progress(request, None)
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        mut on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        let file = netcdf::open(&self.file_path)
            .map_err(|e| format!("Failed to open NetCDF file '{}': {e}", self.file_path))?;

        with_netcdf_variable(&file, &request.variable, |var| {
            let (extents, window) = selection_window(var, request)?;
            let raw_values = read_variable_hyperslab_as_f32(var, &extents)?;
            if let Some(ref mut cb) = on_progress {
                let bytes = raw_values.len().saturating_mul(std::mem::size_of::<f32>());
                cb(bytes as u64);
            }
            Ok(oriented_block(&file, request, var, raw_values, window))
        })
    }

    fn fetch_blocks(&self, requests: &[SliceRequest]) -> Result<BlockResult, BlockStoreError> {
        use rayon::prelude::*;

        let blocks: Result<Vec<OctantBlock>, BlockStoreError> = requests
            .par_iter()
            .map(|request| self.fetch_block(request))
            .collect();

        Ok(BlockResult::new(blocks?))
    }
}

/// The dimension names, origin and shape of a block window.
struct BlockWindow {
    dim_names: Vec<String>,
    origin: Vec<usize>,
    shape: Vec<usize>,
}

/// The extents `request` reads from `var`, clamped to its dimensions, and their window.
fn selection_window(
    var: &netcdf::Variable<'_>,
    request: &SliceRequest,
) -> Result<(Extents, BlockWindow), BlockStoreError> {
    let dims = var.dimensions();
    if request.selections.len() != dims.len() {
        return Err(format!(
            "fetch_block: selection has {} dimension(s) but '{}' has rank {}",
            request.selections.len(),
            request.variable,
            dims.len()
        )
        .into());
    }
    let mut extents = Vec::with_capacity(dims.len());
    let mut window = BlockWindow {
        dim_names: Vec::with_capacity(dims.len()),
        origin: Vec::with_capacity(dims.len()),
        shape: Vec::with_capacity(dims.len()),
    };
    for (dim, sel) in dims.iter().zip(&request.selections) {
        let dim_len = dim.len();
        let (start, end) = sel.bounds();
        let start = start.min(dim_len.saturating_sub(1));
        let end = end.max(start.saturating_add(1)).min(dim_len);
        let count = end.saturating_sub(start).max(1);
        extents.push(Extent::SliceCount {
            start,
            count,
            stride: 1,
        });
        window.dim_names.push(dim.name());
        window.origin.push(start);
        window.shape.push(count);
    }
    Ok((Extents::from(extents), window))
}

/// The block of `raw_values` read from `var` over `window`, with its window coordinates,
/// oriented north-up and west-left.
fn oriented_block(
    file: &netcdf::File,
    request: &SliceRequest,
    var: &netcdf::Variable<'_>,
    raw_values: Vec<f32>,
    mut window: BlockWindow,
) -> OctantBlock {
    let attributes = extract_variable_attributes(var);
    let mut coords = super::coords::extract_sliced_coordinates(
        file,
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
