//! 3D volumetric data and composite subvolume progressive projection routines.

use super::projection_hash::{compute_composite_hash, compute_target_dims};
use super::volume_ring::RingPlane;
use crate::app::OctantApp;
use crate::app::layers::{Layer, LayerId};
use crate::data::VolumeData;
use crate::data::octant_block::OctantBlock;

impl OctantApp {
    /// Projects a 3D scalar or composite block subvolume onto layer `id`'s 3D render pipelines.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_3d_volume_projection(
        &mut self,
        id: LayerId,
        block: &OctantBlock,
        x_dim: usize,
        y_dim: usize,
        z_dim: usize,
        req_x: (usize, usize),
        req_y: (usize, usize),
        req_z: (usize, usize),
        local_x_range: (usize, usize),
        local_y_range: (usize, usize),
        local_z_range: (usize, usize),
        fixed_indices: &[usize],
        is_3d_spatial_anim: bool,
        compute_bounds: bool,
        c_dim: usize,
    ) {
        let Some(layer) = self.layers.get(id) else {
            return;
        };
        let composite = layer.composite.enabled;
        let eff_z = if composite && z_dim == c_dim {
            usize::MAX
        } else {
            z_dim
        };
        let (nx, ny, nz) = compute_target_dims(block, eff_z, req_x, req_y, req_z);
        let ch_hash = compute_composite_hash(&layer.composite, self.layer_is_geotiff(id));
        let target_desc = format!(
            "Vol:[{}] var={} ({nx}x{ny}x{nz}) xr={}..={} yr={}..={} zr={}..={} fx={:?} ch={ch_hash:016x}",
            layer.selection().store_target,
            block.variable_name,
            req_x.0,
            req_x.1,
            req_y.0,
            req_y.1,
            req_z.0,
            req_z.1,
            if is_3d_spatial_anim {
                &[] as &[usize]
            } else {
                fixed_indices
            },
        );

        self.ensure_volume_allocated(id, nx, ny, nz, &target_desc);
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        // Steps along an animated X or Y axis only rotate the ring buffer: the
        // block written last is still in place, so nothing to slice or upload.
        let unchanged = layer
            .data
            .last_volume_block
            .as_ref()
            .is_some_and(|(alloc, origin)| {
                *alloc == layer.data.volume_allocations && *origin == block.origin
            });
        // Along an animated Z axis only the planes the window gained are
        // written (`volume_ring`); a flipped Z keeps whole blocks.
        let ring = z_dim < block.rank() && !block.is_flipped(z_dim) && self.animates_along_z(id);
        if is_3d_spatial_anim && unchanged && !ring {
            return;
        }
        let planes: Vec<Option<RingPlane>> = if ring {
            let planes = self.ring_planes_to_write(id, block, z_dim);
            planes.into_iter().map(Some).collect()
        } else {
            vec![None]
        };
        if planes.is_empty() {
            return;
        }
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        // Raw channel readouts only exist for 2D composites.
        layer.data.composite_probe = None;
        layer.data.composite_stretch = None;
        layer.data.flipped_dims.clone_from(&block.flipped_dims);

        for plane in planes {
            let local_z = plane
                .as_ref()
                .map_or(local_z_range, |p| (p.local, p.local + 1));
            let written = self.write_volume_slab(
                id,
                block,
                (x_dim, y_dim, z_dim, c_dim),
                (req_x, req_y, req_z),
                (local_x_range, local_y_range, local_z),
                fixed_indices,
                compute_bounds,
                &target_desc,
                plane.as_ref().map(|p| p.slot),
            );
            if written && let Some(p) = plane {
                self.record_ring_plane(id, p.slot, p.step);
            }
        }
    }

    /// Whether `layer`'s GPU renderers are missing or encode the wrong format.
    /// Without a GPU (headless) there are none to rebuild.
    fn volume_renderers_stale(&self, layer: &Layer) -> bool {
        self.wgpu_render_state.is_some()
            && (layer
                .renderers
                .volume
                .as_ref()
                .is_none_or(|r| r.encoding() != layer.volume_encoding())
                || layer.renderers.point_cloud.is_none())
    }

    fn ensure_volume_allocated(
        &mut self,
        id: LayerId,
        nx: usize,
        ny: usize,
        nz: usize,
        desc: &str,
    ) {
        let Some(layer) = self.layers.get(id) else {
            return;
        };
        let needs_realloc = match &layer.data.volume {
            Some(ex) => {
                ex.width != nx
                    || ex.height != ny
                    || ex.depth != nz
                    || ex.dataset_name != desc
                    || self.volume_renderers_stale(layer)
            }
            None => true,
        };

        if needs_realloc && let Some(layer) = self.layers.get_mut(id) {
            layer.data.volume_allocations += 1;
            let initial_vdata = VolumeData::new(
                nx,
                ny,
                nz,
                vec![f32::NAN; nx * ny * nz],
                f32::NAN,
                f32::NAN,
                desc.to_string(),
            );
            self.rebuild_pipeline_with_volume_data(id, initial_vdata);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn slice_3d_composite_volume(
        &self,
        id: LayerId,
        block: &OctantBlock,
        c_dim: usize,
        x_dim: usize,
        y_dim: usize,
        z_dim: usize,
        x_rng: (usize, usize),
        y_rng: (usize, usize),
        z_rng: (usize, usize),
        fixed: &[usize],
        desc: &str,
    ) -> Option<VolumeData> {
        let composite = &self.layers.get(id)?.composite;
        if !self.layer_is_geotiff(id) && !composite.channel_configs.is_empty() {
            crate::data::slicing::slice_multichannel_volume_composite_nd(
                block,
                c_dim,
                x_dim,
                y_dim,
                z_dim,
                x_rng,
                y_rng,
                z_rng,
                fixed,
                &composite.channel_configs,
                desc,
            )
        } else {
            let opt_ch = composite.rgb_channels.map(Some);
            crate::data::slicing::slice_rgb_volume_composite_nd(
                block, c_dim, x_dim, y_dim, z_dim, x_rng, y_rng, z_rng, fixed, opt_ch, desc,
            )
        }
    }
}
