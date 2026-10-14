//! Buffered playback: playback that reaches a step not yet loaded in every
//! drawn animated layer waits (`PlaybackState::buffering`) until
//! `PlaybackState::buffer_secs` of steps are loaded ahead, then plays them
//! without stalling, instead of showing each step the moment it arrives.

use super::layer_request::LayerRequest;
use super::pacing::estimate_chunk_bytes;
use crate::app::OctantApp;
use crate::app::layers::LayerId;

impl OctantApp {
    /// Starts or stops playback; a start waits for its buffer.
    pub fn toggle_playback(&mut self) {
        self.playback.is_playing = !self.playback.is_playing;
        self.playback.last_step_time = web_time::Instant::now();
        self.playback.buffering = self.playback.is_playing;
        self.playback.buffer_progress = (0, 0);
        if self.playback.is_playing {
            self.prefetch_animated_ranges();
        }
    }

    /// `buffer_secs` of steps at the playback rate, at least one.
    pub(crate) fn buffer_target_steps(&self) -> usize {
        let fps = self.playback.playback_fps.max(1.0);
        (self.playback.buffer_secs.max(0.0) * fps).ceil().max(1.0) as usize
    }

    /// Drawn layers that animate (at least one), which share the lookahead.
    pub(crate) fn animated_layer_count(&self) -> usize {
        let drawn = self.layers.drawn_ids();
        let animated = drawn
            .iter()
            .filter(|&&id| self.layer_animated_dim(id).is_some());
        animated.count().max(1)
    }

    /// Whether playback may show step `next`: once it is ready, or while
    /// buffering once the buffer ahead of it is (recorded in
    /// `buffer_progress` as steps ready and wanted).
    pub(crate) fn may_show(&mut self, next: usize) -> bool {
        let requests = self.gating_requests();
        let total = self.animated_dim_extent().max(1);
        let wanted = if self.playback.buffering {
            self.buffer_wanted(&requests, next, total)
        } else {
            1
        };
        let ready = requests
            .iter()
            .map(|(id, r)| self.layer_run(*id, r, (next, total), wanted))
            .min()
            .unwrap_or(wanted);
        self.playback.buffer_progress = (ready, wanted);
        ready >= wanted
    }

    /// The layers playback waits for: every drawn animated layer, but an
    /// overlay whose last fetch failed.
    fn gating_requests(&self) -> Vec<(LayerId, LayerRequest)> {
        let failed = |id| id != LayerId::BASE && self.layers.get(id).is_some_and(|l| l.load.failed);
        self.layers
            .drawn_ids()
            .into_iter()
            .filter(|&id| !failed(id))
            .filter_map(|id| {
                let request = self.shown_layer_request(id)?;
                request.anim_dim.map(|_| (id, request))
            })
            .collect()
    }

    /// Steps the buffer wants ahead of `next`: `buffer_target_steps`, every
    /// step left to play, or as many as half the cache holds, whichever is
    /// fewest.
    fn buffer_wanted(
        &self,
        requests: &[(LayerId, LayerRequest)],
        next: usize,
        total: usize,
    ) -> usize {
        let left = if self.playback.loop_playback {
            total
        } else {
            total.saturating_sub(next)
        };
        let step_bytes: usize = requests
            .iter()
            .filter_map(|(_, r)| Some(estimate_chunk_bytes(&r.request, r.anim_dim?, 1)))
            .sum();
        let capacity = self.block_cache.max_bytes() / 2 / step_bytes.max(1);
        self.buffer_target_steps().min(left).min(capacity).max(1)
    }

    /// Steps layer `id` holds from shared step `next` on (wrapping at
    /// `total`), up to `limit`, a resident block at a time.
    fn layer_run(
        &self,
        id: LayerId,
        r: &LayerRequest,
        (next, total): (usize, usize),
        limit: usize,
    ) -> usize {
        let mut k = 0;
        while k < limit {
            let shared = (next + k) % total;
            let step = self.layer_step_at(id, shared);
            let selections = &r.request.selections;
            let until = self.block_cache.covered_until(
                &r.source_id,
                &r.var.name,
                selections,
                r.anim_dim,
                step,
            );
            let Some(end) = until else {
                break;
            };
            // The block holds this layer's steps up to `end`, never past the wrap.
            k += end.saturating_sub(step).clamp(1, total - shared);
        }
        k.min(limit)
    }
}
