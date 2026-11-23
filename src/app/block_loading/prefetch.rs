use super::pacing::{PrefetchDispatchParams, estimate_chunk_bytes, paused_chunks, playing_chunks};
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
        // The window is the shown selection's: the base layer keeps it on arrival.
        let requested = (id == LayerId::BASE).then(|| self.layers.base.selection().clone());
        if let Some(layer) = self.layers.get_mut(id) {
            layer.load.block_key = Some(req.cache_key());
            layer.load.pending_target_step = Some(step);
            layer.load.requested = requested;
        }
        self.block_prefetcher.request_now(req, &self.block_cache);
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
        let failed_overlay =
            id != LayerId::BASE && self.layers.get(id).is_some_and(|l| l.load.failed);
        if !self.playback.enable_prefetch || failed_overlay {
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
        // A fetch decodes whole stored chunks, however little of them it reads.
        let source = store_handle.source();
        let stored_bytes =
            super::arrival_notices::stored_chunk_bytes(var_info, &source.kind, &source.uri);
        let (max_concurrent, budget) = self.prefetch_pacing(
            id,
            cs,
            estimate_chunk_bytes(base_req, anim_dim, cs),
            stored_bytes,
        );
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

    /// The chunk indices of layer `id` to prefetch, in order: while playing,
    /// `budget` chunks from the current one on (it included); while paused,
    /// the selected range of the animated dimension from the current chunk
    /// on, at most `budget` chunks.
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
        // A ring volume's window runs on into the first steps, looping or not.
        let ring = self.has_ring(id);
        if self.playback.is_playing {
            let wraps = self.playback.loop_playback || ring;
            return playing_chunks(current, max_chunk, budget, wraps);
        }
        let (start, end) = shown
            .dim_config
            .get(anim_dim)
            .map(|c| c.range)
            .unwrap_or((0, last_step));
        let range = (start.min(last_step) / cs, end.min(last_step) / cs);
        let mut chunks = self.ring_chunks(id, cs);
        let window = chunks.len();
        for c in paused_chunks(current, range, max_chunk, budget) {
            if !chunks.contains(&c) {
                chunks.push(c);
            }
        }
        chunks.truncate(budget.max(window));
        chunks
    }

    /// Dispatches candidate prefetch requests up to the dynamic concurrency limit.
    fn dispatch_lookahead_chunks(&mut self, indices: &[usize], params: PrefetchDispatchParams<'_>) {
        let source_id = params.source_id;
        for &chunk_idx in indices {
            // The prefetcher's own cap may be lower than the lookahead's.
            if self.block_prefetcher.active_worker_threads() >= params.max_concurrent
                || !self.block_prefetcher.has_free_worker()
            {
                self.block_prefetcher.hear_freed();
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

            // A failed block is asked again every few seconds, not in a loop;
            // full workers stop the loop at its top.
            if !self.block_prefetcher.failed_recently(&req.cache_key()) {
                self.block_prefetcher.request(req, &self.block_cache);
            }
        }
    }
}
