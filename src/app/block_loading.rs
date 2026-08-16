//! Loading path through `DatasetManager`/`BlockCache`/`BlockPrefetcher`.

use crate::data::{BlockRequest, DimensionSelection, SliceRequest};

use super::OctantApp;

impl OctantApp {
    /// Aligned `[start, end)` window along the animated dimension that
    /// contains `self.current_timestep`, clamped to the dataset's actual
    /// extent and `max_steps` limit.
    ///
    /// By default, prefetches in multiples of the variable's chunk size
    /// along the animated dimension.
    fn animated_window(
        &self,
        full_extent: usize,
        chunk_size: usize,
        max_steps: usize,
    ) -> (usize, usize) {
        let cs = if chunk_size == 0 { 1 } else { chunk_size };
        let start = (self.current_timestep / cs) * cs;
        let end = (start + cs).min(full_extent).min(max_steps).max(start + 1);
        (start, end)
    }

    /// Returns the open `StoreHandle` for the currently plotted dataset from `dataset_manager`.
    pub fn plotted_store_handle(&self) -> Option<crate::data::StoreHandle> {
        let source_id = format!(
            "{:?}:{}",
            self.plotted_store_kind, self.plotted_store_target_input
        );
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

        let legacy_request =
            crate::ui::variables_panel::build_slice_request_for_plotted(self, &var_name, &shape);
        let mut selections = legacy_request.selections.clone();

        let anim_chunk_size = self
            .plotted_animated_dim
            .and_then(|dim| var_info.chunk_shape.get(dim))
            .copied()
            .unwrap_or(1) as usize;

        if let Some(anim_dim) = self.plotted_animated_dim {
            let (max_allowed_steps, _, _) =
                crate::ui::variables_panel::calculate_max_animated_steps(
                    var_info,
                    &self.plotted_dim_config,
                    &self.plotted_selected_dim_ranges,
                    anim_dim,
                );

            let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
            if full_extent > 0 && self.current_timestep >= full_extent {
                self.current_timestep = full_extent - 1;
            }
            if anim_dim < self.plotted_selected_dim_indices.len() {
                self.plotted_selected_dim_indices[anim_dim] = self.current_timestep;
            }
            if anim_dim < selections.len() {
                let (start, end) =
                    self.animated_window(full_extent, anim_chunk_size, max_allowed_steps);
                selections[anim_dim] = DimensionSelection::Range { start, end };
            }
        }

        let slice_request = SliceRequest::new(&var_name, selections);
        self.active_slice_request = Some(slice_request.clone());

        let source_id = format!(
            "{:?}:{}",
            self.plotted_store_kind, self.plotted_store_target_input
        );

        // 1. Cache HIT: Check if any resident block in memory (e.g. full dataset array) covers current_timestep
        if let Some(block) = self.block_cache.find_covering_block(
            &source_id,
            &var_name,
            self.plotted_animated_dim,
            self.current_timestep,
        ) {
            self.status_message = format!(
                "🚀 Block cache HIT for '{}' ({} bytes resident)",
                block.variable_name,
                block.bytes_size()
            );
            self.apply_block_projection(&block);
            self.maybe_prefetch_next_window(&shape);
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
            self.maybe_prefetch_next_window(&shape);
            return;
        }

        // 3. Cache MISS: dispatch async prefetch request.
        self.status_message = format!("⏳ [block cache] Downloading window for '{}'...", var_name);
        self.block_prefetcher
            .request(block_request, &self.block_cache);
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

        let source_id = format!(
            "{:?}:{}",
            self.plotted_store_kind, self.plotted_store_target_input
        );

        let Some(anim_dim) = self.plotted_animated_dim else {
            return;
        };
        let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
        if step >= full_extent {
            return;
        }

        if self
            .block_cache
            .covers(&source_id, &var_name, Some(anim_dim), step)
        {
            return;
        }

        let anim_chunk_size = var_info.chunk_shape.get(anim_dim).copied().unwrap_or(1) as usize;
        let cs = if anim_chunk_size == 0 {
            1
        } else {
            anim_chunk_size
        };
        let start = (step / cs) * cs;
        let end = (start + cs).min(full_extent).max(start + 1);

        let legacy_request =
            crate::ui::variables_panel::build_slice_request_for_plotted(self, &var_name, &shape);
        let mut selections = legacy_request.selections;
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
        let source_id = format!(
            "{:?}:{}",
            self.plotted_store_kind, self.plotted_store_target_input
        );
        let var_name = self
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|m| m.variables.get(self.plotted_variable_idx))
            .map(|v| v.name.clone());

        let is_cached = if let Some(ref name) = var_name {
            self.block_cache
                .covers(&source_id, name, self.plotted_animated_dim, target_step)
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
        let target_step = if self.current_timestep > 0 {
            self.current_timestep - 1
        } else if max_steps > 0 {
            max_steps - 1
        } else {
            0
        };
        self.request_step_or_load(target_step);
    }

    /// Requests the next step along the animated dimension.
    pub fn step_next(&mut self) {
        let max_steps = self.animated_dim_extent();
        let target_step = if max_steps > 0 {
            (self.current_timestep + 1) % max_steps
        } else {
            0
        };
        self.request_step_or_load(target_step);
    }

    /// Prefetches upcoming animation windows in the background to ensure buffer is warm ahead of playback.
    fn maybe_prefetch_next_window(&mut self, shape: &[u64]) {
        // 1. Only prefetch lookahead windows if playback is actively playing.
        if !self.is_playing {
            return;
        }

        // 2. Ensure an animated dimension is active.
        let Some(anim_dim) = self.plotted_animated_dim else {
            return;
        };

        // 3. Extract dimension extent and chunk size to calculate window bounds.
        let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
        let anim_chunk_size = self
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|meta| meta.variables.get(self.plotted_variable_idx))
            .and_then(|var| var.chunk_shape.get(anim_dim))
            .copied()
            .unwrap_or(1) as usize;

        let cs = if anim_chunk_size == 0 {
            1
        } else {
            anim_chunk_size
        };
        let current_chunk_idx = self.current_timestep / cs;
        let lookahead_chunks = (self.block_window_size / cs).clamp(2, 6);

        // 4. Retrieve current active slice request to copy non-animated dimension selections.
        let Some(next_legacy_request) = self.active_slice_request.clone() else {
            return;
        };
        if anim_dim >= next_legacy_request.selections.len() {
            return;
        }

        // 5. Get current dataset source ID and open store handle.
        let source_id = format!(
            "{:?}:{}",
            self.plotted_store_kind, self.plotted_store_target_input
        );

        let Some(store_handle) = self.plotted_store_handle() else {
            return;
        };

        // 6. Loop over upcoming lookahead chunks and schedule background requests.
        for i in 1..=lookahead_chunks {
            let chunk_idx = current_chunk_idx + i;
            let chunk_start = chunk_idx * cs;

            // Handle wrap-around prefetching when loop playback is enabled.
            if chunk_start >= full_extent {
                if self.loop_playback && full_extent > 0 {
                    let total_chunks = full_extent.div_ceil(cs);
                    let wrap_chunk_idx = (current_chunk_idx + i) % total_chunks;
                    let wrap_start = wrap_chunk_idx * cs;
                    let wrap_end = (wrap_start + cs).min(full_extent);
                    if self.block_cache.covers(
                        &source_id,
                        &next_legacy_request.variable,
                        self.plotted_animated_dim,
                        wrap_start,
                    ) || self.block_prefetcher.is_pending_timestep(
                        &source_id,
                        &next_legacy_request.variable,
                        self.plotted_animated_dim,
                        wrap_start,
                    ) {
                        continue;
                    }
                    let mut selections = next_legacy_request.selections.clone();
                    selections[anim_dim] = DimensionSelection::Range {
                        start: wrap_start,
                        end: wrap_end,
                    };
                    let req = BlockRequest::new(
                        store_handle.clone(),
                        SliceRequest::new(next_legacy_request.variable.clone(), selections),
                    );
                    if !self.block_cache.contains(&req.cache_key()) {
                        self.block_prefetcher.request(req, &self.block_cache);
                    }
                }
                break;
            }

            let chunk_end = (chunk_start + cs).min(full_extent);
            if self.block_cache.covers(
                &source_id,
                &next_legacy_request.variable,
                self.plotted_animated_dim,
                chunk_start,
            ) || self.block_prefetcher.is_pending_timestep(
                &source_id,
                &next_legacy_request.variable,
                self.plotted_animated_dim,
                chunk_start,
            ) {
                continue;
            }

            let mut selections = next_legacy_request.selections.clone();
            selections[anim_dim] = DimensionSelection::Range {
                start: chunk_start,
                end: chunk_end,
            };
            let req = BlockRequest::new(
                store_handle.clone(),
                SliceRequest::new(next_legacy_request.variable.clone(), selections),
            );
            if !self.block_cache.contains(&req.cache_key())
                && !self.block_prefetcher.request(req, &self.block_cache)
            {
                break; // Stop if prefetch thread pool is full.
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

    /// Projects a resident block into current 2D and 3D views.
    fn apply_block_projection(&mut self, block: &crate::data::octant_block::OctantBlock) {
        let anim_dim = self.plotted_animated_dim;

        let all_dims: Vec<usize> = (0..block.rank()).collect();
        let non_anim: Vec<usize> = (0..block.rank()).filter(|&d| Some(d) != anim_dim).collect();

        let orig_dim_names: Vec<String> = self
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|meta| meta.variables.get(self.plotted_variable_idx))
            .map(|v| v.dimension_names.clone())
            .unwrap_or_else(|| block.dimension_names.clone());

        let find_explicit_spatial = |role: crate::app::SpatialRole| -> Option<usize> {
            (0..block.rank()).find(|&d| {
                let Some(name) = block.dimension_names.get(d) else {
                    return false;
                };
                let Some(orig_idx) = orig_dim_names.iter().position(|n| n == name) else {
                    return false;
                };
                self.plotted_dim_config
                    .get(orig_idx)
                    .is_some_and(|c| c.spatial == role)
            })
        };

        let explicit_x = find_explicit_spatial(crate::app::SpatialRole::X);
        let explicit_y = find_explicit_spatial(crate::app::SpatialRole::Y);
        let explicit_z = find_explicit_spatial(crate::app::SpatialRole::Z);

        let explicit_spatial: Vec<usize> = self
            .plotted_spatial_dims
            .iter()
            .copied()
            .filter(|&d| d < block.rank())
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

        let compute_bounds = !self.lock_color_bounds;

        if let Some(mdata) = block.slice_2d(
            x_dim,
            y_dim,
            &fixed_indices,
            self.animated_dim_extent(),
            &format!("Block Cache [{}]", block.variable_name),
            compute_bounds,
        ) {
            self.rebuild_pipeline_with_matrix_data(mdata);
        }

        let is_3d_plot = self.active_plot_type == crate::plots::PlotType::Volume
            || self.active_plot_type == crate::plots::PlotType::PointCloud;

        let is_3d_spatial_anim = anim_dim.is_some_and(|a| a == x_dim || a == y_dim || a == z_dim);

        let current_volume_desc = format!(
            "Block Cache Volume [{}] origin={:?} shape={:?} fixed={:?}",
            block.variable_name,
            block.origin,
            block.shape,
            if is_3d_spatial_anim {
                vec![]
            } else {
                fixed_indices.clone()
            }
        );

        let needs_volume_update = if let Some(existing) = &self.volume_data {
            let nz = if z_dim < block.rank() && z_dim != x_dim && z_dim != y_dim {
                block.shape[z_dim]
            } else {
                1
            };
            let nx = block.shape.get(x_dim).copied().unwrap_or(0);
            let ny = block.shape.get(y_dim).copied().unwrap_or(0);
            let slice_elements = nx.saturating_mul(ny).max(1);
            let max_z = (crate::plots::common::MAX_GPU_STORAGE_BUFFER_ELEMENTS / slice_elements)
                .clamp(1, nz);
            let eff_nz = nz.min(max_z);

            existing.width != nx
                || existing.height != ny
                || existing.depth != eff_nz
                || self.volume_renderer.is_none()
                || existing.dataset_name != current_volume_desc
        } else {
            true
        };

        if is_3d_plot
            && needs_volume_update
            && let Some(vdata) = block.volume(
                x_dim,
                y_dim,
                z_dim,
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
