//! Primary variable block loading, prefetch polling, and fetch abort controls.

use crate::app::OctantApp;
use crate::app::layers::LayerId;
use crate::data::BlockRequest;
use crate::data::octant_block::OctantBlock;

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

    /// Loads the base layer's block at the current step and selections.
    pub fn load_selected_variable_block(&mut self) {
        self.load_layer_block(LayerId::BASE);
    }

    /// Loads the base layer's block at the current step, then every animated
    /// overlay's.
    pub(crate) fn load_step_blocks(&mut self) {
        self.load_selected_variable_block();
        self.load_animated_overlay_blocks();
    }

    /// Loads the block of every drawn overlay with an animated dimension at
    /// the current step (the others don't change with it).
    pub(crate) fn load_animated_overlay_blocks(&mut self) {
        for id in self.layers.drawn_ids() {
            if id != LayerId::BASE && self.layer_animated_dim(id).is_some() {
                self.load_layer_block(id);
            }
        }
    }

    /// Loads layer `id`'s block at the current step from its staged selection:
    /// shows it from the cache when resident, else requests it. The base layer
    /// clamps the current step into its animated extent; other layers clamp
    /// only their own step (`layer_step`).
    pub fn load_layer_block(&mut self, id: LayerId) {
        self.request_layer_coordinates(id);
        let Some((_, staged)) = self.layer_selections(id) else {
            return;
        };
        if staged.metadata.is_none() {
            self.status_message = "No dataset metadata loaded.".to_string();
            return;
        }
        let Some(layer_request) = self.staged_layer_request(id) else {
            self.status_message = "Invalid selected variable index.".to_string();
            return;
        };
        let anim_dim = self.layer_animated_dim(id);
        let (slice_request, step) = self.request_at_current_step(id, &layer_request, anim_dim);
        let store_handle = self.request_store(&layer_request);
        let block_request = store_handle.map(|h| BlockRequest::new(h, slice_request.clone()));
        let Some(layer) = self.layers.get_mut(id) else {
            return;
        };
        layer.load.block_key = block_request.as_ref().map(BlockRequest::cache_key);
        layer.load.slice_request = Some(slice_request);

        let source_id = &layer_request.source_id;
        if self.show_resident_block(id, source_id, anim_dim, step) {
            return;
        }
        let Some(block_request) = block_request else {
            self.status_message =
                format!("Dataset store not open in DatasetManager for '{source_id}'");
            return;
        };

        // Cache MISS: dispatch async prefetch request for current chunk and launch background prefetching in parallel.
        // Playback may move on meanwhile; the block is shown at this step.
        if let Some(layer) = self.layers.get_mut(id) {
            layer.load.pending_target_step = Some(step);
        }
        self.status_message = format!(
            "[block cache] Downloading window for '{}'...",
            layer_request.var.name
        );
        self.block_prefetcher
            .request(block_request, &self.block_cache);
    }

    /// Shows layer `id`'s latest request (`load.slice_request`, at `step`)
    /// from the cache: a resident block covering the step (e.g. the full
    /// array), else the exact block. Whether one was shown.
    fn show_resident_block(
        &mut self,
        id: LayerId,
        source_id: &str,
        anim_dim: Option<usize>,
        step: usize,
    ) -> bool {
        let Some(layer) = self.layers.get(id) else {
            return false;
        };
        let (Some(request), key) = (&layer.load.slice_request, &layer.load.block_key) else {
            return false;
        };
        let block = self
            .block_cache
            .find_covering_block(
                source_id,
                &request.variable,
                &request.selections,
                anim_dim,
                step,
            )
            .or_else(|| key.as_ref().and_then(|k| self.block_cache.get(k)));
        let Some(block) = block else {
            return false;
        };
        let selections = request.selections.clone();
        self.show_cached_block(id, source_id, &block, &selections, anim_dim);
        true
    }

    /// Drains completed block-cache prefetch results.
    pub fn poll_block_prefetch_results(&mut self) {
        let completed = self.block_prefetcher.poll();
        let had_completed = !completed.is_empty();

        for res in completed {
            match res.result {
                Ok(block) => {
                    let requested = self.layers.find_by_key(&res.key);
                    let shown_in = requested.or_else(|| self.layer_showing_block(&res.key, &block));

                    self.block_cache.put(res.key, block.clone());

                    if let Some(id) = shown_in {
                        let moved_step =
                            requested.is_some() && self.accept_requested_block(id, &block);
                        self.status_message =
                            format!("[block cache] Loaded '{}'", block.variable_name);
                        self.apply_block_projection(id, &block);
                        // The base layer reached a new step: overlays follow it.
                        if moved_step && id == LayerId::BASE {
                            self.load_animated_overlay_blocks();
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("Block cache fetch error: {e}");
                    self.notify(crate::ui::toast::Severity::Error, "Couldn't load data", &e);
                }
            }
        }

        // Whenever worker slots free up, replenish and dispatch remaining chunks in the selection:
        if had_completed {
            self.prefetch_animated_ranges();
        }
    }

    /// Settles layer `id`'s request for `block`: the base layer's staged
    /// selection becomes the plotted one, and the view moves to the step the
    /// request was made at. Whether it moved to that step.
    fn accept_requested_block(&mut self, id: LayerId, block: &OctantBlock) -> bool {
        if let Some(layer) = self.layers.get_mut(id) {
            layer.load.block_key = None;
        }
        if id == LayerId::BASE {
            self.sync_plotted_state_from_selected();
        }
        let Some(layer) = self.layers.get_mut(id) else {
            return false;
        };
        let Some(target) = layer.load.pending_target_step.take() else {
            return false;
        };
        let Some(dim) = layer.selection().animated_dim else {
            return false;
        };
        if !super::view_filter::covers_step(block, dim, target) {
            return false;
        }
        // Only the base layer moves the shared step; an overlay may have
        // clamped its own request into a shorter extent.
        if id == LayerId::BASE {
            self.current_timestep = target;
        }
        let selection = if !layer.selection().dim_indices.is_empty() {
            Some(layer.selection_mut())
        } else {
            self.staged_selection_mut(id)
        };
        if let Some(index) = selection.and_then(|s| s.dim_indices.get_mut(dim)) {
            *index = target;
        }
        true
    }

    /// Aborts all ongoing data transfers and prefetch worker threads.
    pub fn abort_current_fetch(&mut self) {
        self.block_prefetcher.abort();
        self.is_playing = false;
        for layer in self.layers.iter_mut() {
            layer.load.pending_target_step = None;
        }
        self.status_message = "Data fetch aborted by user.".to_string();
    }
}
