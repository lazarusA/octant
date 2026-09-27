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

        let is_channel_dim = |d: usize| -> bool {
            let Some(name) = block_dim_names.get(d) else {
                return false;
            };
            crate::data::coordinates::naming::is_channel_dim_name(name)
        };

        let x_dim = explicit_x.unwrap_or_else(|| {
            (0..rank)
                .rev()
                .find(|&d| Some(d) != anim_dim && !is_channel_dim(d))
                .unwrap_or(0)
        });

        let y_dim = explicit_y.unwrap_or_else(|| {
            (0..rank)
                .rev()
                .find(|&d| d != x_dim && Some(d) != anim_dim && !is_channel_dim(d))
                .unwrap_or_else(|| (0..rank).find(|&d| d != x_dim).unwrap_or(0))
        });

        let z_dim = explicit_z.unwrap_or_else(|| {
            (0..rank)
                .find(|&d| d != x_dim && d != y_dim && Some(d) != anim_dim && !is_channel_dim(d))
                .unwrap_or(usize::MAX)
        });

        (x_dim, y_dim, z_dim)
    }

    /// Projects a resident block into current 2D and 3D views.
    pub fn apply_block_projection(&mut self, block: &crate::data::octant_block::OctantBlock) {
        let dim_configs = if !self.plotted_dim_config.is_empty() {
            &self.plotted_dim_config
        } else {
            &self.dim_config
        };
        let anim_dim = self
            .plotted_animated_dim
            .or_else(|| crate::app::DimConfig::animated_dim(dim_configs));
        let orig_dim_names: Vec<String> = self
            .plotted_dataset_metadata
            .as_ref()
            .or(self.active_dataset_metadata.as_ref())
            .and_then(|meta| {
                meta.variables
                    .get(self.plotted_variable_idx)
                    .or_else(|| meta.variables.get(self.selected_variable_idx))
            })
            .map(|v| v.dimension_names.clone())
            .unwrap_or_else(|| block.dimension_names.clone());

        let (x_dim, y_dim, z_dim) = Self::resolve_spatial_axes(
            block.rank(),
            &block.dimension_names,
            &orig_dim_names,
            dim_configs,
        );

        let sel_indices = if !self.plotted_selected_dim_indices.is_empty() {
            &self.plotted_selected_dim_indices
        } else {
            &self.selected_dim_indices
        };

        let fixed_indices: Vec<usize> = (0..block.rank())
            .map(|i| {
                let name = block.dimension_names.get(i);
                let orig_idx = name
                    .and_then(|n| orig_dim_names.iter().position(|o| o == n))
                    .unwrap_or(i);
                let idx = if Some(orig_idx) == anim_dim {
                    self.current_timestep
                } else {
                    sel_indices.get(orig_idx).copied().unwrap_or(0)
                };
                idx.saturating_sub(block.origin.get(i).copied().unwrap_or(0))
            })
            .collect();

        let sel_ranges = if !self.plotted_selected_dim_ranges.is_empty() {
            &self.plotted_selected_dim_ranges
        } else {
            &self.selected_dim_ranges
        };

        let get_dim_bounds = |dim_idx: usize| -> ((usize, usize), (usize, usize)) {
            let Some(dim_name) = block.dimension_names.get(dim_idx) else {
                let len = block.shape.get(dim_idx).copied().unwrap_or(1);
                return ((0, len.saturating_sub(1)), (0, len));
            };
            let orig_idx = orig_dim_names
                .iter()
                .position(|o| o == dim_name)
                .unwrap_or(dim_idx);
            let dim_len = block.shape.get(dim_idx).copied().unwrap_or(1);
            let block_orig = block.origin.get(dim_idx).copied().unwrap_or(0);
            let (req_start, req_end) = sel_ranges
                .get(orig_idx)
                .copied()
                .unwrap_or((0, dim_len.saturating_sub(1)));

            let local_start = req_start
                .saturating_sub(block_orig)
                .min(dim_len.saturating_sub(1));
            let local_end = (req_end + 1)
                .saturating_sub(block_orig)
                .clamp(local_start + 1, dim_len);
            ((req_start, req_end), (local_start, local_end))
        };

        let (req_x, local_x_range) = get_dim_bounds(x_dim);
        let (req_y, local_y_range) = get_dim_bounds(y_dim);
        let (req_z, local_z_range) = if z_dim < block.rank() {
            get_dim_bounds(z_dim)
        } else {
            ((0, 0), (0, 1))
        };

        let compute_bounds = !self.lock_color_bounds;
        let c_dim = self.channel_dim_index().unwrap_or(0);

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

        if is_3d_plot {
            self.apply_3d_volume_projection(
                block,
                x_dim,
                y_dim,
                z_dim,
                req_x,
                req_y,
                req_z,
                local_x_range,
                local_y_range,
                local_z_range,
                &fixed_indices,
                is_3d_spatial_anim,
                compute_bounds,
                c_dim,
            );
        } else {
            self.apply_2d_projection(
                block,
                x_dim,
                y_dim,
                local_x_range,
                local_y_range,
                &fixed_indices,
                compute_bounds,
                c_dim,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_2d_projection(
        &mut self,
        block: &crate::data::octant_block::OctantBlock,
        x_dim: usize,
        y_dim: usize,
        x_range: (usize, usize),
        y_range: (usize, usize),
        fixed_indices: &[usize],
        compute_bounds: bool,
        c_dim: usize,
    ) {
        let mdata_opt = if self.rgb_composite_mode
            && block.shape.len() >= 2
            && block.shape.get(c_dim).copied().unwrap_or(0) >= 1
        {
            if !self.is_geotiff() && !self.composite_channel_configs.is_empty() {
                crate::data::slicing::slice_multichannel_composite_nd(
                    block,
                    c_dim,
                    x_dim,
                    y_dim,
                    x_range,
                    y_range,
                    fixed_indices,
                    &self.composite_channel_configs,
                    self.animated_dim_extent(),
                )
            } else {
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
                    fixed_indices,
                    opt_channels,
                    self.animated_dim_extent(),
                )
            }
        } else {
            None
        };

        let mdata_opt = mdata_opt.or_else(|| {
            block.slice_2d_with_ranges(
                x_dim,
                y_dim,
                x_range,
                y_range,
                fixed_indices,
                self.animated_dim_extent(),
                &format!("Block Cache [{}]", block.variable_name),
                compute_bounds,
            )
        });

        if let Some(mdata) = mdata_opt {
            self.rebuild_pipeline_with_matrix_data(mdata);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_3d_volume_projection(
        &mut self,
        block: &crate::data::octant_block::OctantBlock,
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
        let target_nx = (req_x.1 + 1).saturating_sub(req_x.0).max(1);
        let target_ny = (req_y.1 + 1).saturating_sub(req_y.0).max(1);
        let full_nz = if z_dim < block.rank() {
            (req_z.1 + 1).saturating_sub(req_z.0).max(1)
        } else {
            1
        };

        let slice_elements = target_nx.saturating_mul(target_ny).max(1);
        let max_z = (crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS / slice_elements)
            .clamp(1, full_nz);
        let target_nz = full_nz.min(max_z);

        let channels_signature: String = if self.rgb_composite_mode {
            if !self.is_geotiff() && !self.composite_channel_configs.is_empty() {
                self.composite_channel_configs
                    .iter()
                    .map(|c| format!("{}:{}:{:?}:{:?}", c.index, c.visible, c.color_rgb, c.window))
                    .collect::<Vec<_>>()
                    .join(";")
            } else {
                format!("rgb:{:?}", self.rgb_composite_channels)
            }
        } else {
            "scalar".to_string()
        };

        let target_volume_desc = format!(
            "Volume [{}] var={} target=({}x{}x{}) xr={}..={} yr={}..={} zr={}..={} fixed={:?} comp={}",
            self.plotted_store_target_input,
            block.variable_name,
            target_nx,
            target_ny,
            target_nz,
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
            channels_signature,
        );

        let needs_full_realloc = match &self.volume_data {
            Some(existing) => {
                existing.width != target_nx
                    || existing.height != target_ny
                    || existing.depth != target_nz
                    || existing.dataset_name != target_volume_desc
                    || self.volume_renderer.is_none()
                    || self.point_cloud_renderer.is_none()
            }
            None => true,
        };

        if needs_full_realloc {
            let total_elements = target_nx * target_ny * target_nz;
            let initial_vdata = crate::data::volume_data::VolumeData::new(
                target_nx,
                target_ny,
                target_nz,
                vec![f32::NAN; total_elements],
                f32::NAN,
                f32::NAN,
                target_volume_desc.clone(),
            );
            self.rebuild_pipeline_with_volume_data(initial_vdata);
        }

        let slab_opt = if self.rgb_composite_mode
            && block.shape.len() >= 3
            && block.shape.get(c_dim).copied().unwrap_or(0) >= 1
        {
            if !self.is_geotiff() && !self.composite_channel_configs.is_empty() {
                crate::data::slicing::slice_multichannel_volume_composite_nd(
                    block,
                    c_dim,
                    x_dim,
                    y_dim,
                    z_dim,
                    local_x_range,
                    local_y_range,
                    local_z_range,
                    fixed_indices,
                    &self.composite_channel_configs,
                    &target_volume_desc,
                )
            } else {
                let opt_channels = [
                    Some(self.rgb_composite_channels[0]),
                    Some(self.rgb_composite_channels[1]),
                    Some(self.rgb_composite_channels[2]),
                ];
                crate::data::slicing::slice_rgb_volume_composite_nd(
                    block,
                    c_dim,
                    x_dim,
                    y_dim,
                    z_dim,
                    local_x_range,
                    local_y_range,
                    local_z_range,
                    fixed_indices,
                    opt_channels,
                    &target_volume_desc,
                )
            }
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
                &target_volume_desc,
                compute_bounds,
            )
        });

        if let Some(slab) = slab_opt {
            let block_orig_x = block.origin.get(x_dim).copied().unwrap_or(0);
            let block_orig_y = block.origin.get(y_dim).copied().unwrap_or(0);
            let block_orig_z = if z_dim < block.rank() {
                block.origin.get(z_dim).copied().unwrap_or(0)
            } else {
                0
            };

            let dest_x = (block_orig_x + local_x_range.0).saturating_sub(req_x.0);
            let dest_y = (block_orig_y + local_y_range.0).saturating_sub(req_y.0);
            let dest_z = (block_orig_z + local_z_range.0).saturating_sub(req_z.0);

            if let Some(vdata) = &mut self.volume_data {
                vdata.update_subvolume(
                    [dest_x, dest_y, dest_z],
                    [slab.width, slab.height, slab.depth],
                    &slab.values,
                );

                if let Some(wgpu_render_state) = &self.wgpu_render_state {
                    if let Some(volume_renderer) = &self.volume_renderer {
                        volume_renderer.update_data(&wgpu_render_state.queue, &vdata.values);
                    }
                    if let Some(point_cloud_renderer) = &self.point_cloud_renderer {
                        point_cloud_renderer.update_data(&wgpu_render_state.queue, &vdata.values);
                    }
                }

                if !self.lock_color_bounds {
                    if vdata.min_val.is_finite() {
                        self.volume_cmin = vdata.min_val;
                        self.color_range_min = vdata.min_val;
                    }
                    if vdata.max_val.is_finite() {
                        self.volume_cmax = vdata.max_val;
                        self.color_range_max = vdata.max_val;
                    }
                }
                if vdata.min_val.is_finite() {
                    self.global_data_min = self.global_data_min.min(vdata.min_val);
                }
                if vdata.max_val.is_finite() {
                    self.global_data_max = self.global_data_max.max(vdata.max_val);
                }
            }
        }
    }
}
