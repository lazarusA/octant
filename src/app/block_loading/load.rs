//! Primary variable block loading, prefetch polling, and fetch abort controls.

use crate::app::OctantApp;
use crate::data::{BlockRequest, DimensionSelection, SliceRequest};

impl OctantApp {
    /// Plots the current slider selection (the Plot button): aborts pending
    /// fetches, moves the step into the selected range of the animated axis so
    /// a new selection never shows a step it excludes, and loads. Playback
    /// loads with `load_selected_variable_block` and may run past that range.
    pub fn plot_selection(&mut self) {
        self.block_prefetcher.abort();
        if let Some(anim_dim) = self.animated_dim
            && let Some(&(start, end)) = self.selected_dim_ranges.get(anim_dim)
        {
            self.current_timestep = self.current_timestep.clamp(start, end.max(start));
        }
        self.load_selected_variable_block();
    }

    /// Loads the block corresponding to the current animated step and selections.
    pub fn load_selected_variable_block(&mut self) {
        let Some(metadata) = &self.active_dataset_metadata else {
            self.status_message = "No dataset metadata loaded.".to_string();
            return;
        };
        let Some(var_info) = metadata.variables.get(self.selected_variable_idx) else {
            self.status_message = "Invalid selected variable index.".to_string();
            return;
        };

        let var_name = var_info.name.clone();
        let shape = var_info.shape.clone();

        let base_request = crate::ui::variables_panel::build_slice_request(self, &var_name, &shape);
        let mut selections = base_request.selections;

        let anim_dim = self.plotted_animated_dim.or(self.animated_dim);
        if let Some(anim_dim) = anim_dim {
            let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
            if full_extent > 0 && self.current_timestep >= full_extent {
                self.current_timestep = full_extent - 1;
            }
            if anim_dim < self.selected_dim_indices.len() {
                self.selected_dim_indices[anim_dim] = self.current_timestep;
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

        let source_id = self.selected_source_id();
        let store_handle = self.selected_store_handle();
        let block_key = store_handle
            .as_ref()
            .map(|h| BlockRequest::new(h.clone(), slice_request.clone()).cache_key());
        self.active_block_key = block_key;

        // 1. Cache HIT: Check if any resident block in memory (e.g. full dataset array) covers current_timestep
        if let Some(block) = self.block_cache.find_covering_block(
            &source_id,
            &var_name,
            &slice_request.selections,
            anim_dim,
            self.current_timestep,
        ) {
            self.status_message = format!(
                "Block cache HIT for '{}' ({} bytes resident)",
                block.variable_name,
                block.bytes_size()
            );
            self.sync_plotted_state_from_selected();
            self.apply_block_projection(&block);
            self.project_cached_volume_blocks(
                &source_id,
                &var_name,
                &slice_request.selections,
                anim_dim,
            );
            self.prefetch_selected_animated_range(&shape);
            return;
        }

        let Some(store_handle) = store_handle else {
            self.status_message =
                format!("Dataset store not open in DatasetManager for '{source_id}'");
            return;
        };

        let selections = slice_request.selections.clone();
        let block_request = BlockRequest::new(store_handle, slice_request);
        let key = block_request.cache_key();

        // 2. Exact Key Cache HIT
        if let Some(block) = self.block_cache.get(&key) {
            self.status_message = format!(
                "Block cache HIT for '{}' ({} bytes resident)",
                block.variable_name,
                block.bytes_size()
            );
            self.sync_plotted_state_from_selected();
            self.apply_block_projection(&block);
            self.project_cached_volume_blocks(&source_id, &var_name, &selections, anim_dim);
            self.prefetch_selected_animated_range(&shape);
            return;
        }

        // 3. Cache MISS: dispatch async prefetch request for current chunk and launch background prefetching in parallel.
        self.status_message = format!("[block cache] Downloading window for '{}'...", var_name);
        self.block_prefetcher
            .request(block_request, &self.block_cache);
    }

    /// Drains completed block-cache prefetch results.
    pub fn poll_block_prefetch_results(&mut self) {
        let completed = self.block_prefetcher.poll();
        let had_completed = !completed.is_empty();

        for res in completed {
            match res.result {
                Ok(block) => {
                    let is_active = self.active_block_key.as_ref() == Some(&res.key);
                    let is_same_var = self
                        .plotted_variable_info()
                        .or_else(|| self.selected_variable_info())
                        .is_some_and(|v| v.name == block.variable_name);
                    let anim_dim = self.plotted_animated_dim.or(self.animated_dim);
                    let covers_current = is_same_var
                        && anim_dim.is_some_and(|dim| {
                            let origin = block.origin.get(dim).copied().unwrap_or(0);
                            let extent = block.shape.get(dim).copied().unwrap_or(0);
                            self.current_timestep >= origin
                                && self.current_timestep < origin + extent
                        });
                    let is_volume_or_point_cloud = is_same_var
                        && (self.active_plot_type == crate::plots::PlotType::Volume
                            || self.active_plot_type == crate::plots::PlotType::PointCloud);

                    self.block_cache.put(res.key, block.clone());

                    // When cache eviction shifts the oldest resident slice forward, update slider start:
                    if let Some(dim) = anim_dim
                        && let Some(meta) = self
                            .plotted_dataset_metadata
                            .as_ref()
                            .or(self.active_dataset_metadata.as_ref())
                        && let Some(var) = meta
                            .variables
                            .get(self.plotted_variable_idx)
                            .or_else(|| meta.variables.get(self.selected_variable_idx))
                        && var.name == block.variable_name
                    {
                        let source_id = self.plotted_source_id();
                        if let Some(min_t) = self
                            .block_cache
                            .min_resident_timestep(&source_id, &var.name, dim)
                        {
                            let sel_ranges = if !self.plotted_selected_dim_ranges.is_empty() {
                                &mut self.plotted_selected_dim_ranges
                            } else {
                                &mut self.selected_dim_ranges
                            };
                            if dim < sel_ranges.len() {
                                let current_start = sel_ranges[dim].0;
                                if min_t > current_start
                                    && self.block_cache.current_bytes()
                                        >= self.block_cache.max_bytes()
                                {
                                    sel_ranges[dim].0 = min_t;
                                }
                            }
                        }
                    }

                    if is_active || (is_same_var && (covers_current || is_volume_or_point_cloud)) {
                        if is_active {
                            self.active_block_key = None;
                            self.sync_plotted_state_from_selected();
                            if let Some(target) = self.pending_target_step.take()
                                && let Some(dim) = anim_dim
                            {
                                let origin = block.origin.get(dim).copied().unwrap_or(0);
                                let extent = block.shape.get(dim).copied().unwrap_or(0);
                                if target >= origin && target < origin + extent {
                                    self.current_timestep = target;
                                    let sel_indices =
                                        if !self.plotted_selected_dim_indices.is_empty() {
                                            &mut self.plotted_selected_dim_indices
                                        } else {
                                            &mut self.selected_dim_indices
                                        };
                                    if dim < sel_indices.len() {
                                        sel_indices[dim] = target;
                                    }
                                }
                            }
                        }
                        self.status_message =
                            format!("[block cache] Loaded '{}'", block.variable_name);
                        self.apply_block_projection(&block);
                    }
                }
                Err(e) => {
                    self.status_message = format!("Block cache fetch error: {e}");
                }
            }
        }

        // Whenever worker slots free up, replenish and dispatch remaining chunks in the selection:
        if had_completed
            && let Some(meta) = &self.plotted_dataset_metadata
            && let Some(var) = meta.variables.get(self.plotted_variable_idx)
        {
            let shape = var.shape.clone();
            self.prefetch_selected_animated_range(&shape);
        }
    }

    /// Aborts all ongoing data transfers and prefetch worker threads.
    pub fn abort_current_fetch(&mut self) {
        self.block_prefetcher.abort();
        self.is_playing = false;
        self.pending_target_step = None;
        self.status_message = "Data fetch aborted by user.".to_string();
    }
}
