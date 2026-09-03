//! Loading path through `DatasetManager`/`BlockCache`/`BlockPrefetcher`.

use crate::data::{BlockRequest, DimensionSelection, SliceRequest};

use super::OctantApp;

impl OctantApp {
    /// Aligned `[start, end)` window along the animated dimension that
    /// contains `step`, clamped to the dataset's actual extent.
    ///
    /// Automatically bounds the window size based on:
    /// 1. Chunk size along the animated dimension (always chunk-aligned).
    /// 2. Memory footprint of a single chunk (caps single block footprint to ~64 MB).
    /// 3. User-configured `block_window_size` (e.g. 8 to 128 steps).
    pub fn animated_window_bounds(
        &self,
        step: usize,
        full_extent: usize,
        anim_dim: usize,
        chunk_shape: &[u64],
        shape: &[u64],
        selections: &[DimensionSelection],
    ) -> (usize, usize, usize) {
        let cs = chunk_shape.get(anim_dim).copied().unwrap_or(1).max(1) as usize;

        // Calculate elements in one slice across all other non-animated dimensions,
        // respecting the user's active slider selections and ranges.
        let other_elements: u64 = selections
            .iter()
            .enumerate()
            .filter(|&(d, _)| d != anim_dim)
            .map(|(d, sel)| match sel {
                DimensionSelection::Index(_) => 1u64,
                DimensionSelection::Range { start, end } => {
                    let extent = shape.get(d).copied().unwrap_or(1) as usize;
                    let eff_end = (*end).min(extent);
                    (eff_end.saturating_sub(*start) as u64).max(1)
                }
            })
            .try_fold(1u64, |acc, d| acc.checked_mul(d))
            .unwrap_or(1);

        let single_chunk_bytes = (cs as u64).saturating_mul(other_elements).saturating_mul(4);

        // Target ~64 MB max per single OctantBlock to avoid huge single-request delays
        let target_block_bytes = 64 * 1024 * 1024u64;
        let max_chunks_by_memory =
            (target_block_bytes / single_chunk_bytes.max(1)).clamp(1, 16) as usize;
        let max_chunks_by_user = (self.block_window_size / cs).max(1);
        let chunks_in_window = max_chunks_by_user.min(max_chunks_by_memory);
        let window_size = chunks_in_window * cs;

        let start = (step / cs) * cs;
        let end = (start + window_size).min(full_extent).max(start + 1);
        (start, end, window_size)
    }

    /// Returns the open `StoreHandle` for the currently plotted dataset from `dataset_manager`.
    pub fn plotted_store_handle(&self) -> Option<crate::data::StoreHandle> {
        let source_id = self.plotted_source_id();
        self.dataset_manager
            .get(&source_id)
            .map(|d| d.store.clone())
    }

    /// Loads the block corresponding to the current animated step and selections.
    pub fn load_selected_variable_block(&mut self) {
        let Some(metadata) = &self.plotted_dataset_metadata else {
            self.status_message = "No plotted dataset metadata loaded.".to_string();
            return;
        };
        let Some(var_info) = metadata.variables.get(self.plotted_variable_idx) else {
            self.status_message = "Invalid plotted variable index.".to_string();
            return;
        };

        let var_name = var_info.name.clone();
        let shape = var_info.shape.clone();

        let base_request =
            crate::ui::variables_panel::build_slice_request_for_plotted(self, &var_name, &shape);
        let mut selections = base_request.selections.clone();

        if let Some(anim_dim) = self.plotted_animated_dim {
            let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
            if full_extent > 0 && self.current_timestep >= full_extent {
                self.current_timestep = full_extent - 1;
            }
            if anim_dim < self.plotted_selected_dim_indices.len() {
                self.plotted_selected_dim_indices[anim_dim] = self.current_timestep;
            }
            if anim_dim < selections.len() {
                let (start, end, _) = self.animated_window_bounds(
                    self.current_timestep,
                    full_extent,
                    anim_dim,
                    &var_info.chunk_shape,
                    &shape,
                    &selections,
                );
                selections[anim_dim] = DimensionSelection::Range { start, end };
            }
        }

        let slice_request = SliceRequest::new(&var_name, selections);
        self.active_slice_request = Some(slice_request.clone());

        let source_id = self.plotted_source_id();

        // 1. Cache HIT: Check if any resident block in memory (e.g. full dataset array) covers current_timestep
        if let Some(block) = self.block_cache.find_covering_block(
            &source_id,
            &var_name,
            &slice_request.selections,
            self.plotted_animated_dim,
            self.current_timestep,
        ) {
            self.status_message = format!(
                "🚀 Block cache HIT for '{}' ({} bytes resident)",
                block.variable_name,
                block.bytes_size()
            );
            self.apply_block_projection(&block);
            self.prefetch_selected_animated_range(&shape);
            return;
        }

        let Some(store_handle) = self.plotted_store_handle() else {
            self.status_message =
                format!("Dataset store not open in DatasetManager for '{source_id}'");
            return;
        };

        let block_request = BlockRequest::new(store_handle, slice_request);
        let key = block_request.cache_key();
        self.active_block_key = Some(key.clone());

        // 2. Exact Key Cache HIT
        if let Some(block) = self.block_cache.get(&key) {
            self.status_message = format!(
                "🚀 Block cache HIT for '{}' ({} bytes resident)",
                block.variable_name,
                block.bytes_size()
            );
            self.apply_block_projection(&block);
            self.prefetch_selected_animated_range(&shape);
            return;
        }

        // 3. Cache MISS: dispatch async prefetch request for current chunk and launch background prefetching in parallel.
        self.status_message = format!("⏳ [block cache] Downloading window for '{}'...", var_name);
        self.block_prefetcher
            .request(block_request, &self.block_cache);
        self.prefetch_selected_animated_range(&shape);
    }

    /// Prefetches the block window containing `step` asynchronously using `plotted_store_handle()`,
    /// without modifying current UI, `matrix_data`, or `current_timestep` state.
    pub fn prefetch_block_window_for_next_steps(&mut self, step: usize) {
        let Some(metadata) = &self.plotted_dataset_metadata else {
            return;
        };
        let Some(var_info) = metadata.variables.get(self.plotted_variable_idx) else {
            return;
        };
        let Some(store_handle) = self.plotted_store_handle() else {
            return;
        };

        let var_name = var_info.name.clone();
        let shape = var_info.shape.clone();

        let source_id = self.plotted_source_id();

        let Some(anim_dim) = self.plotted_animated_dim else {
            return;
        };
        let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
        if step >= full_extent {
            return;
        }

        let base_request =
            crate::ui::variables_panel::build_slice_request_for_plotted(self, &var_name, &shape);

        if self.block_cache.covers(
            &source_id,
            &var_name,
            &base_request.selections,
            Some(anim_dim),
            step,
        ) {
            return;
        }

        let (start, end, _) = self.animated_window_bounds(
            step,
            full_extent,
            anim_dim,
            &var_info.chunk_shape,
            &shape,
            &base_request.selections,
        );

        let mut selections = base_request.selections;
        if anim_dim < selections.len() {
            selections[anim_dim] = DimensionSelection::Range { start, end };
        }

        let slice_request = SliceRequest::new(&var_name, selections);

        let block_request = BlockRequest::new(store_handle, slice_request);
        self.active_block_key = Some(block_request.cache_key());
        self.pending_target_step = Some(step);
        self.block_prefetcher
            .request(block_request, &self.block_cache);
    }

    /// Checks if `target_step` is resident in the block cache.
    /// - If resident: updates `current_timestep` to `target_step` and projects data immediately.
    /// - If not resident: keeps `current_timestep` on the current valid step and prefetches the block window containing `target_step`.
    pub fn request_step_or_load(&mut self, target_step: usize) {
        let source_id = self.plotted_source_id();
        let var_name = self.plotted_variable_info().map(|v| v.name.clone());
        let base_request = self.plotted_variable_info().map(|v| {
            crate::ui::variables_panel::build_slice_request_for_plotted(self, &v.name, &v.shape)
        });
        let selections = base_request
            .as_ref()
            .map(|r| r.selections.as_slice())
            .unwrap_or(&[]);

        let is_cached = if let Some(ref name) = var_name {
            self.block_cache.covers(
                &source_id,
                name,
                selections,
                self.plotted_animated_dim,
                target_step,
            )
        } else {
            false
        };

        if is_cached {
            self.current_timestep = target_step;
            self.load_selected_variable_block();
        } else {
            self.prefetch_block_window_for_next_steps(target_step);
        }
    }

    /// Requests the previous step along the animated dimension.
    pub fn step_prev(&mut self) {
        let max_steps = self.animated_dim_extent();
        if max_steps > 0 {
            let prev_step = if self.current_timestep > 0 {
                self.current_timestep - 1
            } else {
                max_steps - 1
            };
            self.request_step_or_load(prev_step);
        }
    }

    /// Requests the next step along the animated dimension.
    pub fn step_next(&mut self) {
        let max_steps = self.animated_dim_extent();
        if max_steps > 0 {
            let next_step = (self.current_timestep + 1) % max_steps;
            self.request_step_or_load(next_step);
        }
    }

    /// Progressively prefetches all remaining block windows across the selected animated dimension range in the background.
    pub fn prefetch_selected_animated_range(&mut self, shape: &[u64]) {
        if !self.enable_prefetch {
            return;
        }

        let Some(anim_dim) = self.plotted_animated_dim else {
            return;
        };
        let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
        if full_extent <= 1 {
            return;
        }

        let Some(active_req) = self.active_slice_request.clone() else {
            return;
        };
        if anim_dim >= active_req.selections.len() {
            return;
        }

        let chunk_shape = self
            .plotted_variable_info()
            .map(|v| v.chunk_shape.clone())
            .unwrap_or_default();

        let (_, _, window_step) = self.animated_window_bounds(
            self.current_timestep,
            full_extent,
            anim_dim,
            &chunk_shape,
            shape,
            &active_req.selections,
        );
        let cs = window_step.max(1);

        // Determine user's selected slider range for the animated dimension
        let (range_start, range_end) = self
            .plotted_selected_dim_ranges
            .get(anim_dim)
            .copied()
            .unwrap_or((0, full_extent.saturating_sub(1)));
        let range_start = range_start.min(full_extent.saturating_sub(1));
        let range_end = range_end
            .min(full_extent.saturating_sub(1))
            .max(range_start);

        let first_chunk = range_start / cs;
        let last_chunk = range_end / cs;

        let source_id = self.plotted_source_id();

        let Some(store_handle) = self.plotted_store_handle() else {
            return;
        };

        let current_chunk = self.current_timestep / cs;

        // Schedule chunks ordered by proximity to current_timestep within the selected range [first_chunk..=last_chunk]:
        let mut chunk_indices = Vec::new();
        for c in (current_chunk + 1)..=last_chunk {
            chunk_indices.push(c);
        }
        for c in (first_chunk..=current_chunk).rev() {
            if !chunk_indices.contains(&c) {
                chunk_indices.push(c);
            }
        }

        for chunk_idx in chunk_indices {
            let chunk_start = chunk_idx * cs;
            let chunk_end = (chunk_start + cs).min(full_extent);

            if self.block_cache.covers(
                &source_id,
                &active_req.variable,
                &active_req.selections,
                self.plotted_animated_dim,
                chunk_start,
            ) || self.block_prefetcher.is_pending_timestep(
                &source_id,
                &active_req.variable,
                self.plotted_animated_dim,
                chunk_start,
            ) {
                continue;
            }

            let mut selections = active_req.selections.clone();
            selections[anim_dim] = DimensionSelection::Range {
                start: chunk_start,
                end: chunk_end,
            };
            let req = BlockRequest::new(
                store_handle.clone(),
                SliceRequest::new(active_req.variable.clone(), selections),
            );

            if !self.block_cache.contains(&req.cache_key())
                && !self.block_prefetcher.request(req, &self.block_cache)
            {
                break; // Stop when thread pool is saturated; will continue as worker threads finish
            }
        }
    }

    /// Full size of the currently animated dimension in the dataset.
    pub fn animated_dim_extent(&self) -> usize {
        let Some(anim_dim) = self.plotted_animated_dim else {
            return 1;
        };
        self.plotted_dataset_metadata
            .as_ref()
            .and_then(|meta| meta.variables.get(self.plotted_variable_idx))
            .and_then(|v| v.shape.get(anim_dim))
            .map(|&s| s as usize)
            .unwrap_or(1)
    }

    /// Resolves the 3D spatial axis indices `(x_dim, y_dim, z_dim)` for block projections,
    /// honoring explicit user SpatialRoles (X/Y/Z) or falling back to non-animated dimensions.
    pub fn resolve_spatial_axes(
        rank: usize,
        block_dim_names: &[String],
        orig_dim_names: &[String],
        dim_config: &[crate::app::DimConfig],
    ) -> (usize, usize, usize) {
        let anim_dim = crate::app::DimConfig::animated_dim(dim_config);
        let all_dims: Vec<usize> = (0..rank).collect();
        let non_anim: Vec<usize> = (0..rank).filter(|&d| Some(d) != anim_dim).collect();

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

        let explicit_x = find_explicit_spatial(crate::app::SpatialRole::X);
        let explicit_y = find_explicit_spatial(crate::app::SpatialRole::Y);
        let explicit_z = find_explicit_spatial(crate::app::SpatialRole::Z);

        let explicit_spatial: Vec<usize> = crate::app::DimConfig::spatial_dims(dim_config)
            .into_iter()
            .filter(|&d| d < rank)
            .collect();

        let x_dim = explicit_x
            .or_else(|| explicit_spatial.first().copied())
            .unwrap_or_else(|| non_anim.last().copied().unwrap_or(0));

        let y_dim = explicit_y
            .or_else(|| explicit_spatial.iter().copied().find(|&d| d != x_dim))
            .unwrap_or_else(|| {
                non_anim
                    .len()
                    .checked_sub(2)
                    .and_then(|i| non_anim.get(i))
                    .copied()
                    .unwrap_or_else(|| all_dims.iter().copied().find(|&d| d != x_dim).unwrap_or(0))
            });

        let z_dim = explicit_z
            .or_else(|| {
                explicit_spatial
                    .iter()
                    .copied()
                    .find(|&d| d != x_dim && d != y_dim)
            })
            .unwrap_or_else(|| {
                all_dims
                    .iter()
                    .copied()
                    .find(|&d| d != x_dim && d != y_dim)
                    .unwrap_or(usize::MAX)
            });

        (x_dim, y_dim, z_dim)
    }

    /// Projects a resident block into current 2D and 3D views.
    pub fn apply_block_projection(&mut self, block: &crate::data::octant_block::OctantBlock) {
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

        if let Some(mdata) = block.slice_2d_with_ranges(
            x_dim,
            y_dim,
            x_range,
            y_range,
            &fixed_indices,
            self.animated_dim_extent(),
            &format!("Block Cache [{}]", block.variable_name),
            compute_bounds,
        ) {
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
                vec![]
            } else {
                fixed_indices.clone()
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

    /// Drains completed block-cache prefetch results.
    pub fn poll_block_prefetch_results(&mut self) {
        let completed = self.block_prefetcher.poll();

        for res in completed {
            match res.result {
                Ok(block) => {
                    let is_active = self.active_block_key.as_ref() == Some(&res.key);
                    let covers_current = self.plotted_animated_dim.is_some_and(|dim| {
                        let origin = block.origin.get(dim).copied().unwrap_or(0);
                        let extent = block.shape.get(dim).copied().unwrap_or(0);
                        self.current_timestep >= origin && self.current_timestep < origin + extent
                    });
                    self.block_cache.put(res.key, block.clone());
                    if is_active || covers_current {
                        if is_active
                            && let Some(target) = self.pending_target_step.take()
                            && let Some(dim) = self.plotted_animated_dim
                        {
                            let origin = block.origin.get(dim).copied().unwrap_or(0);
                            let extent = block.shape.get(dim).copied().unwrap_or(0);
                            if target >= origin && target < origin + extent {
                                self.current_timestep = target;
                                if dim < self.plotted_selected_dim_indices.len() {
                                    self.plotted_selected_dim_indices[dim] = target;
                                }
                            }
                        }
                        self.status_message =
                            format!("⚡ [block cache] Loaded '{}'", block.variable_name);
                        self.apply_block_projection(&block);

                        if let Some(meta) = &self.plotted_dataset_metadata
                            && let Some(var) = meta.variables.get(self.plotted_variable_idx)
                        {
                            let shape = var.shape.clone();
                            self.prefetch_selected_animated_range(&shape);
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("Block cache fetch error: {e}");
                }
            }
        }
    }

    /// Aborts all ongoing data transfers and prefetch worker threads.
    pub fn abort_current_fetch(&mut self) {
        self.block_prefetcher.abort();
        self.is_playing = false;
        self.pending_target_step = None;
        self.status_message = "⏹ Data fetch aborted by user.".to_string();
    }
}
