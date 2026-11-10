//! `BlockStore` trait implementation for `ProceduralBlockStore`.

use super::ProceduralBlockStore;
use super::healpix::slice_healpix_block;
use super::inspect::{GRIDS_2D, grid_coords, inspect_procedural};
use super::slice_2d::slice_2d_grid_block;
use super::slice_3d::{slice_gaussian_wave_packet_4d, slice_procedural_matrix_block};
use crate::data::blocks::{BlockStore, BlockStoreError};
use crate::data::metadata::DatasetMetadata;
use crate::data::octant_block::OctantBlock;
use crate::data::procedural::{
    generate_clenshaw_curtis_2d, generate_gaussian_grid_2d, generate_stepped_resolution_2d,
    generate_stretched_regional_2d,
};
use crate::data::slice_request::SliceRequest;

impl BlockStore for ProceduralBlockStore {
    fn backend_name(&self) -> &str {
        "Procedural / Known-Truth"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        let meta = inspect_procedural(&self.uri)?;
        Ok(meta.variables.into_iter().map(|v| v.name).collect())
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        inspect_procedural(&self.uri)
    }

    fn fetch_block(&self, request: &SliceRequest) -> Result<OctantBlock, BlockStoreError> {
        if self.uri.contains("healpix") {
            return slice_healpix_block(request);
        }

        if let Some((size, values, (xs, ys))) = grid_2d(&request.variable) {
            let coords = (xs.as_slice(), ys.as_slice());
            return slice_2d_grid_block(&request.variable, size, &values, coords, request);
        }

        if request.variable == "procedural_matrix_2d" {
            return slice_procedural_matrix_block(request);
        }

        // Default 4D Known-Truth Block slicing
        slice_gaussian_wave_packet_4d(request)
    }
}

/// A 2D test grid: (width, height), values and x/y coordinates.
type Grid2d = ((usize, usize), Vec<f32>, (Vec<f64>, Vec<f64>));

/// A 2D test grid's (width, height), values and x/y coordinates, sized by
/// `GRIDS_2D` as its inspection lists it.
fn grid_2d(name: &str) -> Option<Grid2d> {
    let &(_, w, h) = GRIDS_2D.iter().find(|g| g.0 == name)?;
    let values = match name {
        "clenshaw_curtis_2d" => generate_clenshaw_curtis_2d(w, h, 0).0,
        "gaussian_grid_2d" => generate_gaussian_grid_2d(w, h, 0).0,
        "stretched_regional_2d" => generate_stretched_regional_2d(w, h).0,
        _ => generate_stepped_resolution_2d(w, h).0,
    };
    let coords = grid_coords(name, w, h);
    Some(((w, h), values, coords))
}
