//! A composite brick's apron from its neighbours (`apron.rs`): the
//! neighbour's block is blended over just the box the apron needs, with the
//! plot's stretches (`CompositeLod::slice`), so filtering across bricks reads
//! the colors each brick shows.

use std::ops::Range;

use super::Lod;
use crate::app::OctantApp;
use crate::app::block_loading::block_axes;
use crate::app::detail::LevelRead;
use crate::data::OctantBlock;
use crate::data::multiscale::bricks::{APRON, BRICK, BrickKey};
use crate::plots::volume::encode::packed_rgba;

impl Lod {
    /// RGBA8 colors of `block` over the stored box `region` of brick `key`'s
    /// cube (x fastest), blended as the plotted composite; `None` for a
    /// scalar plot or a block of other dimensions.
    pub(super) fn region_colors(
        &self,
        key: BrickKey,
        region: &[Range<usize>; 3],
        block: &OctantBlock,
        read: &LevelRead,
    ) -> Option<Vec<[u8; 4]>> {
        let (dims, config) = (&read.dims, &read.sel.dim_config);
        if block.dimension_names != *dims {
            return None;
        }
        let (bx, by, bz) =
            OctantApp::resolve_spatial_axes(block.rank(), &block.dimension_names, dims, config);
        let fixed =
            block_axes::fixed_indices(block, dims, read.sel.animated_dim(), config, read.step);
        self.colors_over(key, region, block, ([bx, by, bz], &fixed))
    }

    /// [`Self::region_colors`] with the block's x, y, z dimensions `axes`
    /// and the indices `fixed` of its others.
    pub(super) fn colors_over(
        &self,
        key: BrickKey,
        region: &[Range<usize>; 3],
        block: &OctantBlock,
        (axes, fixed): ([usize; 3], &[usize]),
    ) -> Option<Vec<[u8; 4]>> {
        let composite = self.composite.as_ref()?;
        if region.iter().any(|r| r.is_empty()) {
            return None;
        }
        let [bx, by, bz] = axes;
        // Block-local stored index of the cube's stored cell `s` along `i`.
        let local = |i: usize, s: usize| {
            let cell = (u64::from(key.b[i]) * BRICK + s as u64).saturating_sub(APRON);
            let origin = block.origin.get(axes[i]).copied().unwrap_or(0) as u64;
            let len = block.shape.get(axes[i]).copied().unwrap_or(1);
            (cell.saturating_sub(origin) as usize).min(len.saturating_sub(1))
        };
        let span: [(usize, usize); 3] = std::array::from_fn(|i| {
            let (a, b) = (local(i, region[i].start), local(i, region[i].end - 1));
            block.oriented_range(axes[i], (a.min(b), a.max(b) + 1))
        });
        let volume = composite.slice(block, (bx, by, bz), (span[0], span[1], span[2]), fixed)?;
        let n = [volume.width, volume.height, volume.depth];
        let at = |i: usize, s: usize| {
            let o = block
                .oriented_index(axes[i], local(i, s))
                .saturating_sub(span[i].0);
            o.min(n[i].saturating_sub(1))
        };
        let mut out = Vec::new();
        for z in region[2].clone() {
            for y in region[1].clone() {
                let row = (at(2, z) * n[1] + at(1, y)) * n[0];
                for x in region[0].clone() {
                    let v = volume
                        .values
                        .get(row + at(0, x))
                        .copied()
                        .unwrap_or(f32::NAN);
                    out.push(packed_rgba(v));
                }
            }
        }
        Some(out)
    }
}
