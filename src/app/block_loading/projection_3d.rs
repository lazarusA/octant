//! 3D volumetric data and composite subvolume progressive projection routines.

use super::projection_hash::{compute_composite_hash, compute_target_dims};
use crate::app::OctantApp;
use crate::data::octant_block::OctantBlock;
use crate::data::volume_data::VolumeData;

impl OctantApp {
    /// Projects a 3D scalar or composite block subvolume onto the 3D render pipelines.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_3d_volume_projection(
        &mut self,
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
        let eff_z = if self.rgb_composite_mode && z_dim == c_dim {
            usize::MAX
        } else {
            z_dim
        };
        let (nx, ny, nz) = compute_target_dims(block, eff_z, req_x, req_y, req_z);
        let ch_hash = compute_composite_hash(self);
        let target_desc = format!(
            "Vol:[{}] var={} ({nx}x{ny}x{nz}) xr={}..={} yr={}..={} zr={}..={} fx={:?} ch={ch_hash:016x}",
            self.plotted_store_target_input,
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

        self.ensure_volume_allocated(nx, ny, nz, &target_desc);

        let slab_opt = if self.rgb_composite_mode
            && block.shape.len() >= 3
            && block.shape.get(c_dim).copied().unwrap_or(0) >= 1
        {
            self.slice_3d_composite_volume(
                block,
                c_dim,
                x_dim,
                y_dim,
                z_dim,
                local_x_range,
                local_y_range,
                local_z_range,
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
                local_x_range,
                local_y_range,
                local_z_range,
                fixed_indices,
                &target_desc,
                compute_bounds,
            )
        });

        if let Some(slab) = slab_opt {
            self.commit_volume_slab(
                block,
                x_dim,
                y_dim,
                z_dim,
                req_x.0,
                req_y.0,
                req_z.0,
                local_x_range.0,
                local_y_range.0,
                local_z_range.0,
                slab,
            );
        }
    }

    /// Whether the GPU renderers are missing or encode the wrong format. Without
    /// a GPU (headless) there are none to rebuild.
    fn volume_renderers_stale(&self) -> bool {
        self.wgpu_render_state.is_some()
            && (self
                .volume_renderer
                .as_ref()
                .is_none_or(|r| r.encoding() != self.volume_encoding())
                || self.point_cloud_renderer.is_none())
    }

    fn ensure_volume_allocated(&mut self, nx: usize, ny: usize, nz: usize, desc: &str) {
        let needs_realloc = match &self.volume_data {
            Some(ex) => {
                ex.width != nx
                    || ex.height != ny
                    || ex.depth != nz
                    || ex.dataset_name != desc
                    || self.volume_renderers_stale()
            }
            None => true,
        };

        if needs_realloc {
            self.volume_allocations += 1;
            let initial_vdata = VolumeData::new(
                nx,
                ny,
                nz,
                vec![f32::NAN; nx * ny * nz],
                f32::NAN,
                f32::NAN,
                desc.to_string(),
            );
            self.rebuild_pipeline_with_volume_data(initial_vdata);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn slice_3d_composite_volume(
        &self,
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
        if !self.is_geotiff() && !self.composite_channel_configs.is_empty() {
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
                &self.composite_channel_configs,
                desc,
            )
        } else {
            let opt_ch = [
                Some(self.rgb_composite_channels[0]),
                Some(self.rgb_composite_channels[1]),
                Some(self.rgb_composite_channels[2]),
            ];
            crate::data::slicing::slice_rgb_volume_composite_nd(
                block, c_dim, x_dim, y_dim, z_dim, x_rng, y_rng, z_rng, fixed, opt_ch, desc,
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn commit_volume_slab(
        &mut self,
        block: &OctantBlock,
        x_dim: usize,
        y_dim: usize,
        z_dim: usize,
        req_x0: usize,
        req_y0: usize,
        req_z0: usize,
        local_x0: usize,
        local_y0: usize,
        local_z0: usize,
        slab: VolumeData,
    ) {
        let orig_x = block.origin.get(x_dim).copied().unwrap_or(0);
        let orig_y = block.origin.get(y_dim).copied().unwrap_or(0);
        let orig_z = if z_dim < block.rank() {
            block.origin.get(z_dim).copied().unwrap_or(0)
        } else {
            0
        };

        let dest_x = (orig_x + local_x0).saturating_sub(req_x0);
        let dest_y = (orig_y + local_y0).saturating_sub(req_y0);
        let raw_dest_z = (orig_z + local_z0).saturating_sub(req_z0);
        let depth_max = self.volume_data.as_ref().map(|v| v.depth).unwrap_or(1);
        let dest_z = raw_dest_z % depth_max.max(1);

        let mut bounds_opt = None;
        if let Some(vdata) = &mut self.volume_data {
            vdata.update_subvolume(
                [dest_x, dest_y, dest_z],
                [slab.width, slab.height, slab.depth.min(vdata.depth)],
                &slab.values,
            );

            bounds_opt = Some((vdata.min_val, vdata.max_val));
        }
        // Uploaded before the next paint, to the renderer on screen only.
        self.mark_volume_dirty(dest_z..(dest_z + slab.depth).min(depth_max));

        if let Some((min_val, max_val)) = bounds_opt {
            self.sync_volume_color_bounds(min_val, max_val);
        }
    }

    fn sync_volume_color_bounds(&mut self, min_val: f32, max_val: f32) {
        if !self.lock_color_bounds {
            if min_val.is_finite() {
                self.volume_cmin = min_val;
                self.color_range_min = min_val;
            }
            if max_val.is_finite() {
                self.volume_cmax = max_val;
                self.color_range_max = max_val;
            }
        }
        if min_val.is_finite() {
            self.global_data_min = self.global_data_min.min(min_val);
        }
        if max_val.is_finite() {
            self.global_data_max = self.global_data_max.max(max_val);
        }
    }
}
