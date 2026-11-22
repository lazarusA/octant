//! Which renderer the base volume draws with: the bricked one once every root brick arrived, with the same
//! parameters (it fills the same box), else the plotted volume; and what
//! the hover reads there: the level and value of the finest brick drawn
//! under a voxel of the plotted volume.

use std::sync::Arc;

use super::Lod;
use crate::app::OctantApp;
use crate::app::layers::{Layer, LayerId};
use crate::data::OctantBlock;
use crate::data::multiscale::bricks::BRICK;
use crate::plots::{VolumeRenderer, VolumeUniformParams};

impl OctantApp {
    /// `layer`'s volume renderer and parameters on a canvas `screen_aspect`
    /// wide per unit of height; `moving` while the view or a setting is
    /// being dragged.
    pub(crate) fn volume_draw(
        &self,
        layer: &Layer,
        screen_aspect: f32,
        moving: bool,
    ) -> Option<(Arc<VolumeRenderer>, VolumeUniformParams)> {
        let params = self.get_volume_uniform_params(layer, screen_aspect);
        let lod = layer.data.volume_lod.lod.as_ref().filter(|l| l.ready());
        let bricked = layer.renderers.volume_bricked.as_ref();
        if let (Some(lod), Some(renderer), LayerId::BASE) = (lod, bricked, layer.id()) {
            // Moving, rays step as the plotted volume's: the frame keeps up;
            // at rest, as the finest level drawn.
            let level = match moving {
                true => lod.root_level,
                false => lod.finest,
            };
            if let Some(bricks) = renderer.bricks() {
                bricks.set_dims(lod.level_dims(level));
            }
            return Some((renderer.clone(), params));
        }
        Some((layer.renderers.volume.as_ref()?.clone(), params))
    }

    /// The level and value of the brick drawn at voxel `voxel` (x, y, z in
    /// drawn order) of the plotted volume of `dims` voxels; `None` while the
    /// volume is not bricked or the brick's block left the cache.
    pub(crate) fn brick_reading(
        &self,
        voxel: [usize; 3],
        dims: [usize; 3],
    ) -> Option<(usize, f32)> {
        let lod = self.layers.base.data.volume_lod.lod.as_ref();
        // A composite's hover reads its channels from the plotted volume.
        let lod = lod.filter(|l| l.ready() && l.composite.is_none())?;
        let q = lod.level0_center(voxel, dims);
        let bricks = lod.bricks();
        let grid = bricks.grid();
        let cell =
            std::array::from_fn(|i| ((q[i] / BRICK as f64) as u32).min(grid[i].saturating_sub(1)));
        let (key, placed) = lod.residency.lookup(&bricks, cell)?;
        let block = self.block_cache.peek(&placed.block)?;
        let level = usize::from(key.level);
        // MIP draws the largest of the finer voxels inside the hit voxel.
        let value = match self.plot_configs.volume.algorithm {
            1 => lod.block_extreme(block, level, lod.level0_box(voxel, dims), f32::max),
            // MinIP draws the smallest.
            2 => lod.block_extreme(block, level, lod.level0_box(voxel, dims), f32::min),
            _ => lod.block_value(block, level, q),
        };
        Some((level, value?))
    }
}

impl Lod {
    /// The level-0 position (storage order) of the center of the plotted
    /// volume's voxel `voxel` (drawn order) of `dims` voxels.
    fn level0_center(&self, voxel: [usize; 3], dims: [usize; 3]) -> [f64; 3] {
        self.level0_at(std::array::from_fn(|i| {
            (voxel[i] as f64 + 0.5) / dims[i].max(1) as f64
        }))
    }

    /// The level-0 position (storage order) of the plotted window's point
    /// `t` (0..1 along x, y, z in drawn order).
    fn level0_at(&self, t: [f64; 3]) -> [f64; 3] {
        std::array::from_fn(|i| {
            let s = if self.flip[i] { 1.0 - t[i] } else { t[i] };
            self.place.lo[i] + s * (self.place.hi[i] - self.place.lo[i])
        })
    }

    /// The level-0 box (storage order, low and high corner) the plotted
    /// volume's voxel `voxel` of `dims` voxels covers.
    fn level0_box(&self, voxel: [usize; 3], dims: [usize; 3]) -> [[f64; 3]; 2] {
        let n = |i: usize| dims[i].max(1) as f64;
        let a = self.level0_at(std::array::from_fn(|i| voxel[i] as f64 / n(i)));
        let b = self.level0_at(std::array::from_fn(|i| (voxel[i] + 1) as f64 / n(i)));
        [
            std::array::from_fn(|i| a[i].min(b[i])),
            std::array::from_fn(|i| a[i].max(b[i])),
        ]
    }

    /// The largest (`pick` = `f32::max`) or smallest of `block`'s values (a
    /// brick of `level`) whose cells' centers lie in the level-0 box
    /// `lo..hi`; the center's cell when none.
    fn block_extreme(
        &self,
        block: &OctantBlock,
        level: usize,
        [lo, hi]: [[f64; 3]; 2],
        pick: fn(f32, f32) -> f32,
    ) -> Option<f32> {
        let center = std::array::from_fn(|i| (lo[i] + hi[i]) / 2.0);
        let lvl = self.set.level(level)?;
        let axes = self.block_axes(block)?;
        // The box's cells inside the block: the hover runs every frame.
        let cells: [std::ops::Range<usize>; 3] = std::array::from_fn(|i| {
            let d = self.dims[i];
            let f = lvl.factor.get(d).copied().unwrap_or(1.0);
            let off = lvl.offset.get(d).copied().unwrap_or(0.0);
            let first = ((lo[i] - off) / f - 0.5).ceil().max(0.0) as usize;
            let end = ((hi[i] - off) / f - 0.5).ceil().max(0.0) as usize;
            let (_, origin, len) = axes[i];
            first.max(origin)..end.min(origin + len)
        });
        let mut best: Option<f32> = None;
        for z in cells[2].clone() {
            for y in cells[1].clone() {
                for x in cells[0].clone() {
                    let v = value_at(block, &axes, [x, y, z]).filter(|v| v.is_finite());
                    best = match (best, v) {
                        (Some(b), Some(v)) => Some(pick(b, v)),
                        (b, v) => b.or(v),
                    };
                }
            }
        }
        best.or_else(|| self.block_value(block, level, center))
    }

    /// `block`'s value (a brick of `level`) at level-0 position `q`.
    fn block_value(&self, block: &OctantBlock, level: usize, q: [f64; 3]) -> Option<f32> {
        let level = self.set.level(level)?;
        let cell: [u64; 3] = std::array::from_fn(|i| {
            let d = self.dims[i];
            let (f, off) = (
                level.factor.get(d).copied().unwrap_or(1.0),
                level.offset.get(d).copied().unwrap_or(0.0),
            );
            ((q[i] - off) / f).floor().max(0.0) as u64
        });
        self.cell_value(block, cell)
    }

    /// `block`'s value at its level's cell `cell` (x, y, z), clamped into
    /// the block.
    pub(super) fn cell_value(&self, block: &OctantBlock, cell: [u64; 3]) -> Option<f32> {
        value_at(block, &self.block_axes(block)?, cell.map(|c| c as usize))
    }

    /// `block`'s x, y and z: its dimension, where it starts on the level
    /// and its length (resolved once for a loop over its cells).
    pub(super) fn block_axes(&self, block: &OctantBlock) -> Option<BlockAxes> {
        let mut axes = [(0, 0, 0); 3];
        for (i, axis) in axes.iter_mut().enumerate() {
            let name = self.set.dims.get(self.dims[i])?;
            let k = block.dimension_names.iter().position(|n| n == name)?;
            *axis = (k, *block.origin.get(k)?, *block.shape.get(k)?);
        }
        Some(axes)
    }
}

/// Per x, y, z: a block's dimension, its start on the level and its length.
pub(super) type BlockAxes = [(usize, usize, usize); 3];

/// `block`'s value at its level's cell `cell` (x, y, z) along `axes`,
/// clamped into the block; other dimensions at their first index.
pub(super) fn value_at(block: &OctantBlock, axes: &BlockAxes, cell: [usize; 3]) -> Option<f32> {
    // On the stack: the hover and aprons read many.
    let mut index = [0usize; 8];
    let rank = block.rank();
    if rank > index.len() {
        return None;
    }
    for (i, &(k, origin, len)) in axes.iter().enumerate() {
        let local = cell[i].saturating_sub(origin).min(len.saturating_sub(1));
        index[k] = block.oriented_index(k, local);
    }
    block.get(&index[..rank])
}
