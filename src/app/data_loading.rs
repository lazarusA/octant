use crate::cache::SliceCacheKey;
use crate::data::matrix_data::MatrixData;
use crate::stores::{
    DataStore, DatasetMetadata, VariableInfo, icechunk_local::IcechunkLocalStore,
    icechunk_remote::IcechunkRemoteStore, zarr_local::ZarrLocalStore, zarr_remote::ZarrRemoteStore,
};
use crate::ui::variables_panel::build_slice_request;

use super::OctantApp;
use super::state::StoreKind;

impl OctantApp {
    pub(super) fn get_line_profile_payload(&self) -> (Vec<f32>, u32, u32) {
        if let Some(matrix) = &self.matrix_data {
            let (profile_length, line_count, slice_idx) = if self.line_profile_dim_idx == 0 {
                (
                    matrix.width,
                    matrix.height,
                    self.line_profile_slice_idx
                        .min(matrix.height.saturating_sub(1)),
                )
            } else {
                (
                    matrix.height,
                    matrix.width,
                    self.line_profile_slice_idx
                        .min(matrix.width.saturating_sub(1)),
                )
            };

            if self.line_plot_all_series {
                let mut payload = Vec::with_capacity(profile_length.max(1) * line_count.max(1));
                for idx in 0..line_count {
                    payload.extend(matrix.extract_1d_line_profile(self.line_profile_dim_idx, idx));
                }
                (payload, profile_length as u32, line_count as u32)
            } else {
                (
                    matrix.extract_1d_line_profile(self.line_profile_dim_idx, slice_idx),
                    profile_length as u32,
                    1,
                )
            }
        } else {
            (Vec::new(), 0, 0)
        }
    }

    pub fn inspect_active_store(&mut self) {
        self.is_loading = true;
        self.status_message = format!("Inspecting {:?} metadata...", self.selected_store_kind);

        let store_kind = self.selected_store_kind;
        let target_input = self.store_target_input.clone();

        let (tx, rx) = std::sync::mpsc::channel();
        self.metadata_rx = Some(rx);

        std::thread::spawn(move || {
            let store: Box<dyn DataStore> = match store_kind {
                StoreKind::RemoteZarr => Box::new(ZarrRemoteStore::new(&target_input)),
                StoreKind::LocalZarr => Box::new(ZarrLocalStore::new(&target_input)),
                StoreKind::RemoteIcechunk => Box::new(IcechunkRemoteStore::new(&target_input)),
                StoreKind::LocalIcechunk => Box::new(IcechunkLocalStore::new(&target_input)),
                StoreKind::ProceduralRandom => {
                    let meta = DatasetMetadata {
                        name: "Procedural Test Store".to_string(),
                        store_type: "Random Procedural".to_string(),
                        variables: vec![VariableInfo {
                            name: "random_matrix".to_string(),
                            data_type: "float32".to_string(),
                            shape: vec![64, 64],
                            dimension_names: vec!["y".to_string(), "x".to_string()],
                            chunk_shape: vec![64, 64],
                            file_size: crate::utils::calculate_variable_size_bytes(
                                &[64, 64],
                                "float32",
                            ),
                            ..Default::default()
                        }],
                        dimension_coordinates: std::collections::HashMap::new(),
                    };
                    let _ = tx.send(Ok(meta));
                    return;
                }
            };

            let res = store.inspect().map_err(|e| e.to_string());
            let _ = tx.send(res);
        });
    }

    pub fn load_selected_variable_slice(&mut self) {
        self.show_settings_panel = true;

        let (var_name, max_steps, chunk_time_size, slice_bytes_hint) = {
            if let Some(metadata) = &self.active_dataset_metadata {
                if let Some(var_info) = metadata.variables.get(self.selected_variable_idx) {
                    let max_steps = if var_info.shape.len() <= 2 {
                        if let Some(first_dim) = var_info.dimension_names.first() {
                            let name = first_dim.to_lowercase();
                            if name == "time" || name == "t" || name.contains("step") {
                                var_info.shape.first().copied().unwrap_or(1) as usize
                            } else {
                                1
                            }
                        } else {
                            1
                        }
                    } else {
                        var_info.shape.first().copied().unwrap_or(1) as usize
                    };
                    let chunk_time_size = if !var_info.chunk_shape.is_empty() {
                        var_info.chunk_shape[0] as usize
                    } else {
                        46
                    };

                    let slice_bytes = if var_info.shape.len() >= 2 {
                        let w = var_info.shape[var_info.shape.len() - 1] as usize;
                        let h = var_info.shape[var_info.shape.len() - 2] as usize;
                        w * h * std::mem::size_of::<f32>()
                    } else {
                        64 * 64 * std::mem::size_of::<f32>()
                    };

                    (
                        var_info.name.clone(),
                        max_steps,
                        chunk_time_size,
                        slice_bytes,
                    )
                } else {
                    return;
                }
            } else {
                return;
            }
        };
        // We still have metadata in scope, so get var_info again:
        let var_info = if let Some(metadata) = &self.active_dataset_metadata {
            if let Some(v) = metadata.variables.get(self.selected_variable_idx) {
                v
            } else {
                self.status_message = "Invalid variable index.".to_string();
                return;
            }
        } else {
            self.status_message = "No dataset metadata loaded.".to_string();
            return;
        };

        // Build the hyperslab request from UI state
        let slice_request = build_slice_request(self, &var_name, &var_info.shape);
        self.active_slice_request = Some(slice_request);
        // Update current_timestep based on animated dimension (if any)
        if let Some(anim_dim) = self.animated_dim {
            self.current_timestep = self.selected_dim_indices[anim_dim];
        }

        // Enforce bounds on current_timestep for new variable
        if self.current_timestep >= max_steps {
            self.current_timestep = 0;
        }

        let cache_key = SliceCacheKey {
            store_kind: self.selected_store_kind,
            store_target: self.store_target_input.clone(),
            variable_name: var_name.clone(),
            timestep: self.current_timestep,
        };

        self.active_requested_key = Some(cache_key.clone());

        // 1. Check LRU Cache HIT
        if let Some(slice) = self.lru_cache.get(&cache_key) {
            let mdata = MatrixData::new(
                slice.width,
                slice.height,
                slice.values.clone(),
                slice.min_val,
                slice.max_val,
                format!("{} ({})", slice.dataset_name, slice.variable_name),
                slice.max_timesteps,
            );
            self.rebuild_pipeline_with_matrix_data(mdata);
            self.is_fetching_slice = false;
            self.status_message = format!(
                "🚀 Cache HIT for '{}' (step {})",
                slice.variable_name,
                self.current_timestep + 1
            );

            // Trigger chunk-aligned prefetching ahead
            let total_steps = max_steps.max(slice.max_timesteps);
            self.prefetcher.prefetch_chunk_aligned(
                self.selected_store_kind,
                &self.store_target_input,
                &var_name,
                self.current_timestep,
                total_steps,
                chunk_time_size,
                slice_bytes_hint,
                self.prefetch_lookahead,
                &self.lru_cache,
            );
            return;
        }

        // Procedural random bypass
        if self.selected_store_kind == StoreKind::ProceduralRandom {
            if let Ok(data) = MatrixData::create_random_matrix(64, 64) {
                self.rebuild_pipeline_with_matrix_data(data);
            }
            self.is_fetching_slice = false;
            return;
        }

        // 2. Cache MISS: Dispatch Non-Blocking Async Background Request for active slice ONLY!
        self.is_fetching_slice = true;
        self.status_message = format!(
            "⏳ Downloading slice for '{}' (step {})...",
            var_name,
            self.current_timestep + 1
        );

        // Fetch active slice first so 2D map renders immediately on screen
        self.prefetcher.request_slice(cache_key, &self.lru_cache);
    }

    pub fn trigger_background_prefetch(&mut self) {
        if let Some(metadata) = &self.active_dataset_metadata
            && let Some(var_info) = metadata.variables.get(self.selected_variable_idx)
        {
            let max_steps = if var_info.shape.len() <= 2 {
                1
            } else {
                var_info.shape.first().copied().unwrap_or(1) as usize
            };
            if max_steps <= 1 {
                return;
            }
            let chunk_time_size = if !var_info.chunk_shape.is_empty() {
                var_info.chunk_shape[0] as usize
            } else {
                46
            };
            let slice_bytes = if var_info.shape.len() >= 2 {
                let w = var_info.shape[var_info.shape.len() - 1] as usize;
                let h = var_info.shape[var_info.shape.len() - 2] as usize;
                w * h * std::mem::size_of::<f32>()
            } else {
                64 * 64 * std::mem::size_of::<f32>()
            };

            self.prefetcher.prefetch_chunk_aligned(
                self.selected_store_kind,
                &self.store_target_input,
                &var_info.name,
                self.current_timestep,
                max_steps,
                chunk_time_size,
                slice_bytes,
                self.prefetch_lookahead,
                &self.lru_cache,
            );
        }
    }
}
