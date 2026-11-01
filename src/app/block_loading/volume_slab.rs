//! Writing a block's 3D slab into a layer's volume, mirrored along flipped dimensions.

use super::projection_channels_3d::SlabRanges;
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::VolumeData;
use crate::data::block_orientation::slab_destination;
use crate::data::octant_block::OctantBlock;

impl OctantApp {
    /// Slices `block` over the local ranges and writes it into layer `id`'s
    /// volume, into plane `ring_slot` when given (`volume_ring`). Returns
    /// whether it wrote.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn write_volume_slab(
        &mut self,
        id: LayerId,
        block: &OctantBlock,
        (x_dim, y_dim, z_dim, c_dim): (usize, usize, usize, usize),
        (req_x, req_y, req_z): ((usize, usize), (usize, usize), (usize, usize)),
        (local_x_range, local_y_range, local_z_range): (
            (usize, usize),
            (usize, usize),
            (usize, usize),
        ),
        fixed_indices: &[usize],
        compute_bounds: bool,
        target_desc: &str,
        ring_slot: Option<usize>,
    ) -> bool {
        let layer = self.layers.get(id);
        let composite = layer.is_some_and(|l| l.composite.enabled);
        let gpu_blend = layer.is_some_and(|l| l.data.channel_volume.is_some());
        // Slices read the block in its oriented (flipped) order.
        let oriented = (
            block.oriented_range(x_dim, local_x_range),
            block.oriented_range(y_dim, local_y_range),
            block.oriented_range(z_dim, local_z_range),
        );
        let dims = (x_dim, y_dim, z_dim, c_dim);
        // The GPU blends raw channels; the volume keeps the selected one's.
        let channels = if gpu_blend {
            let Some(slabs) = self.channel_slabs(id, block, dims, oriented, fixed_indices) else {
                return false;
            };
            slabs
        } else {
            Vec::new()
        };
        let cpu_composite = composite && !gpu_blend;
        let sliced = self.slice_slab(
            id,
            block,
            dims,
            oriented,
            fixed_indices,
            compute_bounds,
            target_desc,
            cpu_composite,
        );
        let Some(slab) = sliced else {
            return false;
        };
        self.commit_volume_slab(
            id,
            block,
            x_dim,
            y_dim,
            z_dim,
            req_x.0,
            req_y.0,
            req_z,
            local_x_range.0,
            local_y_range.0,
            local_z_range.0,
            slab,
            ring_slot,
            &channels,
        );
        true
    }

    /// `block`'s slab over the `oriented` ranges of its `(x, y, z, c)`
    /// `dims`: the colors of a composite the CPU blends (`cpu_composite`),
    /// else its values.
    #[allow(clippy::too_many_arguments)]
    fn slice_slab(
        &self,
        id: LayerId,
        block: &OctantBlock,
        dims: (usize, usize, usize, usize),
        oriented: SlabRanges,
        fixed: &[usize],
        compute_bounds: bool,
        target_desc: &str,
        cpu_composite: bool,
    ) -> Option<VolumeData> {
        let (x_dim, y_dim, z_dim, c_dim) = dims;
        let (x, y, z) = oriented;
        let composite = (cpu_composite
            && block.shape.len() >= 3
            && block.shape.get(c_dim).copied().unwrap_or(0) >= 1)
            .then(|| {
                self.slice_3d_composite_volume(
                    id,
                    block,
                    c_dim,
                    x_dim,
                    y_dim,
                    z_dim,
                    x,
                    y,
                    z,
                    fixed,
                    target_desc,
                )
            })
            .flatten();
        composite.or_else(|| {
            block.volume_with_ranges(
                x_dim,
                y_dim,
                z_dim,
                x,
                y,
                z,
                fixed,
                target_desc,
                compute_bounds,
            )
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn commit_volume_slab(
        &mut self,
        id: LayerId,
        block: &OctantBlock,
        x_dim: usize,
        y_dim: usize,
        z_dim: usize,
        req_x0: usize,
        req_y0: usize,
        req_z: (usize, usize),
        local_x0: usize,
        local_y0: usize,
        local_z0: usize,
        slab: VolumeData,
        ring_slot: Option<usize>,
        channels: &[VolumeData],
    ) {
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        let (vol_w, vol_h, depth_max) = layer
            .data
            .volume
            .as_ref()
            .map_or((1, 1, 1), |v| (v.width, v.height, v.depth));
        // A dimension outside the block (no Z) has origin 0.
        let dest = |dim: usize, local: usize, len: usize, req: usize, vol: usize| {
            let origin = block.origin.get(dim).copied().unwrap_or(0);
            slab_destination(block.is_flipped(dim), (origin, local, len), (req, vol))
        };
        let dest_x = dest(x_dim, local_x0, slab.width, req_x0, vol_w);
        let dest_y = dest(y_dim, local_y0, slab.height, req_y0, vol_h);
        // A flipped Z mirrors across the whole requested range, then wraps into the ring
        // buffer, which may hold fewer planes than requested.
        let req_z_len = req_z.1.saturating_sub(req_z.0).saturating_add(1);
        let raw_dest_z = dest(z_dim, local_z0, slab.depth, req_z.0, req_z_len);
        let dest_z = ring_slot.unwrap_or(raw_dest_z % depth_max.max(1));

        let mut bounds_opt = None;
        if let Some(vdata) = &mut layer.data.volume {
            vdata.update_subvolume(
                [dest_x, dest_y, dest_z],
                [slab.width, slab.height, slab.depth.min(vdata.depth)],
                &slab.values,
            );

            bounds_opt = Some((vdata.min_val, vdata.max_val));
            layer.data.touch_volume();
            layer.data.last_volume_block =
                Some((layer.data.volume_allocations, block.origin.clone()));
        }
        if let Some(cv) = &mut layer.data.channel_volume {
            cv.write_slabs([dest_x, dest_y, dest_z], depth_max, slab.depth, channels);
        }
        // Uploaded before the next paint, to the renderer on screen only.
        layer
            .renderers
            .mark_volume_dirty(dest_z..(dest_z + slab.depth).min(depth_max));

        if let Some((min_val, max_val)) = bounds_opt {
            layer.color.follow_extent(min_val, max_val);
        }
    }
}
