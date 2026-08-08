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
use crate::utils::zarr::DimensionSelection;

use super::OctantApp;
use super::state::StoreKind;

impl OctantApp {
    /// Aligned `[start, end)` window along the animated dimension that
    /// contains `self.current_timestep`, clamped to the dataset's actual
    /// extent. Windows are aligned to `block_window_size` (not centered on
    /// the current frame) so repeated playback re-hits the same window
    /// boundaries instead of drifting.
    fn animated_window(&self, full_extent: usize) -> (usize, usize) {
        let window = self.block_window_size.max(1);
        let start = (self.current_timestep / window) * window;
        let end = (start + window).min(full_extent).max(start + 1);
        (start, end)
    }

    /// Block-cache equivalent of `load_selected_variable_slice`.
    ///
    /// On a cache hit, immediately projects the resident `OctantBlock` into
    /// a `MatrixSlice` via `OctantBlock::matrix_slice` and rebuilds the GPU
    /// pipeline — no I/O. On a miss, dispatches an async `fetch_block` and
    /// returns; the result is picked up by the poll loop (see
    /// `app/ui.rs`'s per-frame update once wired in).
    ///
    /// The animated dimension (if any) is fetched as a *window* of frames
    /// rather than a single index, so most calls during playback are cache
    /// hits — see `animated_window`.
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
        // uses, then override the animated dimension's selection with a
        // window rather than a single collapsed index (see animated_window).
        let mut request = build_slice_request(self, &var_name, &shape);
        if let Some(anim_dim) = self.animated_dim {
            self.selected_dim_indices[anim_dim] = self.current_timestep;
            if anim_dim < request.selections.len() {
                let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
                let (start, end) = self.animated_window(full_extent);
                request.selections[anim_dim] = DimensionSelection::Range(start..end);
            }
        }

        let key = BlockCacheKey::from_request(
            self.selected_store_kind,
            self.store_target_input.clone(),
            &request,
        );
        self.active_block_key = Some(key.clone());
        self.active_slice_request = Some(request.clone());

        // 1. Cache HIT: project in memory, no I/O.
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

        self.status_message = format!("⏳ [block cache] Downloading window for '{}'...", var_name);
        self.block_prefetcher
            .request_block(key, request, storage, &self.block_cache);
    }

    /// While playing back inside an already-loaded window, kicks off a
    /// background fetch for the *next* window once playback is past its
    /// midpoint — so by the time playback crosses the boundary, the next
    /// window is often already resident instead of causing a visible stall.
    fn maybe_prefetch_next_window(&mut self, shape: &[u64]) {
        if !self.is_playing {
            return;
        }
        let Some(anim_dim) = self.animated_dim else {
            return;
        };
        let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
        let (start, end) = self.animated_window(full_extent);
        if end >= full_extent {
            return; // no next window
        }
        // Only start prefetching once past the midpoint of the current
        // window, so this isn't re-requesting on every single frame.
        if self.current_timestep < start + (end - start) / 2 {
            return;
        }

        let next_start = end;
        let next_end = (next_start + self.block_window_size.max(1)).min(full_extent);

        let Some(mut next_request) = self.active_slice_request.clone() else {
            return;
        };
        if anim_dim >= next_request.selections.len() {
            return;
        }
        next_request.selections[anim_dim] = DimensionSelection::Range(next_start..next_end);

        let next_key = BlockCacheKey::from_request(
            self.selected_store_kind,
            self.store_target_input.clone(),
            &next_request,
        );

        if self.selected_store_kind == StoreKind::ProceduralRandom {
            return;
        }
        let Ok(storage) = build_storage_for(self.selected_store_kind, &self.store_target_input)
        else {
            return;
        };
        self.block_prefetcher
            .request_block(next_key, next_request, storage, &self.block_cache);
    }

    /// Full size of the currently animated dimension in the *dataset*
    /// (from metadata), not the resident block.
    ///
    /// This matters because the resident block only covers one window of
    /// the animated dimension (see `animated_window`) — its own shape along
    /// that axis is the window size, not the dataset's full extent. Using
    /// the block's shape directly would make playback think the animation
    /// only has as many frames as one window.
    fn animated_dim_extent(&self) -> usize {
        let Some(anim_dim) = self.animated_dim else {
            return 1;
        };
        self.active_dataset_metadata
            .as_ref()
            .and_then(|meta| meta.variables.get(self.selected_variable_idx))
            .and_then(|v| v.shape.get(anim_dim))
            .map(|&s| s as usize)
            .unwrap_or(1)
    }

    /// Projects a resident block into the current 2D view (x/y from
    /// `spatial_dims`, everything else fixed at `selected_dim_indices`) and
    /// pushes it through the existing GPU pipeline rebuild.
    ///
    /// Falls back to the block's first two dimensions as x/y if the user
    /// hasn't assigned spatial roles yet, so this degrades gracefully rather
    /// than silently doing nothing.
    fn apply_block_projection(&mut self, block: &crate::data::octant_block::OctantBlock) {
        let anim_dim = self.animated_dim;

        // Guard against the animated dimension being (mis)assigned as a
        // spatial role too — it's collapsed to size 1 per fetch, so using
        // it as x_dim/y_dim would silently render a 1px-wide/tall slice
        // every frame: fetches genuinely differ, but the picture looks frozen.
        let explicit_spatial = match (self.spatial_dims.first(), self.spatial_dims.get(1)) {
            (Some(&x), Some(&y)) => Some((x, y)),
            _ => None,
        };

        let (x_dim, y_dim) = if let Some((x, y)) = explicit_spatial {
            if Some(x) == anim_dim || Some(y) == anim_dim {
                self.status_message = format!(
                    "Block cache: X/Y spatial role collides with the animated dimension (dim {}) — pick a different X or Y dim.",
                    anim_dim.unwrap_or(usize::MAX)
                );
                return;
            }
            (x, y)
        } else {
            // Fallback: the block's last two dimensions, excluding the
            // animated one (previously this didn't exclude it — the bug).
            let non_anim: Vec<usize> = (0..block.rank()).filter(|&d| Some(d) != anim_dim).collect();
            match (
                non_anim.last().copied(),
                non_anim
                    .len()
                    .checked_sub(2)
                    .and_then(|i| non_anim.get(i))
                    .copied(),
            ) {
                (Some(x), Some(y)) => (x, y),
                _ => {
                    self.status_message =
                        "Block cache: not enough non-animated dimensions to project a 2D view."
                            .to_string();
                    return;
                }
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
            self.animated_dim_extent(),
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
        self.status_message = format!(
            "{}  [x_dim={x_dim} y_dim={y_dim} anim_dim={anim_dim:?} t={}]",
            self.status_message, self.current_timestep
        );
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
