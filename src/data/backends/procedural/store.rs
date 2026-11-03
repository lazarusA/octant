//! `BlockStore` trait implementation for `ProceduralBlockStore`.

use super::ProceduralBlockStore;
use super::healpix::slice_healpix_block;
use super::inspect::inspect_procedural;
use super::slice_2d::slice_2d_grid_block;
use super::slice_3d::{slice_gaussian_wave_packet_4d, slice_procedural_matrix_block};
use crate::data::blocks::{BlockStore, BlockStoreError, ProgressCallback};
use crate::data::metadata::DatasetMetadata;
use crate::data::octant_block::OctantBlock;
use crate::data::procedural::{
    generate_clenshaw_curtis_2d, generate_clenshaw_curtis_coords, generate_gaussian_coords,
    generate_gaussian_grid_2d, generate_stepped_resolution_2d, generate_stepped_resolution_coords,
    generate_stretched_regional_2d, generate_stretched_regional_coords,
};
use crate::data::slice_request::SliceRequest;

impl BlockStore for ProceduralBlockStore {
    fn backend_name(&self) -> &str {
        "Procedural / Known-Truth"
    }

    fn variables(&self) -> Result<Vec<String>, BlockStoreError> {
        if self.uri.contains("healpix") {
            return Ok(vec![
                "temp".to_string(),
                "mslp".to_string(),
                "lat".to_string(),
                "lon".to_string(),
                "ring".to_string(),
            ]);
        }
        let is_4d = self.uri.contains("volume") || self.uri.contains("4d");
        if is_4d {
            Ok(vec![
                "gaussian_wave_packet_4d".to_string(),
                "procedural_matrix_2d".to_string(),
            ])
        } else {
            Ok(vec![
                "clenshaw_curtis_2d".to_string(),
                "gaussian_grid_2d".to_string(),
                "stretched_regional_2d".to_string(),
                "stepped_resolution_2d".to_string(),
                "gaussian_wave_packet_4d".to_string(),
                "procedural_matrix_2d".to_string(),
            ])
        }
    }

    fn inspect(&self) -> Result<DatasetMetadata, BlockStoreError> {
        inspect_procedural(&self.uri)
    }

    fn fetch_block_with_progress(
        &self,
        request: &SliceRequest,
        mut on_progress: ProgressCallback,
    ) -> Result<OctantBlock, BlockStoreError> {
        if self.uri.contains("healpix") {
            return slice_healpix_block(request, &mut on_progress);
        }

        if request.variable == "clenshaw_curtis_2d" {
            let (h_full, w_full) = (64, 128);
            let (full_data, _, _) = generate_clenshaw_curtis_2d(w_full, h_full, 0);
            let (xs, ys) = generate_clenshaw_curtis_coords(w_full, h_full);
            return Ok(slice_2d_grid_block(
                &request.variable,
                (w_full, h_full),
                &full_data,
                (&xs, &ys),
                request,
                &mut on_progress,
            ));
        }

        if request.variable == "gaussian_grid_2d" {
            let (h_full, w_full) = (64, 128);
            let (full_data, _, _) = generate_gaussian_grid_2d(w_full, h_full, 0);
            let (xs, ys) = generate_gaussian_coords(w_full, h_full);
            return Ok(slice_2d_grid_block(
                &request.variable,
                (w_full, h_full),
                &full_data,
                (&xs, &ys),
                request,
                &mut on_progress,
            ));
        }

        if request.variable == "stretched_regional_2d" {
            let (h_full, w_full) = (32, 48);
            let (full_data, _, _) = generate_stretched_regional_2d(w_full, h_full);
            let (xs, ys) = generate_stretched_regional_coords(w_full, h_full);
            return Ok(slice_2d_grid_block(
                &request.variable,
                (w_full, h_full),
                &full_data,
                (&xs, &ys),
                request,
                &mut on_progress,
            ));
        }

        if request.variable == "stepped_resolution_2d" {
            let (h_full, w_full) = (32, 64);
            let (full_data, _, _) = generate_stepped_resolution_2d(w_full, h_full);
            let (xs, ys) = generate_stepped_resolution_coords(w_full, h_full);
            return Ok(slice_2d_grid_block(
                &request.variable,
                (w_full, h_full),
                &full_data,
                (&xs, &ys),
                request,
                &mut on_progress,
            ));
        }

        if request.variable == "procedural_matrix_2d" {
            return Ok(slice_procedural_matrix_block(request, &mut on_progress));
        }

        // Default 4D Known-Truth Block slicing
        Ok(slice_gaussian_wave_packet_4d(request, &mut on_progress))
    }
}
