//! Background prefetching logic and animation lookahead range queueing.

use crate::app::OctantApp;
use crate::data::{BlockRequest, DimensionSelection, SliceRequest};

impl OctantApp {
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

        let base_req = if let Some(req) = self.active_slice_request.clone() {
            req
        } else if let Some(var_info) = self.plotted_variable_info() {
            crate::ui::variables_panel::build_slice_request_for_plotted(self, &var_info.name, shape)
        } else {
            return;
        };

        if anim_dim >= base_req.selections.len() {
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
            &base_req.selections,
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

        let _first_chunk = range_start / cs;
        let last_chunk = range_end / cs;

        let source_id = self.plotted_source_id();

        let Some(store_handle) = self.plotted_store_handle() else {
            return;
        };

        let current_chunk = self.current_timestep / cs;
        let max_dataset_chunk = full_extent.saturating_sub(1) / cs;

        let mut chunk_indices = Vec::new();
        if self.is_playing {
            // Sliding lookahead window during active animation playback:
            let lookahead_chunks = (self.block_window_size / cs).max(1);
            let max_lookahead_chunk = (current_chunk + lookahead_chunks).min(max_dataset_chunk);
            for c in (current_chunk + 1)..=max_lookahead_chunk {
                chunk_indices.push(c);
            }

            // If loop playback is active and approaching end of dataset, buffer starting wrap-around chunks:
            if self.loop_playback && current_chunk + lookahead_chunks >= max_dataset_chunk {
                let wrap_end = lookahead_chunks.saturating_sub(1);
                for c in 0..=wrap_end.min(max_dataset_chunk) {
                    if !chunk_indices.contains(&c) && c != current_chunk {
                        chunk_indices.push(c);
                    }
                }
            }
        } else {
            // When paused / on initial load with a multi-chunk range selection:
            // Queue all chunks across the user's requested range in parallel across Rayon workers:
            for c in (current_chunk + 1)..=last_chunk {
                chunk_indices.push(c);
            }
        }

        for chunk_idx in chunk_indices {
            let chunk_start = chunk_idx * cs;
            let chunk_end = (chunk_start + cs).min(full_extent);

            if self.block_cache.covers(
                &source_id,
                &base_req.variable,
                &base_req.selections,
                self.plotted_animated_dim,
                chunk_start,
            ) {
                continue;
            }

            if self.block_prefetcher.is_pending_timestep(
                &source_id,
                &base_req.variable,
                &base_req.selections,
                self.plotted_animated_dim,
                chunk_start,
            ) {
                continue;
            }

            let mut selections = base_req.selections.clone();
            selections[anim_dim] = DimensionSelection::Range {
                start: chunk_start,
                end: chunk_end,
            };
            let req = BlockRequest::new(
                store_handle.clone(),
                SliceRequest::new(base_req.variable.clone(), selections),
            );

            if !self.block_cache.contains(&req.cache_key())
                && !self.block_prefetcher.request(req, &self.block_cache)
            {
                break;
            }
        }
    }
}
