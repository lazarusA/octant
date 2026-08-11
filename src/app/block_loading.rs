//! Opt-in loading path through `DatasetManager`/`BlockCache`/`BlockPrefetcher`.
//!
//! This exists side-by-side with `data_loading.rs`'s
//! `load_selected_variable_slice`, which is unchanged and remains the
//! default. Nothing here runs unless `OctantApp::use_block_cache` is `true`
//! and something calls `load_selected_variable_block`.

use crate::data::{
    BlockRequest, DataSource, DataSourceKind, Dataset, DimensionSelection, SliceRequest,
    SourceFactory,
};

use super::OctantApp;
use super::state::StoreKind;

impl OctantApp {
    /// Aligned `[start, end)` window along the animated dimension that
    /// contains `self.current_timestep`, clamped to the dataset's actual
    /// extent.
    /// Aligned `[start, end)` window along the animated dimension that
    /// contains `self.current_timestep`, clamped to the dataset's actual
    /// extent.
    ///
    /// By default, prefetches in multiples of the variable's chunk size
    /// along the animated dimension.
    fn animated_window(&self, full_extent: usize, chunk_size: usize) -> (usize, usize) {
        let cs = if chunk_size == 0 { 1 } else { chunk_size };
        let multiplier = (self.block_window_size / cs).max(1);
        let window = cs * multiplier;
        let start = (self.current_timestep / window) * window;
        let end = (start + window).min(full_extent).max(start + 1);
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

    /// Block-cache equivalent of `load_selected_variable_slice`.
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
            let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
            if full_extent > 0 && self.current_timestep >= full_extent {
                self.current_timestep = full_extent - 1;
            }
            if anim_dim < self.plotted_selected_dim_indices.len() {
                self.plotted_selected_dim_indices[anim_dim] = self.current_timestep;
            }
            if anim_dim < selections.len() {
                let (user_start, user_end) = self
                    .plotted_selected_dim_ranges
                    .get(anim_dim)
                    .copied()
                    .unwrap_or((0, full_extent.saturating_sub(1)));

                if user_end > user_start {
                    selections[anim_dim] = DimensionSelection::Range {
                        start: user_start,
                        end: (user_end + 1).min(full_extent),
                    };
                } else {
                    let (start, end) = self.animated_window(full_extent, anim_chunk_size);
                    selections[anim_dim] = DimensionSelection::Range { start, end };
                }
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

    /// Prefetches the block window containing `timestep` asynchronously using `plotted_store_handle()`,
    /// without modifying current UI, `matrix_data`, or `current_timestep` state.
    pub fn prefetch_block_window_for_timestep(&mut self, timestep: usize) {
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
        if timestep >= full_extent {
            return;
        }

        if self
            .block_cache
            .covers(&source_id, &var_name, Some(anim_dim), timestep)
        {
            return;
        }

        let anim_chunk_size = var_info.chunk_shape.get(anim_dim).copied().unwrap_or(1) as usize;
        let cs = if anim_chunk_size == 0 {
            1
        } else {
            anim_chunk_size
        };
        let multiplier = (self.block_window_size / cs).max(1);
        let window = cs * multiplier;

        let start = (timestep / window) * window;
        let end = (start + window).min(full_extent).max(start + 1);

        let legacy_request =
            crate::ui::variables_panel::build_slice_request_for_plotted(self, &var_name, &shape);
        let mut selections = legacy_request.selections;
        if anim_dim < selections.len() {
            selections[anim_dim] = DimensionSelection::Range { start, end };
        }

        let slice_request = SliceRequest::new(&var_name, selections);

        let block_request = BlockRequest::new(store_handle, slice_request);
        self.active_block_key = Some(block_request.cache_key());
        self.block_prefetcher
            .request(block_request, &self.block_cache);
    }

    /// Prefetches upcoming animation windows in the background to ensure buffer is warm ahead of playback.
    fn maybe_prefetch_next_window(&mut self, shape: &[u64]) {
        if !self.is_playing {
            return;
        }
        let Some(anim_dim) = self.plotted_animated_dim else {
            return;
        };
        let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
        let anim_chunk_size = self
            .plotted_dataset_metadata
            .as_ref()
            .and_then(|meta| meta.variables.get(self.plotted_variable_idx))
            .and_then(|var| var.chunk_shape.get(anim_dim))
            .copied()
            .unwrap_or(1) as usize;
        let (_start, end) = self.animated_window(full_extent, anim_chunk_size);

        let cs = if anim_chunk_size == 0 {
            1
        } else {
            anim_chunk_size
        };
        let window = cs * (self.block_window_size / cs).max(1);
        let lookahead_windows = 4; // Look ahead up to 4 windows in parallel

        let Some(next_legacy_request) = self.active_slice_request.clone() else {
            return;
        };
        if anim_dim >= next_legacy_request.selections.len() {
            return;
        }

        let source_id = format!(
            "{:?}:{}",
            self.plotted_store_kind, self.plotted_store_target_input
        );

        let Some(store_handle) = self.get_or_open_plotted_store() else {
            return;
        };

        for i in 0..lookahead_windows {
            let ns = end + i * window;
            if ns >= full_extent {
                if self.loop_playback && full_extent > 0 {
                    let wrap_start = (i * window) % full_extent;
                    let wrap_end = (wrap_start + window).min(full_extent);
                    if self.block_cache.covers(
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

            if self.block_cache.covers(
                &source_id,
                &next_legacy_request.variable,
                self.plotted_animated_dim,
                ns,
            ) {
                continue; // Already resident in memory! Skip prefetching!
            }

            let ne = (ns + window).min(full_extent);
            let mut selections = next_legacy_request.selections.clone();
            selections[anim_dim] = DimensionSelection::Range { start: ns, end: ne };

            let req = BlockRequest::new(
                store_handle.clone(),
                SliceRequest::new(next_legacy_request.variable.clone(), selections),
            );
            if !self.block_cache.contains(&req.cache_key())
                && !self.block_prefetcher.request(req, &self.block_cache)
            {
                break; // Stop if prefetch thread pool is full
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

        let explicit_spatial: Vec<usize> = self
            .plotted_spatial_dims
            .iter()
            .copied()
            .filter(|&d| d < block.rank())
            .collect();

        let x_dim = explicit_spatial
            .first()
            .copied()
            .unwrap_or_else(|| non_anim.last().copied().unwrap_or(0));

        let y_dim = explicit_spatial.get(1).copied().unwrap_or_else(|| {
            non_anim
                .len()
                .checked_sub(2)
                .and_then(|i| non_anim.get(i))
                .copied()
                .unwrap_or_else(|| all_dims.iter().copied().find(|&d| d != x_dim).unwrap_or(0))
        });

        let z_dim = explicit_spatial.get(2).copied().unwrap_or_else(|| {
            all_dims
                .iter()
                .copied()
                .find(|&d| d != x_dim && d != y_dim)
                .unwrap_or(usize::MAX)
        });

        let fixed_indices: Vec<usize> = self
            .plotted_selected_dim_indices
            .iter()
            .enumerate()
            .map(|(i, &idx)| idx.saturating_sub(block.origin.get(i).copied().unwrap_or(0)))
            .collect();

        if let Some(mdata) = block.slice_2d(
            x_dim,
            y_dim,
            &fixed_indices,
            self.animated_dim_extent(),
            &format!("Block Cache [{}]", block.variable_name),
        ) {
            self.rebuild_pipeline_with_matrix_data(mdata);
        }

        let is_3d_plot = self.active_plot_type == crate::plots::PlotType::Volume
            || self.active_plot_type == crate::plots::PlotType::PointCloud;

        if !is_3d_plot {
            return;
        }

        if let Some(vdata) = block.volume(
            x_dim,
            y_dim,
            z_dim,
            &fixed_indices,
            &format!("Block Cache Volume [{}]", block.variable_name),
        ) {
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
}
