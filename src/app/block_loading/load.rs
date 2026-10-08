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
        if let Some(anim_dim) = self.selected.animated_dim
            && let Some(&(start, end)) = self.selected.dim_ranges.get(anim_dim)
        {
            self.current_timestep = self.current_timestep.clamp(start, end.max(start));
        }
        self.load_selected_variable_block();
    }

    /// Loads the block corresponding to the current animated step and selections.
    pub fn load_selected_variable_block(&mut self) {
        // A plotted variable needs its coordinates for axes, sliders and the hover.
        self.request_variable_coordinates(self.selected.variable_idx);
        let Some(metadata) = &self.selected.metadata else {
            self.status_message = "No dataset metadata loaded.".to_string();
            return;
        };
        let Some(var_info) = metadata.variables.get(self.selected.variable_idx) else {
            self.status_message = "Invalid selected variable index.".to_string();
            return;
        };

        let var_name = var_info.name.clone();
        let shape = var_info.shape.clone();

        let base_request = crate::ui::variables_panel::build_slice_request(self, &var_name, &shape);
        let mut selections = base_request.selections;

        let anim_dim = self.plotted().animated_dim.or(self.selected.animated_dim);
        if let Some(anim_dim) = anim_dim {
            let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
            if full_extent > 0 && self.current_timestep >= full_extent {
                self.current_timestep = full_extent - 1;
            }
            if anim_dim < self.selected.dim_indices.len() {
                self.selected.dim_indices[anim_dim] = self.current_timestep;
            }
            if anim_dim < self.plotted().dim_indices.len() {
                self.layers.base.selection_mut().dim_indices[anim_dim] = self.current_timestep;
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
        self.layers.base.load.slice_request = Some(slice_request.clone());

        let source_id = self.selected_source_id();
        let store_handle = self.selected_store_handle();
        let block_key = store_handle
            .as_ref()
            .map(|h| BlockRequest::new(h.clone(), slice_request.clone()).cache_key());
        self.layers.base.load.block_key = block_key;

        // 1. Cache HIT: Check if any resident block in memory (e.g. full dataset array) covers current_timestep
        if let Some(block) = self.block_cache.find_covering_block(
            &source_id,
            &var_name,
            &slice_request.selections,
            anim_dim,
            self.current_timestep,
        ) {
            self.show_cached_block(&block, &slice_request.selections, anim_dim, &shape);
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
            self.show_cached_block(&block, &selections, anim_dim, &shape);
            return;
        }

        // 3. Cache MISS: dispatch async prefetch request for current chunk and launch background prefetching in parallel.
        // Playback may move on meanwhile; the block is shown at this step.
        self.layers.base.load.pending_target_step = Some(self.current_timestep);
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
                    let is_active = self.layers.find_by_key(&res.key).is_some();
                    let is_same_var = self
                        .plotted_variable_info()
                        .or_else(|| self.selected_variable_info())
                        .is_some_and(|v| v.name == block.variable_name);
                    let anim_dim = self.plotted().animated_dim.or(self.selected.animated_dim);
                    let covers_current = is_same_var
                        && anim_dim.is_some_and(|dim| {
                            let origin = block.origin.get(dim).copied().unwrap_or(0);
                            let extent = block.shape.get(dim).copied().unwrap_or(0);
                            self.current_timestep >= origin
                                && self.current_timestep < origin + extent
                        });
                    // Only blocks of the requested view: not of an older
                    // selection, variable or plot layout.
                    let is_same_var = is_same_var && self.key_matches_view(&res.key, anim_dim);
                    let is_volume_or_point_cloud = is_same_var
                        && (self.selected.plot_type == crate::plots::PlotType::Volume
                            || self.selected.plot_type == crate::plots::PlotType::PointCloud);

                    self.block_cache.put(res.key, block.clone());

                    if is_active || (is_same_var && (covers_current || is_volume_or_point_cloud)) {
                        if is_active {
                            self.layers.base.load.block_key = None;
                            self.sync_plotted_state_from_selected();
                            if let Some(target) = self.layers.base.load.pending_target_step.take()
                                && let Some(dim) = self.plotted().animated_dim
                            {
                                let origin = block.origin.get(dim).copied().unwrap_or(0);
                                let extent = block.shape.get(dim).copied().unwrap_or(0);
                                if target >= origin && target < origin + extent {
                                    self.current_timestep = target;
                                    let sel_indices = if !self.plotted().dim_indices.is_empty() {
                                        &mut self.layers.base.selection_mut().dim_indices
                                    } else {
                                        &mut self.selected.dim_indices
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
                    self.notify(crate::ui::toast::Severity::Error, "Couldn't load data", &e);
                }
            }
        }

        // Whenever worker slots free up, replenish and dispatch remaining chunks in the selection:
        if had_completed
            && let Some(meta) = &self.plotted().metadata
            && let Some(var) = meta.variables.get(self.plotted().variable_idx)
        {
            let shape = var.shape.clone();
            self.prefetch_selected_animated_range(&shape);
        }
    }

    /// Aborts all ongoing data transfers and prefetch worker threads.
    pub fn abort_current_fetch(&mut self) {
        self.block_prefetcher.abort();
        self.is_playing = false;
        self.layers.base.load.pending_target_step = None;
        self.status_message = "Data fetch aborted by user.".to_string();
    }
}
