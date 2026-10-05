use super::pacing::{
    PrefetchDispatchParams, adaptive_concurrency, adaptive_lookahead, estimate_chunk_bytes,
};
use crate::app::OctantApp;
use crate::data::{BlockRequest, DimensionSelection, SliceRequest};

impl OctantApp {
    /// Prefetches the block window containing `step` asynchronously.
    pub fn prefetch_block_window_for_next_steps(&mut self, step: usize) {
        let (Some(metadata), Some(store_handle), Some(anim_dim)) = (
            &self.plotted_dataset_metadata,
            self.plotted_store_handle(),
            self.plotted_animated_dim,
        ) else {
            return;
        };
        let Some(var_info) = metadata.variables.get(self.plotted_variable_idx) else {
            return;
        };

        let var_name = var_info.name.clone();
        let shape = var_info.shape.clone();
        let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
        if step >= full_extent {
            return;
        }

        let base_request =
            crate::ui::variables_panel::build_slice_request_for_plotted(self, &var_name, &shape);
        if self.block_cache.covers(
            &self.plotted_source_id(),
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

        let req = BlockRequest::new(store_handle, SliceRequest::new(&var_name, selections));
        self.active_block_key = Some(req.cache_key());
        self.pending_target_step = Some(step);
        self.block_prefetcher.request(req, &self.block_cache);
    }

    /// Checks if `target_step` is resident in the block cache; loads or prefetches it.
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

        let is_cached = var_name.is_some_and(|name| {
            self.block_cache.covers(
                &source_id,
                &name,
                selections,
                self.plotted_animated_dim,
                target_step,
            )
        });

        if is_cached {
            self.current_timestep = target_step;
            self.load_selected_variable_block();
        } else {
            self.prefetch_block_window_for_next_steps(target_step);
        }
    }

    /// Progressively prefetches lookahead block windows along the animated dimension.
    pub fn prefetch_selected_animated_range(&mut self, shape: &[u64]) {
        if !self.enable_prefetch {
            return;
        }
        let (Some(anim_dim), Some(var_info), Some(store_handle)) = (
            self.plotted_animated_dim,
            self.plotted_variable_info(),
            self.plotted_store_handle(),
        ) else {
            return;
        };

        let full_extent = shape.get(anim_dim).copied().unwrap_or(1) as usize;
        let base_req = crate::ui::variables_panel::build_slice_request_for_plotted(
            self,
            &var_info.name,
            shape,
        );
        if full_extent <= 1 || anim_dim >= base_req.selections.len() {
            return;
        }

        let (_, _, window_step) = self.animated_window_bounds(
            self.current_timestep,
            full_extent,
            anim_dim,
            &var_info.chunk_shape,
            shape,
            &base_req.selections,
        );
        let cs = window_step.max(1);
        let chunk_bytes = estimate_chunk_bytes(&base_req, anim_dim, cs);
        let max_concurrent = adaptive_concurrency(chunk_bytes, self.prefetch_threads);
        let lookahead = adaptive_lookahead(
            self.block_window_size,
            cs,
            chunk_bytes,
            self.block_cache.max_bytes(),
        );

        let chunk_indices = self.collect_lookahead_chunks(anim_dim, cs, full_extent, lookahead);
        let params = PrefetchDispatchParams {
            cs,
            full_extent,
            anim_dim,
            max_concurrent,
            base_req: &base_req,
            store_handle: &store_handle,
        };
        self.dispatch_lookahead_chunks(&chunk_indices, params);
    }

    /// Collects the ordered sequence of chunk indices to prefetch.
    fn collect_lookahead_chunks(
        &self,
        anim_dim: usize,
        cs: usize,
        full_extent: usize,
        lookahead: usize,
    ) -> Vec<usize> {
        let current_chunk = self.current_timestep / cs;
        let max_dataset_chunk = full_extent.saturating_sub(1) / cs;
        let is_spatial = self
            .plotted_dim_config
            .get(anim_dim)
            .or_else(|| self.dim_config.get(anim_dim))
            .is_some_and(|c| c.spatial != crate::app::SpatialRole::None);

        let mut indices = Vec::new();
        if is_spatial {
            let (r_start, r_end) = self
                .plotted_selected_dim_ranges
                .get(anim_dim)
                .copied()
                .unwrap_or((0, full_extent.saturating_sub(1)));
            let (first, last) = (r_start / cs, r_end.min(full_extent.saturating_sub(1)) / cs);
            indices.extend((first..=last).filter(|&c| c != current_chunk));
            indices.truncate(lookahead);
        } else if self.is_playing {
            let max_lookahead = (current_chunk + lookahead).min(max_dataset_chunk);
            indices.extend(current_chunk..=max_lookahead);
            if self.loop_playback && current_chunk + lookahead >= max_dataset_chunk {
                let wrap_end = lookahead.saturating_sub(1).min(max_dataset_chunk);
                for c in 0..=wrap_end {
                    if !indices.contains(&c) {
                        indices.push(c);
                    }
                }
            }
            indices.truncate(lookahead + 1);
        } else {
            let max_paused = (current_chunk + lookahead.min(2)).min(max_dataset_chunk);
            indices.extend(current_chunk..=max_paused);
        }
        indices
    }

    /// Dispatches candidate prefetch requests up to the dynamic concurrency limit.
    fn dispatch_lookahead_chunks(&mut self, indices: &[usize], params: PrefetchDispatchParams<'_>) {
        let source_id = self.plotted_source_id();
        for &chunk_idx in indices {
            if self.block_prefetcher.active_worker_threads() >= params.max_concurrent {
                break;
            }
            let chunk_start = chunk_idx * params.cs;
            let chunk_end = (chunk_start + params.cs).min(params.full_extent);

            if self.block_cache.covers(
                &source_id,
                &params.base_req.variable,
                &params.base_req.selections,
                Some(params.anim_dim),
                chunk_start,
            ) || self.block_prefetcher.is_pending_timestep(
                &source_id,
                &params.base_req.variable,
                &params.base_req.selections,
                Some(params.anim_dim),
                chunk_start,
            ) {
                continue;
            }

            let mut selections = params.base_req.selections.clone();
            selections[params.anim_dim] = DimensionSelection::Range {
                start: chunk_start,
                end: chunk_end,
            };
            let req = BlockRequest::new(
                params.store_handle.clone(),
                SliceRequest::new(params.base_req.variable.clone(), selections),
            );

            if !self.block_cache.contains(&req.cache_key())
                && !self.block_prefetcher.request(req, &self.block_cache)
                && self.block_prefetcher.active_worker_threads() >= params.max_concurrent
            {
                break;
            }
        }
    }
}
