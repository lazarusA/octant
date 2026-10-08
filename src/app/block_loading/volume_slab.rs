//! Writing a block's 3D slab into a layer's volume, mirrored along flipped dimensions.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::block_orientation::slab_destination;
use crate::data::octant_block::OctantBlock;
use crate::data::volume_data::VolumeData;

impl OctantApp {
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
        let dest_z = raw_dest_z % depth_max.max(1);

        let mut bounds_opt = None;
        if let Some(vdata) = &mut layer.data.volume {
            vdata.update_subvolume(
                [dest_x, dest_y, dest_z],
                [slab.width, slab.height, slab.depth.min(vdata.depth)],
                &slab.values,
            );

            bounds_opt = Some((vdata.min_val, vdata.max_val));
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
