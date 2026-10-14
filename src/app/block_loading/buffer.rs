//! Buffered playback: playback that reaches a step not yet loaded in every
//! drawn animated layer waits (`PlaybackState::buffering`) until
//! `PlaybackState::buffer_secs` of steps are loaded ahead, then plays them
//! without stalling, instead of showing each step the moment it arrives.

use super::layer_request::LayerRequest;
use super::pacing::estimate_chunk_bytes;
use crate::app::OctantApp;

impl OctantApp {
    /// Starts or stops playback; a start waits for its buffer.
    pub fn toggle_playback(&mut self) {
        self.playback.is_playing = !self.playback.is_playing;
        self.playback.last_step_time = web_time::Instant::now();
        self.playback.buffering = self.playback.is_playing;
        if self.playback.is_playing {
            self.prefetch_animated_ranges();
        }
    }

    /// `buffer_secs` of steps at the playback rate, at least one.
    pub(crate) fn buffer_target_steps(&self) -> usize {
        let fps = self.playback.playback_fps.max(1.0);
        (self.playback.buffer_secs.max(0.0) * fps).ceil().max(1.0) as usize
    }

    /// The drawn layers that animate, with their requests.
    fn animated_requests(&self) -> Vec<(crate::app::layers::LayerId, LayerRequest)> {
        self.layers
            .drawn_ids()
            .into_iter()
            .filter_map(|id| {
                let request = self.shown_layer_request(id)?;
                request.anim_dim.map(|_| (id, request))
            })
            .collect()
    }

    /// Drawn layers that animate (at least one), which share the lookahead.
    pub(crate) fn animated_layer_count(&self) -> usize {
        self.animated_requests().len().max(1)
    }

    /// Whether every drawn animated layer holds its step for shared step `step`.
    pub(crate) fn step_ready(&self, step: usize) -> bool {
        let requests = self.animated_requests();
        self.ready_in(&requests, step)
    }

    fn ready_in(
        &self,
        requests: &[(crate::app::layers::LayerId, LayerRequest)],
        step: usize,
    ) -> bool {
        requests
            .iter()
            .all(|(id, r)| self.request_resident(r, self.layer_step_at(*id, step)))
    }

    /// Whether the buffer ahead of step `next` is full: the steps from `next`
    /// on, in playback order (wrapping when looping), are ready for
    /// `buffer_target_steps`, for every step left to play, or for as many
    /// as half the cache holds, whichever is fewest.
    pub(crate) fn buffer_filled(&self, next: usize) -> bool {
        let requests = self.animated_requests();
        let total = self.animated_dim_extent().max(1);
        let left = if self.playback.loop_playback {
            total
        } else {
            total.saturating_sub(next)
        };
        let wanted = self
            .buffer_target_steps()
            .min(left)
            .min(self.buffer_capacity_steps(&requests))
            .max(1);
        (0..wanted).all(|k| self.ready_in(&requests, (next + k) % total))
    }

    /// Steps of every animated layer that half the block cache holds (the
    /// playing lookahead's share), at least one.
    fn buffer_capacity_steps(
        &self,
        requests: &[(crate::app::layers::LayerId, LayerRequest)],
    ) -> usize {
        let step_bytes: usize = requests
            .iter()
            .filter_map(|(_, r)| Some(estimate_chunk_bytes(&r.request, r.anim_dim?, 1)))
            .sum();
        (self.block_cache.max_bytes() / 2 / step_bytes.max(1)).max(1)
    }
}
