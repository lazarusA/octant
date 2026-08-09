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
use crate::ui::variables_panel::build_slice_request;

use super::OctantApp;
use super::state::StoreKind;

impl OctantApp {
    /// Aligned `[start, end)` window along the animated dimension that
    /// contains `self.current_timestep`, clamped to the dataset's actual
    /// extent.
    fn animated_window(&self, full_extent: usize) -> (usize, usize) {
        let window = self.block_window_size.max(1);
        let start = (self.current_timestep / window) * window;
        let end = (start + window).min(full_extent).max(start + 1);
        (start, end)
    }

    /// Block-cache equivalent of `load_selected_variable_slice`.
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

        let legacy_request = build_slice_request(self, &var_name, &shape);
        let mut selections: Vec<DimensionSelection> = legacy_request
            .selections
            .iter()
            .map(|sel| match sel {
                crate::utils::zarr::DimensionSelection::Index(i) => DimensionSelection::Index(*i),
                crate::utils::zarr::DimensionSelection::Range(r) => DimensionSelection::Range {
                    start: r.start,
                    end: r.end,
                },
            })
            .collect();

        if let Some(anim_dim) = self.animated_dim {
            self.selected_dim_indices[anim_dim] = self.current_timestep;
            if anim_dim < selections.len() {
                let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
                let (start, end) = self.animated_window(full_extent);
                selections[anim_dim] = DimensionSelection::Range { start, end };
            }
        }

        let slice_request = SliceRequest::new(&var_name, selections);
        self.active_slice_request = Some(crate::utils::zarr::SliceRequest {
            variable: var_name.clone(),
            selections: slice_request
                .selections
                .iter()
                .map(|s| match s {
                    DimensionSelection::Index(i) => {
                        crate::utils::zarr::DimensionSelection::Index(*i)
                    }
                    DimensionSelection::Range { start, end } => {
                        crate::utils::zarr::DimensionSelection::Range(*start..*end)
                    }
                })
                .collect(),
        });

        let source_id = format!("{:?}:{}", self.selected_store_kind, self.store_target_input);

        let store_handle = if let Some(dataset) = self.dataset_manager.get(&source_id) {
            dataset.store.clone()
        } else {
            let kind = match self.selected_store_kind {
                StoreKind::RemoteZarr => DataSourceKind::RemoteZarr,
                StoreKind::LocalZarr => DataSourceKind::LocalZarr,
                StoreKind::RemoteIcechunk => DataSourceKind::RemoteIcechunk,
                StoreKind::LocalIcechunk => DataSourceKind::LocalIcechunk,
                StoreKind::ProceduralRandom => DataSourceKind::Other("ProceduralRandom".into()),
            };
            let data_source =
                DataSource::new(&source_id, kind, &self.store_target_input, &metadata.name);

            match SourceFactory::open(data_source.clone()) {
                Ok(store) => {
                    let dataset = Dataset::new(&source_id, data_source, store.clone());
                    self.dataset_manager.add(dataset);
                    store
                }
                Err(e) => {
                    self.status_message = format!("Block cache open error: {e}");
                    return;
                }
            }
        };

        let block_request = BlockRequest::new(store_handle, slice_request);
        let key = block_request.cache_key();
        self.active_block_key = Some(key.clone());

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

        // 2. Cache MISS: dispatch async prefetch request.
        self.status_message = format!("⏳ [block cache] Downloading window for '{}'...", var_name);
        self.block_prefetcher
            .request(block_request, &self.block_cache);
    }

    /// Prefetches next animation window in the background once past midpoint.
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
            return;
        }
        if self.current_timestep < start + (end - start) / 2 {
            return;
        }

        let next_start = end;
        let next_end = (next_start + self.block_window_size.max(1)).min(full_extent);

        let Some(next_legacy_request) = self.active_slice_request.clone() else {
            return;
        };
        if anim_dim >= next_legacy_request.selections.len() {
            return;
        }

        let mut selections: Vec<DimensionSelection> = next_legacy_request
            .selections
            .iter()
            .map(|sel| match sel {
                crate::utils::zarr::DimensionSelection::Index(i) => DimensionSelection::Index(*i),
                crate::utils::zarr::DimensionSelection::Range(r) => DimensionSelection::Range {
                    start: r.start,
                    end: r.end,
                },
            })
            .collect();

        selections[anim_dim] = DimensionSelection::Range {
            start: next_start,
            end: next_end,
        };

        let next_slice_request = SliceRequest::new(next_legacy_request.variable, selections);
        let source_id = format!("{:?}:{}", self.selected_store_kind, self.store_target_input);

        let Some(dataset) = self.dataset_manager.get(&source_id) else {
            return;
        };

        let next_block_request = dataset.request(next_slice_request);
        self.block_prefetcher
            .request(next_block_request, &self.block_cache);
    }

    /// Full size of the currently animated dimension in the dataset.
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

    /// Projects a resident block into the current 2D view.
    fn apply_block_projection(&mut self, block: &crate::data::octant_block::OctantBlock) {
        let anim_dim = self.animated_dim;

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
            .map(|(i, &idx)| idx.saturating_sub(block.origin.get(i).copied().unwrap_or(0)))
            .collect();

        eprintln!(
            "[apply_block_projection] block var='{}', shape={:?}, dim_names={:?}, x_dim={x_dim}, y_dim={y_dim}, fixed_indices={:?}",
            block.variable_name, block.shape, block.dimension_names, fixed_indices
        );

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

    /// Drains completed block-cache prefetch results.
    pub fn poll_block_prefetch_results(&mut self) {
        if !self.use_block_cache {
            return;
        }

        let completed = self.block_prefetcher.poll();
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
