//! Spatial axis resolution and block projection onto 2D / 3D render pipelines.

use crate::app::OctantApp;

impl OctantApp {
    /// Resolves the 3D spatial axis indices `(x_dim, y_dim, z_dim)` for block projections,
    /// honoring explicit user SpatialRoles (X/Y/Z) or falling back to non-animated dimensions.
    pub fn resolve_spatial_axes(
        rank: usize,
        block_dim_names: &[String],
        orig_dim_names: &[String],
        dim_config: &[crate::app::DimConfig],
    ) -> (usize, usize, usize) {
        if rank <= 1 {
            return (0, 0, usize::MAX);
        }

        let anim_dim = crate::app::DimConfig::animated_dim(dim_config);

        let find_explicit_spatial = |role: crate::app::SpatialRole| -> Option<usize> {
            (0..rank).find(|&d| {
                let Some(name) = block_dim_names.get(d) else {
                    return false;
                };
                let Some(orig_idx) = orig_dim_names.iter().position(|n| n == name) else {
                    return false;
                };
                dim_config.get(orig_idx).is_some_and(|c| c.spatial == role)
            })
        };

        let explicit_grid = find_explicit_spatial(crate::app::SpatialRole::Grid);
        let explicit_x = find_explicit_spatial(crate::app::SpatialRole::X);
        let explicit_y = find_explicit_spatial(crate::app::SpatialRole::Y);
        let explicit_z = find_explicit_spatial(crate::app::SpatialRole::Z);

        if let Some(grid_dim) = explicit_grid {
            let z_dim = explicit_z.unwrap_or(usize::MAX);
            return (grid_dim, grid_dim, z_dim);
        }

        let x_dim = explicit_x
            .unwrap_or_else(|| (0..rank).rev().find(|&d| Some(d) != anim_dim).unwrap_or(0));

        let y_dim = explicit_y.unwrap_or_else(|| {
            (0..rank)
                .rev()
                .find(|&d| d != x_dim && Some(d) != anim_dim)
                .unwrap_or_else(|| (0..rank).find(|&d| d != x_dim).unwrap_or(0))
        });

        let z_dim = explicit_z.unwrap_or_else(|| {
            (0..rank)
                .find(|&d| d != x_dim && d != y_dim)
                .unwrap_or(usize::MAX)
        });

        (x_dim, y_dim, z_dim)
    }

    /// Projects a resident block into current 2D and 3D views.
    pub fn apply_block_projection(&mut self, block: &crate::data::octant_block::OctantBlock) {
        // Synchronize plotted state now that the new block has arrived and is being rendered
        self.sync_plotted_state_from_selected();

        let anim_dim = crate::app::DimConfig::animated_dim(&self.plotted_dim_config);
        let orig_dim_names: Vec<String> = self
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|meta| meta.variables.get(self.plotted_variable_idx))
            .map(|v| v.dimension_names.clone())
            .unwrap_or_else(|| block.dimension_names.clone());

        let (x_dim, y_dim, z_dim) = Self::resolve_spatial_axes(
            block.rank(),
            &block.dimension_names,
            &orig_dim_names,
            &self.plotted_dim_config,
        );

        let fixed_indices: Vec<usize> = (0..block.rank())
            .map(|i| {
                let name = block.dimension_names.get(i);
                let orig_idx = name
                    .and_then(|n| orig_dim_names.iter().position(|o| o == n))
                    .unwrap_or(i);
                let idx = self
                    .plotted_selected_dim_indices
                    .get(orig_idx)
                    .copied()
                    .unwrap_or(0);
                idx.saturating_sub(block.origin.get(i).copied().unwrap_or(0))
            })
            .collect();

        let get_local_range = |dim_idx: usize| -> (usize, usize) {
            let Some(dim_name) = block.dimension_names.get(dim_idx) else {
                return (0, block.shape.get(dim_idx).copied().unwrap_or(1));
            };
            let orig_idx = orig_dim_names
                .iter()
                .position(|o| o == dim_name)
                .unwrap_or(dim_idx);
            let dim_len = block.shape.get(dim_idx).copied().unwrap_or(1);
            let block_orig = block.origin.get(dim_idx).copied().unwrap_or(0);
            let (req_start, req_end) = self
                .plotted_selected_dim_ranges
                .get(orig_idx)
                .copied()
                .unwrap_or((0, dim_len.saturating_sub(1)));

            let local_start = req_start
                .saturating_sub(block_orig)
                .min(dim_len.saturating_sub(1));
            let local_end = (req_end + 1)
                .saturating_sub(block_orig)
                .clamp(local_start + 1, dim_len);
            (local_start, local_end)
        };

        let x_range = get_local_range(x_dim);
        let y_range = get_local_range(y_dim);
        let z_range = if z_dim < block.rank() {
            get_local_range(z_dim)
        } else {
            (0, 1)
        };

        let compute_bounds = !self.lock_color_bounds;

        let c_dim = self.channel_dim_index().unwrap_or(0);
        let mdata_opt = if self.rgb_composite_mode
            && block.shape.len() >= 2
            && block.shape.get(c_dim).copied().unwrap_or(0) >= 2
        {
            let opt_channels = [
                Some(self.rgb_composite_channels[0]),
                Some(self.rgb_composite_channels[1]),
                Some(self.rgb_composite_channels[2]),
            ];
            crate::data::slicing::slice_rgb_composite_nd(
                block,
                c_dim,
                x_dim,
                y_dim,
                x_range,
                y_range,
                &fixed_indices,
                opt_channels,
                self.animated_dim_extent(),
            )
        } else {
            if self.rgb_composite_mode {
                self.rgb_composite_mode = false;
                if self.active_colormap == 1000 {
                    self.active_colormap = 0;
                }
            }
            block.slice_2d_with_ranges(
                x_dim,
                y_dim,
                x_range,
                y_range,
                &fixed_indices,
                self.animated_dim_extent(),
                &format!("Block Cache [{}]", block.variable_name),
                compute_bounds,
            )
        };

        if let Some(mdata) = mdata_opt {
            self.rebuild_pipeline_with_matrix_data(mdata);
        }

        let is_volume_allowed = crate::ui::variables_panel::is_volume_allowed_for_selection(self);
        if !is_volume_allowed
            && (self.active_plot_type == crate::plots::PlotType::Volume
                || self.active_plot_type == crate::plots::PlotType::PointCloud)
        {
            self.active_plot_type = crate::plots::PlotType::Heatmap;
        }

        let is_3d_plot = (self.active_plot_type == crate::plots::PlotType::Volume
            || self.active_plot_type == crate::plots::PlotType::PointCloud)
            && is_volume_allowed;

        let is_3d_spatial_anim = anim_dim.is_some_and(|a| a == x_dim || a == y_dim || a == z_dim);

        let current_volume_desc = format!(
            "Block Cache Volume [{}] origin={:?} shape={:?} fixed={:?} xr={:?} yr={:?} zr={:?}",
            block.variable_name,
            block.origin,
            block.shape,
            if is_3d_spatial_anim {
                &[] as &[usize]
            } else {
                fixed_indices.as_slice()
            },
            x_range,
            y_range,
            z_range,
        );

        let req_nx = x_range.1.saturating_sub(x_range.0);
        let req_ny = y_range.1.saturating_sub(y_range.0);
        let req_nz = z_range.1.saturating_sub(z_range.0);

        let needs_volume_update = if let Some(existing) = &self.volume_data {
            let slice_elements = req_nx.saturating_mul(req_ny).max(1);
            let max_z = (crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS / slice_elements)
                .clamp(1, req_nz);
            let eff_nz = req_nz.min(max_z);

            existing.width != req_nx
                || existing.height != req_ny
                || existing.depth != eff_nz
                || self.volume_renderer.is_none()
                || existing.dataset_name != current_volume_desc
        } else {
            true
        };

        if is_3d_plot
            && needs_volume_update
            && let Some(vdata) = block.volume_with_ranges(
                x_dim,
                y_dim,
                z_dim,
                x_range,
                y_range,
                z_range,
                &fixed_indices,
                &current_volume_desc,
                compute_bounds,
            )
        {
            let depth = vdata.depth;
            self.rebuild_pipeline_with_volume_data(vdata);
            self.status_message = format!(
                "{}  [x_dim={x_dim} y_dim={y_dim} z_dim={z_dim} depth={depth} anim_dim={anim_dim:?} t={}]",
                self.status_message, self.current_timestep
            );
        }
    }
}
