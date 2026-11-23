//! Placing an arrived brick (`fetch.rs` `place_arrivals`): its stored cube
//! and apron, a slot, its ranges, the upload and the table cells it changed,
//! then its neighbours' aprons and its ancestors' ranges.

use super::encode::{Cube, brick_values};
use super::fetch::Gpu;
use super::residency::CellBox;
use super::subblocks::{SubRanges, open_subs, sub_ranges};
use super::{Lod, Placed};
use crate::app::detail::LevelRead;
use crate::data::OctantBlock;
use crate::data::multiscale::bricks::{BrickKey, Bricks, STORED};
use crate::plots::volume::bricks::BrickTextures;

impl Lod {
    /// Puts `block` (`key`'s read) in a slot and uploads it and the table
    /// cells it changed (when there is a GPU); nothing while every slot is
    /// wanted. `false` when the block does not hold the brick.
    pub(super) fn place(
        &mut self,
        key: BrickKey,
        read: &LevelRead,
        block: OctantBlock,
        generation: u64,
        (gpu, cache): (Gpu, &crate::data::blocks::BlockCache),
    ) -> bool {
        let Some(mut cube) = brick_values(self, key, read, &block) else {
            return false;
        };
        self.stitch_own(key, &mut cube, cache, Some(read));
        let (range, missing, subs) = self.cube_ranges(&cube);
        // Not `self.bricks()`: the residency is borrowed apart from the set.
        let bricks = Bricks {
            set: &self.set,
            dims: self.dims,
        };
        let pinned = self.roots.contains(&key);
        let placed = Placed {
            block: read.key.clone(),
            range,
            missing,
            subs,
        };
        let Some((slot, changed)) = self
            .residency
            .insert(&bricks, key, placed, generation, pinned)
        else {
            return true;
        };
        if let Cube::Values(_) = cube {
            self.absorb_descendants(key);
        }
        let Some((gpu, queue)) = gpu else {
            return true;
        };
        self.upload_placed(key, (slot, &changed), &cube, (gpu, queue));
        self.stitch_neighbours(key, (&block, read), gpu, queue);
        if let Cube::Values(_) = cube {
            self.widen_ancestors(key, range, missing, Some((gpu, queue)));
        }
        true
    }

    /// The range of `cube`'s valid values, whether any voxel is missing, and
    /// its sub-blocks' ranges (computed in a single pass).
    fn cube_ranges(&self, cube: &Cube) -> ((f32, f32), bool, SubRanges) {
        match cube {
            Cube::Values(values) => sub_ranges(values, self.encoding()),
            // Never skipped as drawing nothing (composites are always open).
            Cube::Colors(_) => ((f32::MIN, f32::MAX), false, open_subs()),
        }
    }

    /// Uploads placed `key`'s `cube`, ranges and the table cells `changed`
    /// into `slot`.
    fn upload_placed(
        &self,
        key: BrickKey,
        (slot, changed): ([u32; 3], &[CellBox]),
        cube: &Cube,
        (gpu, queue): (&BrickTextures, &wgpu::Queue),
    ) {
        match cube {
            Cube::Values(values) => {
                let whole = ([0; 3], [STORED as u32; 3]);
                self.upload_values((gpu, queue), slot, whole, values);
            }
            Cube::Colors(colors) => gpu.upload_brick(queue, slot, colors),
        }
        if let Some(placed) = self.residency.get(&key) {
            let (lo, hi) = placed.range;
            gpu.write_range(queue, slot, [lo, hi], placed.missing);
            gpu.write_subs(queue, slot, &placed.subs);
        }
        for cells in changed {
            gpu.write_table(queue, cells, &self.residency.table_region(cells));
        }
    }
}
