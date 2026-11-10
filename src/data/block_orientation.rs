//! Orienting a block as it is read (`BlockCoords`), and index ranges of blocks whose
//! orientation reversed some dimensions (`OctantBlock::flipped_dims`).

use std::collections::HashMap;
use std::sync::Arc;

use crate::data::blocks::BlockStoreError;
use crate::data::coordinates::naming::{is_lat_name, is_lon_name};
use crate::data::octant_block::OctantBlock;
use crate::utils::grid::check_and_orient_block_grid;
use crate::utils::grid_flips::OrientHints;

impl OctantBlock {
    /// Whether dimension `dim` runs opposite to storage order in this block.
    pub fn is_flipped(&self, dim: usize) -> bool {
        self.dimension_names
            .get(dim)
            .is_some_and(|name| self.flipped_dims.contains(name))
    }

    /// The block-local index holding stored-order local index `index` along `dim`.
    pub fn oriented_index(&self, dim: usize, index: usize) -> usize {
        match self.shape.get(dim).and_then(|len| len.checked_sub(1)) {
            Some(last) if self.is_flipped(dim) => last - index.min(last),
            _ => index,
        }
    }

    /// The block-local range `start..end` that holds stored-order local range `range` along
    /// `dim`: mirrored for flipped dimensions, unchanged otherwise.
    pub fn oriented_range(&self, dim: usize, (start, end): (usize, usize)) -> (usize, usize) {
        let len = self.shape.get(dim).copied().unwrap_or(0);
        // Dimensions outside the block (a volume without Z) pass `(0, 1)` through.
        debug_assert!(
            dim >= self.shape.len() || (start <= end && end <= len),
            "range {start}..{end} outside {len}"
        );
        if self.is_flipped(dim) && start <= end && end <= len {
            (len - end, len - start)
        } else {
            (start, end)
        }
    }
}

/// Where a slab of `slab_len` indices, starting at stored local index `local_start` of a
/// block at `block_origin`, goes in a volume of `volume_len` indices requested from
/// `request_start`. A flipped dimension runs opposite to storage across the whole volume,
/// so its slab is placed mirrored.
pub fn slab_destination(
    flipped: bool,
    (block_origin, local_start, slab_len): (usize, usize, usize),
    (request_start, volume_len): (usize, usize),
) -> usize {
    let stored_start = block_origin.saturating_add(local_start);
    if flipped {
        volume_len.saturating_sub(
            stored_start
                .saturating_add(slab_len)
                .saturating_sub(request_start),
        )
    } else {
        stored_start.saturating_sub(request_start)
    }
}

/// Window coordinates of a block, keyed by dimension name (and its lowercase form), plus
/// the first and last coordinate of the whole latitude and longitude dimensions, which
/// decide the block's orientation. Filled by every reader that orients its blocks.
#[derive(Default)]
pub struct BlockCoords {
    pub coordinates: HashMap<String, Vec<f64>>,
    pub lat_extent: Option<(f64, f64)>,
    pub lon_extent: Option<(f64, f64)>,
}

impl BlockCoords {
    /// Keeps `values` as the coordinates of `name`, also under its lowercase form.
    pub fn insert(&mut self, name: &str, values: Vec<f64>) {
        let clean = name.trim().to_lowercase();
        if clean != name {
            self.coordinates.insert(clean, values.clone());
        }
        self.coordinates.insert(name.to_string(), values);
    }

    /// Records `extent` when `name` is the first latitude or longitude seen.
    pub fn note_extent(&mut self, name: &str, extent: Option<(f64, f64)>) {
        if is_lat_name(name) && self.lat_extent.is_none() {
            self.lat_extent = extent;
        } else if is_lon_name(name) && self.lon_extent.is_none() {
            self.lon_extent = extent;
        }
    }

    /// The block of `values` over the window `(dim_names, origin, shape)` with these
    /// coordinates, oriented north-up and west-left (`check_and_orient_block_grid`).
    pub fn oriented_block(
        mut self,
        variable: String,
        values: Vec<f32>,
        (mut dim_names, mut origin, mut shape): (Vec<String>, Vec<usize>, Vec<usize>),
        attributes: Arc<HashMap<String, String>>,
    ) -> Result<OctantBlock, BlockStoreError> {
        let hints = OrientHints {
            attributes: &attributes,
            lat_extent: self.lat_extent,
            lon_extent: self.lon_extent,
        };
        let (values, flipped_dims) = check_and_orient_block_grid(
            values,
            &mut shape,
            &mut dim_names,
            &mut origin,
            hints,
            &mut self.coordinates,
        );
        let mut block = OctantBlock::new(
            variable,
            shape,
            dim_names,
            origin,
            values,
            self.coordinates,
            attributes,
        )?;
        block.flipped_dims = flipped_dims;
        Ok(block)
    }
}
