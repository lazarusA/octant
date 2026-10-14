use super::pacing::{
    PrefetchDispatchParams, adaptive_concurrency, estimate_chunk_bytes, paused_chunks,
    playing_chunks, playing_lookahead, range_budget,
};
use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::{BlockRequest, DimensionSelection, SliceRequest};

impl OctantApp {
    /// Prefetches the block window containing `step` of layer `id`'s shown
    /// selection asynchronously; the block is shown at `step` when it arrives.
    pub(crate) fn prefetch_layer_window(&mut self, id: LayerId, step: usize) {
        let Some(layer_request) = self.shown_layer_request(id) else {
            return;
        };
        let Some(anim_dim) = layer_request.anim_dim else {
            return;
        };
        let var_info = &layer_request.var;
        let full_extent = var_info.shape.get(anim_dim).copied().unwrap_or(1) as usize;
        if step >= full_extent || self.request_resident(&layer_request, step) {
            return;
        }
        let Some(store_handle) = self.request_store(&layer_request) else {
            return;
        };

        let (start, end, _) = self.animated_window_bounds(
            step,
            full_extent,
            anim_dim,
            &var_info.chunk_shape,
            &var_info.shape,
            &layer_request.request.selections,
        );

        let mut selections = layer_request.request.selections;
        if anim_dim < selections.len() {
            selections[anim_dim] = DimensionSelection::Range { start, end };
        }

        let req = BlockRequest::new(store_handle, SliceRequest::new(&var_info.name, selections));
        if let Some(layer) = self.layers.get_mut(id) {
            layer.load.block_key = Some(req.cache_key());
            layer.load.pending_target_step = Some(step);
        }
        self.block_prefetcher.request(req, &self.block_cache);
    }

    /// Shows `target_step` when the base layer holds it (loading the animated
    /// overlays at it too), else prefetches every drawn layer's window there.
    pub fn request_step_or_load(&mut self, target_step: usize) {
        if self.layer_step_resident(LayerId::BASE, target_step) {
            self.playback.current_timestep = target_step;
            self.load_step_blocks();
        } else {
            for id in self.layers.drawn_ids() {
                self.prefetch_layer_window(id, target_step);
            }
        }
    }

    /// Progressively prefetches lookahead block windows along every drawn
    /// layer's animated dimension.
    pub fn prefetch_animated_ranges(&mut self) {
        for id in self.layers.drawn_ids() {
            self.prefetch_layer_animated_range(id);
        }
    }

    /// Progressively prefetches lookahead block windows along layer `id`'s
    /// animated dimension.
    pub(crate) fn prefetch_layer_animated_range(&mut self, id: LayerId) {
        if !self.playback.enable_prefetch {
            return;
        }
        let Some(layer_request) = self.shown_layer_request(id) else {
            return;
        };
        let Some(anim_dim) = layer_request.anim_dim else {
            return;
        };
        let (var_info, base_req) = (&layer_request.var, &layer_request.request);
        let full_extent = var_info.shape.get(anim_dim).copied().unwrap_or(1) as usize;
        if full_extent <= 1 || anim_dim >= base_req.selections.len() {
            return;
        }
        let Some(store_handle) = self.request_store(&layer_request) else {
            return;
        };

        let (_, _, window_step) = self.animated_window_bounds(
            self.playback.current_timestep,
            full_extent,
            anim_dim,
            &var_info.chunk_shape,
            &var_info.shape,
            &base_req.selections,
        );
        let cs = window_step.max(1);
        let (max_concurrent, budget) =
            self.prefetch_pacing(cs, estimate_chunk_bytes(base_req, anim_dim, cs));
        let chunk_indices = self.collect_lookahead_chunks(id, anim_dim, cs, full_extent, budget);
        let params = PrefetchDispatchParams {
            cs,
            full_extent,
            anim_dim,
            max_concurrent,
            base_req,
            store_handle: &store_handle,
            source_id: &layer_request.source_id,
        };
        self.dispatch_lookahead_chunks(&chunk_indices, params);
    }

    /// The lookahead's workers and its chunk budget for chunks of `cs` steps
    /// and `chunk_bytes`: the playing lookahead, else the paused range's.
    /// Foreground requests (each layer's shown block) never take the
    /// lookahead's workers.
    fn prefetch_pacing(&self, cs: usize, chunk_bytes: usize) -> (usize, usize) {
        let max_concurrent =
            adaptive_concurrency(chunk_bytes, self.prefetch_threads) + self.foreground_requests();
        let cache_bytes = self.block_cache.max_bytes();
        let budget = if self.playback.is_playing {
            playing_lookahead(self.block_window_size, cs, chunk_bytes, cache_bytes)
        } else {
            range_budget(chunk_bytes, cache_bytes)
        };
        (max_concurrent, budget)
    }

    /// Layers whose shown block is being fetched.
    fn foreground_requests(&self) -> usize {
        self.layers
            .iter()
            .filter(|l| {
                l.load
                    .block_key
                    .as_ref()
                    .is_some_and(|k| self.block_prefetcher.is_pending(k))
            })
            .count()
    }

    /// The chunk indices of layer `id` to prefetch, in order: while playing,
    /// the current chunk and the `budget` after it; while paused, the
    /// selected range of the animated dimension from the current chunk on,
    /// at most `budget` chunks.
    fn collect_lookahead_chunks(
        &self,
        id: LayerId,
        anim_dim: usize,
        cs: usize,
        full_extent: usize,
        budget: usize,
    ) -> Vec<usize> {
        let Some((shown, _)) = self.layer_selections(id) else {
            return Vec::new();
        };
        let current = self.layer_step(id) / cs;
        let last_step = full_extent.saturating_sub(1);
        let max_chunk = last_step / cs;
        if self.playback.is_playing {
            return playing_chunks(current, max_chunk, budget, self.playback.loop_playback);
        }
        let (start, end) = shown
            .dim_ranges
            .get(anim_dim)
            .copied()
            .unwrap_or((0, last_step));
        let range = (start.min(last_step) / cs, end.min(last_step) / cs);
        paused_chunks(current, range, max_chunk, budget)
    }

    /// Dispatches candidate prefetch requests up to the dynamic concurrency limit.
    fn dispatch_lookahead_chunks(&mut self, indices: &[usize], params: PrefetchDispatchParams<'_>) {
        let source_id = params.source_id;
        for &chunk_idx in indices {
            if self.block_prefetcher.active_worker_threads() >= params.max_concurrent {
                break;
            }
            let chunk_start = chunk_idx * params.cs;
            let chunk_end = (chunk_start + params.cs).min(params.full_extent);

            if self.block_cache.covers(
                source_id,
                &params.base_req.variable,
                &params.base_req.selections,
                Some(params.anim_dim),
                chunk_start,
            ) || self.block_prefetcher.is_pending_timestep(
                source_id,
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
