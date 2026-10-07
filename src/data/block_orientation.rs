//! Index ranges of blocks whose data orientation reversed some dimensions
//! (`OctantBlock::flipped_dims`).

use crate::data::octant_block::OctantBlock;

impl OctantBlock {
    /// Whether dimension `dim` runs opposite to storage order in this block.
    pub fn is_flipped(&self, dim: usize) -> bool {
        self.dimension_names
            .get(dim)
            .is_some_and(|name| self.flipped_dims.contains(name))
    }

    /// The block-local range `start..end` that holds stored-order local range `range` along
    /// `dim`: mirrored for flipped dimensions, unchanged otherwise.
    pub fn oriented_range(&self, dim: usize, (start, end): (usize, usize)) -> (usize, usize) {
        let len = self.shape.get(dim).copied().unwrap_or(0);
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
    let stored_start = block_origin + local_start;
    if flipped {
        volume_len.saturating_sub((stored_start + slab_len).saturating_sub(request_start))
    } else {
        stored_start.saturating_sub(request_start)
    }
}
