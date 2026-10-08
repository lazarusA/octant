//! 3D volumetric data and composite subvolume progressive projection routines.

use super::projection_hash::{compute_composite_hash, compute_target_dims};
use crate::app::OctantApp;
use crate::app::layers::{Layer, LayerId};
use crate::data::octant_block::OctantBlock;
use crate::data::volume_data::VolumeData;

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
        // Raw channel readouts only exist for 2D composites.
        layer.data.composite_probe = None;
        layer.data.flipped_dims.clone_from(&block.flipped_dims);

        // Slices read the block in its oriented (flipped) order.
        let oriented = (
            block.oriented_range(x_dim, local_x_range),
            block.oriented_range(y_dim, local_y_range),
            block.oriented_range(z_dim, local_z_range),
        );
        let slab_opt = if composite
            && block.shape.len() >= 3
            && block.shape.get(c_dim).copied().unwrap_or(0) >= 1
        {
            self.slice_3d_composite_volume(
                id,
                block,
                c_dim,
                x_dim,
                y_dim,
                z_dim,
                oriented.0,
                oriented.1,
                oriented.2,
                fixed_indices,
                &target_desc,
            )
        } else {
            None
        };

        let slab_opt = slab_opt.or_else(|| {
            block.volume_with_ranges(
                x_dim,
                y_dim,
                z_dim,
                oriented.0,
                oriented.1,
                oriented.2,
                fixed_indices,
                &target_desc,
                compute_bounds,
            )
        });

        if let Some(slab) = slab_opt {
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
            );
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
    fn slice_3d_composite_volume(
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
