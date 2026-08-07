//! Opt-in loading path through `BlockLruCache`/`BlockPrefetcher`.
//!
//! This exists side-by-side with `data_loading.rs`'s
//! `load_selected_variable_slice`, which is unchanged and remains the
//! default. Nothing here runs unless `OctantApp::use_block_cache` is `true`
//! and something calls `load_selected_variable_block` — call it from the
//! same places `load_selected_variable_slice` is called today (e.g. behind
//! a debug toggle in the settings panel) to A/B the two paths.
//!
//! Currently only `StoreKind::RemoteZarr` has a working storage backend
//! (see `crate::cache::storage::build_storage_for`); other store kinds
//! report a clear "not yet implemented" status message instead of silently
//! falling back to the legacy path.

use crate::cache::block_cache::BlockCacheKey;
use crate::cache::storage::build_storage_for;
use crate::ui::variables_panel::build_slice_request;

use super::OctantApp;
use super::state::StoreKind;

impl OctantApp {
    /// Block-cache equivalent of `load_selected_variable_slice`.
    ///
    /// On a cache hit, immediately projects the resident `OctantBlock` into
    /// a `MatrixSlice` via `OctantBlock::matrix_slice` and rebuilds the GPU
    /// pipeline — no I/O. On a miss, dispatches an async `fetch_block` and
    /// returns; the result is picked up by the poll loop (see
    /// `app/ui.rs`'s per-frame update once wired in).
    pub fn load_selected_variable_block(&mut self) {
        if !self.use_block_cache {
            return;
        }

        self.show_settings_panel = true;

        let Some(metadata) = &self.active_dataset_metadata else {
            self.status_message = "No dataset metadata loaded.".to_string();
            return;
        };
        let Some(var_info) = metadata.variables.get(self.selected_variable_idx) else {
            self.status_message = "Invalid variable index.".to_string();
            return;
        };

        let var_name = var_info.name.clone();
        let shape = var_info.shape.clone();

        // Reuse the same hyperslab-from-UI-state builder the legacy path
        // uses — DimensionSelection/SliceRequest are already the shared
        // vocabulary between both paths.
        let request = build_slice_request(self, &var_name, &shape);

        let key = BlockCacheKey::from_request(
            self.selected_store_kind,
            self.store_target_input.clone(),
            &request,
        );
        self.active_block_key = Some(key.clone());
        self.active_slice_request = Some(request.clone());

        // 1. Cache HIT: project in memory, no I/O.
        if let Some(block) = self.block_cache.get(&key) {
            self.apply_block_projection(&block);
            self.status_message = format!(
                "🚀 Block cache HIT for '{}' ({} bytes resident)",
                block.variable_name,
                block.bytes_size()
            );
            return;
        }

        if self.selected_store_kind == StoreKind::ProceduralRandom {
            self.status_message =
                "Block cache: ProceduralRandom is not backed by a store; use the legacy path for it.".to_string();
            return;
        }

        // 2. Cache MISS: build the storage handle and dispatch async fetch.
        let storage = match build_storage_for(self.selected_store_kind, &self.store_target_input) {
            Ok(s) => s,
            Err(e) => {
                self.status_message = format!("Block cache: {e}");
                return;
            }
        };

        self.status_message = format!("⏳ [block cache] Downloading '{}'...", var_name);
        self.block_prefetcher
            .request_block(key, request, storage, &self.block_cache);
    }

    /// Projects a resident block into the current 2D view (x/y from
    /// `spatial_dims`, everything else fixed at `selected_dim_indices`) and
    /// pushes it through the existing GPU pipeline rebuild.
    ///
    /// Falls back to the block's first two dimensions as x/y if the user
    /// hasn't assigned spatial roles yet, so this degrades gracefully rather
    /// than silently doing nothing.
    fn apply_block_projection(&mut self, block: &crate::data::octant_block::OctantBlock) {
        let (x_dim, y_dim) = match (self.spatial_dims.first(), self.spatial_dims.get(1)) {
            (Some(&x), Some(&y)) => (x, y),
            _ if block.rank() >= 2 => (block.rank() - 1, block.rank() - 2),
            _ => {
                self.status_message =
                    "Block cache: variable has fewer than 2 dimensions to project.".to_string();
                return;
            }
        };

        let fixed_indices: Vec<usize> = self
            .selected_dim_indices
            .iter()
            .enumerate()
            .map(|(i, &idx)| {
                // Indices in selected_dim_indices are global; the block may
                // be windowed, so offset by its origin along that dimension.
                idx.saturating_sub(block.origin.get(i).copied().unwrap_or(0))
            })
            .collect();

        let Some(matrix_slice) = block.matrix_slice(
            x_dim,
            y_dim,
            &fixed_indices,
            self.current_timestep,
            block.shape.first().copied().unwrap_or(1),
            &format!("Block Cache [{}]", block.variable_name),
        ) else {
            self.status_message =
                "Block cache: projection failed (dimension mismatch).".to_string();
            return;
        };

        let mdata = crate::data::matrix_data::MatrixData::new(
            matrix_slice.width,
            matrix_slice.height,
            matrix_slice.values,
            matrix_slice.min_val,
            matrix_slice.max_val,
            matrix_slice.dataset_name,
            matrix_slice.max_timesteps,
        );
        self.rebuild_pipeline_with_matrix_data(mdata);
    }

    /// Drains completed block-cache prefetch results. Call this from the
    /// same per-frame spot `ui.rs` drains `self.prefetcher.poll_results()`
    /// once you're ready to test this path live.
    pub fn poll_block_prefetch_results(&mut self) {
        if !self.use_block_cache {
            return;
        }

        let completed = self.block_prefetcher.poll_results();
        for res in completed {
            match res.result {
                Ok(block) => {
                    let is_active = self.active_block_key.as_ref() == Some(&res.key);
                    self.block_cache.put(res.key, block.clone());
                    if is_active {
                        self.apply_block_projection(&block);
                        self.status_message =
                            format!("⚡ [block cache] Loaded '{}'", block.variable_name);
                    }
                }
                Err(e) => {
                    self.status_message = format!("Block cache fetch error: {e}");
                }
            }
        }
    }
}
